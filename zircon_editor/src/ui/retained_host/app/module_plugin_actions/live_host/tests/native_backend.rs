use super::*;

#[test]
fn development_reload_uses_the_selection_scoped_authority_resolver() {
    let source = include_str!("../native_backend.rs");

    assert!(source.contains("authority_resolver"));
    assert!(source.contains("hot_reload_editor_plugin_with_authority"));
    assert!(source.contains("DevelopmentPluginWatch::start"));
}

#[test]
fn development_watch_cleanup_failure_is_reported_without_replacing_the_host_outcome() {
    let mut diagnostics = vec!["native plugin unloaded".to_string()];

    append_development_watch_cleanup_diagnostic(
        &mut diagnostics,
        Err("watch registry is poisoned".to_string()),
    );

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0], "native plugin unloaded");
    assert!(diagnostics[1].contains("native.development_watch.cleanup_failed"));
}

#[test]
fn development_watch_replaces_an_old_project_root_for_the_same_plugin() {
    let base = std::env::temp_dir().join(format!(
        "zircon-editor-development-watch-registry-{}",
        std::process::id()
    ));
    let first_root = base.join("first");
    let second_root = base.join("second");
    std::fs::create_dir_all(&first_root).unwrap();
    std::fs::create_dir_all(&second_root).unwrap();
    let first_artifact = first_root.join("demo.dll");
    let second_artifact = second_root.join("demo.dll");
    let other_artifact = first_root.join("other.dll");
    std::fs::write(&first_artifact, []).unwrap();
    std::fs::write(&second_artifact, []).unwrap();
    std::fs::write(&other_artifact, []).unwrap();
    let first = DevelopmentPluginWatchKey::new(
        ModulePluginLiveHostProject::new(first_root.clone(), "first".to_string(), 1),
        "demo",
        &first_artifact,
    )
    .unwrap();
    let second = DevelopmentPluginWatchKey::new(
        ModulePluginLiveHostProject::new(second_root.clone(), "second".to_string(), 2),
        "demo",
        &second_artifact,
    )
    .unwrap();
    let other = DevelopmentPluginWatchKey::new(
        ModulePluginLiveHostProject::new(first_root.clone(), "first".to_string(), 1),
        "other",
        &other_artifact,
    )
    .unwrap();
    let mut watches = BTreeMap::from([(first.clone(), 1), (other.clone(), 2)]);

    replace_development_watch(&mut watches, second.clone(), 3);

    assert_eq!(watches.len(), 2);
    assert!(!watches.contains_key(&first));
    assert_eq!(watches.get(&second), Some(&3));
    assert_eq!(watches.get(&other), Some(&2));
    let reopened = DevelopmentPluginWatchKey::new(
        ModulePluginLiveHostProject::new(second_root.clone(), "second".to_string(), 3),
        "demo",
        &second_artifact,
    )
    .unwrap();
    replace_development_watch(&mut watches, reopened.clone(), 4);
    assert!(!watches.contains_key(&second));
    assert_eq!(watches.get(&reopened), Some(&4));
    std::fs::remove_dir_all(base).unwrap();
}
