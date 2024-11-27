#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(non_upper_case_globals)]

use eframe::egui;
use egui::{vec2, Align, Align2, Label, Pos2, Rect, RichText, Vec2};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use std::{
    ffi::{CStr, CString},
    fs,
    os::raw::c_int,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use torrent::{Torrent, TorrentPieceState, TorrentState};
mod torrent;
include!("../bindings.rs");
macro_rules! dummy_str {
    () => {
        "Exercitation deserunt eu qui eu pariatur dolore duis velit amet adipisicing ea excepteur cupidatat.".to_owned()
    };
}

fn prepare_data_dir() -> PathBuf {
    let data_dir_base = dirs::data_dir().expect("Failed to get the data dir.");
    let data_dir = data_dir_base.join("com.github.nahiyan").join("torrenter");
    fs::create_dir_all(&data_dir).expect("Failed to create the data dir.");
    fs::create_dir_all(data_dir.join("resume_files")).expect("Failed to create resume files dir.");

    data_dir
}

fn refresh(torrents: Arc<Mutex<Vec<Torrent>>>) {
    torrent::refresh(torrents);
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    log::info!("Application started");

    let data_dir = prepare_data_dir();
    unsafe {
        let resume_dir = data_dir
            .join("resume_files")
            .to_str()
            .expect("Failed to str of path")
            .to_string();
        let resume_dir_cstr = CString::new(resume_dir).expect("Failed to convert to CString");
        // Load torrents from resume files
        initiate(resume_dir_cstr.as_ptr());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_maximize_button(true),
        // .with_inner_size([700.0, 700.0])

        // .with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "Torrenter",
        options,
        Box::new(|_cc| Ok(Box::<AppState>::default())),
    )
}

struct AppState {
    magnet_url: String,
    torrents: Arc<Mutex<Vec<Torrent>>>,
    selection_index: Option<usize>,
    should_stop: Arc<Mutex<bool>>,
    // last_refresh: std::time::Instant,
    // counter: std::sync::Arc<std::sync::Mutex<i32>>,
}

impl Default for AppState {
    fn default() -> Self {
        // let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        // let counter_clone = counter.clone();
        let should_stop = Arc::new(Mutex::new(false));
        let should_stop_clone = should_stop.clone();

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
        let torrents_clone = torrents.clone();

        // Perform torrent-related tasks in the background
        thread::spawn(move || loop {
            if *should_stop_clone.lock().unwrap() {
                println!("Stopping.");
                break;
            }

            unsafe {
                handle_alerts();
            }
            torrent::refresh(torrents_clone.to_owned());
            thread::sleep(std::time::Duration::from_secs(1));
        });

        Self {
            magnet_url: "".to_owned(),
            torrents: torrents,
            selection_index: None,
            should_stop: should_stop,
            // last_refresh: Instant::now(),
            // counter: counter.clone(),
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut torrents = self.torrents.lock().unwrap();

        // Bottom panel
        if let Some(index) = self.selection_index {
            let torrent = &torrents[index - 1];
            egui::TopBottomPanel::bottom("torrent_info")
                .resizable(true)
                .min_height(200.0)
                // .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(0.0, 5.0)))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(5.0);
                        ui.heading("Torrent Details");
                        ui.add_space(5.0);
                        ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Hey"));
                        ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Hey"));
                        ui.label(torrent.name.to_owned());
                        ui.label(dummy_str!());
                        ui.label(dummy_str!());
                        ui.label(dummy_str!());
                        ui.label(dummy_str!());
                    });
                });
        }

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.3);

            let mut toasts = Toasts::new()
                .anchor(Align2::LEFT_TOP, (10.0, 10.0))
                .direction(egui::Direction::TopDown);

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
                let h_width = ui.available_width() - ui.spacing().item_spacing.x;
                let magnet_url_textbox = egui::TextEdit::singleline(&mut self.magnet_url)
                    .hint_text("Enter magnet URL here.")
                    .vertical_align(Align::Center);
                let add_button = egui::Button::new("Add Torrent");

                // Add magnet URL handler
                ui.add_sized(Vec2::new(h_width * 0.8, 30.0), magnet_url_textbox);
                if ui
                    .add_sized(Vec2::new(h_width * 0.2, 30.0), add_button)
                    .clicked()
                {
                    let downloads_dir = dirs::download_dir()
                        .expect("Failed to get downloads dir.")
                        .to_str()
                        .expect("Failed to convert to string")
                        .to_owned();
                    let magnet_url_cstr =
                        CString::new(self.magnet_url.clone()).expect("Failed to create CString");
                    let downloads_dir_cstr =
                        CString::new(downloads_dir.clone()).expect("Failed to create CString");
                    let mut torrent = Torrent::new("".to_owned(), downloads_dir);
                    torrent.hash = unsafe {
                        let hash_cstr =
                            add_magnet_url(magnet_url_cstr.as_ptr(), downloads_dir_cstr.as_ptr());
                        CStr::from_ptr(hash_cstr)
                            .to_str()
                            .expect("Failed to work with cstr")
                            .to_string()
                    };
                    torrents.push(torrent);
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

            egui::ScrollArea::vertical().show(ui, |ui| {
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
                            let remove_btn = ui.button("✖").on_hover_text("Remove".to_owned());
                            if remove_btn.clicked() {
                                unsafe {
                                    torrent_remove(index as c_int);
                                }

                                toasts.add(Toast {
                                    text: "Removed torrent.".into(),
                                    kind: ToastKind::Success,
                                    options: ToastOptions::default()
                                        .duration(Duration::from_secs(5))
                                        .show_progress(true),
                                    ..Default::default()
                                });
                            }

                            let stream_btn = ui
                                .button(if torrent.is_streaming {
                                    RichText::new("📶").strong()
                                } else {
                                    RichText::new("📶")
                                })
                                .on_hover_text("Stream".to_owned());
                            if stream_btn.clicked() {
                                unsafe {
                                    toggle_stream(index as c_int);
                                }
                            }
                            let info_btn = ui.button("ℹ").on_hover_text("Details".to_owned());
                            let toggle_state_btn = ui
                                .button(if torrent.state == torrent::TorrentState::Paused {
                                    "▶"
                                } else {
                                    "⏸"
                                })
                                .on_hover_text("Pause/Resume".to_owned());
                            if toggle_state_btn.clicked() {
                                if torrent.state == torrent::TorrentState::Paused {
                                    unsafe {
                                        torrent_resume(index as c_int);
                                    }
                                } else {
                                    unsafe {
                                        torrent_pause(index as c_int);
                                    }
                                }
                            }

                            let is_selected = Some(index + 1) == self.selection_index;
                            if is_selected {
                                info_btn.clone().highlight();
                            }
                            if info_btn.clicked() {
                                self.selection_index =
                                    if !is_selected { Some(index + 1) } else { None };
                            }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                ui.add(torrent_title);
                            })
                        });

                        // Status
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{} • {:.2} {} • ⬇ {:.2} {} • ⬆ {:.2} {} • {} seeds • {} peers",
                                torrent.state.to_string(),
                                torrent.total_size,
                                torrent.total_size_unit,
                                torrent.download_rate,
                                torrent.download_rate_unit,
                                torrent.upload_rate,
                                torrent.upload_rate_unit,
                                torrent.num_seeds,
                                torrent.num_seeds
                            ));
                        });

                        // // Progress bar
                        // ui.horizontal(|ui| {
                        //     ui.add(
                        //         ProgressBar::new(torrent.progress)
                        //             .rounding(egui::Rounding::from(3.0))
                        //             .show_percentage()
                        //             .desired_height(15.0),
                        //     );
                        // });

                        // Complex progress bar
                        if !(torrent.state == TorrentState::DownloadingMetaData
                            || torrent.state == TorrentState::Allocating)
                        {
                            ui.horizontal(|ui| {
                                ui.label(format!("{:.1}%", torrent.progress * 100.0));

                                let width = ui.available_width();
                                let piece_width = width / torrent.pieces.len() as f32;
                                let start_x = ui.next_widget_position().x;
                                let start_y = ui.min_rect().top();

                                // let background_rect = Rect::from_min_size(
                                //     Pos2::new(start_x, start_y),
                                //     vec2(width, 15.0),
                                // );
                                // ui.painter().rect_filled(
                                //     background_rect,
                                //     egui::Rounding::from(0.0),
                                //     egui::Color32::WHITE,
                                // );

                                for (index, piece) in torrent.pieces.iter().enumerate() {
                                    // if *piece == TorrentPieceState::Incomplete {
                                    //     continue;
                                    // }

                                    let rect = Rect::from_min_size(
                                        Pos2::new(start_x + (index as f32 * piece_width), start_y),
                                        vec2(piece_width, 15.0),
                                    );

                                    let color = match piece {
                                        TorrentPieceState::Complete => egui::Color32::BLUE,
                                        TorrentPieceState::Queued => egui::Color32::GREEN,
                                        _ => egui::Color32::WHITE,
                                    };

                                    // Draw rectangle for each piece
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::Rounding::from(0.0),
                                        color,
                                    );
                                }
                            });
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
        *self.should_stop.lock().unwrap() = true;
        unsafe {
            destroy();
        };
    }
}
