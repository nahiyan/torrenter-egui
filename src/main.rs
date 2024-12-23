#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(non_upper_case_globals)]

use controllers::add_torrent;
use controllers::message::MessageController;
use eframe::egui;
use egui::Align2;
use egui_toast::Toasts;
use models::message::Message;
use models::tab::{Tab, TabView};
use models::torrent::Torrent;
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
use views::add_torrent::AddTorrentWidget;
use views::tab::TabWidget;
use views::torrent::TorrentWidget;
mod bytes;
mod controllers;
mod duration;
mod models;
mod tests;
mod toasts;
mod views;
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

struct AppState {
    torrents: Arc<Mutex<Vec<Torrent>>>,
    sel_torrent: Arc<Mutex<Option<usize>>>,
    channel_tx: Sender<Message>,
    can_exit: Arc<Mutex<bool>>,
    tab_view: TabView,
    toasts: Arc<Mutex<Toasts>>,
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
        let sel_torrent = Arc::new(Mutex::new(None));
        let toasts = Arc::new(Mutex::new(
            Toasts::new()
                .anchor(Align2::CENTER_TOP, (10.0, 10.0))
                .direction(egui::Direction::TopDown),
        ));

        // Perform torrent-related tasks in the background
        let mut msg_controller = MessageController {
            tx: tx.clone(),
            torrents: torrents.clone(),
            last_refresh,
            can_exit: can_exit.clone(),
            sel_torrent: sel_torrent.clone(),
            toasts: toasts.clone(),
        };
        let can_exit_clone = can_exit.clone();
        thread::spawn(move || loop {
            let message = rx.recv().unwrap();
            msg_controller.process(message);
            if *can_exit_clone.lock().unwrap() {
                break;
            }
        });

        Self {
            torrents,
            sel_torrent,
            channel_tx: tx,
            can_exit,
            tab_view: TabView {
                tabs: [
                    (Tab::General, "General".to_owned(), false),
                    (Tab::Files, "Files".to_owned(), false),
                    (Tab::Peers, "Peers".to_owned(), false),
                ],
                selected: Tab::General,
            },
            toasts,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.channel_tx.send(Message::Refresh).unwrap();
        let torrents = self.torrents.lock().unwrap();
        let mut toasts = self.toasts.lock().unwrap();

        // Bottom panel
        let sel_torrent = *self.sel_torrent.lock().unwrap();
        if let Some(index) = sel_torrent {
            let index = index - 1;
            let torrent = &torrents[index];
            egui::TopBottomPanel::bottom("torrent_info")
                .resizable(true)
                .min_height(200.0)
                .show(ctx, |ui| {
                    ui.add_space(5.0);
                    ui.add(TabWidget {
                        tab_view: &mut self.tab_view,
                        channel_tx: &self.channel_tx,
                        torrent,
                        index,
                    });
                });
        }

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.3);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Handle drag and drop
                let has_hovering_files = ctx.input(|i| !i.raw.hovered_files.is_empty());
                let mut add_btn_clicked = false;
                ui.add(AddTorrentWidget::new(
                    has_hovering_files,
                    &mut add_btn_clicked,
                    ctx,
                ));
                ui.add_space(10.0);
                let dropped_files = ctx.input(|r| r.raw.dropped_files.clone());
                add_torrent::handle_file_drop(&dropped_files, &self.channel_tx);

                // Handle "torrent add" from a file
                if add_btn_clicked {
                    add_torrent::handle_file_add(&mut toasts, &self.channel_tx);
                }

                // Listen for pasted magnet URLs
                add_torrent::handle_magnet_pastes(ctx, &self.channel_tx);

                // Show the torrents
                if !torrents.is_empty() {
                    ui.heading("Torrents");
                    ui.add_space(5.0);
                    for (index, torrent) in torrents.iter().enumerate() {
                        ui.add(TorrentWidget {
                            torrent,
                            sel_torrent: *self.sel_torrent.lock().unwrap(),
                            index,
                            channel_tx: &self.channel_tx,
                        });
                        ui.add_space(10.0);
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
