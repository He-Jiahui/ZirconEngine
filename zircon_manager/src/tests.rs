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
fn manager_public_surface_excludes_asset_display_taxonomy() {
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let asset_records_source = include_str!("records/asset.rs");

    assert!(
        !lib_source.contains("AssetRecordKind"),
        "zircon_manager lib.rs should not re-export AssetRecordKind after canonical ResourceKind migration"
    );
    assert!(
        !lib_source.contains("PreviewStateRecord"),
        "zircon_manager lib.rs should not re-export PreviewStateRecord after asset-owned preview migration"
    );
    assert!(
        !records_mod_source.contains("AssetRecordKind"),
        "zircon_manager records mod should not re-export AssetRecordKind after canonical ResourceKind migration"
    );
    assert!(
        !records_mod_source.contains("PreviewStateRecord"),
        "zircon_manager records mod should not re-export PreviewStateRecord after asset-owned preview migration"
    );
    assert!(
        !asset_records_source.contains("pub enum AssetRecordKind"),
        "zircon_manager asset records should not define AssetRecordKind after canonical ResourceKind migration"
    );
    assert!(
        !asset_records_source.contains("pub enum PreviewStateRecord"),
        "zircon_manager asset records should not define PreviewStateRecord after asset-owned preview migration"
    );
}

#[test]
fn manager_public_surface_excludes_input_protocol_types() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let input_records_path = crate_root.join("src/records/input.rs");

    for forbidden in ["InputButton", "InputEvent", "InputEventRecord", "InputSnapshot"] {
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
    let handles_source = include_str!("handles.rs");
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
fn manager_public_surface_excludes_resource_state_mirror() {
    let lib_source = include_str!("lib.rs");
    let records_mod_source = include_str!("records/mod.rs");
    let resource_records_source = include_str!("records/resource.rs");
    let traits_source = include_str!("traits.rs");

    assert!(
        !lib_source.contains("ResourceStateRecord"),
        "zircon_manager lib.rs should not re-export ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !records_mod_source.contains("ResourceStateRecord"),
        "zircon_manager records mod should not re-export ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !resource_records_source.contains("pub enum ResourceStateRecord"),
        "zircon_manager resource records should not define ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !traits_source.contains("ResourceStateRecord"),
        "zircon_manager traits should refer to zircon_resource::ResourceState instead of ResourceStateRecord"
    );
}
