use crate::torrent::{TorrentFilePriority, TorrentState};

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
    ToggleStream(usize),
    ChangeFilePriority(usize, usize, TorrentFilePriority),
    FetchPeers(usize),
}
