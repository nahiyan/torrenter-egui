use egui::{Color32, Label, RichText, Widget};

use crate::{
    format_bytes,
    models::torrent::{Torrent, TorrentState},
};

use super::progress_bar::CompoundProgressBar;

pub struct TorrentWidget<'a> {
    pub torrent: &'a Torrent,
}

impl<'a> Widget for TorrentWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let torrent_title = {
            let name = if self.torrent.name == "".to_string() {
                &self.torrent.hash
            } else {
                &self.torrent.name
            };
            let rich_text = RichText::new(name).size(14.0).strong();
            Label::new(rich_text).truncate().halign(egui::Align::LEFT)
        };
        ui.vertical(|ui| {
            // Title and controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                // Remove torrent
                let remove_btn = ui.button("✖").on_hover_text("Remove".to_owned());
                // if remove_btn.clicked() {
                //     self.channel_tx
                //         .send(Message::RemoveTorrent(index.clone()))
                //         .unwrap();
                //     self.sel_torrent = None;

                //     toasts::success(&mut toasts, "Removed the torrent.");
                // }

                // Toggle strewam
                let stream_btn = ui
                    .button(if self.torrent.is_streaming {
                        RichText::new("📶").strong()
                    } else {
                        RichText::new("📶")
                    })
                    .on_hover_text("Stream");
                // if stream_btn.clicked() {
                //     self.channel_tx
                //         .send(Message::ToggleStream(index.clone()))
                //         .unwrap();
                // }

                // Open directory
                if ui
                    .button("📂")
                    .on_hover_text("Open containing directory")
                    .clicked()
                {
                    // open::that(torrent.save_path.clone()).unwrap();
                }

                // Info button
                let info_btn = ui.button("ℹ").on_hover_text("Details");
                // let is_selected = Some(index + 1) == self.sel_torrent;

                // if is_selected {
                //     info_btn.clone().highlight();
                // }
                // if info_btn.clicked() {
                //     self.sel_torrent = if !is_selected {
                //         self.channel_tx.send(Message::ForcedRefresh).unwrap();
                //         Some(index + 1)
                //     } else {
                //         None
                //     };
                // }

                // Pause/Resume btn
                let state_btn_text = if self.torrent.state == TorrentState::Paused {
                    "▶"
                } else {
                    "⏸"
                };
                let toggle_state_btn = ui.button(state_btn_text).on_hover_text("Pause/Resume");
                // if toggle_state_btn.clicked() {
                //     self.channel_tx
                //         .send(Message::UpdateState(torrent.state.clone(), index.clone()))
                //         .unwrap();
                // }

                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.add(torrent_title);
                })
            });

            // Status
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                // Color
                let state_color = match self.torrent.state {
                    TorrentState::Seeding => Color32::BLUE.lerp_to_gamma(Color32::WHITE, 0.6),
                    TorrentState::Downloading => Color32::GREEN.lerp_to_gamma(Color32::WHITE, 0.5),
                    TorrentState::Paused => Color32::ORANGE.lerp_to_gamma(Color32::WHITE, 0.3),
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
                let state_emoji = match self.torrent.state {
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
                        self.torrent.state.to_string()
                    ))
                    .color(state_color),
                );

                // Label
                ui.label(format!(
                    " • {} • ⬇ {} • ⬆ {} • {} seeds • {} peers",
                    format_bytes!(self.torrent.total_size),
                    format_bytes!(self.torrent.download_rate, "/s"),
                    format_bytes!(self.torrent.upload_rate, "/s"),
                    self.torrent.num_seeds,
                    self.torrent.num_seeds
                ));
            });

            // Compound progress bar
            if self.torrent.state == TorrentState::DownloadingMetaData
                || self.torrent.state == TorrentState::Allocating
            {
            } else {
                ui.add(CompoundProgressBar::new(self.torrent));
            }
            ui.add_space(15.0);
        });
        ui.response()
    }
}
