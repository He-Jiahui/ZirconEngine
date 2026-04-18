#[test]
fn level_protocol_types_live_in_scene_framework_layer() {
    let scene_lib_source = include_str!("../lib.rs");
    let level_system_source = include_str!("../level_system.rs");
    let render_extract_source = include_str!("../render_extract.rs");
    let default_level_manager_source = include_str!("../module/default_level_manager.rs");
    let level_manager_facade_source = include_str!("../module/level_manager_facade.rs");
    let level_manager_lifecycle_source = include_str!("../module/level_manager_lifecycle.rs");
    let level_manager_project_io_source = include_str!("../module/level_manager_project_io.rs");
    let manager_lib_source = include_str!("../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../zircon_manager/src/resolver.rs");

    for required in ["LevelSummary", "WorldHandle"] {
        assert!(
            scene_lib_source.contains(required),
            "zircon_scene root should publicly re-export {required}"
        );
    }

    for source in [
        level_system_source,
        render_extract_source,
        default_level_manager_source,
        level_manager_facade_source,
        level_manager_lifecycle_source,
        level_manager_project_io_source,
    ] {
        assert!(
            !source.contains("use zircon_manager::WorldHandle"),
            "scene runtime files should not source WorldHandle from zircon_manager after scene protocol migration"
        );
        assert!(
            !source.contains("zircon_manager::WorldHandle"),
            "scene runtime files should not use zircon_manager::WorldHandle after scene protocol migration"
        );
        assert!(
            !source.contains("use zircon_manager::{LevelManager as LevelManagerFacade, LevelSummary, WorldHandle};"),
            "scene manager facade should not source LevelSummary or WorldHandle from zircon_manager after framework migration"
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
