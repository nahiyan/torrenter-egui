#[cfg(test)]
mod tests {
    use crate::fs_tree::FSTree;

    use std::path::Path;

    #[test]
    fn test_fs_tree() {
        let paths = vec![
            Path::new("Season 1/a.mkv"),
            Path::new("Season 1/b.mkv"),
            Path::new("Season 1/c.mkv"),
            Path::new("Season 1/d.mkv"),
            Path::new("Season 2/a.mkv"),
            Path::new("Season 2/b.mkv"),
            Path::new("Season 2/c.mkv"),
            Path::new("Season 2/d.mkv"),
            Path::new("Season 2/bonus/a.mkv"),
            Path::new("Season 2/bonus/b.mkv"),
            Path::new("Season 2/bonus/c.mkv"),
            Path::new("Season 2/bonus/d.mkv"),
        ];

        let tree = FSTree::from_paths(paths);
        assert!(tree.is_ok());
        let tree = tree.unwrap();

        // Tree should have 2 children: Season 1, Season 2.
        let root = &tree.nodes[0];
        assert_eq!(root.name, "root".to_string());
        assert!(root.is_dir);
        assert!(root.children_indices.len() == 2);

        // Season 1 and 2 should exist
        let s1 = root.children_names.get("Season 1");
        let s2 = root.children_names.get("Season 2");
        assert!(s1.is_some());
        assert!(s2.is_some());

        // Season 1 should have 4 children
        let s1 = &tree.nodes[*s1.unwrap()];
        assert!(s1.is_dir);
        assert!(s1.children_indices.len() == 4);
        assert!(s1.children_names.contains_key("a.mkv"));
        assert!(s1.children_names.contains_key("b.mkv"));
        assert!(s1.children_names.contains_key("c.mkv"));
        assert!(s1.children_names.contains_key("d.mkv"));

        // Season 2 should have 5 children
        let s2 = &tree.nodes[*s2.unwrap()];
        assert!(s2.is_dir);
        assert!(s2.children_indices.len() == 5);
        assert!(s2.children_names.contains_key("a.mkv"));
        assert!(s2.children_names.contains_key("b.mkv"));
        assert!(s2.children_names.contains_key("c.mkv"));
        assert!(s2.children_names.contains_key("d.mkv"));
        assert!(s2.children_names.contains_key("bonus"));

        // Bonus should have 4 children
        let bonus = &tree.nodes[*s2.children_names.get("bonus").unwrap()];
        assert!(bonus.is_dir);
        assert!(bonus.children_names.contains_key("a.mkv"));
        assert!(bonus.children_names.contains_key("b.mkv"));
        assert!(bonus.children_names.contains_key("c.mkv"));
        assert!(bonus.children_names.contains_key("d.mkv"));
    }
}
