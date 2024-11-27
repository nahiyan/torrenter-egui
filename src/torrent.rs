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
    pub total_size: f32,
    pub total_size_unit: String,
    pub download_rate: f32,
    pub download_rate_unit: String,
    pub upload_rate: f32,
    pub upload_rate_unit: String,
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
            total_size: 0.0,
            total_size_unit: "B/s".to_string(),
            download_rate: 0.0,
            download_rate_unit: "B/s".to_string(),
            upload_rate: 0.0,
            upload_rate_unit: "B/s".to_string(),
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

fn auto_transfer_rate(byte_rate: i32) -> (f32, String) {
    let gb = i32::pow(10, 9);
    let mb = i32::pow(10, 6);
    let kb = i32::pow(10, 3);

    let rate: f32;
    let unit: &str;

    if byte_rate >= gb {
        rate = byte_rate as f32 / gb as f32;
        unit = "GB/s"
    } else if byte_rate >= mb {
        rate = byte_rate as f32 / mb as f32;
        unit = "MB/s"
    } else if byte_rate >= kb {
        rate = byte_rate as f32 / kb as f32;
        unit = "KB/s"
    } else {
        rate = byte_rate as f32;
        unit = "B/s";
    }

    (rate, unit.to_owned())
}

// Use macro to reduce redundancy
fn auto_transfer_size(bytes: i64) -> (f32, String) {
    let tb = i64::pow(10, 12);
    let gb = i64::pow(10, 9);
    let mb = i64::pow(10, 6);
    let kb = i64::pow(10, 3);

    let rate: f32;
    let unit: &str;

    if bytes >= tb {
        rate = bytes as f32 / tb as f32;
        unit = "TB"
    } else if bytes >= gb {
        rate = bytes as f32 / gb as f32;
        unit = "GB"
    } else if bytes >= mb {
        rate = bytes as f32 / mb as f32;
        unit = "MB"
    } else if bytes >= kb {
        rate = bytes as f32 / kb as f32;
        unit = "KB"
    } else {
        rate = bytes as f32;
        unit = "B";
    }

    (rate, unit.to_owned())
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
        (torrent.download_rate, torrent.download_rate_unit) =
            auto_transfer_rate(info.download_rate);
        (torrent.upload_rate, torrent.upload_rate_unit) = auto_transfer_rate(info.upload_rate);
        (torrent.total_size, torrent.total_size_unit) = auto_transfer_size(info.total_size);
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
