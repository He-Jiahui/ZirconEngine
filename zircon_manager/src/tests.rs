use crate::{
    CONFIG_MANAGER_NAME, EVENT_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME,
    RENDER_FRAMEWORK_NAME, RENDERING_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

#[test]
fn zircon_manager_stays_resolver_only_after_framework_extraction() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = include_str!("lib.rs");
    let resolver_source = include_str!("resolver.rs");

    assert!(
        !crate_root.join("src/traits.rs").exists(),
        "zircon_manager should delete src/traits.rs after framework trait extraction"
    );
    assert!(
        !crate_root.join("src/records").exists(),
        "zircon_manager should delete src/records after framework DTO extraction"
    );
    assert!(
        !lib_source.contains("pub trait "),
        "zircon_manager lib.rs should not define or re-export traits after framework extraction"
    );
    assert!(
        resolver_source.contains("zircon_framework"),
        "zircon_manager resolver should source all manager traits from zircon_framework"
    );
    assert!(
        !resolver_source.contains("zircon_input_protocol"),
        "zircon_manager resolver should not depend on zircon_input_protocol after framework migration"
    );
    assert!(
        !resolver_source.contains("zircon_scene_protocol"),
        "zircon_manager resolver should not depend on zircon_scene_protocol after framework migration"
    );
}

#[test]
fn manager_public_service_names_match_converged_runtime_architecture() {
    assert_eq!(RESOURCE_MANAGER_NAME, "AssetModule.Manager.ResourceManager");
    assert_eq!(INPUT_MANAGER_NAME, "InputModule.Manager.InputManager");
    assert_eq!(
        CONFIG_MANAGER_NAME,
        "FoundationModule.Manager.ConfigManager"
    );
    assert_eq!(EVENT_MANAGER_NAME, "FoundationModule.Manager.EventManager");
    assert_eq!(
        RENDERING_MANAGER_NAME,
        "GraphicsModule.Manager.RenderingManager"
    );
    assert_eq!(
        RENDER_FRAMEWORK_NAME,
        "GraphicsModule.Manager.RenderFramework"
    );
    assert_eq!(LEVEL_MANAGER_NAME, "SceneModule.Manager.LevelManager");
}

#[test]
fn manager_public_surface_excludes_removed_asset_editor_and_protocol_apis() {
    let lib_source = include_str!("lib.rs");
    let resolver_source = include_str!("resolver.rs");
    let service_names_source = include_str!("service_names.rs");

    for forbidden in [
        "AssetManager",
        "EditorAssetManager",
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
        "LevelSummary",
        "WorldHandle",
        "RenderServer",
        "RenderServerError",
        "RENDER_SERVER_NAME",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_manager lib.rs should not expose {forbidden} after convergence refactor"
        );
        assert!(
            !resolver_source.contains(forbidden),
            "zircon_manager resolver should not mention {forbidden} after convergence refactor"
        );
        assert!(
            !service_names_source.contains(forbidden),
            "zircon_manager service names should not mention {forbidden} after convergence refactor"
        );
    }
}
