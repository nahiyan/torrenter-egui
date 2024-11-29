#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(non_upper_case_globals)]

use eframe::egui;
use egui::Vec2;
use egui::{Align, Align2, Color32, Label, RichText};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use progress_bar::CompoundProgressBar;
use std::{
    ffi::{CStr, CString},
    fs,
    os::raw::c_int,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use torrent::{Torrent, TorrentState};
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

struct AppState {
    magnet_url: String,
    torrents: Arc<Mutex<Vec<Torrent>>>,
    selection_index: Option<usize>,
    should_stop: Arc<Mutex<bool>>,
}

impl Default for AppState {
    fn default() -> Self {
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
            torrents,
            selection_index: None,
            should_stop,
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

                        for file in &torrent.files {
                            ui.label(file);
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
                let magnet_url_width = ui.available_width() - ui.spacing().item_spacing.x - 100.0;
                let magnet_url_textbox = egui::TextEdit::singleline(&mut self.magnet_url)
                    .hint_text("Enter magnet URL here.")
                    .vertical_align(Align::Center);
                let add_button = egui::Button::new("Add Torrent");

                // Add magnet URL handler
                ui.add_sized(Vec2::new(magnet_url_width, 30.0), magnet_url_textbox);
                if ui.add_sized(Vec2::new(100.0, 30.0), add_button).clicked() {
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
                            // Remove torrent
                            let remove_btn = ui.button("✖").on_hover_text("Remove".to_owned());
                            if remove_btn.clicked() {
                                self.selection_index = None;
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

                            // Toggle strewam
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

                            // Info button
                            let info_btn = ui.button("ℹ").on_hover_text("Details".to_owned());
                            let is_selected = Some(index + 1) == self.selection_index;
                            if is_selected {
                                info_btn.clone().highlight();
                            }
                            if info_btn.clicked() {
                                self.selection_index =
                                    if !is_selected { Some(index + 1) } else { None };
                            }

                            let state_btn_text = if torrent.state == torrent::TorrentState::Paused {
                                "▶"
                            } else {
                                "⏸"
                            };
                            let toggle_state_btn = ui
                                .button(state_btn_text)
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

                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                ui.add(torrent_title);
                            })
                        });

                        // Status
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            // ui.add(egui::Image::new(egui::include_image!(
                            //     "../assets/seeding.svg"
                            // )));

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
                            ui.label(format!(
                                " • {} • ⬇ {} • ⬆ {} • {} seeds • {} peers",
                                format_bytes!(torrent.total_size),
                                format_bytes!(torrent.download_rate, "/s"),
                                format_bytes!(torrent.upload_rate, "/s"),
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

                        // Compound progress bar
                        if !(torrent.state == TorrentState::DownloadingMetaData
                            || torrent.state == TorrentState::Allocating)
                        {
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
        *self.should_stop.lock().unwrap() = true;
        unsafe {
            destroy();
        };
    }
}
