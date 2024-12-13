use egui::{Response, RichText, Ui, Widget};
use egui_extras::{Column, TableBuilder};

use crate::{format_bytes, models::peer::Peer};

pub struct PeersWidget<'a> {
    peers: &'a Vec<Peer>,
}

impl<'a> PeersWidget<'a> {
    pub fn new(peers: &'a Vec<Peer>) -> Self {
        Self { peers }
    }
}

impl<'a> Widget for PeersWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        TableBuilder::new(ui)
            .striped(true)
            .auto_shrink(true)
            .vscroll(false)
            .column(Column::remainder().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true).at_least(100.0))
            .column(Column::auto().resizable(true))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("IP Address").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Download Rate").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Upload Rate").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Client").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Progress").strong());
                });
            })
            .body(|mut body| {
                self.peers.iter().for_each(|p| {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(p.ip_address.clone());
                        });
                        row.col(|ui| {
                            ui.label(format_bytes!(p.download_rate, "/s"));
                        });
                        row.col(|ui| {
                            ui.label(format_bytes!(p.upload_rate, "/s"));
                        });
                        row.col(|ui| {
                            ui.label(p.client.clone());
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.2}%", p.progress * 100.0));
                        });
                    });
                });
            });
        ui.response()
    }
}
