use super::torrent::TorrentFilePriority;

#[derive(Clone)]
pub struct File {
    pub path: String,
    pub priority: TorrentFilePriority,
}
