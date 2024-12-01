#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(non_upper_case_globals)]

use eframe::egui;
use egui::{Align, Align2, Color32, Label, RichText, Sense};
use egui::{Layout, Vec2};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use progress_bar::CompoundProgressBar;
use std::sync::mpsc::Sender;
use std::time::Instant;
use std::{
    ffi::{CStr, CString},
    fs,
    os::raw::c_int,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use torrent::{Torrent, TorrentFilePriority, TorrentState};
mod progress_bar;
mod torrent;
include!("../bindings.rs");

macro_rules! format_bytes {
    ($bytes: expr, $prefix: literal) => {{
        let tb = i64::pow(10, 12);
        let gb = i64::pow(10, 9);
        let mb = i64::pow(10, 6);
        let kb = i64::pow(10, 3);

        if $bytes >= tb {
            format!("{:.2} TB{}", $bytes as f32 / tb as f32, $prefix)
        } else if $bytes >= gb {
            format!("{:.2} GB{}", $bytes as f32 / gb as f32, $prefix)
        } else if $bytes >= mb {
            format!("{:.2} MB{}", $bytes as f32 / mb as f32, $prefix)
        } else if $bytes >= kb {
            format!("{:.2} KB{}", $bytes as f32 / kb as f32, $prefix)
        } else {
            format!("{:.2} B{}", $bytes as f32 / mb as f32, $prefix)
        }
    }};

    ($bytes: expr) => {
        format_bytes!($bytes, "")
    };
}

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
    magnet_url: String,
    torrents: Arc<Mutex<Vec<Torrent>>>,
    selection_index: Option<usize>,
    channel_tx: Sender<Message>,
    safe_to_exit: Arc<Mutex<bool>>,
    tab_view: TabView,
}

#[derive(PartialEq, Debug)]
enum Message {
    Stop,
    Refresh,
    ForcedRefresh,
    AddTorrent(String),
    RemoveTorrent(usize),
    UpdateState(TorrentState, usize),
    ToggleStream(usize),
    ChangeFilePriority(usize, usize, TorrentFilePriority),
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

        let safe_to_exit = Arc::new(Mutex::new(false));
        let (tx, rx) = std::sync::mpsc::channel::<Message>();
        let mut last_refresh = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
        let tx_clone = tx.clone();

        // Perform torrent-related tasks in the background
        let torrents_clone = torrents.clone();
        let safe_to_exit_clone = safe_to_exit.clone();
        thread::spawn(move || loop {
            let message = rx.recv().unwrap();
            match message {
                Message::Stop => {
                    println!("Stopping.");
                    unsafe {
                        destroy();
                    };
                    *safe_to_exit_clone.lock().unwrap() = true;

                    break;
                }
                Message::Refresh | Message::ForcedRefresh => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_refresh).as_secs_f32();

                    if elapsed >= 0.9 || message == Message::ForcedRefresh {
                        unsafe { handle_alerts() }
                        torrent::refresh(torrents_clone.clone());
                        last_refresh = now;
                    }
                }
                Message::AddTorrent(magnet_url) => {
                    let downloads_dir = dirs::download_dir()
                        .expect("Failed to get downloads dir.")
                        .to_str()
                        .expect("Failed to convert to string")
                        .to_owned();
                    let magnet_url_cstr =
                        CString::new(magnet_url).expect("Failed to create CString");
                    let downloads_dir_cstr =
                        CString::new(downloads_dir.clone()).expect("Failed to create CString");
                    let mut torrent = Torrent::new("".to_owned(), downloads_dir);
                    // TODO: Handle errors
                    torrent.hash = unsafe {
                        let hash_cstr =
                            add_magnet_url(magnet_url_cstr.as_ptr(), downloads_dir_cstr.as_ptr());
                        CStr::from_ptr(hash_cstr)
                            .to_str()
                            .expect("Failed to work with cstr")
                            .to_string()
                    };
                    tx_clone.send(Message::ForcedRefresh).unwrap();
                }
                Message::UpdateState(state, index) => {
                    if state == torrent::TorrentState::Paused {
                        unsafe {
                            torrent_resume(index as c_int);
                        }
                    } else {
                        unsafe {
                            torrent_pause(index as c_int);
                        }
                    }
                    tx_clone.send(Message::ForcedRefresh).unwrap();
                }
                Message::RemoveTorrent(index) => unsafe {
                    torrent_remove(index as c_int);
                    tx_clone.send(Message::ForcedRefresh).unwrap();
                },
                Message::ToggleStream(index) => unsafe {
                    toggle_stream(index as c_int);
                    tx_clone.send(Message::ForcedRefresh).unwrap();
                },
                Message::ChangeFilePriority(index, f_index, priority) => unsafe {
                    let lt_download_priority = match priority {
                        TorrentFilePriority::Skip => 0,
                        TorrentFilePriority::Low => 1,
                        TorrentFilePriority::Default => 4,
                        TorrentFilePriority::High => 7,
                    };
                    change_file_priority(
                        index as c_int,
                        f_index as c_int,
                        lt_download_priority as c_int,
                    );
                    tx_clone.send(Message::ForcedRefresh).unwrap();
                },
            }
        });

        Self {
            magnet_url: "".to_owned(),
            torrents,
            selection_index: None,
            channel_tx: tx,
            safe_to_exit: safe_to_exit.clone(),
            // selected_tab: Tab::General,
            tab_view: TabView {
                tabs: [
                    (Tab::General, "General".to_owned(), false),
                    (Tab::Files, "Files".to_owned(), false),
                    (Tab::Peers, "Peers".to_owned(), false),
                    (Tab::Trackers, "Trackers".to_owned(), false),
                ],
                selected: Tab::General,
            },
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.channel_tx.send(Message::Refresh).unwrap();
        let torrents = self.torrents.lock().unwrap();

        // Bottom panel
        if let Some(index) = self.selection_index {
            let torrent = &torrents[index - 1];
            egui::TopBottomPanel::bottom("torrent_info")
                .resizable(true)
                .min_height(200.0)
                // .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(0.0, 5.0)))
                .show(ctx, |ui| {
                    // let tabs = ["tab1", "tab2", "tab3"]
                    //     .map(str::to_string)
                    //     .into_iter()
                    //     .collect();
                    // let dock_state = DockState::new(tabs);
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                            if ui.button("✖").clicked() {
                                self.selection_index = None;
                            }
                        });
                    });
                    ui.add_space(5.0);
                    egui::ScrollArea::both().show(ui, |ui| {
                        // Force the scroll area to expand horizontally
                        ui.allocate_at_least(
                            Vec2::new(ui.available_width(), 0.0),
                            Sense::focusable_noninteractive(),
                        );

                        ui.horizontal(|ui| {
                            self.tab_view
                                .tabs
                                .iter_mut()
                                .for_each(|(tab, text, is_hovered)| {
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
                                });
                        });

                        ui.add_space(5.0);
                        ui.heading("Torrent Details");
                        ui.add_space(5.0);

                        match self.tab_view.selected {
                            Tab::General => {
                                ui.vertical(|ui| {
                                    ui.label("General");
                                });
                            }
                            Tab::Files => {
                                let mut files_enabled: Vec<bool> = torrent
                                    .files
                                    .iter()
                                    .map(|f| match f.1 {
                                        TorrentFilePriority::Skip => false,
                                        _ => true,
                                    })
                                    .collect();
                                for (f_index, file) in torrent.files.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .checkbox(&mut files_enabled[f_index], &file.0)
                                            .changed()
                                        {
                                            let is_enabled = files_enabled[f_index];
                                            let priority = if is_enabled {
                                                TorrentFilePriority::Default
                                            } else {
                                                TorrentFilePriority::Skip
                                            };
                                            self.channel_tx
                                                .send(Message::ChangeFilePriority(
                                                    index - 1,
                                                    f_index,
                                                    priority,
                                                ))
                                                .unwrap();
                                        }
                                    });
                                }
                            }
                            _ => {
                                ui.vertical(|ui| {
                                    ui.label("Elit Lorem commodo Lorem proident voluptate sunt ad consequat enim.");
                                });
                            }
                        }
                    });
                });
        }

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.3);

            let mut toasts = Toasts::new()
                .anchor(Align2::LEFT_TOP, (10.0, 10.0))
                .direction(egui::Direction::TopDown);
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Drag and drop guide
                // ui.horizontal(|ui| {
                //     let start_pos = Pos2::new(ui.next_widget_position().x, ui.min_rect().top());
                //     let drop_rect =
                //         Rect::from_min_size(start_pos, Vec2::new(ui.available_width(), 50.0));
                //     let rect = ui.allocate_rect(drop_rect, Sense::hover());
                //     let hovering_files = ctx.input(|i| i.raw.hovered_files.clone());
                //     // let pasted_content = ctx.input(|i| i.raw.);
                //     let about_to_drop = !hovering_files.is_empty();
                //     let color = if about_to_drop {
                //         Color32::WHITE.gamma_multiply(0.2)
                //     } else {
                //         Color32::WHITE.gamma_multiply(0.5)
                //     };
                //     let stroke = if about_to_drop {
                //         Stroke::new(2.0, Color32::GREEN)
                //     } else {
                //         Stroke::new(2.0, Color32::WHITE)
                //     };
                //     ui.painter()
                //         .rect(rect.rect, Rounding::from(0.0), color, stroke);
                // });
                // ui.add_space(10.0);

                // if ui.button("Open File").clicked() {
                //     let file = rfd::FileDialog::new()
                //         .add_filter("torrent", &["torrent"])
                //         .pick_file();
                //     if let Some(f) = file {
                //         println!(
                //             "File selected: {}",
                //             f.to_str().expect("Failed to get string from file path.")
                //         );
                //     } else {
                //         println!("No file selected");
                //     }
                // }

                ui.heading("Add Torrent ");
                ui.horizontal(|ui| {
                    let magnet_url_width =
                        ui.available_width() - ui.spacing().item_spacing.x - 100.0;
                    let magnet_url_textbox = egui::TextEdit::singleline(&mut self.magnet_url)
                        .hint_text("Enter magnet URL here.")
                        .vertical_align(Align::Center);
                    let add_btn = egui::Button::new("Add Torrent");

                    // Add magnet URL handler
                    ui.add_sized(Vec2::new(magnet_url_width, 30.0), magnet_url_textbox);
                    if ui.add_sized(Vec2::new(100.0, 30.0), add_btn).clicked() {
                        self.channel_tx
                            .send(Message::AddTorrent(self.magnet_url.clone()))
                            .unwrap();
                        self.magnet_url = "".to_owned();

                        toasts.add(Toast {
                            text: "Added new torrent.".into(),
                            kind: ToastKind::Success,
                            options: ToastOptions::default()
                                .duration(Duration::from_secs(5))
                                .show_progress(true),
                            ..Default::default()
                        });
                    }
                });
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
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
                                self.selection_index = None;

                                toasts.add(Toast {
                                    text: "Removed torrent.".into(),
                                    kind: ToastKind::Success,
                                    options: ToastOptions::default()
                                        .duration(Duration::from_secs(5))
                                        .show_progress(true),
                                    ..Default::default()
                                });
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
                            let is_selected = Some(index + 1) == self.selection_index;

                            if is_selected {
                                info_btn.clone().highlight();
                            }
                            if info_btn.clicked() {
                                self.selection_index = if !is_selected {
                                    self.channel_tx.send(Message::ForcedRefresh).unwrap();
                                    Some(index + 1)
                                } else {
                                    None
                                };
                            }

                            // Pause/Resume btn
                            let state_btn_text = if torrent.state == torrent::TorrentState::Paused {
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

                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                ui.add(torrent_title);
                            })
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
            });

            toasts.show(ctx);
            ctx.request_repaint_after_secs(1.0);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.channel_tx.send(Message::Stop).unwrap();
        loop {
            if *self.safe_to_exit.lock().unwrap() {
                break;
            }
        }
    }
}
