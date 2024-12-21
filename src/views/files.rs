use std::{collections::HashSet, sync::mpsc::Sender};

use egui::{CollapsingHeader, Response, Ui, Widget};

use crate::{
    fs_tree::{FSTree, FSTreeNode},
    models::{message::Message, torrent::TorrentFilePriority},
};

pub struct FilesWidget<'a> {
    files: &'a Vec<(String, TorrentFilePriority)>,
    channel_tx: &'a Sender<Message>,
    torrent_index: usize,
}

impl<'a> FilesWidget<'a> {
    pub fn new(
        files: &'a Vec<(String, TorrentFilePriority)>,
        channel_tx: &'a Sender<Message>,
        index: usize,
    ) -> Self {
        Self {
            files,
            channel_tx,
            torrent_index: index,
        }
    }
}

impl<'a> Widget for FilesWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let paths = self.files.iter().map(|(name, _)| name).collect();
        let mut file_priorities: Vec<bool> = self
            .files
            .iter()
            .map(|(_, priority)| !matches!(priority, TorrentFilePriority::Skip))
            .collect();
        let tree = FSTree::from_paths(paths);
        match tree {
            Ok(tree) => {
                assert!(!tree.nodes.is_empty());

                let mut draw_tree = |tree: FSTree, priorities: &mut Vec<bool>| {
                    fn draw_node(
                        node: &FSTreeNode,
                        tree: &FSTree,
                        ui: &mut Ui,
                        channel_tx: &Sender<Message>,
                        priorities: &mut Vec<bool>,
                        torrent_index: usize,
                    ) {
                        if node.is_dir {
                            // Directory
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                let mut ids = HashSet::<usize>::new();
                                tree.path_ids(node, &mut ids);
                                let mut is_checked = { ids.iter().all(|i| priorities[*i]) };
                                let checkbox = ui.checkbox(&mut is_checked, "");
                                if checkbox.changed() {
                                    let new_priority = if is_checked {
                                        TorrentFilePriority::Default
                                    } else {
                                        TorrentFilePriority::Skip
                                    };

                                    for id in ids {
                                        channel_tx
                                            .send(Message::UpdateFilePriority(
                                                torrent_index,
                                                id,
                                                new_priority.clone(),
                                            ))
                                            .unwrap()
                                    }
                                }

                                CollapsingHeader::new(&node.name).show(ui, |ui| {
                                    for index in &node.children_indices {
                                        let child_node = &tree.nodes[*index];
                                        draw_node(
                                            child_node,
                                            tree,
                                            ui,
                                            channel_tx,
                                            priorities,
                                            torrent_index,
                                        );
                                    }
                                });
                            });
                        } else {
                            // File
                            let path_id = node.path_id;
                            let checkbox = ui.checkbox(&mut priorities[path_id], &node.name);
                            if checkbox.changed() {
                                let new_priority = if priorities[path_id] {
                                    TorrentFilePriority::Default
                                } else {
                                    TorrentFilePriority::Skip
                                };
                                channel_tx
                                    .send(Message::UpdateFilePriority(
                                        torrent_index,
                                        path_id,
                                        new_priority,
                                    ))
                                    .unwrap();
                            }
                        }
                    }

                    let root = &tree.nodes[0];
                    for index in &root.children_indices {
                        let root_child = &tree.nodes[*index];
                        draw_node(
                            root_child,
                            &tree,
                            ui,
                            self.channel_tx,
                            priorities,
                            self.torrent_index,
                        );
                    }
                };

                draw_tree(tree, &mut file_priorities);
            }
            Err(()) => {
                ui.label("Failed to load files.");
            }
        };
        ui.response()
    }
}
