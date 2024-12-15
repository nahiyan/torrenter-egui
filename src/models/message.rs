use super::torrent::{TorrentFilePriority, TorrentState};

#[derive(PartialEq)]
pub enum AddTorrentKind {
    File,
    MagnetUrl,
}

#[derive(PartialEq)]
pub enum Message {
    Stop,
    Refresh,
    ForcedRefresh,
    AddTorrent(String, AddTorrentKind),
    RemoveTorrent(usize),
    UpdateState(TorrentState, usize),
    UpdateSelTorrent(Option<usize>),
    ToggleStreamMode(usize),
    ChangeFilePriority(usize, usize, TorrentFilePriority),
    FetchPeers(usize),
    OpenDir(String),
}
