use std::{collections::HashMap, path::PathBuf};

enum ErrAddChild {
    ParentNotFound,
    ChildExists(usize),
}

pub struct FSTreeNode {
    pub name: String,
    pub is_dir: bool,
    pub children_indices: Vec<usize>,
    pub children_names: HashMap<String, usize>,
}

pub struct FSTree {
    pub nodes: Vec<FSTreeNode>,
}

impl FSTree {
    fn new() -> Self {
        let root = FSTreeNode {
            name: "root".to_string(),
            is_dir: true,
            children_indices: vec![],
            children_names: HashMap::new(),
        };
        FSTree { nodes: vec![root] }
    }

    fn add_child(
        &mut self,
        parent_id: usize,
        name: String,
        is_dir: bool,
    ) -> Result<usize, ErrAddChild> {
        assert!(!self.nodes.is_empty());
        let id = self.nodes.len() as usize;

        let new_node = FSTreeNode {
            name: name.clone(),
            is_dir,
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

    pub fn from_paths<T>(paths: Vec<T>) -> Result<Self, ()>
    where
        T: Into<PathBuf>,
    {
        let mut tree = FSTree::new();
        for path in paths {
            let path: PathBuf = path.into();
            let mut parent_id = 0;
            let comps = path.components();
            let num_comps = comps.clone().count();
            for (index, path_comp) in comps.enumerate() {
                let name = path_comp.as_os_str().to_str().unwrap().to_string();
                let is_dir = if index < num_comps - 1 {
                    true
                } else {
                    path.is_dir()
                };

                let result = tree.add_child(parent_id, name.clone(), is_dir);
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
