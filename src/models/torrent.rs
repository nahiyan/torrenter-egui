use std::fmt;

use super::peer::Peer;

#[derive(PartialEq, Clone, Debug)]
pub enum TorrentFilePriority {
    Skip,
    Default,
    Low,
    High,
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
    pub hash: String,
    pub total_size: i64,
    pub download_rate: i64,
    pub upload_rate: i64,
    pub num_peers: i32,
    pub num_seeds: i32,
    pub pieces: Vec<TorrentPieceState>,
    pub is_streaming: bool,
    pub num_files: i32,
    pub files: Vec<(String, TorrentFilePriority)>,
    pub peers: Vec<Peer>,
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
            num_files: 0,
            files: vec![],
            peers: vec![],
        }
    }
}
