#[test]
fn embedded_play_keeps_startup_cleanup_explicit_and_non_blocking() {
    let source = include_str!("../play_session_factory.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("play session factory production source must precede its tests");
    assert!(production.contains("error.diagnostic_with_recovery()"));
    assert!(!production.contains("error.retry_cleanup()"));
}

#[test]
fn script_binding_payload_survives_play_snapshot() {
    use zircon_editor::core::play::PlaySceneSource;
    use zircon_runtime::scene::{components::NodeKind, DynamicScene, World};

    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let bindings = serde_json::json!([
        {"package":"probe", "module":"main", "enabled":true,
         "update":true, "fixed_update":false,
         "properties":{"role":"player", "speed":2.2, "nested":{"value":[1,2,3]}}},
        {"package":"probe", "module":"other", "enabled":false,
         "update":false, "fixed_update":true, "properties":{}}
    ]);
    world
        .set_dynamic_component(entity, "script.bindings", bindings.clone())
        .unwrap();
    let PlaySceneSource::Snapshot(document) = PlaySceneSource::from_world(&world).unwrap() else {
        panic!("expected in-memory Play snapshot");
    };
    let mut restored = World::empty();
    DynamicScene::from_versioned_json(&document)
        .unwrap()
        .spawn_into(&mut restored)
        .unwrap();
    assert_eq!(
        restored.dynamic_component(entity, "script.bindings"),
        Some(&bindings)
    );
}

#[cfg(feature = "editor-scripting")]
#[test]
fn linked_play_catalog_rejects_missing_required_external_providers() {
    use zircon_runtime::builtin::{RuntimeModuleCompositionCompiler, RuntimePluginId};
    use zircon_runtime::core::framework::{
        platform::RuntimeTargetMode,
        project::{ProjectPluginManifest, ProjectPluginSelection},
    };
    use zircon_runtime::plugin::RuntimePluginCatalog;

    for selection in [
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Ui, true, true)
            .with_runtime_crate("external_ui"),
        ProjectPluginSelection::runtime_plugin("orbrun.missing", true, true)
            .with_runtime_crate("builtin_unknown"),
    ] {
        let catalog = RuntimePluginCatalog::default();
        let plan = catalog.compiled_project_plan(
            &ProjectPluginManifest {
                selections: vec![selection],
            },
            RuntimeTargetMode::ClientRuntime,
        );
        assert!(RuntimeModuleCompositionCompiler::new(&plan)
            .compile()
            .is_err());
    }
    let ui = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Ui, true, true)
        .with_runtime_crate("builtin_ui");
    let catalog = RuntimePluginCatalog::default();
    let duplicate = catalog.compiled_project_plan(
        &ProjectPluginManifest {
            selections: vec![ui.clone(), ui],
        },
        RuntimeTargetMode::ClientRuntime,
    );
    assert!(RuntimeModuleCompositionCompiler::new(&duplicate)
        .compile()
        .is_err());
}

#[cfg(feature = "editor-scripting")]
#[test]
#[ignore = "requires the admitted Linux editor stage (ZIRCON_RUNTIME_LIBRARY and ZIRCON_ASSET_ROOT)"]
fn scripted_play_runs_materialized_snapshot_and_retires() {
    use super::*;
    use zircon_editor::core::play::{PlaySceneSource, PlaySnapshotStore};
    use zircon_runtime::asset::{project::ProjectManifest, AssetUri};
    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::core::framework::{
        platform::RuntimeTargetMode, project::ProjectPluginSelection,
    };
    use zircon_runtime::scene::{components::NodeKind, World};
    use zircon_runtime_interface::world_sync::{WorldQuery, WorldQueryResult};

    let root = std::env::temp_dir().join(format!(
        "scripted-play-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::create_dir_all(root.join("scripts/probe")).unwrap();
    let disk_scene = "[[entities]]\nentity = 999\nname = \"DiskOnly\"\n";
    std::fs::write(root.join("assets/scenes/main.scene.toml"), disk_scene).unwrap();
    let mut manifest = ProjectManifest::new(
        "Scripted Play regression",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.plugins.selections.push(
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::ZrVmLanguage, true, true)
            .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    );
    manifest.scripts.package_roots = vec!["scripts".into()];
    manifest.scripts.startup_packages = vec!["probe".into()];
    manifest.save(root.join("zircon-project.toml")).unwrap();
    std::fs::write(
        root.join("scripts/probe/plugin.toml"),
        r#"
name = "probe"
version = "0.1.0"
entry = "main"
backend = "zr_vm:project"
[capabilities]
capabilities = ["gameplay.entity"]
[zr_vm]
project = "plugin.zrp"
entry_module = "main"
execution_mode = "interp"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("scripts/probe/plugin.zrp"),
        r#"{"name":"probe","source":".","binary":"bin","entry":"main"}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("scripts/probe/main.zr"),
        r#"
let gameplay = import("zr.zircon.gameplay");
pub fn activate(): int { return 0; }
pub fn deactivate(): int { return 0; }
pub fn onStart(entity: int, dt: float): int {
    gameplay.set_position(entity, 7.0, gameplay.script_number(entity, "target_y", 0.0), 9.0); return 0;
}
pub fn onUpdate(entity: int, dt: float): int {
    gameplay.translate(entity, 1.0, 0.0, 0.0); return 0;
}
"#,
    )
    .unwrap();
    let mut world = World::empty();
    let probe = world.spawn_node(NodeKind::Empty).unwrap();
    world.rename_node(probe, "SnapshotProbe").unwrap();
    world
        .set_dynamic_component(
            probe,
            "script.bindings",
            serde_json::json!([{
                "package": "probe", "module": "main", "enabled": true,
                "fixed_update": false, "properties": {"target_y": 8.0}
            }]),
        )
        .unwrap();
    let mut snapshot = PlaySnapshotStore::default()
        .materialize(&root, &PlaySceneSource::from_world(&world).unwrap())
        .unwrap();
    let test_runtime = stage_test_runtime(&root);
    let preflight = crate::entry::runtime_library::LoadedRuntime::preflight_for_request(
        &test_runtime,
        test_runtime.display().to_string(),
    )
    .unwrap();
    let factory = AppPlaySessionFactory::new(preflight, RuntimeCapabilities::editor_default());
    let mut identities = Vec::new();
    for _ in 0..2 {
        let mut lease = factory
            .create(&PlaySessionLaunchRequest::new(
                &root,
                snapshot.relative_path().clone(),
            ))
            .unwrap();
        let gateway = lease.gateway();
        identities.push(gateway.session_identity());
        let read_position = || match gateway
            .query_world(WorldQuery::transform_snapshot(probe))
            .unwrap()
        {
            WorldQueryResult::TransformSnapshot { transform, .. } => transform.translation,
            other => panic!("snapshot probe missing: {other:?}"),
        };
        gateway.tick_frame().unwrap();
        let first = read_position();
        assert!(
            first.x >= 7.0,
            "onStart must execute in the linked snapshot"
        );
        assert_eq!((first.y, first.z), (8.0, 9.0));
        gateway.tick_frame().unwrap();
        assert!(
            read_position().x > first.x,
            "onUpdate must continue executing"
        );
        assert!(
            lease.retire().is_err(),
            "external gateway ownership must prevent retirement"
        );
        drop(gateway);
        lease.retire().unwrap();
    }
    assert_ne!(identities[0], identities[1]);
    assert_eq!(
        std::fs::read_to_string(root.join("assets/scenes/main.scene.toml")).unwrap(),
        disk_scene
    );
    snapshot.cleanup().unwrap();
}

#[cfg(feature = "editor-scripting")]
fn stage_test_runtime(root: &std::path::Path) -> std::path::PathBuf {
    use zircon_runtime_interface::runtime_build_set::{
        ZrRuntimeArtifactIdentityV1, ZrRuntimeArtifactManifestV1, ZrRuntimeDigestV1,
    };

    let source = std::path::PathBuf::from(
        std::env::var_os("ZIRCON_RUNTIME_LIBRARY").expect("select the admitted editor stage"),
    );
    let name = source.file_name().unwrap().to_str().unwrap();
    let sidecar = format!("{name}.manifest.json");
    let mut manifest: ZrRuntimeArtifactManifestV1 =
        serde_json::from_slice(&std::fs::read(source.with_file_name(&sidecar)).unwrap()).unwrap();
    let host = std::env::current_exe().unwrap();
    manifest.host_artifacts.push(ZrRuntimeArtifactIdentityV1 {
        file_name: host.file_name().unwrap().to_str().unwrap().to_owned(),
        sha256: ZrRuntimeDigestV1::sha256_reader(std::fs::File::open(host).unwrap()).unwrap(),
    });
    manifest.build_set_id = manifest.derived_build_set_id().unwrap();
    let directory = root.join("test-runtime");
    std::fs::create_dir(&directory).unwrap();
    let runtime = directory.join(name);
    std::fs::copy(source, &runtime).unwrap();
    std::fs::write(
        directory.join(sidecar),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    runtime
}
