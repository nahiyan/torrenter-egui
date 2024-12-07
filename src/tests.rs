#[cfg(test)]
mod tests {
    use crate::fs_tree::Tree;

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

        let tree = Tree::from_paths(paths);
        // Tree should have 2 children: Season 1, Season 2.
        assert!(tree.children.borrow().len() == 2);
        let root_children = tree.children.borrow();
        let s1 = root_children.get(&Tree::from_name("Season 1".to_string()));
        assert!(s1.is_some());

        let s2 = root_children.get(&Tree::from_name("Season 2".to_string()));
        assert!(s2.is_some());

        let s1 = s1.expect("Failed to unwrap Season 1");
        let s1_children = s1.children.borrow();
        let s2 = s2.expect("Failed to unwrap Season 2");
        let s2_children = s2.children.borrow();

        // Season 1 and 2 should have a, b, c, and d.
        let a = Tree::from_name("a.mkv".to_string());
        let b = Tree::from_name("b.mkv".to_string());
        let c = Tree::from_name("c.mkv".to_string());
        let d = Tree::from_name("d.mkv".to_string());
        assert!(s1_children.contains(&a));
        assert!(s1_children.contains(&b));
        assert!(s1_children.contains(&c));
        assert!(s1_children.contains(&d));
        assert!(s2_children.contains(&a));
        assert!(s2_children.contains(&b));
        assert!(s2_children.contains(&c));
        assert!(s2_children.contains(&d));

        // Season 2 should have bonus.
        let bonus = Tree::from_name("bonus".to_string());
        assert!(s2_children.contains(&bonus));

        // Bonus should have a, b, c, and d.
        let bonus = s2_children
            .get(&bonus)
            .expect("Failed to get Season 2/bonus");
        let bonus_children = bonus.children.borrow();
        assert!(bonus_children.contains(&a));
        assert!(bonus_children.contains(&b));
        assert!(bonus_children.contains(&c));
        assert!(bonus_children.contains(&d));
    }
}
