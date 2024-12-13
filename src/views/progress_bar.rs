use egui::{Color32, Pos2, Rect, Rounding, Vec2, Widget};

use crate::torrent::{Torrent, TorrentPieceState};

pub struct CompoundProgressBar<'a> {
    torrent: &'a Torrent,
}

impl<'a> CompoundProgressBar<'a> {
    pub fn new(torrent: &'a Torrent) -> Self {
        CompoundProgressBar { torrent }
    }
}

impl Widget for CompoundProgressBar<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label(format!("{:.1}%", self.torrent.progress * 100.0));

            let bar_width = ui.available_width();
            // let ppp = ui.ctx().pixels_per_point();
            // let groups_count =
            //     (f32::min(bar_width * ppp, self.torrent.pieces.len() as f32)).floor() as u32;
            let groups_count = 100;
            let group_size = self.torrent.pieces.len() as f32 / groups_count as f32;
            let rect_width = bar_width / groups_count as f32;
            let start_pos = Pos2::new(ui.next_widget_position().x, ui.min_rect().top());

            let mut groups: Vec<(u32, u32, u32)> = (0..groups_count).map(|_| (0, 0, 0)).collect();
            for (i, piece) in self.torrent.pieces.iter().enumerate() {
                let group_index = (i as f32 / group_size as f32).floor() as usize;
                let group = &mut groups[group_index];
                let c = match piece {
                    &TorrentPieceState::Complete => &mut group.0,
                    &TorrentPieceState::Queued => &mut group.1,
                    &TorrentPieceState::Incomplete => &mut group.2,
                };
                *c += 1;
            }

            let rects: Vec<Rect> = groups
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    Rect::from_min_size(
                        Pos2::new(start_pos.x + (i as f32 * rect_width), start_pos.y),
                        Vec2::new(rect_width, 15.0),
                    )
                })
                .collect();

            let mut i = 0;
            for rect in rects {
                // c -> complete
                // q -> queued
                // i -> incomplete
                let (c_pieces, q_pieces, i_pieces) = groups[i];
                let total = c_pieces + q_pieces + i_pieces;
                let color = if c_pieces > q_pieces {
                    Color32::WHITE.lerp_to_gamma(
                        Color32::from_rgb(83, 61, 204),
                        c_pieces as f32 / total as f32,
                    )
                } else if q_pieces >= c_pieces {
                    Color32::WHITE.lerp_to_gamma(Color32::GREEN, q_pieces as f32 / total as f32)
                } else {
                    Color32::WHITE
                };
                // TODO: Allocate this space in the ui
                ui.painter().rect_filled(
                    ui.painter().round_rect_to_pixels(rect),
                    Rounding::from(0.0),
                    color,
                );
                i += 1;
            }
        })
        .response
    }
}
