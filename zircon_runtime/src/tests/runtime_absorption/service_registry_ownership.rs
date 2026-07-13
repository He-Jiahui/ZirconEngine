use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime workspace parent")
        .to_path_buf()
}

#[test]
fn registry_owned_services_store_only_weak_runtime_back_references() {
    let root = workspace_root();
    let service_fields = [
        (
            "zircon_runtime/src/foundation/runtime/config_manager.rs",
            "core: CoreHandle",
            "core: CoreWeak",
        ),
        (
            "zircon_runtime/src/foundation/runtime/event_manager.rs",
            "core: CoreHandle",
            "core: CoreWeak",
        ),
        (
            "zircon_runtime/src/animation/manager/mod.rs",
            "core: Option<CoreHandle>",
            "core: Option<CoreWeak>",
        ),
        (
            "zircon_editor/src/ui/host/editor_ui_host.rs",
            "core: CoreHandle",
            "core: CoreWeak",
        ),
        (
            "zircon_plugins/animation/runtime/src/manager.rs",
            "core: Option<CoreHandle>",
            "core: Option<CoreWeak>",
        ),
        (
            "zircon_plugins/physics/runtime/src/manager.rs",
            "core: Arc<Mutex<Option<CoreHandle>>>",
            "core: Arc<Mutex<Option<CoreWeak>>>",
        ),
        (
            "zircon_plugins/sound/runtime/src/service_types/manager_state.rs",
            "core: Option<CoreHandle>",
            "core: Option<CoreWeak>",
        ),
    ];

    for (relative_path, forbidden, required) in service_fields {
        let source = std::fs::read_to_string(root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        assert!(
            !source.contains(forbidden),
            "registry-owned service {relative_path} must not store strong runtime field `{forbidden}`"
        );
        assert!(
            source.contains(required),
            "registry-owned service {relative_path} must store weak runtime field `{required}`"
        );
    }

    let factory_borrow_sites = [
        (
            "zircon_runtime/src/foundation/module.rs",
            "DefaultConfigManager::new(core)",
        ),
        (
            "zircon_runtime/src/foundation/module.rs",
            "DefaultEventManager::new(core)",
        ),
        (
            "zircon_runtime/src/animation/module.rs",
            "DefaultAnimationManager::new(Some(core))",
        ),
        (
            "zircon_editor/src/ui/host/module.rs",
            "EditorManager::new(core)",
        ),
        (
            "zircon_plugins/animation/runtime/src/module.rs",
            "DefaultAnimationManager::new(Some(core))",
        ),
        (
            "zircon_plugins/physics/runtime/src/module.rs",
            "manager.attach_core(core)",
        ),
        (
            "zircon_plugins/sound/runtime/src/module.rs",
            "DefaultSoundManager::new(Some(core))",
        ),
    ];

    for (relative_path, borrowed_factory_call) in factory_borrow_sites {
        let source = std::fs::read_to_string(root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        assert!(
            source.contains(borrowed_factory_call),
            "registry-owned service factory {relative_path} must keep borrowed construction `{borrowed_factory_call}`"
        );
        assert!(
            !source.contains("core.clone()"),
            "registry-owned service factory {relative_path} must not clone the Runtime root"
        );
    }
}
