use std::{
    cell::RefCell,
    collections::HashSet,
    hash::{Hash, Hasher},
    path::Path,
    rc::Rc,
};

#[derive(Clone, Eq)]
pub struct Tree {
    pub name: String,
    pub level: u32,
    // is_dir: bool,
    pub children: Rc<RefCell<HashSet<Tree>>>,
}

impl Hash for Tree {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Tree {
    fn new(name: String, level: u32) -> Self {
        Self {
            name,
            level,
            children: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn from_name(name: String) -> Self {
        Self {
            name,
            level: 0,
            children: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn from_paths(paths: Vec<&Path>) -> Self {
        let root = Tree::new("root".to_owned(), 0);
        for path in paths {
            let path_comps = path.components();
            // let num_comps = path_comps.clone().count();
            let mut parent = root.children.clone();
            for (index, path_comp) in path_comps.enumerate() {
                let name = path_comp.as_os_str().to_str().unwrap().to_string();
                let level = index as u32 + 1;
                // let is_dir = index != num_comps - 1;

                let comp = {
                    let new_comp = Tree::new(name, level);
                    if let Some(existing_comp) = parent.borrow().get(&new_comp) {
                        existing_comp.to_owned()
                    } else {
                        new_comp
                    }
                };

                let comp_children = comp.children.clone();
                parent.borrow_mut().insert(comp);
                parent = comp_children;
            }
        }
        root
    }

    pub fn display(&self) {
        let indent = (0..self.level)
            .map(|_| "——".to_owned())
            .fold("".to_owned(), |a, b| a + &b)
            .to_owned();
        println!(
            "{}{} | {} children",
            indent,
            self.name,
            self.children.borrow().len()
        );
        for child in self.children.borrow().iter() {
            child.display();
        }
    }
}
