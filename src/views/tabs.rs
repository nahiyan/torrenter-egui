use std::sync::mpsc::Sender;

use egui::{Align, Layout, RichText, Sense, Vec2, Widget};

use crate::models::{
    message::Message,
    peer::Peer,
    tab::{Tab, TabView},
    torrent::TorrentFilePriority,
};

use super::{files::FilesWidget, peers::PeersWidget};

pub struct TabWidget<'a> {
    pub tab_view: &'a mut TabView,
    pub channel_tx: &'a Sender<Message>,
    pub files: &'a Vec<(String, TorrentFilePriority)>,
    pub peers: &'a Vec<Peer>,
    pub index: usize,
}

impl<'a> Widget for TabWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                // Close button
                if ui.button("✖").clicked() {
                    self.channel_tx
                        .send(Message::UpdateSelTorrent(None))
                        .unwrap();
                }

                // Tabs
                ui.with_layout(Layout::left_to_right(Align::RIGHT), |ui| {
                    self.tab_view
                        .tabs
                        .iter_mut()
                        .for_each(|(tab, text, is_hovered)| {
                            let rt = {
                                let rt = RichText::new(text.clone());
                                if tab.clone() == self.tab_view.selected {
                                    rt.strong().underline()
                                } else if *is_hovered {
                                    rt.underline()
                                } else {
                                    rt
                                }
                            };
                            let label =
                                ui.label(rt).on_hover_cursor(egui::CursorIcon::PointingHand);
                            if label.clicked() {
                                self.tab_view.selected = tab.clone();
                            }
                            *is_hovered = label.hovered();
                        });
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
                    ui.add(FilesWidget::new(self.files, self.channel_tx, self.index));
                }
                Tab::Peers => {
                    self.channel_tx
                        .send(Message::FetchPeers(self.index))
                        .unwrap();

                    ui.add(PeersWidget::new(self.peers));
                }

                Tab::Trackers => {
                    todo!("Implement trackers tab")
                }
            }
        });
        ui.response()
    }
}
