use std::{collections::HashMap, path::Path};

enum ErrAddChild {
    ParentNotFound,
    ChildExists(usize),
}

pub struct TreeNode {
    pub name: String,
    pub children_indices: Vec<usize>,
    pub children_names: HashMap<String, usize>,
}

pub struct Tree {
    pub nodes: Vec<TreeNode>,
}

impl Tree {
    fn new() -> Self {
        let root = TreeNode {
            name: "root".to_string(),
            children_indices: vec![],
            children_names: HashMap::new(),
        };
        Tree { nodes: vec![root] }
    }

    fn add_child(&mut self, parent_id: usize, name: String) -> Result<usize, ErrAddChild> {
        assert!(!self.nodes.is_empty());
        let id = self.nodes.len() as usize;

        let new_node = TreeNode {
            name: name.clone(),
            children_indices: vec![],
            children_names: HashMap::new(),
        };

        // Add to the parent
        let parent = self.nodes.get_mut(parent_id);
        match parent {
            Some(parent) => {
                // Add only if parent has no such child
                let child_lookup = parent.children_names.get(&name);
                match child_lookup {
                    None => {
                        parent.children_names.insert(name, id);
                        parent.children_indices.push(id);
                        self.nodes.push(new_node);
                        Ok(id)
                    }
                    Some(existing_id) => Err(ErrAddChild::ChildExists(*existing_id)),
                }
            }
            None => Err(ErrAddChild::ParentNotFound),
        }
    }

    pub fn from_paths(paths: Vec<&Path>) -> Result<Self, ()> {
        let mut tree = Tree::new();
        for path in paths {
            let mut parent_id = 0;
            for path_comp in path.components() {
                let name = path_comp.as_os_str().to_str().unwrap().to_string();

                let result = tree.add_child(parent_id, name.clone());
                match result {
                    Ok(id) => {
                        parent_id = id;
                    }
                    Err(ErrAddChild::ChildExists(id)) => {
                        parent_id = id;
                    }
                    Err(ErrAddChild::ParentNotFound) => return Err(()),
                }
            }
        }
        Ok(tree)
    }
}
