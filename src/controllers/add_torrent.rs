use std::sync::mpsc::Sender;

use egui::{Context, DroppedFile, Event};
use egui_toast::Toasts;
use rfd::FileDialog;

use crate::{
    models::message::{AddTorrentKind, Message},
    toasts,
};

pub fn handle_file_drop(dropped_files: &[DroppedFile], channel_tx: &Sender<Message>) {
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
        channel_tx
            .send(Message::AddTorrent(file_path, AddTorrentKind::File))
            .unwrap();
    }
}

pub fn handle_file_add(toasts: &mut Toasts, channel_tx: &Sender<Message>) {
    let file_path = FileDialog::new()
        .add_filter("torrent", &["torrent"])
        .pick_file();
    if let Some(file_path) = file_path {
        match file_path.extension().and_then(|e| e.to_str()) {
            Some("torrent") => {
                channel_tx
                    .send(Message::AddTorrent(
                        file_path
                            .to_str()
                            .expect("Failed to convert path to str")
                            .to_string(),
                        AddTorrentKind::File,
                    ))
                    .unwrap();
            }
            _ => {
                toasts::error(toasts, "Only .torrent files are accepted.");
            }
        }
    }
}

pub fn handle_magnet_pastes(ctx: &Context, channel_tx: &Sender<Message>) {
    ctx.input(|r| {
        for event in &r.events {
            if let Event::Paste(text) = event {
                let magnet_url = text.trim().to_string();
                channel_tx
                    .send(Message::AddTorrent(magnet_url, AddTorrentKind::MagnetUrl))
                    .unwrap();
            }
        }
    });
}
