use std::{
    ffi::{c_int, CStr, CString},
    sync::{mpsc::Sender, Arc, Mutex},
    time::Instant,
};

use egui_toast::Toasts;

use crate::{
    models::{
        message::Message,
        peer,
        torrent::{Torrent, TorrentFilePriority, TorrentState},
    },
    AddTorrentKind,
};

use super::torrent;

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
                let downloads_dir = dirs::download_dir()
                    .expect("Failed to get downloads dir.")
                    .to_str()
                    .expect("Failed to convert to string")
                    .to_owned();
                let downloads_dir_cstr =
                    CString::new(downloads_dir.clone()).expect("Failed to create CString");
                let mut torrent = Torrent::new("".to_owned(), downloads_dir);
                let path_cstr = CString::new(path).expect("Failed to create CString");

                match kind {
                    AddTorrentKind::MagnetUrl => {
                        let magnet_url_cstr = path_cstr;
                        // TODO: Handle errors
                        torrent.hash = unsafe {
                            let hash_cstr = add_magnet_url(
                                magnet_url_cstr.as_ptr(),
                                downloads_dir_cstr.as_ptr(),
                            );
                            CStr::from_ptr(hash_cstr)
                                .to_str()
                                .expect("Failed to work with cstr")
                                .to_string()
                        };
                    }
                    AddTorrentKind::File => {
                        let file_path_cstr = path_cstr;
                        // TODO: Handle errors
                        torrent.hash = unsafe {
                            let hash_cstr =
                                add_file(file_path_cstr.as_ptr(), downloads_dir_cstr.as_ptr());
                            // hash_cstr.as_ref().is_none();
                            CStr::from_ptr(hash_cstr)
                                .to_str()
                                .expect("Failed to work with cstr")
                                .to_string()
                        };
                    }
                }
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::UpdateState(state, index) => {
                if state == TorrentState::Paused {
                    unsafe {
                        torrent_resume(index as c_int);
                    }
                } else {
                    unsafe {
                        torrent_pause(index as c_int);
                    }
                }
                self.tx.send(Message::ForcedRefresh).unwrap();
            }
            Message::RemoveTorrent(index) => unsafe {
                torrent_remove(index as c_int);
                self.tx.send(Message::ForcedRefresh).unwrap();
            },
            Message::ToggleStream(index) => unsafe {
                toggle_stream(index as c_int);
                self.tx.send(Message::ForcedRefresh).unwrap();
            },
            Message::ChangeFilePriority(index, f_index, priority) => unsafe {
                let lt_download_priority = match priority {
                    TorrentFilePriority::Skip => 0,
                    TorrentFilePriority::Low => 1,
                    TorrentFilePriority::Default => 4,
                    TorrentFilePriority::High => 7,
                };
                change_file_priority(
                    index as c_int,
                    f_index as c_int,
                    lt_download_priority as c_int,
                );
                self.tx.send(Message::ForcedRefresh).unwrap();
            },
            Message::FetchPeers(index) => {
                let mut num_peers: c_int = 0;
                let num_peers_ptr = &mut num_peers;
                let torrents = self.torrents.clone();
                let mut torrents = torrents.lock().unwrap();
                let peers: &mut Vec<peer::Peer> = &mut torrents[index].peers;
                peers.clear();
                unsafe {
                    let c_peers = get_peers(index as c_int, num_peers_ptr);
                    for i in 0..num_peers {
                        let c_peer = *c_peers.add(i as usize);
                        let ip_address = CStr::from_ptr(c_peer.ip_address)
                            .to_str()
                            .expect("Failed to process C str")
                            .to_string();
                        let client = CStr::from_ptr(c_peer.client)
                            .to_str()
                            .expect("Failed to process C str")
                            .to_string();
                        let download_rate = c_peer.download_rate;
                        let upload_rate = c_peer.upload_rate;
                        let progress = c_peer.progress;
                        let peer = peer::Peer {
                            ip_address,
                            progress,
                            client,
                            download_rate,
                            upload_rate,
                        };
                        peers.push(peer);
                    }
                    free_peers(c_peers, num_peers);
                }
            }
            Message::OpenDir(dir) => open::that(dir.to_string()).expect("Failed to open directory"),
            Message::UpdateSelTorrent(new_sel) => *self.sel_torrent.lock().unwrap() = new_sel,
        }
    }
}
