use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::framework::foundation::ConfigManager;
use crate::core::manager::{ManagerResolver, CONFIG_MANAGER_NAME};
use crate::core::CoreRuntime;
use serde_json::json;

#[cfg(feature = "animation")]
use crate::animation::{
    module_descriptor as animation_module_descriptor, DefaultAnimationManager,
    ANIMATION_MODULE_NAME, ANIMATION_PLAYBACK_CONFIG_KEY, DEFAULT_ANIMATION_MANAGER_NAME,
};
#[cfg(feature = "animation")]
use crate::core::framework::animation::{AnimationManager, AnimationPlaybackSettings};
use crate::foundation::{module_descriptor, FOUNDATION_MODULE_NAME};

#[test]
fn foundation_descriptor_exposes_only_behavioral_managers() {
    let descriptor = module_descriptor();

    assert!(
        descriptor.drivers.is_empty(),
        "foundation must not publish placeholder drivers as ready capabilities"
    );
    assert_eq!(descriptor.managers.len(), 1);
    assert!(descriptor
        .managers
        .iter()
        .any(|manager| manager.name.as_str() == CONFIG_MANAGER_NAME));
}

#[test]
fn foundation_root_stays_structural_after_module_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("foundation")
            .join("mod.rs"),
    )
    .expect("foundation mod source");

    for forbidden in [
        "pub struct FoundationModule",
        "impl EngineModule for FoundationModule",
        "fn module_name(&self)",
        "fn module_description(&self)",
        "fn descriptor(&self)",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected foundation/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn config_manager_roundtrip_works_through_resolver() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();

    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    config
        .set_value("editor.layout", json!({"dock": "main"}))
        .unwrap();

    assert_eq!(
        config.get_value("editor.layout").unwrap()["dock"],
        json!("main")
    );
}

#[test]
fn versioned_manager_handle_rejects_the_unloaded_generation() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let resolver = ManagerResolver::new(runtime.handle());
    let stale_handle = resolver.config_handle().unwrap();

    resolver.resolve(stale_handle.clone()).unwrap();
    runtime.deactivate_module(FOUNDATION_MODULE_NAME).unwrap();

    assert!(matches!(
        resolver.config_handle(),
        Err(crate::core::CoreError::ServiceUnavailable(name)) if name == CONFIG_MANAGER_NAME
    ));

    let error = match resolver.resolve(stale_handle.clone()) {
        Ok(_) => panic!("stale manager generation unexpectedly resolved"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        crate::core::CoreError::StaleServiceHandle {
            expected_index,
            expected_generation,
            actual_index,
            actual_generation,
            ..
        } if expected_index == stale_handle.index
            && expected_generation == stale_handle.generation
            && actual_index == stale_handle.index
            && actual_generation == stale_handle.generation + 1
    ));

    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let current_handle = resolver.config_handle().unwrap();
    assert_eq!(current_handle.index, stale_handle.index);
    assert_eq!(current_handle.generation, stale_handle.generation + 1);
    resolver.resolve(current_handle).unwrap();
}

#[test]
fn foundation_registry_services_do_not_retain_the_runtime_root() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let weak = runtime.weak();

    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();

    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "foundation registry services must not keep CoreRuntime alive"
    );
    assert_eq!(config.get_value("runtime.gone"), None);
    assert_eq!(
        config.set_value("runtime.gone", json!(true)),
        Err(crate::core::framework::foundation::ConfigManagerError::RuntimeUnavailable)
    );
}

#[test]
fn config_manager_persists_values_to_disk() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_config_{unique}.json"));
    let _config_path_override = super::runtime::override_config_file_path_for_test(path.clone());

    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    config
        .set_value("editor.workbench.default_layout", json!({"page": "main"}))
        .unwrap();
    config.flush(Duration::from_secs(2)).unwrap();

    let second_runtime = CoreRuntime::new();
    second_runtime.register_module(module_descriptor()).unwrap();
    second_runtime
        .activate_module(FOUNDATION_MODULE_NAME)
        .unwrap();
    let second_resolver = ManagerResolver::new(second_runtime.handle());
    let second_config = second_resolver
        .resolve(second_resolver.config_handle().unwrap())
        .unwrap();

    assert_eq!(
        second_config.get_value("editor.workbench.default_layout"),
        Some(json!({"page": "main"}))
    );

    let _ = std::fs::remove_file(path);
}

#[cfg(feature = "animation")]
#[test]
fn animation_playback_settings_use_config_manager_persistence() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_animation_config_{unique}.json"));
    let _config_path_override = super::runtime::override_config_file_path_for_test(path.clone());
    let mut settings = AnimationPlaybackSettings::default();
    settings.enabled = false;

    {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime
            .register_module(animation_module_descriptor())
            .unwrap();
        runtime.activate_module(ANIMATION_MODULE_NAME).unwrap();

        let core = runtime.handle();
        let resolver = ManagerResolver::new(core.clone());
        let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
        let dirty_generation = config.persistence_report().dirty_generation;
        let animation = core
            .resolve_manager::<DefaultAnimationManager>(DEFAULT_ANIMATION_MANAGER_NAME)
            .unwrap();

        animation
            .store_playback_settings(settings.clone())
            .expect("animation playback settings should persist through ConfigManager");

        assert_eq!(animation.playback_settings(), settings);
        assert_eq!(
            config
                .get_value(ANIMATION_PLAYBACK_CONFIG_KEY)
                .and_then(|value| value.get("enabled").and_then(serde_json::Value::as_bool)),
            Some(false)
        );
        assert!(
            config.persistence_report().dirty_generation > dirty_generation,
            "animation persistence must advance the ConfigManager dirty generation"
        );
        config.flush(Duration::from_secs(2)).unwrap();
    }

    {
        let runtime = CoreRuntime::new();
        runtime.register_module(module_descriptor()).unwrap();
        runtime
            .register_module(animation_module_descriptor())
            .unwrap();
        runtime.activate_module(ANIMATION_MODULE_NAME).unwrap();

        let animation = runtime
            .resolve_manager::<DefaultAnimationManager>(DEFAULT_ANIMATION_MANAGER_NAME)
            .unwrap();
        assert_eq!(animation.playback_settings(), settings);
    }

    let _ = std::fs::remove_file(path);
}

#[test]
fn public_manager_services_use_foundation_module_registry_names() {
    assert_eq!(
        CONFIG_MANAGER_NAME,
        "FoundationModule.Manager.ConfigManager"
    );
}

#[test]
fn foundation_config_paths_isolate_concurrent_runtime_persistence() {
    use std::sync::mpsc;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-foundation-config-path-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let mut releases = Vec::new();
    let mut workers = Vec::new();
    for owner in ["left", "right"] {
        let path = root.join(format!("{owner}.json"));
        std::fs::write(
            &path,
            serde_json::to_vec(&json!({"fixture.owner": owner})).unwrap(),
        )
        .unwrap();
        let (release_tx, release_rx) = mpsc::channel();
        releases.push(release_tx);
        let ready_tx = ready_tx.clone();
        workers.push(std::thread::spawn(move || {
            let weak = {
                let runtime = CoreRuntime::new();
                let mut descriptor = module_descriptor();
                crate::foundation::bind_config_file_path(&mut descriptor, path.clone()).unwrap();
                runtime.register_module(descriptor).unwrap();
                runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
                let resolver = ManagerResolver::new(runtime.handle());
                let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
                assert_eq!(config.get_value("fixture.owner"), Some(json!(owner)));
                ready_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                config.set_value("fixture.written", json!(owner)).unwrap();
                config.flush(Duration::from_secs(2)).unwrap();
                let disk: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                assert_eq!(disk["fixture.owner"], json!(owner));
                assert_eq!(disk["fixture.written"], json!(owner));
                runtime.weak()
            };
            assert!(
                weak.upgrade().is_none(),
                "path factory must not retain CoreRuntime"
            );
        }));
    }
    drop(ready_tx);
    for _ in 0..2 {
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    for release in releases {
        release.send(()).unwrap();
    }
    for worker in workers {
        worker.join().unwrap();
    }
    for owner in ["left", "right"] {
        let runtime = CoreRuntime::new();
        let mut descriptor = module_descriptor();
        crate::foundation::bind_config_file_path(
            &mut descriptor,
            root.join(format!("{owner}.json")),
        )
        .unwrap();
        runtime.register_module(descriptor).unwrap();
        runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        let resolver = ManagerResolver::new(runtime.handle());
        let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
        assert_eq!(config.get_value("fixture.written"), Some(json!(owner)));
    }
    // This uniquely created fixture owns both files; no shared directory is removed.
    for owner in ["left", "right"] {
        std::fs::remove_file(root.join(format!("{owner}.json"))).unwrap();
    }
    std::fs::remove_dir(root).unwrap();
}

#[test]
fn foundation_config_path_binding_rejects_invalid_authority() {
    use std::path::PathBuf;
    use std::sync::Arc;

    let mut descriptor = module_descriptor();
    let original_factory = Arc::clone(&descriptor.managers[0].factory);
    for path in [PathBuf::new(), PathBuf::from("relative-config.json")] {
        assert!(crate::foundation::bind_config_file_path(&mut descriptor, path.clone()).is_err());
        assert!(Arc::ptr_eq(
            &original_factory,
            &descriptor.managers[0].factory
        ));
        assert!(crate::foundation::DefaultConfigManager::new_with_file_path(
            &CoreRuntime::new().handle(),
            path
        )
        .is_err());
    }
    let absolute = std::env::temp_dir().join("zircon-invalid-foundation-descriptor.json");
    let mut other = crate::core::ModuleDescriptor::new("OtherModule", "not Foundation");
    assert!(crate::foundation::bind_config_file_path(&mut other, absolute.clone()).is_err());
    let mut missing = module_descriptor();
    missing.managers.clear();
    assert!(crate::foundation::bind_config_file_path(&mut missing, absolute.clone()).is_err());
    let mut duplicate = module_descriptor();
    duplicate.managers.push(duplicate.managers[0].clone());
    assert!(crate::foundation::bind_config_file_path(&mut duplicate, absolute).is_err());
}
