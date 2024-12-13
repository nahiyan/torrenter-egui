#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(non_upper_case_globals)]

use controllers::message::MessageController;
use eframe::egui;
use egui::{Align, Align2, Color32, DroppedFile, Event, Label, RichText, Sense};
use egui::{Layout, Vec2};
use egui_toast::Toasts;
use models::message::{AddTorrentKind, Message};
use progress_bar::CompoundProgressBar;
use rfd::FileDialog;
use std::sync::mpsc::Sender;
use std::time::Instant;
use std::{
    ffi::CString,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use torrent::{Torrent, TorrentState};
use views::add_torrent::AddTorrentWidget;
mod bytes;
pub mod controllers;
mod fs_tree;
pub mod models;
mod peers;
mod progress_bar;
mod tests;
mod toasts;
mod torrent;
mod views;
use views::files::FilesWidget;
use views::peers::PeersWidget;
include!("../bindings.rs");

fn prepare_data_dir() -> PathBuf {
    let data_dir_base = dirs::data_dir().expect("Failed to get the data dir.");
    let data_dir = data_dir_base.join("com.github.nahiyan").join("torrenter");
    fs::create_dir_all(&data_dir).expect("Failed to create the data dir.");
    fs::create_dir_all(data_dir.join("resume_files")).expect("Failed to create resume files dir.");

    data_dir
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    log::info!("Application started");

    // Load torrents from resume files
    let data_dir = prepare_data_dir();
    let resume_dir = data_dir
        .join("resume_files")
        .to_str()
        .expect("Failed to str of path")
        .to_string();
    unsafe {
        let resume_dir_cstr = CString::new(resume_dir).expect("Failed to convert to CString");
        initiate(resume_dir_cstr.as_ptr());
    }

    // Spawn the frame
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximize_button(true),
        ..Default::default()
    };
    eframe::run_native(
        "Torrenter",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<AppState>::default())
        }),
    )
}

#[derive(PartialEq, Clone)]
enum Tab {
    General,
    Files,
    Peers,
    Trackers,
}

struct TabView {
    tabs: [(Tab, String, bool); 4],
    selected: Tab,
}

struct AppState {
    torrents: Arc<Mutex<Vec<Torrent>>>,
    sel_torrent: Option<usize>,
    channel_tx: Sender<Message>,
    can_exit: Arc<Mutex<bool>>,
    tab_view: TabView,
}

impl Default for AppState {
    fn default() -> Self {
        let torrents = Arc::new(Mutex::new({
            let torrents_count = unsafe { get_count() };
            let mut torrents = Vec::new();
            for _ in 0..torrents_count {
                torrents.push(Torrent::new(
                    "".to_owned(),
                    dirs::download_dir()
                        .expect("Failed to get downloads dir.")
                        .to_str()
                        .to_owned()
                        .expect("Failed to get downloads dir.")
                        .to_owned(),
                ));
            }
            torrents
        }));

        let can_exit = Arc::new(Mutex::new(false));
        let (tx, rx) = std::sync::mpsc::channel::<Message>();
        let last_refresh = Box::new(Instant::now().checked_sub(Duration::from_secs(1)).unwrap());

        // Perform torrent-related tasks in the background
        let mut message_controller = MessageController {
            tx: tx.clone(),
            torrents: torrents.clone(),
            last_refresh,
            can_exit: can_exit.clone(),
        };
        let can_exit_clone = can_exit.clone();
        thread::spawn(move || loop {
            let message = rx.recv().unwrap();
            message_controller.process(message);
            if *can_exit_clone.lock().unwrap() {
                break;
            }
        });

        Self {
            torrents,
            sel_torrent: None,
            channel_tx: tx,
            can_exit,
            tab_view: TabView {
                tabs: [
                    (Tab::General, "General".to_owned(), false),
                    (Tab::Files, "Files".to_owned(), false),
                    (Tab::Peers, "Peers".to_owned(), false),
                    (Tab::Trackers, "Trackers".to_owned(), false),
                ],
                selected: Tab::Files,
            },
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.channel_tx.send(Message::Refresh).unwrap();
        let torrents = self.torrents.lock().unwrap();

        // Bottom panel
        if let Some(index) = self.sel_torrent {
            let index = index - 1;
            let torrent = &torrents[index];
            egui::TopBottomPanel::bottom("torrent_info")
                .resizable(true)
                .min_height(200.0)
                // .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(0.0, 5.0)))
                .show(ctx, |ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                            // Close button
                            if ui.button("✖").clicked() {
                                self.sel_torrent = None;
                            }

                            // Tabs
                            ui.with_layout(Layout::left_to_right(Align::RIGHT), |ui| {
                                self.tab_view.tabs.iter_mut().for_each(
                                    |(tab, text, is_hovered)| {
                                        let rt = {
                                            let rt = RichText::new(text.clone());
                                            if tab.clone() == self.tab_view.selected {
                                                rt.strong().underline()
                                            } else if is_hovered.clone() {
                                                rt.underline()
                                            } else {
                                                rt
                                            }
                                        };
                                        let label = ui
                                            .label(rt)
                                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                                        if label.clicked() {
                                            self.tab_view.selected = tab.clone();
                                        }
                                        *is_hovered = label.hovered();
                                    },
                                );
                            });
                        });
                    });
                    ui.add_space(5.0);
                    egui::ScrollArea::both().show(ui, |ui| {
                        // Force the scroll area to expand horizontally
                        ui.allocate_at_least(
                            Vec2::new(ui.available_width(), 0.0),
                            Sense::focusable_noninteractive(),
                        );

                        ui.add_space(5.0);

                        match self.tab_view.selected {
                            Tab::General => {
                                todo!("Implement general tab")
                            }
                            Tab::Files => {
                                ui.add(FilesWidget::new(&torrent.files));
                            }
                            Tab::Peers => {
                                self.channel_tx.send(Message::FetchPeers(index)).unwrap();

                                ui.add(PeersWidget::new(&torrent.peers));
                            }

                            Tab::Trackers => {
                                todo!("Implement trackers tab")
                            }
                        }
                    });
                });
        }

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.3);

            let mut toasts = Toasts::new()
                .anchor(Align2::CENTER_TOP, (10.0, 10.0))
                .direction(egui::Direction::TopDown);
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Handle drag and drop
                let has_hovering_files = ctx.input(|i| !i.raw.hovered_files.is_empty());
                let mut add_btn_clicked = false;
                ui.add(AddTorrentWidget::new(
                    has_hovering_files,
                    &mut add_btn_clicked,
                ));
                let dropped_files = ctx.input(|r| r.raw.dropped_files.clone());
                if let Some(DroppedFile {
                    path: Some(file_path),
                    mime: _,
                    name: _,
                    last_modified: _,
                    bytes: _,
                }) = dropped_files.first()
                {
                    let file_path = file_path
                        .to_str()
                        .expect("Failed to convert path to str")
                        .to_string();
                    self.channel_tx
                        .send(Message::AddTorrent(file_path, AddTorrentKind::File))
                        .unwrap();
                    toasts::success(&mut toasts, "Added the new torrent.");
                }
                ui.add_space(10.0);

                // Handle "torrent add" from a file
                if add_btn_clicked {
                    let file_path = FileDialog::new()
                        .add_filter("torrent", &["torrent"])
                        .pick_file();
                    if let Some(file_path) = file_path {
                        match file_path.extension() {
                            Some(extension) => {
                                if extension == "torrent" {
                                    self.channel_tx
                                        .send(Message::AddTorrent(
                                            file_path
                                                .to_str()
                                                .expect("Failed to convert path to str")
                                                .to_string(),
                                            AddTorrentKind::File,
                                        ))
                                        .unwrap();
                                    toasts::success(&mut toasts, "Added the new torrent.");
                                } else {
                                    toasts::error(&mut toasts, "Only .torrent files are accepted.");
                                }
                            }
                            None => {}
                        }
                    }
                }

                // Listen for pasted magnet URLs
                ctx.input(|r| {
                    for event in &r.events {
                        match event {
                            Event::Paste(text) => {
                                let magnet_url = text.trim().to_string();
                                self.channel_tx
                                    .send(Message::AddTorrent(
                                        magnet_url,
                                        AddTorrentKind::MagnetUrl,
                                    ))
                                    .unwrap();

                                toasts::success(&mut toasts, "Added the new torrent.");
                            }
                            _ => {}
                        }
                    }
                });

                if !torrents.is_empty() {
                    ui.heading("Torrents");
                    ui.add_space(5.0);
                    for (index, torrent) in torrents.iter().enumerate() {
                        let torrent_title = {
                            let name = if torrent.name == "".to_string() {
                                &torrent.hash
                            } else {
                                &torrent.name
                            };
                            let rich_text = RichText::new(name).size(14.0).strong();
                            Label::new(rich_text).truncate().halign(egui::Align::LEFT)
                        };
                        ui.vertical(|ui| {
                            // Title and controls
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                // Remove torrent
                                let remove_btn = ui.button("✖").on_hover_text("Remove".to_owned());
                                if remove_btn.clicked() {
                                    self.channel_tx
                                        .send(Message::RemoveTorrent(index.clone()))
                                        .unwrap();
                                    self.sel_torrent = None;

                                    toasts::success(&mut toasts, "Removed the torrent.");
                                }

                                // Toggle strewam
                                let stream_btn = ui
                                    .button(if torrent.is_streaming {
                                        RichText::new("📶").strong()
                                    } else {
                                        RichText::new("📶")
                                    })
                                    .on_hover_text("Stream");
                                if stream_btn.clicked() {
                                    self.channel_tx
                                        .send(Message::ToggleStream(index.clone()))
                                        .unwrap();
                                }

                                // Open directory
                                if ui
                                    .button("📂")
                                    .on_hover_text("Open containing directory")
                                    .clicked()
                                {
                                    open::that(torrent.save_path.clone()).unwrap();
                                }

                                // Info button
                                let info_btn = ui.button("ℹ").on_hover_text("Details");
                                let is_selected = Some(index + 1) == self.sel_torrent;

                                if is_selected {
                                    info_btn.clone().highlight();
                                }
                                if info_btn.clicked() {
                                    self.sel_torrent = if !is_selected {
                                        self.channel_tx.send(Message::ForcedRefresh).unwrap();
                                        Some(index + 1)
                                    } else {
                                        None
                                    };
                                }

                                // Pause/Resume btn
                                let state_btn_text = if torrent.state == TorrentState::Paused {
                                    "▶"
                                } else {
                                    "⏸"
                                };
                                let toggle_state_btn =
                                    ui.button(state_btn_text).on_hover_text("Pause/Resume");
                                if toggle_state_btn.clicked() {
                                    self.channel_tx
                                        .send(Message::UpdateState(
                                            torrent.state.clone(),
                                            index.clone(),
                                        ))
                                        .unwrap();
                                }

                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::TOP),
                                    |ui| {
                                        ui.add(torrent_title);
                                    },
                                )
                            });

                            // Status
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;

                                // Color
                                let state_color = match torrent.state {
                                    TorrentState::Seeding => {
                                        Color32::BLUE.lerp_to_gamma(Color32::WHITE, 0.6)
                                    }
                                    TorrentState::Downloading => {
                                        Color32::GREEN.lerp_to_gamma(Color32::WHITE, 0.5)
                                    }
                                    TorrentState::Paused => {
                                        Color32::ORANGE.lerp_to_gamma(Color32::WHITE, 0.3)
                                    }
                                    TorrentState::QueuedForChecking
                                    | TorrentState::CheckingFiles
                                    | TorrentState::DownloadingMetaData
                                    | TorrentState::Allocating
                                    | TorrentState::CheckingResumeData => {
                                        Color32::RED.lerp_to_gamma(Color32::WHITE, 0.5)
                                    }
                                    _ => ui.visuals().text_color(),
                                };

                                // Emoji
                                let state_emoji = match torrent.state {
                                    TorrentState::Finished => "✅",
                                    TorrentState::Seeding => "🍒",
                                    TorrentState::Downloading => "📩",
                                    TorrentState::Paused => "⏸",
                                    _ => "⭕",
                                };
                                ui.label(
                                    RichText::new(format!(
                                        "{} {}",
                                        state_emoji,
                                        torrent.state.to_string()
                                    ))
                                    .color(state_color),
                                );

                                // Label
                                ui.label(format!(
                                    " • {} • ⬇ {} • ⬆ {} • {} seeds • {} peers",
                                    format_bytes!(torrent.total_size),
                                    format_bytes!(torrent.download_rate, "/s"),
                                    format_bytes!(torrent.upload_rate, "/s"),
                                    torrent.num_seeds,
                                    torrent.num_seeds
                                ));
                            });

                            // Compound progress bar
                            if torrent.state == TorrentState::DownloadingMetaData
                                || torrent.state == TorrentState::Allocating
                            {
                            } else {
                                ui.add(CompoundProgressBar::new(torrent));
                            }
                            ui.add_space(15.0);
                        });
                    }
                }
            });

            toasts.show(ctx);
            ctx.request_repaint_after_secs(1.0);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.channel_tx.send(Message::Stop).unwrap();
        loop {
            if *self.can_exit.lock().unwrap() {
                break;
            }
        }
    }
}
