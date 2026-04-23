use std::fmt;

use super::{file::File, peer::Peer};

#[derive(PartialEq, Clone, Debug)]
pub enum TorrentFilePriority {
    Skip,
    Default,
    Low,
    High,
}

impl From<i32> for TorrentFilePriority {
    fn from(value: i32) -> Self {
        match value {
            0 => TorrentFilePriority::Skip,
            1 => TorrentFilePriority::Low,
            4 => TorrentFilePriority::Default,
            7 => TorrentFilePriority::High,
            _ => TorrentFilePriority::Default,
        }
    }
}

impl From<TorrentFilePriority> for i32 {
    fn from(priority: TorrentFilePriority) -> Self {
        match priority {
            TorrentFilePriority::Skip => 0,
            TorrentFilePriority::Low => 1,
            TorrentFilePriority::Default => 4,
            TorrentFilePriority::High => 7,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
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

impl From<i32> for TorrentState {
    fn from(value: i32) -> Self {
        match value {
            0 => TorrentState::QueuedForChecking,
            1 => TorrentState::CheckingFiles,
            2 => TorrentState::DownloadingMetaData,
            3 => TorrentState::Downloading,
            4 => TorrentState::Finished,
            5 => TorrentState::Seeding,
            6 => TorrentState::Allocating,
            7 => TorrentState::CheckingResumeData,
            _ => TorrentState::Paused,
        }
    }
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

#[derive(PartialEq, Clone)]
pub enum TorrentPieceState {
    Complete,
    Incomplete,
    Queued,
}

#[derive(Clone)]
pub struct Torrent {
    pub name: String,
    pub save_path: String,
    pub progress: f32,
    pub state: TorrentState,
    pub total_size: i64,
    pub download_rate: i64,
    pub upload_rate: i64,
    pub num_peers: i32,
    pub num_seeds: i32,
    pub pieces: Vec<TorrentPieceState>,
    pub is_streaming: bool,
    pub files: Vec<File>,
    pub peers: Vec<Peer>,
    pub hash: String,
    pub comment: String,
    pub piece_len: i64,
    pub pieces_downloaded: i32,
    pub active_duration: i32,
    pub seeding_duration: i32,
    pub next_announce: i64,
    pub total_download: i64,
    pub total_upload: i64,
    pub total_ses_download: i64,
    pub total_ses_upload: i64,
    pub eta: i64,
}

impl Torrent {
    pub fn new(name: String, save_path: String) -> Self {
        Self {
            name,
            save_path,
            progress: 0.0,
            state: TorrentState::CheckingResumeData,
            total_size: 0,
            download_rate: 0,
            upload_rate: 0,
            num_peers: 0,
            num_seeds: 0,
            pieces: vec![],
            is_streaming: false,
            files: vec![],
            peers: vec![],
            hash: "".to_string(),
            comment: "".to_string(),
            piece_len: 0,
            pieces_downloaded: 0,
            active_duration: 0,
            seeding_duration: 0,
            next_announce: 0,
            total_download: 0,
            total_upload: 0,
            total_ses_download: 0,
            total_ses_upload: 0,
            eta: 0,
        }
    }
}
