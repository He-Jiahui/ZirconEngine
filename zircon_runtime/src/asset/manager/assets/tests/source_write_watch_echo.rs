use super::*;

#[test]
fn matching_transaction_echo_is_suppressed_and_sidecar_change_retargets_model() {
    let root = std::env::temp_dir().join(format!(
        "zircon_transaction_watch_echo_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let model_path = root.join("mesh.obj");
    let material_path = root.join("mesh.mtl");
    std::fs::write(&model_path, b"model").unwrap();
    std::fs::write(&material_path, b"initial material").unwrap();
    let model_uri = AssetUri::parse("res://models/mesh.obj").unwrap();
    let material_uri = AssetUri::parse("res://models/mesh.mtl").unwrap();
    let mut echoes = TransactionWatchEchoes::default();
    echoes.register([
        ImportSourceWatchEcho::new(model_uri.clone(), model_uri.clone(), model_path, b"model"),
        ImportSourceWatchEcho::new(
            material_uri.clone(),
            model_uri.clone(),
            material_path.clone(),
            b"initial material",
        ),
    ]);

    assert!(echoes
        .filter(vec![AssetChange::new(
            AssetChangeKind::Added,
            model_uri.clone(),
            None,
        )])
        .is_empty());

    std::fs::write(&material_path, b"changed material").unwrap();
    let changes = echoes.filter(vec![AssetChange::new(
        AssetChangeKind::Modified,
        material_uri,
        None,
    )]);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].uri, model_uri);

    let _ = std::fs::remove_dir_all(root);
}
