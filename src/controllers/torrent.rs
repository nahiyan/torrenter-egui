use crate::models::{
    message::{AddTorrentKind, Message},
    peer,
    torrent::{Torrent, TorrentFilePriority, TorrentPieceState, TorrentState},
};
use std::{
    cell::Cell,
    ffi::{c_int, CStr, CString},
    sync::{mpsc::Sender, Arc, Mutex},
};

include!("../../bindings.rs");

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

pub fn add_torrent(path: String, kind: AddTorrentKind) {
    let downloads_dir = dirs::download_dir()
        .expect("Failed to get downloads dir.")
        .to_str()
        .expect("Failed to convert to string")
        .to_owned();
    let downloads_dir_cstr = CString::new(downloads_dir.clone()).expect("Failed to create CString");
    let path_cstr = CString::new(path).expect("Failed to create CString");

    match kind {
        AddTorrentKind::MagnetUrl => {
            let magnet_url_cstr = path_cstr;
            // TODO: Handle errors
            unsafe {
                let hash_cstr =
                    add_magnet_url(magnet_url_cstr.as_ptr(), downloads_dir_cstr.as_ptr());
                CStr::from_ptr(hash_cstr)
                    .to_str()
                    .expect("Failed to work with cstr")
                    .to_string()
            };
        }
        AddTorrentKind::File => {
            let file_path_cstr = path_cstr;
            // TODO: Handle errors
            unsafe {
                let hash_cstr = add_file(file_path_cstr.as_ptr(), downloads_dir_cstr.as_ptr());
                // hash_cstr.as_ref().is_none();
                CStr::from_ptr(hash_cstr)
                    .to_str()
                    .expect("Failed to work with cstr")
                    .to_string()
            };
        }
    }
}

pub fn remove(index: usize) {
    unsafe {
        torrent_remove(index as c_int);
    }
}

pub fn toggle_stream_mode(index: usize) {
    unsafe {
        toggle_stream(index as c_int);
    }
}

pub fn set_file_priority(index: usize, f_index: usize, priority: TorrentFilePriority) {
    let lt_download_priority = match priority {
        TorrentFilePriority::Skip => 0,
        TorrentFilePriority::Low => 1,
        TorrentFilePriority::Default => 4,
        TorrentFilePriority::High => 7,
    };
    unsafe {
        change_file_priority(
            index as c_int,
            f_index as c_int,
            lt_download_priority as c_int,
        );
    }
}

pub fn fetch_peers(index: usize, torrents: Arc<Mutex<Vec<Torrent>>>) {
    let mut num_peers: c_int = 0;
    let num_peers_ptr = &mut num_peers;
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
