use super::*;

#[test]
fn metadata_key_normalizes_separators_and_trailing_slashes() {
    let key = project_metadata_key("E:\\Projects\\Game\\");

    assert_eq!(key, "e:/projects/game");
}

#[test]
fn project_paths_match_uses_metadata_key_normalization() {
    assert!(project_paths_match(
        "E:\\Projects\\Game\\",
        "E:/Projects/Game"
    ));
    assert!(project_paths_match("E:/Projects/Game", "e:/projects/game/"));
}

#[test]
fn filesystem_path_key_canonicalizes_when_possible() {
    let root = std::env::temp_dir().join(format!(
        "zircon-hub-path-key-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let project = root.join("Project");
    std::fs::create_dir_all(&project).unwrap();

    assert_eq!(
        project_filesystem_path_key(project.join(".")),
        project_filesystem_path_key(&project)
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn normalize_project_root_resolves_dot_components_and_strips_extended_prefix() {
    let root = std::env::temp_dir().join(format!(
        "zircon-hub-normalize-root-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let project = root.join("Project");
    std::fs::create_dir_all(&project).unwrap();

    assert_eq!(
        normalize_project_root(project.join(".")),
        strip_windows_extended_length_prefix(project.canonicalize().unwrap())
    );
    assert_eq!(
        normalize_project_root(r"\\?\E:\Projects\Game"),
        PathBuf::from(r"E:\Projects\Game")
    );
    assert_eq!(
        normalize_project_root(r"\\?\UNC\server\share\Game"),
        PathBuf::from(r"\\server\share\Game")
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_metadata_can_be_pruned() {
    let mut metadata = ProjectMetadataMap::new();
    metadata.insert("empty".to_string(), ProjectMetadata::default());
    metadata.insert(
        "pinned".to_string(),
        ProjectMetadata {
            pinned: true,
            ..ProjectMetadata::default()
        },
    );

    prune_empty_metadata(&mut metadata);

    assert!(!metadata.contains_key("empty"));
    assert!(metadata.contains_key("pinned"));
}
