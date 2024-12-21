#ifndef __API_H__
#define __API_H__

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

struct File {
  char *path;
  int priority;
};

struct TorrentInfo {
  const char *name;
  const char *save_path;
  int state;
  float progress;
  int peers, seeds;
  long total_size, download_rate, upload_rate, total_pieces;
  char *pieces;
  bool is_streaming;
};

struct Peer {
  const char *region;
  const char *ip_address;
  const char *client;
  float progress;
  long download_rate;
  long upload_rate;
};

struct Tracker {
  int tier;
  const char *url;
  int status;
  int num_peers;
  int num_seeds;
  const char *message;
};

// Lifecycle
void initiate(const char *resume_dir);
void destroy();

// Torrent management
bool add_file(const char *file_path, const char *save_path);
bool add_magnet_url(const char *url, const char *save_path);
int get_count();
void handle_alerts();
struct TorrentInfo get_torrent_info(int index);
bool torrent_pause(int index);
bool torrent_resume(int index);
bool torrent_remove(int index);
bool toggle_stream(int index);
bool change_file_priority(int, int, int);
struct File *get_files(int index, int *num_files);
void free_files(struct File *files, int num_files);
struct Peer *get_peers(int, int *);
void free_peers(struct Peer *, int);

// Utilities
const char *libtorrent_version();
void free_torrent_info(struct TorrentInfo info);

#ifdef __cplusplus
}
#endif
#endif // __API_H__