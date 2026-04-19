#[test]
fn level_protocol_types_live_in_scene_framework_layer() {
    let scene_lib_source = include_str!("../lib.rs");
    let render_extract_source = include_str!("../render_extract.rs");
    let runtime_level_system_source =
        include_str!("../../../zircon_runtime/src/scene/level_system.rs");
    let runtime_default_level_manager_source =
        include_str!("../../../zircon_runtime/src/scene/module/default_level_manager.rs");
    let runtime_level_manager_facade_source =
        include_str!("../../../zircon_runtime/src/scene/module/level_manager_facade.rs");
    let runtime_level_manager_lifecycle_source =
        include_str!("../../../zircon_runtime/src/scene/module/level_manager_lifecycle.rs");
    let runtime_level_manager_project_io_source =
        include_str!("../../../zircon_runtime/src/scene/module/level_manager_project_io.rs");
    let manager_lib_source = include_str!("../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../zircon_manager/src/resolver.rs");

    for forbidden in ["LevelSummary", "WorldHandle"] {
        assert!(
            !scene_lib_source.contains(forbidden),
            "zircon_scene root should not re-export framework-owned {forbidden}"
        );
    }

    for source in [
        render_extract_source,
        runtime_level_system_source,
        runtime_default_level_manager_source,
        runtime_level_manager_facade_source,
        runtime_level_manager_lifecycle_source,
        runtime_level_manager_project_io_source,
    ] {
        assert!(
            !source.contains("use zircon_manager::WorldHandle"),
            "scene/runtime files should not source WorldHandle from zircon_manager after scene protocol migration"
        );
        assert!(
            !source.contains("zircon_manager::WorldHandle"),
            "scene/runtime files should not use zircon_manager::WorldHandle after scene protocol migration"
        );
        assert!(
            !source.contains("use zircon_manager::{LevelManager as LevelManagerFacade, LevelSummary, WorldHandle};"),
            "scene/runtime manager facade should not source LevelSummary or WorldHandle from zircon_manager after framework migration"
        );
    }

    for forbidden in ["WorldHandle", "LevelSummary"] {
        assert!(
            !manager_lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after framework migration"
        );
    }

    assert!(
        manager_resolver_source.contains("zircon_framework"),
        "zircon_manager resolver should depend on zircon_framework for level protocol types"
    );
    assert!(
        !manager_resolver_source.contains("zircon_scene_protocol"),
        "zircon_manager resolver should not depend on zircon_scene_protocol after framework migration"
    );
}

#[test]
fn scene_root_no_longer_owns_runtime_orchestration_files() {
    let scene_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_source = std::fs::read_to_string(scene_root.join("src/lib.rs")).unwrap_or_default();

    assert!(
        !lib_source.contains("DefaultLevelManager"),
        "zircon_scene root should stop exporting DefaultLevelManager"
    );
    assert!(
        !lib_source.contains("WorldDriver"),
        "zircon_scene root should stop exporting WorldDriver"
    );
    assert!(
        !scene_root.join("src/level_system.rs").exists(),
        "LevelSystem should move out of zircon_scene"
    );
    assert!(
        !scene_root
            .join("src/module/default_level_manager.rs")
            .exists(),
        "scene module manager implementation should move to zircon_runtime"
    );
    assert!(
        !scene_root.join("src/module/world_driver.rs").exists(),
        "scene module driver implementation should move to zircon_runtime"
    );
}

#[test]
fn scene_root_promotes_world_component_and_semantics_namespaces() {
    let lib_source = include_str!("../lib.rs");

    for required in [
        "pub mod components;",
        "pub mod semantics;",
        "pub mod serializer;",
        "pub mod world;",
        "pub type Scene = world::World;",
    ] {
        assert!(
            lib_source.contains(required),
            "zircon_scene root should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use components::{",
        "pub use semantics::{ComponentData, EntityIdentity};",
        "pub use serializer::SceneAssetSerializer;",
        "pub use world::{SceneProjectError, World};",
        "pub use components::default_render_layer_mask;",
        "pub use components::Mobility;",
        "pub use components::NodeKind;",
        "pub use components::NodeRecord;",
        "pub use components::Schedule;",
        "pub use components::SystemStage;",
        "pub use world::World;",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_scene root should stop flattening namespace-owned surface `{forbidden}`"
        );
    }
}
