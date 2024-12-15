use crate::models::{
    message::Message,
    torrent::{Torrent, TorrentFilePriority, TorrentPieceState, TorrentState},
};
use std::{
    cell::Cell,
    ffi::{c_int, CStr},
    sync::{mpsc::Sender, Arc, Mutex},
};

include!("../../bindings.rs");

pub struct TorrentController {
    pub torrents: Arc<Mutex<Vec<Torrent>>>,
    pub channel_tx: Sender<Message>,
    pub sel_torrent: Arc<Cell<Option<usize>>>,
}

pub fn refresh(torrents: Arc<Mutex<Vec<Torrent>>>) {
    let torrents_count = unsafe { get_count() as usize };
    let mut torrents = torrents.lock().unwrap();
    torrents.resize(torrents_count, Torrent::new("".to_owned(), "".to_owned()));
    assert!(torrents_count == torrents.len());

    for index in 0..torrents_count {
        let torrent = torrents
            .get_mut(index as usize)
            .expect("Failed to get torrent by index");
        let info = unsafe { get_torrent_info(index as c_int) };
        torrent.progress = info.progress;
        torrent.name = unsafe {
            let c_str = info.name;
            CStr::from_ptr(c_str)
                .to_str()
                .expect("Failed to work with cstr")
                .to_string()
        };
        torrent.state = match info.state {
            0 => TorrentState::QueuedForChecking,
            1 => TorrentState::CheckingFiles,
            2 => TorrentState::DownloadingMetaData,
            3 => TorrentState::Downloading,
            4 => TorrentState::Finished,
            5 => TorrentState::Seeding,
            6 => TorrentState::Allocating,
            7 => TorrentState::CheckingResumeData,
            _ => TorrentState::Paused,
        };
        torrent.total_size = info.total_size;
        torrent.download_rate = info.download_rate;
        torrent.upload_rate = info.upload_rate;
        torrent.num_peers = info.peers;
        torrent.num_seeds = info.seeds;
        torrent.pieces = unsafe {
            let total_pieces = info.total_pieces;
            let mut pieces = vec![];
            for i in 0..total_pieces {
                let piece = *info.pieces.add(i as usize) as u8 as char;
                pieces.push(match piece {
                    'c' => TorrentPieceState::Complete,
                    'i' => TorrentPieceState::Incomplete,
                    'q' => TorrentPieceState::Queued,
                    _ => TorrentPieceState::Incomplete,
                });
            }
            pieces
        };
        torrent.is_streaming = info.is_streaming;
        torrent.num_files = info.num_files;
        torrent.files = unsafe {
            if torrent.num_files > 0 {
                let files = std::slice::from_raw_parts(info.files, info.num_files as usize);
                files
                    .iter()
                    .map(|file| {
                        let path = CStr::from_ptr(file.path)
                            .to_str()
                            .expect("Failed to get C string")
                            .to_string();
                        let priority = match file.priority {
                            0 => TorrentFilePriority::Skip,
                            1 => TorrentFilePriority::Low,
                            4 => TorrentFilePriority::Default,
                            7 => TorrentFilePriority::High,
                            _ => TorrentFilePriority::Default,
                        };
                        (path, priority)
                    })
                    .collect()
            } else {
                vec![]
            }
        };
        torrent.save_path = unsafe {
            CStr::from_ptr(info.save_path)
                .to_str()
                .expect("Failed to process C str")
                .to_string()
        };

        unsafe {
            free_torrent_info(info);
        }
    }
}

impl TorrentController {
    pub fn remove(&self, index: &usize) {
        self.channel_tx
            .send(Message::RemoveTorrent(index.clone()))
            .unwrap();
        self.sel_torrent.set(None);

        // toasts::success(&mut toasts, "Removed the torrent.");
    }

    pub fn toggle_stream(&self, index: &usize) {
        self.channel_tx
            .send(Message::ToggleStream(index.clone()))
            .unwrap();
    }

    pub fn open_dir(&self, torrent: &Torrent) {
        open::that(torrent.save_path.clone()).unwrap();
    }

    pub fn show_info(&self, index: &usize, is_selected: bool) {
        self.sel_torrent.set(if !is_selected {
            Some(index.clone() + 1)
        } else {
            None
        });
        self.channel_tx.send(Message::ForcedRefresh).unwrap();
    }

    pub fn set_state(&self, index: &usize, state: TorrentState) {
        self.channel_tx
            .send(Message::UpdateState(state, index.clone()))
            .unwrap();
    }
}
