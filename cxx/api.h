#ifndef __API_H__
#define __API_H__

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

struct File {
  char *name;
};

struct TorrentInfo {
  const char *name;
  int state;
  float progress;
  int peers, seeds;
  long total_size, download_rate, upload_rate, total_pieces;
  char *pieces;
  bool is_streaming;
  int num_files;
  struct File *files;
};

// Lifecycle
void initiate(const char *resume_dir);
void destroy();

// Torrent management
const char *add_magnet_url(const char *url, const char *save_path);
int get_count();
void handle_alerts();
struct TorrentInfo get_torrent_info(int index);
void torrent_pause(int index);
void torrent_resume(int index);
void torrent_remove(int index);
void toggle_stream(int index);

// Utilities
const char *libtorrent_version();
void free_torrent_info(struct TorrentInfo info);

#ifdef __cplusplus
}
#endif
#endif // __API_H__