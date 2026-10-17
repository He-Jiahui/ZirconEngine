use std::time::Duration;

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
    let fixture = ManagedConfigFixture::new("config_manager_roundtrip_works_through_resolver");
    let runtime = CoreRuntime::new();
    let mut descriptor = module_descriptor();
    crate::foundation::bind_config_file_path(&mut descriptor, fixture.config_path()).unwrap();
    runtime.register_module(descriptor).unwrap();
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
    let fixture =
        ManagedConfigFixture::new("versioned_manager_handle_rejects_the_unloaded_generation");
    let runtime = CoreRuntime::new();
    let mut descriptor = module_descriptor();
    crate::foundation::bind_config_file_path(&mut descriptor, fixture.config_path()).unwrap();
    runtime.register_module(descriptor).unwrap();
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
    let fixture =
        ManagedConfigFixture::new("foundation_registry_services_do_not_retain_the_runtime_root");
    let runtime = CoreRuntime::new();
    let mut descriptor = module_descriptor();
    crate::foundation::bind_config_file_path(&mut descriptor, fixture.config_path()).unwrap();
    runtime.register_module(descriptor).unwrap();
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
    let fixture = ManagedConfigFixture::new("public-persistence");
    let path = fixture.config_path();
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
    // Close the actual final worker before admitting a new independent owner.
    drop(config);
    drop(resolver);
    drop(runtime);

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
    drop(second_config);
    drop(second_resolver);
    drop(second_runtime);
}

#[test]
fn foundation_failed_config_activation_preserves_the_existing_path_owner() {
    let fixture = ManagedConfigFixture::new("failed-activation");
    let path = fixture.config_path();
    let (existing_weak, failed_weak) = {
        let existing = CoreRuntime::new();
        let mut descriptor = module_descriptor();
        crate::foundation::bind_config_file_path(&mut descriptor, path.clone()).unwrap();
        existing.register_module(descriptor).unwrap();
        existing.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        let resolver = ManagerResolver::new(existing.handle());
        let retained_handle = resolver.config_handle().unwrap();
        let config = resolver.resolve(retained_handle.clone()).unwrap();
        config.set_value("fixture.owner", json!("before")).unwrap();
        config.flush(Duration::from_secs(2)).unwrap();
        std::fs::write(&path, b"{malformed startup json").unwrap();

        let failed = CoreRuntime::new();
        let mut descriptor = module_descriptor();
        crate::foundation::bind_config_file_path(&mut descriptor, path.clone()).unwrap();
        failed.register_module(descriptor).unwrap();
        let error = failed.activate_module(FOUNDATION_MODULE_NAME).unwrap_err();
        assert!(matches!(&error, crate::core::CoreError::Initialization(..)));
        // Exclusive ownership now rejects before parsing the malformed replacement JSON.
        // Unowned prepare Err/panic and malformed-startup cases retain that separate coverage.
        assert!(error.to_string().contains("config owner conflict"));
        assert!(error.to_string().contains(path.to_str().unwrap()));
        let failed_resolver = ManagerResolver::new(failed.handle());
        assert!(failed_resolver.config_handle().is_err());

        let retained = resolver.resolve(retained_handle).unwrap();
        retained
            .set_value("fixture.owner", json!("after failed activation"))
            .unwrap();
        retained.flush(Duration::from_secs(2)).unwrap();
        let disk: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(disk["fixture.owner"], json!("after failed activation"));
        (existing.weak(), failed.weak())
    };
    // No runtime, resolver, service, handle or worker owner escapes this block.
    // Their real final worker Drop joins before the fixture guard removes its unique root.
    assert!(existing_weak.upgrade().is_none());
    assert!(failed_weak.upgrade().is_none());
    let entries = std::fs::read_dir(fixture.root.operation_path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries, vec![path]);
}

#[cfg(feature = "animation")]
#[test]
fn animation_playback_settings_use_config_manager_persistence() {
    let fixture = ManagedConfigFixture::new("animation-config");
    let path = fixture.config_path();
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

    let fixture = ManagedConfigFixture::new("concurrent-host-files");
    let root = fixture.root.operation_path().to_path_buf();
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
    let ready = (0..2)
        .map(|_| ready_rx.recv_timeout(Duration::from_secs(5)))
        .collect::<Vec<_>>();
    let released = releases
        .into_iter()
        .map(|release| release.send(()))
        .collect::<Vec<_>>();
    let joined = workers
        .into_iter()
        .map(|worker| worker.join())
        .collect::<Vec<_>>();
    // Release and join every worker before an observation failure can unwind the fixture.
    for result in ready {
        result.unwrap();
    }
    for result in released {
        result.unwrap();
    }
    for result in joined {
        result.unwrap();
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
    // All worker threads and reopened runtimes closed before the fixture guard cleans both files.
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
    let absolute =
        managed_test_binary_directory().join("zircon-invalid-foundation-descriptor.json");
    let mut other = crate::core::ModuleDescriptor::new("OtherModule", "not Foundation");
    assert!(crate::foundation::bind_config_file_path(&mut other, absolute.clone()).is_err());
    let mut missing = module_descriptor();
    missing.managers.clear();
    assert!(crate::foundation::bind_config_file_path(&mut missing, absolute.clone()).is_err());
    let mut duplicate = module_descriptor();
    duplicate.managers.push(duplicate.managers[0].clone());
    assert!(crate::foundation::bind_config_file_path(&mut duplicate, absolute).is_err());
}

fn managed_test_binary_directory() -> std::path::PathBuf {
    use crate::asset::ProjectPaths;
    let approved = ProjectPaths::resolve_existing("E:/cargo-targets/zircon-local")
        .expect("the managed E drive fixture root must already exist");
    let display = ProjectPaths::display_path(approved.operation_path());
    assert!(
        display
            .to_str()
            .expect("managed root must be Unicode")
            .replace('\\', "/")
            .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
        "the approved root must resolve to the physical E drive root"
    );
    let approved = ProjectPaths::resolve_identity(approved.operation_path()).unwrap();
    let executable = std::env::current_exe().unwrap();
    let binary = ProjectPaths::resolve_identity(executable.parent().unwrap()).unwrap();
    assert!(
        binary.is_within(&approved),
        "test binary must physically live under the managed E root"
    );
    binary.operation_path().to_path_buf()
}

struct ManagedConfigFixture {
    root: crate::asset::project::ResolvedProjectPathIdentity,
}

impl ManagedConfigFixture {
    fn new(label: &str) -> Self {
        use crate::asset::ProjectPaths;
        static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let approved = ProjectPaths::resolve_existing("E:/cargo-targets/zircon-local")
            .expect("the managed E drive fixture root must already exist");
        let display = ProjectPaths::display_path(approved.operation_path());
        assert!(
            display
                .to_str()
                .expect("managed root must be Unicode")
                .replace('\\', "/")
                .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
            "the approved root must resolve to the physical E drive root"
        );
        let approved = ProjectPaths::resolve_identity(approved.operation_path()).unwrap();
        let executable = std::env::current_exe().unwrap();
        let binary = ProjectPaths::resolve_identity(executable.parent().unwrap()).unwrap();
        assert!(
            binary.is_within(&approved),
            "test binary must physically live under the managed E root"
        );
        let id = NEXT_FIXTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(binary.operation_path().join(format!(
            "runtime55-{label}-{}-{nanos}-{id}",
            std::process::id()
        )))
        .unwrap();
        assert!(root.is_within(&binary));
        // Every authority check precedes mkdir/write. create_dir never adopts an existing root.
        std::fs::create_dir(root.operation_path()).unwrap();
        let fixture = Self { root };
        assert_eq!(
            ProjectPaths::resolve_identity(fixture.root.operation_path()).unwrap(),
            fixture.root
        );
        fixture
    }

    fn config_path(&self) -> std::path::PathBuf {
        self.root.operation_path().join("config.json")
    }
}

impl Drop for ManagedConfigFixture {
    fn drop(&mut self) {
        // The declaring scope/wrapper closes all runtimes, handles and worker owners first.
        let current =
            crate::asset::ProjectPaths::resolve_identity(self.root.operation_path()).unwrap();
        assert_eq!(
            current, self.root,
            "cleanup must retain this fixture's physical identity"
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}

#[test]
fn foundation_config_owner_lease_survives_runtime_drop_until_the_last_manager_closes() {
    let fixture = ManagedConfigFixture::new("public-owner-last-worker");
    let path = fixture.config_path();
    {
        let runtime = CoreRuntime::new();
        let weak = runtime.weak();
        let manager = crate::foundation::DefaultConfigManager::new_with_file_path(
            &runtime.handle(),
            path.clone(),
        )
        .unwrap();
        let retained = manager.clone();
        manager.set_value("fixture.retained", json!(true)).unwrap();
        drop(runtime);
        assert!(
            weak.upgrade().is_none(),
            "CoreWeak death must not release a live writer lease"
        );
        retained.flush(Duration::from_secs(2)).unwrap();

        let next = CoreRuntime::new();
        let mut descriptor = module_descriptor();
        crate::foundation::bind_config_file_path(&mut descriptor, path.clone()).unwrap();
        next.register_module(descriptor).unwrap();
        let error = next.activate_module(FOUNDATION_MODULE_NAME).unwrap_err();
        assert!(matches!(&error, crate::core::CoreError::Initialization(..)));
        assert!(error.to_string().contains("config owner conflict"));
        drop(manager);
        let error = next.activate_module(FOUNDATION_MODULE_NAME).unwrap_err();
        assert!(error.to_string().contains("config owner conflict"));
        // DefaultConfigManager clones share one worker. This real last-owner Drop joins it.
        drop(retained);
        next.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        let resolver = ManagerResolver::new(next.handle());
        let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
        assert_eq!(config.get_value("fixture.retained"), Some(json!(true)));
        config.flush(Duration::from_secs(2)).unwrap();
    }
    // The next runtime, resolver, service and both manager clones have all closed above.
}
