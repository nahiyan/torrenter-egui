use crate::models::torrent::{Torrent, TorrentFilePriority, TorrentPieceState, TorrentState};
use std::{
    ffi::{c_int, CStr},
    sync::{Arc, Mutex},
};

include!("../../bindings.rs");

pub struct TorrentController {
    pub torrents: Arc<Mutex<Vec<Torrent>>>,
}

impl TorrentController {
    pub fn refresh(&mut self) {
        let torrents_count = unsafe { get_count() as usize };
        let mut torrents = self.torrents.lock().unwrap();
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
}
