use egui::{CollapsingHeader, Response, Ui, Widget};

use crate::{
    fs_tree::{FSTree, FSTreeNode},
    torrent::{Torrent, TorrentFilePriority},
};

pub struct FilesWidget<'a> {
    torrent: &'a Torrent,
}

impl<'a> FilesWidget<'a> {
    pub fn new(torrent: &'a Torrent) -> Self {
        Self { torrent }
    }
}

impl<'a> Widget for FilesWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let paths = self.torrent.files.iter().map(|(name, _)| name).collect();
        let mut file_priorities: Vec<bool> = self
            .torrent
            .files
            .iter()
            .map(|(_, priority)| !matches!(priority, TorrentFilePriority::Skip))
            .collect();
        let tree = FSTree::from_paths(paths);
        match tree {
            Ok(tree) => {
                assert!(!tree.nodes.is_empty());

                let mut draw_tree = |tree: FSTree, priorities: &mut Vec<bool>| {
                    // TODO: Make it dynamic
                    let mut checked = true;

                    fn draw_node(
                        node: &FSTreeNode,
                        tree: &FSTree,
                        ui: &mut Ui,
                        checked: &mut bool,
                        priorities: &mut Vec<bool>,
                    ) {
                        if node.is_dir {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.checkbox(checked, "");
                                CollapsingHeader::new(&node.name).show(ui, |ui| {
                                    for index in &node.children_indices {
                                        let child_node = &tree.nodes[*index];
                                        draw_node(child_node, tree, ui, checked, priorities);
                                    }
                                });
                            });
                        } else {
                            let path_id = node.path_id;
                            let checkbox = ui.checkbox(&mut priorities[path_id], &node.name);
                            if checkbox.changed() {}
                        }
                    }

                    let root = &tree.nodes[0];
                    for index in &root.children_indices {
                        let root_child = &tree.nodes[*index];
                        draw_node(root_child, &tree, ui, &mut checked, priorities);
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
