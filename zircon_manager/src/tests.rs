use crate::{CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME};

#[test]
fn zircon_manager_stays_contract_only_and_does_not_host_builtin_modules() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");

    assert!(
        !crate_root.join("src/builtins").exists(),
        "zircon_manager should not keep builtins/ after manager implementation split"
    );
    assert!(
        !lib_source.contains("mod builtins;"),
        "zircon_manager lib.rs should not declare builtins after manager implementation split"
    );
    assert!(
        !lib_source.contains("pub struct ManagerModule"),
        "zircon_manager should not own a concrete module type after manager implementation split"
    );
    assert!(
        !lib_source.contains("module_descriptor"),
        "zircon_manager should not export module_descriptor after manager implementation split"
    );
    assert!(
        !lib_source.contains("DefaultConfigManager"),
        "zircon_manager should not expose concrete config manager implementations"
    );
    assert!(
        !lib_source.contains("DefaultEventManager"),
        "zircon_manager should not expose concrete event manager implementations"
    );
}

#[test]
fn public_manager_services_use_foundation_module_registry_names() {
    assert_eq!(
        CONFIG_MANAGER_NAME,
        "FoundationModule.Manager.ConfigManager"
    );
    assert_eq!(EVENT_MANAGER_NAME, "FoundationModule.Manager.EventManager");
}

#[test]
fn manager_public_surface_excludes_editor_asset_api() {
    let lib_source = include_str!("lib.rs");
    let traits_source = include_str!("traits.rs");
    let resolver_source = include_str!("resolver.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let service_names_source = include_str!("service_names.rs");

    assert!(
        !lib_source.contains("EditorAssetManager"),
        "zircon_manager lib.rs should not re-export EditorAssetManager after boundary cleanup"
    );
    assert!(
        !traits_source.contains("pub trait EditorAssetManager"),
        "zircon_manager traits should not define EditorAssetManager after boundary cleanup"
    );
    assert!(
        !resolver_source.contains("resolve_editor_asset_manager"),
        "zircon_manager resolver should not expose resolve_editor_asset_manager after boundary cleanup"
    );
    assert!(
        !resolver_source.contains("EditorAssetManagerHandle"),
        "zircon_manager resolver should not expose EditorAssetManagerHandle after boundary cleanup"
    );
    assert!(
        !records_mod_source.contains("editor_asset"),
        "zircon_manager records mod should not reference editor_asset records after boundary cleanup"
    );
    assert!(
        !service_names_source.contains("EDITOR_ASSET_MANAGER_NAME"),
        "zircon_manager service names should not define EDITOR_ASSET_MANAGER_NAME after boundary cleanup"
    );
}

#[test]
fn manager_public_surface_excludes_asset_manager_api() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let handles_source =
        std::fs::read_to_string(crate_root.join("src/handles.rs")).unwrap_or_default();
    let traits_source = include_str!("traits.rs");
    let resolver_source = include_str!("resolver.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let service_names_source = include_str!("service_names.rs");
    let asset_records_path = crate_root.join("src/records/asset.rs");
    let project_records_path = crate_root.join("src/records/project.rs");

    for forbidden in [
        "AssetManager",
        "AssetManagerHandle",
        "resolve_asset_manager",
        "ASSET_MANAGER_NAME",
        "AssetPipelineInfo",
        "AssetStatusRecord",
        "ProjectInfo",
        "AssetChangeRecord",
        "AssetChangeKind",
        "AssetRecordKind",
        "PreviewStateRecord",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after asset boundary cleanup"
        );
        assert!(
            !traits_source.contains(forbidden),
            "zircon_manager traits should not mention {forbidden} after asset boundary cleanup"
        );
        assert!(
            !resolver_source.contains(forbidden),
            "zircon_manager resolver should not mention {forbidden} after asset boundary cleanup"
        );
        assert!(
            !records_mod_source.contains(forbidden),
            "zircon_manager records mod should not mention {forbidden} after asset boundary cleanup"
        );
        assert!(
            !service_names_source.contains(forbidden),
            "zircon_manager service names should not mention {forbidden} after asset boundary cleanup"
        );
    }

    assert!(
        !handles_source.contains("define_handle!(AssetHandle);"),
        "zircon_manager handles should not define AssetHandle after asset boundary cleanup"
    );
    assert!(
        !asset_records_path.exists(),
        "zircon_manager should delete src/records/asset.rs after asset boundary cleanup"
    );
    assert!(
        !project_records_path.exists(),
        "zircon_manager should delete src/records/project.rs after asset boundary cleanup"
    );
}

#[test]
fn manager_public_surface_excludes_input_protocol_types() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let input_records_path = crate_root.join("src/records/input.rs");

    for forbidden in [
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after input protocol migration"
        );
        assert!(
            !records_mod_source.contains(forbidden),
            "zircon_manager records mod should not re-export {forbidden} after input protocol migration"
        );
    }

    assert!(
        !records_mod_source.contains("mod input;"),
        "zircon_manager records mod should not declare input records after input protocol migration"
    );
    assert!(
        !input_records_path.exists(),
        "zircon_manager should delete src/records/input.rs after input protocol migration"
    );
}

#[test]
fn manager_public_surface_excludes_vm_plugin_protocol_types() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let handles_source =
        std::fs::read_to_string(crate_root.join("src/handles.rs")).unwrap_or_default();
    let capability_set_path = crate_root.join("src/records/capability_set.rs");

    for forbidden in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after vm plugin boundary cleanup"
        );
    }

    assert!(
        !records_mod_source.contains("CapabilitySet"),
        "zircon_manager records mod should not re-export CapabilitySet after vm plugin boundary cleanup"
    );
    assert!(
        !records_mod_source.contains("mod capability_set;"),
        "zircon_manager records mod should not declare capability_set after vm plugin boundary cleanup"
    );
    assert!(
        !handles_source.contains("define_handle!(PluginSlotId);"),
        "zircon_manager handles should not define PluginSlotId after vm plugin boundary cleanup"
    );
    assert!(
        !handles_source.contains("define_handle!(HostHandle);"),
        "zircon_manager handles should not define HostHandle after vm plugin boundary cleanup"
    );
    assert!(
        !capability_set_path.exists(),
        "zircon_manager should delete src/records/capability_set.rs after vm plugin boundary cleanup"
    );
}

#[test]
fn manager_public_surface_excludes_scene_protocol_types() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let traits_source = include_str!("traits.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let handles_path = crate_root.join("src/handles.rs");
    let level_records_path = crate_root.join("src/records/level.rs");

    for forbidden in ["WorldHandle", "LevelSummary"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after scene protocol migration"
        );
        assert!(
            !records_mod_source.contains(forbidden),
            "zircon_manager records mod should not re-export {forbidden} after scene protocol migration"
        );
    }

    assert!(
        traits_source.contains("zircon_scene_protocol"),
        "zircon_manager traits should source scene protocol types from zircon_scene_protocol"
    );
    assert!(
        !handles_path.exists(),
        "zircon_manager should delete src/handles.rs after scene protocol migration"
    );
    assert!(
        !level_records_path.exists(),
        "zircon_manager should delete src/records/level.rs after scene protocol migration"
    );
}

#[test]
fn manager_public_surface_excludes_resource_state_mirror() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let traits_source = include_str!("traits.rs");
    let resource_records_path = crate_root.join("src/records/resource.rs");

    assert!(
        !lib_source.contains("ResourceStateRecord"),
        "zircon_manager lib.rs should not re-export ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !records_mod_source.contains("ResourceStateRecord"),
        "zircon_manager records mod should not re-export ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !resource_records_path.exists(),
        "zircon_manager should delete src/records/resource.rs after canonical ResourceState migration"
    );
    assert!(
        !traits_source.contains("ResourceStateRecord"),
        "zircon_manager traits should refer to zircon_resource::ResourceState instead of ResourceStateRecord"
    );
}

#[test]
fn manager_public_surface_excludes_resource_status_wrapper() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let traits_source = include_str!("traits.rs");
    let resource_records_path = crate_root.join("src/records/resource.rs");

    assert!(
        !lib_source.contains("ResourceStatusRecord"),
        "zircon_manager lib.rs should not re-export ResourceStatusRecord after canonical ResourceRecord migration"
    );
    assert!(
        !records_mod_source.contains("ResourceStatusRecord"),
        "zircon_manager records mod should not re-export ResourceStatusRecord after canonical ResourceRecord migration"
    );
    assert!(
        !resource_records_path.exists(),
        "zircon_manager should delete src/records/resource.rs after canonical ResourceRecord migration"
    );
    assert!(
        !traits_source.contains("ResourceStatusRecord"),
        "zircon_manager traits should refer to zircon_resource::ResourceRecord instead of ResourceStatusRecord"
    );
    assert!(
        traits_source.contains("ResourceRecord"),
        "zircon_manager traits should expose zircon_resource::ResourceRecord after canonical ResourceRecord migration"
    );
}

#[test]
fn manager_public_surface_excludes_resource_change_wrapper() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let traits_source = include_str!("traits.rs");
    let resource_records_path = crate_root.join("src/records/resource.rs");

    for forbidden in ["ResourceChangeKind", "ResourceChangeRecord"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after canonical ResourceEvent migration"
        );
        assert!(
            !records_mod_source.contains(forbidden),
            "zircon_manager records mod should not re-export {forbidden} after canonical ResourceEvent migration"
        );
        assert!(
            !traits_source.contains(forbidden),
            "zircon_manager traits should not mention {forbidden} after canonical ResourceEvent migration"
        );
    }

    assert!(
        !records_mod_source.contains("mod resource;"),
        "zircon_manager records mod should not declare resource records after canonical ResourceEvent migration"
    );
    assert!(
        !resource_records_path.exists(),
        "zircon_manager should delete src/records/resource.rs after canonical ResourceEvent migration"
    );
    assert!(
        traits_source.contains("ResourceEvent"),
        "zircon_manager traits should expose zircon_resource::ResourceEvent after canonical ResourceEvent migration"
    );
}
