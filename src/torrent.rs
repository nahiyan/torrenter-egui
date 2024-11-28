use std::{
    ffi::{c_int, CStr},
    fmt,
    sync::{Arc, Mutex},
};

include!("../bindings.rs");

pub struct Torrent {
    pub name: String,
    pub save_path: String,
    pub progress: f32,
    pub state: TorrentState,
    pub hash: String,
    pub total_size: i64,
    pub download_rate: i64,
    pub upload_rate: i64,
    pub num_peers: i32,
    pub num_seeds: i32,
    pub pieces: Vec<TorrentPieceState>,
    pub is_streaming: bool,
}

impl Torrent {
    pub fn new(name: String, save_path: String) -> Self {
        Self {
            name,
            save_path,
            progress: 0.0,
            state: TorrentState::CheckingResumeData,
            hash: "".to_owned(),
            total_size: 0,
            download_rate: 0,
            upload_rate: 0,
            num_peers: 0,
            num_seeds: 0,
            pieces: vec![],
            is_streaming: false,
        }
    }
}

#[derive(PartialEq)]
pub enum TorrentState {
    QueuedForChecking,
    CheckingFiles,
    DownloadingMetaData,
    Downloading,
    Finished,
    Seeding,
    Allocating,
    CheckingResumeData,
    Paused,
}

impl fmt::Display for TorrentState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            TorrentState::QueuedForChecking => "Queued for checking",
            TorrentState::CheckingFiles => "Checking files",
            TorrentState::DownloadingMetaData => "Downloading metadata",
            TorrentState::Downloading => "Downloading",
            TorrentState::Finished => "Finished",
            TorrentState::Seeding => "Seeding",
            TorrentState::Allocating => "Allocating",
            TorrentState::CheckingResumeData => "Checking resume data",
            TorrentState::Paused => "Paused",
        };
        write!(f, "{}", str)
    }
}

#[derive(PartialEq)]
pub enum TorrentPieceState {
    Complete,
    Incomplete,
    Queued,
}

pub fn refresh(torrents: Arc<Mutex<Vec<Torrent>>>) {
    let torrents_count = unsafe { get_count() as usize };
    let mut torrents = torrents.lock().unwrap();
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

        unsafe {
            free_torrent_info(info);
        }
    }

    torrents.truncate(torrents_count);
}
