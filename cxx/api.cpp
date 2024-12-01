#include "api.h"
#include <chrono>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <libtorrent/add_torrent_params.hpp>
#include <libtorrent/alert.hpp>
#include <libtorrent/alert_types.hpp>
#include <libtorrent/download_priority.hpp>
#include <libtorrent/file_storage.hpp>
#include <libtorrent/magnet_uri.hpp>
#include <libtorrent/read_resume_data.hpp>
#include <libtorrent/session.hpp>
#include <libtorrent/settings_pack.hpp>
#include <libtorrent/torrent_flags.hpp>
#include <libtorrent/torrent_handle.hpp>
#include <libtorrent/torrent_info.hpp>
#include <libtorrent/version.hpp>
#include <libtorrent/write_resume_data.hpp>
#include <stdexcept>
#include <thread>
#include <vector>

using namespace std;
namespace fs = std::filesystem;

struct Torrent {
  lt::torrent_handle h;
  lt::add_torrent_params atp;
  string hash;
  string name;

  Torrent(lt::torrent_handle &h, lt::add_torrent_params &atp, string hash) {
    this->h = h;
    this->atp = atp;
    this->hash = hash;
  }
};

struct State {
  lt::session *ses;
  vector<Torrent *> torrents;
  string resume_dir;
  bool should_stop = false;
  int pending_save_alerts = 0;
} state;

const char *libtorrent_version() { return lt::version(); }

string to_hex(const string binary_data) {
  ostringstream oss;
  for (unsigned char c : binary_data)
    oss << hex << setw(2) << setfill('0') << static_cast<int>(c);
  return oss.str();
}

string get_hash(lt::torrent_handle &h) {
  lt::info_hash_t hashes = h.info_hashes();
  string hash = to_hex(hashes.get_best().to_string());
  return hash;
}

fs::path get_resume_file_path(lt::torrent_handle &h) {
  string hash = get_hash(h);
  assert(!hash.empty());
  fs::path resume_file_path =
      fs::path(state.resume_dir).append(hash + ".resume");
  return resume_file_path;
}

const char *read_resume_file(const char *path) {
  std::ifstream ifs(path, std::ios_base::binary);
  ifs.unsetf(std::ios_base::skipws);
  std::vector<char> buf{std::istream_iterator<char>(ifs),
                        std::istream_iterator<char>()};
  if (buf.size()) {
    lt::add_torrent_params atp = lt::read_resume_data(buf);
    lt::torrent_handle h = state.ses->add_torrent(atp);
    string hash = get_hash(h);
    Torrent *t = new Torrent(h, atp, hash);
    state.torrents.push_back(t);

    return t->hash.c_str();
  }

  printf("Failed to read resume data.\n");
  return "";
}

void write_resume_file(lt::torrent_handle &h, lt::add_torrent_params &atp) {
  try {
    fs::path resume_file_path = get_resume_file_path(h);
    printf("Writing resume file: %s\n", resume_file_path.c_str());
    ofstream of(resume_file_path, ios_base::binary);
    if (!of)
      printf("Failed to write resume file.\n");
    of.unsetf(ios_base::skipws);
    auto const buf = write_resume_data_buf(atp);
    of.write(buf.data(), int(buf.size()));
    of.close();
    cout << "Resume file path: " << resume_file_path << endl;
    // printf("Saved resume file %s.\n", resume_file_path.c_str());
  } catch (...) {
    printf("Failed to write resume file.\n");
  }
}

void initiate(const char *resume_dir) {
  state.ses = new lt::session;

  lt::settings_pack sp = lt::default_settings();
  sp.set_int(sp.active_downloads, -1);
  sp.set_int(sp.connections_limit, 1000);
  sp.set_int(sp.active_seeds, -1);
  sp.set_int(sp.stop_tracker_timeout, 0);
  state.ses->apply_settings(sp);

  state.resume_dir = string(resume_dir);
  try {
    for (const auto &entry : fs::directory_iterator(state.resume_dir))
      read_resume_file(entry.path().c_str());
  } catch (const fs::filesystem_error &e) {
    printf("Failed to read resume files.\n");
  }
}

const char *add_magnet_url(const char *url, const char *save_path) {
  lt::add_torrent_params atp = lt::parse_magnet_uri(url);
  atp.save_path = save_path;
  lt::torrent_handle h = state.ses->add_torrent(atp);
  string hash = get_hash(h);
  Torrent *t = new Torrent(h, atp, hash);
  state.torrents.push_back(t);
  write_resume_file(h, atp);
  return t->hash.c_str();
}

int get_count() { return (int)state.torrents.size(); }

void handle_alerts() {
  std::vector<lt::alert *> alerts;
  state.ses->pop_alerts(&alerts);

  for (lt::alert *alert : alerts) {
    if (auto *at = lt::alert_cast<lt::save_resume_data_alert>(alert)) {
      write_resume_file(at->handle, at->params);
      state.pending_save_alerts--;
    } else if (auto *at =
                   lt::alert_cast<lt::save_resume_data_failed_alert>(alert)) {
      cout << "Failed to save resume data" << endl;
      state.pending_save_alerts--;
    }
  }
}

void torrent_pause(int index) {
  assert(index < state.torrents.size());

  lt::torrent_handle &h = state.torrents[index]->h;
  h.unset_flags(lt::torrent_flags::auto_managed);
  h.set_flags(lt::torrent_flags::paused, lt::torrent_flags::paused);
  h.pause();
}

void torrent_resume(int index) {
  assert(index < state.torrents.size());

  lt::torrent_handle &h = state.torrents[index]->h;
  h.unset_flags(lt::torrent_flags::paused);
  h.set_flags(lt::torrent_flags::auto_managed, lt::torrent_flags::auto_managed);
  h.resume();
}

void torrent_remove(int index) {
  assert(index < state.torrents.size());

  // Remove resume file
  fs::path rf_path = get_resume_file_path(state.torrents[index]->h);
  std::remove(rf_path.c_str());

  // Remove from lt::session
  state.ses->remove_torrent(state.torrents[index]->h);

  // Remove from memory
  delete state.torrents[index];
  state.torrents.erase(state.torrents.begin() + index);
}

void toggle_stream(int index) {
  assert(index < state.torrents.size());

  lt::torrent_handle &h = state.torrents[index]->h;
  bool is_seq = (h.flags() & lt::torrent_flags::sequential_download) ==
                lt::torrent_flags::sequential_download;
  // Roughly guess if we're streaming by priority of the last piece
  int num_pieces = h.status().pieces.size();
  assert(num_pieces > 0);
  // TODO: Check last 1% of the pieces for priority
  bool is_streaming =
      is_seq && h.piece_priority(num_pieces - 1) == lt::top_priority;

  if (!is_streaming)
    h.set_flags(lt::torrent_flags::sequential_download,
                lt::torrent_flags::sequential_download);
  else
    h.unset_flags(lt::torrent_flags::sequential_download);

  // Set the priority for 1% (by size) of last pieces
  int piece_length = h.torrent_file()->piece_length();
  int torrent_size = h.torrent_file()->total_size();
  int last_pieces_count = ceil(((float)torrent_size * 0.01) / piece_length);
  for (int i = max(num_pieces - last_pieces_count, 0); i < num_pieces; i++)
    h.piece_priority(i,
                     !is_streaming ? lt::top_priority : lt::default_priority);
}

void change_file_priority(int index, int f_index, int priority) {
  assert(index < state.torrents.size());

  lt::torrent_handle &h = state.torrents[index]->h;
  h.file_priority(f_index, (lt::download_priority_t)priority);
}

struct TorrentInfo get_torrent_info(int index) {
  assert(index < state.torrents.size());

  Torrent *t = state.torrents[index];
  lt::torrent_handle &h = t->h;
  lt::torrent_status status = h.status();
  auto torrent_info = h.torrent_file();
  TorrentInfo info;

  // Name
  t->name = status.name;
  info.name = t->name.c_str();

  info.progress = status.progress;
  info.peers = status.num_peers;
  info.seeds = status.num_seeds;
  info.download_rate = status.download_rate;
  info.upload_rate = status.upload_rate;

  // State
  bool ses_paused = state.ses->is_paused();
  bool torrent_paused =
      (status.flags & (lt::torrent_flags::auto_managed |
                       lt::torrent_flags::paused)) == lt::torrent_flags::paused;
  info.state = ses_paused || torrent_paused ? -1 : status.state;

  // Size
  info.total_size = torrent_info != nullptr ? torrent_info->total_size()
                                            : status.total_wanted;

  // Pieces: for each char, 'c' -> complete, 'i' -> incomplete, 'q' -> queued.
  info.total_pieces = status.pieces.size();
  info.pieces = new char[info.total_pieces];
  auto &bitfield = status.pieces;
  int i = 0;
  for (bool b : bitfield)
    info.pieces[i++] = b ? 'c' : 'i';

  std::vector<lt::partial_piece_info> queue = h.get_download_queue();
  for (auto &q : queue)
    info.pieces[q.piece_index] = 'q';

  // Streaming
  info.is_streaming = (h.flags() & lt::torrent_flags::sequential_download) ==
                      lt::torrent_flags::sequential_download;

  // Files
  auto file_priorities = h.get_file_priorities();
  if (torrent_info != nullptr) {
    info.num_files = torrent_info->files().num_files();
    info.files = new File[info.num_files];
    for (int i = 0; i < info.num_files; i++) {
      string fpath = torrent_info->files().file_path(i);
      assert(!fpath.empty());
      File &file = info.files[i];
      file.path = new char[fpath.size() + 1];
      copy(fpath.begin(), fpath.end(), info.files[i].path);
      file.path[fpath.size()] = '\0';
      file.priority = file_priorities[i];
    }
  } else {
    info.num_files = 0;
  }

  return info;
}

void free_torrent_info(TorrentInfo info) {
  delete[] info.pieces;
  for (int i = 0; i < info.num_files; i++) {
    delete[] info.files[i].path;
  }
  if (info.num_files > 0)
    delete[] info.files;
}

void destroy() {
  state.ses->pause();
  printf("Session paused.\n");
  for (auto &torrent : state.torrents) {
    torrent->h.pause();
    try {
      if (torrent->h.need_save_resume_data()) {
        torrent->h.save_resume_data(lt::torrent_handle::only_if_modified |
                                    lt::torrent_handle::save_info_dict |
                                    lt::torrent_handle::flush_disk_cache);
        state.pending_save_alerts++;
      }
    } catch (lt::system_error &e) {
      printf("Failed to save resume data.\n");
    }
    delete torrent;
  }
  while (state.pending_save_alerts > 0) {
    handle_alerts();
    this_thread::sleep_for(chrono::milliseconds(100));
  }
  printf("Done with saving.\n");
  state.ses->abort();
  delete state.ses;
  printf("Deleted session.\n");
}