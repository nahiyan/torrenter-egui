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
#[allow(warnings)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use bindings::*;

const TRNT_ADD_FAIL: &str = "Failed to add new torrent.";
const TRNT_ADD_OK: &str = "Added new torrent.";
const TRNT_REMOVE_OK: &str = "Removed torrent.";
const TRNT_REMOVE_FAIL: &str = "Failed to remove torrent.";
const TRNT_PRIORITY_FAIL: &str = "Failed to change priority.";
const TRNT_STATE_FAIL: &str = "Failed to pause/resume torrent state.";

/// Safely convert a C string pointer to a Rust String.
/// Returns an empty string if the pointer is null or contains invalid UTF-8.
unsafe fn c_str_to_string(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr)
        .to_str()
        .unwrap_or("")
        .to_string()
}

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
        torrent.name = unsafe { c_str_to_string(info.name) };
        torrent.state = TorrentState::from(info.state);
        torrent.total_size = info.total_size;
        torrent.download_rate = info.download_rate;
        torrent.upload_rate = info.upload_rate;
        torrent.num_peers = info.peers;
        torrent.num_seeds = info.seeds;
        torrent.pieces = if info.pieces.is_null() || info.total_pieces <= 0 {
            vec![]
        } else {
            unsafe {
                (0..info.total_pieces as usize)
                    .map(|i| match *info.pieces.add(i) as u8 as char {
                        'c' => TorrentPieceState::Complete,
                        'i' => TorrentPieceState::Incomplete,
                        'q' => TorrentPieceState::Queued,
                        _ => TorrentPieceState::Incomplete,
                    })
                    .collect()
            }
        };
        torrent.is_streaming = info.is_streaming;
        torrent.save_path = unsafe { c_str_to_string(info.save_path) };
        torrent.hash = unsafe { c_str_to_string(info.hash) };
        torrent.comment = unsafe { c_str_to_string(info.comment) };
        torrent.piece_len = info.piece_len;
        torrent.pieces_downloaded = info.pieces_downloaded;

        torrent.active_duration = info.active_duration;
        torrent.seeding_duration = info.seeding_duration;

        torrent.next_announce = info.next_announce;

        torrent.total_download = info.total_download;
        torrent.total_upload = info.total_upload;
        torrent.total_ses_download = info.total_ses_download;
        torrent.total_ses_upload = info.total_ses_upload;

        torrent.eta = info.eta;

        unsafe {
            free_torrent_info(info);
        }
    }
}

pub fn add_torrent(path: String, kind: AddTorrentKind, toasts: Arc<Mutex<Toasts>>) {
    let Some(downloads_dir) = dirs::download_dir().and_then(|d| d.to_str().map(String::from))
    else {
        toasts::error(&mut toasts.lock().unwrap(), TRNT_ADD_FAIL);
        return;
    };
    let Ok(downloads_dir_cstr) = CString::new(downloads_dir) else {
        toasts::error(&mut toasts.lock().unwrap(), TRNT_ADD_FAIL);
        return;
    };
    let Ok(path_cstr) = CString::new(path) else {
        toasts::error(&mut toasts.lock().unwrap(), TRNT_ADD_FAIL);
        return;
    };
    let mut toasts = toasts.lock().unwrap();

    let res = match kind {
        AddTorrentKind::MagnetUrl => {
            let magnet_url_cstr = path_cstr;
            unsafe { add_magnet_url(magnet_url_cstr.as_ptr(), downloads_dir_cstr.as_ptr()) }
        }
        AddTorrentKind::File => {
            let file_path_cstr = path_cstr;
            unsafe { add_file(file_path_cstr.as_ptr(), downloads_dir_cstr.as_ptr()) }
        }
    };

    if res {
        toasts::success(&mut toasts, TRNT_ADD_OK);
    } else {
        toasts::error(&mut toasts, TRNT_ADD_FAIL);
    }
}

pub fn remove(index: usize, toasts: Arc<Mutex<Toasts>>) {
    let mut toasts = toasts.lock().unwrap();
    let res = unsafe { torrent_remove(index as c_int) };
    if res {
        toasts::success(&mut toasts, TRNT_REMOVE_OK);
    } else {
        toasts::error(&mut toasts, TRNT_REMOVE_FAIL);
    }
}

pub fn toggle_stream_mode(index: usize, toasts: Arc<Mutex<Toasts>>) {
    let mut toasts = toasts.lock().unwrap();
    let res = unsafe { toggle_stream(index as c_int) };
    if !res {
        toasts::error(&mut toasts, TRNT_REMOVE_FAIL);
    }
}

pub fn set_file_priority(
    index: usize,
    f_index: usize,
    priority: TorrentFilePriority,
    toasts: Arc<Mutex<Toasts>>,
) {
    let mut toasts = toasts.lock().unwrap();
    let lt_download_priority: i32 = priority.into();
    let res = unsafe {
        change_file_priority(
            index as c_int,
            f_index as c_int,
            lt_download_priority as c_int,
        )
    };
    if !res {
        toasts::error(&mut toasts, TRNT_PRIORITY_FAIL);
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
        toasts::error(&mut toasts, TRNT_STATE_FAIL);
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
        if !c_peers.is_null() && num_peers > 0 {
            for i in 0..num_peers {
                let c_peer = *c_peers.add(i as usize);
                peers.push(peer::Peer {
                    ip_address: c_str_to_string(c_peer.ip_address),
                    client: c_str_to_string(c_peer.client),
                    progress: c_peer.progress,
                    download_rate: c_peer.download_rate,
                    upload_rate: c_peer.upload_rate,
                });
            }
            free_peers(c_peers, num_peers);
        }
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
        if !c_files.is_null() && num_files > 0 {
            for i in 0..num_files {
                let c_file = *c_files.add(i as usize);
                files.push(file::File {
                    path: c_str_to_string(c_file.path),
                    priority: TorrentFilePriority::from(c_file.priority),
                });
            }
            free_files(c_files, num_files);
        }
    }
}
