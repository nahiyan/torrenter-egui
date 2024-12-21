use std::{
    ffi::c_int,
    sync::{mpsc::Sender, Arc, Mutex},
    time::Instant,
};

use egui_toast::Toasts;

use super::torrent;
use crate::models::{
    message::Message,
    torrent::{Torrent, TorrentState},
};
include!("../../bindings.rs");

pub struct MessageController {
    pub tx: Sender<Message>,
    pub torrents: Arc<Mutex<Vec<Torrent>>>,
    pub last_refresh: Box<Instant>,
    pub can_exit: Arc<Mutex<bool>>,
    pub sel_torrent: Arc<Mutex<Option<usize>>>,
    pub toasts: Arc<Mutex<Toasts>>,
}

impl MessageController {
    pub fn process(&mut self, message: Message) {
        match message {
            Message::Stop => {
                println!("Stopping.");
                unsafe {
                    destroy();
                };
                *self.can_exit.lock().unwrap() = true;
            }
            Message::Refresh | Message::ForcedRefresh => {
                let now = Instant::now();
                let elapsed = now.duration_since(*self.last_refresh).as_secs_f32();

                if elapsed >= 0.9 || message == Message::ForcedRefresh {
                    unsafe { handle_alerts() }
                    torrent::refresh(self.torrents.clone());
                    self.last_refresh = Box::new(now);
                }
            }
            Message::AddTorrent(path, kind) => {
                torrent::add_torrent(path, kind, self.toasts.clone());
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::UpdateState(state, index) => {
                torrent::toggle_state(index, state, self.toasts.clone());
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::RemoveTorrent(index) => {
                torrent::remove(index, self.toasts.clone());
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::ToggleStreamMode(index) => {
                torrent::toggle_stream_mode(index, self.toasts.clone());
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::UpdateFilePriority(index, f_index, priority) => {
                torrent::set_file_priority(index, f_index, priority, self.toasts.clone());
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::FetchPeers(index) => {
                let torrents = self.torrents.clone();
                torrent::fetch_peers(index, torrents);
            }
            Message::OpenDir(dir) => open::that(dir.to_string()).expect("Failed to open directory"),
            Message::UpdateSelTorrent(new_sel) => *self.sel_torrent.lock().unwrap() = new_sel,
        }
    }
}
