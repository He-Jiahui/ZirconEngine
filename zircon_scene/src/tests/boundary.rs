#[test]
fn level_protocol_types_live_in_scene_protocol_layer() {
    let scene_lib_source = include_str!("../lib.rs");
    let level_system_source = include_str!("../level_system.rs");
    let render_extract_source = include_str!("../render_extract.rs");
    let default_level_manager_source = include_str!("../module/default_level_manager.rs");
    let level_manager_facade_source = include_str!("../module/level_manager_facade.rs");
    let level_manager_lifecycle_source = include_str!("../module/level_manager_lifecycle.rs");
    let level_manager_project_io_source = include_str!("../module/level_manager_project_io.rs");
    let manager_lib_source = include_str!("../../../zircon_manager/src/lib.rs");
    let manager_traits_source = include_str!("../../../zircon_manager/src/traits.rs");
    let manager_records_mod_source = include_str!("../../../zircon_manager/src/records/mod.rs");
    let manager_handles_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../zircon_manager/src/handles.rs");
    let manager_level_records_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_manager/src/records/level.rs");

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
            "scene manager facade should not source LevelSummary or WorldHandle from zircon_manager after scene protocol migration"
        );
    }

    for forbidden in ["WorldHandle", "LevelSummary"] {
        assert!(
            !manager_lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after scene protocol migration"
        );
        assert!(
            !manager_records_mod_source.contains(forbidden),
            "zircon_manager records mod should not re-export {forbidden} after scene protocol migration"
        );
    }

    assert!(
        manager_traits_source.contains("zircon_scene_protocol"),
        "zircon_manager traits should depend on zircon_scene_protocol for level protocol types"
    );
    assert!(
        !manager_handles_path.exists(),
        "zircon_manager should delete src/handles.rs after scene protocol migration"
    );
    assert!(
        !manager_level_records_path.exists(),
        "zircon_manager should delete src/records/level.rs after scene protocol migration"
    );
}
