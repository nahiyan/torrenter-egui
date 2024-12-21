#[cfg(test)]
mod tests {
    use crate::models::fs_tree::FSTree;

    use std::{collections::HashSet, path::Path};

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
        assert_eq!(root.children_indices.len(), 2);

        // Season 1 and 2 should exist
        let s1 = root.children_names.get("Season 1");
        let s2 = root.children_names.get("Season 2");
        assert!(s1.is_some());
        assert!(s2.is_some());

        // Season 1 should have 4 children
        let s1 = &tree.nodes[*s1.unwrap()];
        assert_eq!(s1.path_id, 0);
        assert!(s1.is_dir);
        assert_eq!(s1.children_indices.len(), 4);
        assert!(s1.children_names.contains_key("a.mkv"));
        assert!(s1.children_names.contains_key("b.mkv"));
        assert!(s1.children_names.contains_key("c.mkv"));
        assert!(s1.children_names.contains_key("d.mkv"));
        assert_eq!(tree.nodes[s1.children_names["a.mkv"]].path_id, 0);
        assert_eq!(tree.nodes[s1.children_names["b.mkv"]].path_id, 1);
        assert_eq!(tree.nodes[s1.children_names["c.mkv"]].path_id, 2);
        assert_eq!(tree.nodes[s1.children_names["d.mkv"]].path_id, 3);

        let mut s1_path_ids = HashSet::<usize>::new();
        tree.path_ids(s1, &mut s1_path_ids);
        assert_eq!(
            s1_path_ids,
            vec![0, 1, 2, 3].into_iter().collect::<HashSet<usize>>()
        );

        // Season 2 should have 5 children
        let s2 = &tree.nodes[*s2.unwrap()];
        assert_eq!(s2.path_id, 4);
        assert!(s2.is_dir);
        assert_eq!(s2.children_indices.len(), 5);
        assert!(s2.children_names.contains_key("a.mkv"));
        assert!(s2.children_names.contains_key("b.mkv"));
        assert!(s2.children_names.contains_key("c.mkv"));
        assert!(s2.children_names.contains_key("d.mkv"));
        assert!(s2.children_names.contains_key("bonus"));
        assert_eq!(tree.nodes[s2.children_names["a.mkv"]].path_id, 4);
        assert_eq!(tree.nodes[s2.children_names["b.mkv"]].path_id, 5);
        assert_eq!(tree.nodes[s2.children_names["c.mkv"]].path_id, 6);
        assert_eq!(tree.nodes[s2.children_names["d.mkv"]].path_id, 7);

        let mut s2_path_ids = HashSet::<usize>::new();
        tree.path_ids(s2, &mut s2_path_ids);
        assert_eq!(
            s2_path_ids,
            vec![4, 5, 6, 7, 8, 9, 10, 11]
                .into_iter()
                .collect::<HashSet<usize>>()
        );

        // Bonus should have 4 children
        let bonus = &tree.nodes[*s2.children_names.get("bonus").unwrap()];
        assert_eq!(bonus.path_id, 8);
        assert!(bonus.is_dir);
        assert!(bonus.children_names.contains_key("a.mkv"));
        assert!(bonus.children_names.contains_key("b.mkv"));
        assert!(bonus.children_names.contains_key("c.mkv"));
        assert!(bonus.children_names.contains_key("d.mkv"));
        assert_eq!(tree.nodes[bonus.children_names["a.mkv"]].path_id, 8);
        assert_eq!(tree.nodes[bonus.children_names["b.mkv"]].path_id, 9);
        assert_eq!(tree.nodes[bonus.children_names["c.mkv"]].path_id, 10);
        assert_eq!(tree.nodes[bonus.children_names["d.mkv"]].path_id, 11);

        let mut bonus_path_ids = HashSet::<usize>::new();
        tree.path_ids(bonus, &mut bonus_path_ids);
        assert_eq!(
            bonus_path_ids,
            vec![8, 9, 10, 11].into_iter().collect::<HashSet<usize>>()
        );
    }
}
