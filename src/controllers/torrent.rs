use egui_toast::Toasts;

use crate::{
    models::{
        file,
        message::AddTorrentKind,
        peer,
        torrent::{Torrent, TorrentFilePriority, TorrentPieceState, TorrentState},
    },
    toasts,
};
use std::{
    ffi::{c_int, CStr, CString},
    sync::{Arc, Mutex},
};
include!("../../bindings.rs");
const trnt_add_fail_msg: &str = "Failed to add new torrent.";
const trnt_add_success_msg: &str = "Added new torrent.";
const trnt_remove_success_msg: &str = "Removed torrent.";
const trnt_remove_fail_msg: &str = "Failed to remove torrent.";
const trnt_set_file_priority_fail_msg: &str = "Failed to change priority.";
const trnt_set_state_fail_msg: &str = "Failed to pause/resume torrent state.";

pub fn refresh(torrents: Arc<Mutex<Vec<Torrent>>>) {
    let torrents_count = unsafe { get_count() as usize };
    let mut torrents = torrents.lock().unwrap();
    torrents.resize(torrents_count, Torrent::new("".to_owned(), "".to_owned()));
    assert!(torrents_count == torrents.len());

    for index in 0..torrents_count {
        let torrent = torrents
            .get_mut(index)
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

pub fn add_torrent(path: String, kind: AddTorrentKind, toasts: Arc<Mutex<Toasts>>) {
    let downloads_dir = dirs::download_dir()
        .expect("Failed to get downloads dir.")
        .to_str()
        .expect("Failed to convert to string")
        .to_owned();
    let downloads_dir_cstr = CString::new(downloads_dir.clone()).expect("Failed to create CString");
    let path_cstr = CString::new(path).expect("Failed to create CString");
    let mut toasts = toasts.lock().unwrap();

    let res = match kind {
        AddTorrentKind::MagnetUrl => {
            let magnet_url_cstr = path_cstr;
            // TODO: Handle errors
            unsafe { add_magnet_url(magnet_url_cstr.as_ptr(), downloads_dir_cstr.as_ptr()) }
        }
        AddTorrentKind::File => {
            let file_path_cstr = path_cstr;
            // TODO: Handle errors
            unsafe { add_file(file_path_cstr.as_ptr(), downloads_dir_cstr.as_ptr()) }
        }
    };

    if res {
        toasts::success(&mut toasts, trnt_add_success_msg);
    } else {
        toasts::error(&mut toasts, trnt_add_fail_msg);
    }
}

pub fn remove(index: usize, toasts: Arc<Mutex<Toasts>>) {
    let mut toasts = toasts.lock().unwrap();
    let res = unsafe { torrent_remove(index as c_int) };
    if res {
        toasts::success(&mut toasts, trnt_remove_success_msg);
    } else {
        toasts::error(&mut toasts, trnt_remove_fail_msg);
    }
}

pub fn toggle_stream_mode(index: usize, toasts: Arc<Mutex<Toasts>>) {
    let mut toasts = toasts.lock().unwrap();
    let res = unsafe { toggle_stream(index as c_int) };
    if !res {
        toasts::error(&mut toasts, trnt_remove_fail_msg);
    }
}

pub fn set_file_priority(
    index: usize,
    f_index: usize,
    priority: TorrentFilePriority,
    toasts: Arc<Mutex<Toasts>>,
) {
    let mut toasts = toasts.lock().unwrap();
    let lt_download_priority = match priority {
        TorrentFilePriority::Skip => 0,
        TorrentFilePriority::Low => 1,
        TorrentFilePriority::Default => 4,
        TorrentFilePriority::High => 7,
    };
    let res = unsafe {
        change_file_priority(
            index as c_int,
            f_index as c_int,
            lt_download_priority as c_int,
        )
    };
    if !res {
        toasts::error(&mut toasts, trnt_set_file_priority_fail_msg);
    }
}

pub fn toggle_state(index: usize, state: TorrentState, toasts: Arc<Mutex<Toasts>>) {
    let mut toasts = toasts.lock().unwrap();
    let res = if state == TorrentState::Paused {
        unsafe { torrent_resume(index as c_int) }
    } else {
        unsafe { torrent_pause(index as c_int) }
    };
    if !res {
        toasts::error(&mut toasts, trnt_set_state_fail_msg);
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

pub fn fetch_files(index: usize, torrents: Arc<Mutex<Vec<Torrent>>>) {
    let mut num_files: c_int = 0;
    let num_files_ptr = &mut num_files;
    let mut torrents = torrents.lock().unwrap();
    let files = &mut torrents[index].files;
    files.clear();
    unsafe {
        let c_files = get_files(index as c_int, num_files_ptr);
        for i in 0..num_files {
            let c_file = *c_files.add(i as usize);
            let path = CStr::from_ptr(c_file.path)
                .to_str()
                .expect("Failed to process C str")
                .to_string();
            let priority = match c_file.priority {
                0 => TorrentFilePriority::Skip,
                1 => TorrentFilePriority::Low,
                4 => TorrentFilePriority::Default,
                7 => TorrentFilePriority::High,
                _ => TorrentFilePriority::Default,
            };
            let file = file::File { path, priority };
            files.push(file);
        }
        free_files(c_files, num_files);
    }
}
