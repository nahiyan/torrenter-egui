use egui::{Grid, Widget};

use crate::{duration::format_duration, format_bytes, models::torrent::Torrent};

pub struct GeneralWidget<'a> {
    pub torrent: &'a Torrent,
}

impl<'a> Widget for GeneralWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Grid::new("Information").num_columns(2).show(ui, |ui| {
            // ETA
            ui.label("ETA: ");
            ui.label(format_duration(self.torrent.eta));
            ui.end_row();

            // Time active
            ui.label("Time Active: ");
            ui.label(format!(
                "{} (seeding for {})",
                format_duration(self.torrent.active_duration),
                format_duration(self.torrent.seeding_duration)
            ));
            ui.end_row();

            ui.label("Downloaded: ");
            ui.label(format!(
                "{} ({} in this session)",
                format_bytes!(self.torrent.total_download),
                format_bytes!(self.torrent.total_ses_download)
            ));
            ui.end_row();

            ui.label("Uploaded: ");
            ui.label(format!(
                "{} ({} in this session)",
                format_bytes!(self.torrent.total_upload),
                format_bytes!(self.torrent.total_ses_upload)
            ));
            ui.end_row();

            // Reannounce In
            ui.label("Reannounce In: ");
            ui.label(format_duration(self.torrent.next_announce));
            ui.end_row();

            // Save Path
            ui.label("Save Path: ");
            ui.label(self.torrent.save_path.clone());
            ui.end_row();

            // Hash
            ui.label("Hash: ");
            ui.label(self.torrent.hash.clone());
            ui.end_row();

            // Pieces
            ui.label("Pieces: ");
            ui.label(format!(
                "{} x {} (have {})",
                self.torrent.pieces.len(),
                format_bytes!(self.torrent.piece_len),
                self.torrent.pieces_downloaded
            ));
            ui.end_row();

            // Comment
            ui.label("Comment: ");
            ui.label(self.torrent.comment.clone());
            ui.end_row();
        });
        ui.response()
    }
}
