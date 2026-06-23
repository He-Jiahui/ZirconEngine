#[test]
fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner() {
    fn occurrence_count(source: &str, needle: &str) -> usize {
        source.matches(needle).count()
    }

    let scene_mod = include_str!("../../../asset/assets/scene/mod.rs");
    let scene_lighting = include_str!("../../../asset/assets/scene/lighting.rs");
    let scene_physics = include_str!("../../../asset/assets/scene/physics.rs");
    let asset_assets_mod = include_str!("../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../asset/mod.rs");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let scene_doc = include_str!("../../../../../docs/zircon_runtime/asset/assets/scene.md");

    for module_decl in [
        "mod animation;",
        "mod asset;",
        "mod camera;",
        "mod defaults;",
        "mod entity;",
        "mod extensions;",
        "mod lighting;",
        "mod management;",
        "mod mesh;",
        "mod physics;",
        "mod post_process;",
        "mod transform;",
    ] {
        assert!(
            scene_mod.contains(module_decl),
            "scene/mod.rs should keep folder-backed declaration `{module_decl}`"
        );
    }

    assert_eq!(
        occurrence_count(scene_mod, "pub enum SceneMobilityAsset"),
        1,
        "scene/mod.rs should be the only SceneMobilityAsset enum owner"
    );
    assert!(
        !scene_physics.contains("SceneMobilityAsset"),
        "scene/physics.rs should not reintroduce a duplicate SceneMobilityAsset owner"
    );

    for export_anchor in [
        "pub use lighting::{",
        "SceneSpotLightAsset",
        "pub use scene::{",
        "SceneMobilityAsset",
    ] {
        assert!(
            scene_mod.contains(export_anchor)
                || asset_assets_mod.contains(export_anchor)
                || asset_mod.contains(export_anchor),
            "scene asset export chain should retain `{export_anchor}`"
        );
    }

    for spot_light_anchor in [
        "pub struct SceneSpotLightAsset",
        "pub direction: [Real; 3]",
        "pub outer_angle_radians: Real",
    ] {
        assert!(
            scene_lighting.contains(spot_light_anchor),
            "SceneSpotLightAsset should retain public field anchor `{spot_light_anchor}`"
        );
    }

    for doc_anchor in [
        "scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs",
        "SceneMobilityAsset",
        "SceneSpotLightAsset",
        "split-drift repair",
        "split_drift_static_passed_cargo_deferred_active_lanes",
        "scene asset split-drift repair",
    ] {
        assert!(
            runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || scene_doc.contains(doc_anchor),
            "Runtime 07 scene split docs should retain `{doc_anchor}`"
        );
    }
}

#[test]
fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners() {
    let project_io_root = include_str!("../../../scene/world/project_io.rs");
    let camera = include_str!("../../../scene/world/project_io/camera.rs");
    let physics = include_str!("../../../scene/world/project_io/physics.rs");
    let post_process = include_str!("../../../scene/world/project_io/post_process.rs");
    let references = include_str!("../../../scene/world/project_io/references.rs");
    let script = include_str!("../../../scene/world/project_io/script.rs");
    let transform = include_str!("../../../scene/world/project_io/transform.rs");
    let project_io_doc =
        include_str!("../../../../../docs/zircon_runtime/scene/world/project_io.md");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let large_file_doc =
        include_str!("../../../../../docs/engine-architecture/large-file-ownership-m1.md");
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");

    for root_anchor in [
        "mod camera;",
        "mod physics;",
        "mod post_process;",
        "mod references;",
        "mod script;",
        "mod transform;",
        "pub fn from_scene_asset",
        "pub fn to_scene_asset",
    ] {
        assert!(
            project_io_root.contains(root_anchor),
            "project_io.rs should keep entry orchestration anchor `{root_anchor}`"
        );
    }

    for moved_helper in [
        "fn camera_target_from_asset",
        "fn collider_shape_from_asset",
        "fn post_process_settings_from_asset",
        "fn model_handle_for_reference",
        "fn script_bindings_for_record",
        "fn transform_from_asset",
    ] {
        assert!(
            !project_io_root.contains(moved_helper),
            "project_io.rs should not reclaim converter helper `{moved_helper}`"
        );
    }

    for (module_name, module_source, expected_anchor) in [
        ("camera", camera, "pub(super) fn camera_to_asset"),
        ("physics", physics, "pub(super) fn collider_shape_to_asset"),
        (
            "post_process",
            post_process,
            "pub(super) fn post_process_volume_to_asset",
        ),
        (
            "references",
            references,
            "pub(super) fn reference_for_model_handle",
        ),
        ("script", script, "pub(super) fn script_bindings_for_record"),
        ("transform", transform, "pub(super) fn transform_to_asset"),
    ] {
        assert!(
            module_source.contains(expected_anchor),
            "project_io/{module_name}.rs should own `{expected_anchor}`"
        );
    }

    for doc_anchor in [
        "Project I/O Folder Split",
        "project_io/{camera,physics,post_process,references,script,transform}.rs",
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
        "project_io.rs 772 行",
        "project_io folder split",
    ] {
        assert!(
            project_io_doc.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || large_file_doc.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor),
            "Project I/O split docs should retain `{doc_anchor}`"
        );
    }
}

#[test]
fn runtime_07_dynamic_session_event_split_keeps_abi_entry_and_event_owner() {
    let session_root = include_str!("../../../dynamic_api/session.rs");
    let session_events = include_str!("../../../dynamic_api/session/events.rs");
    let runtime_07_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let dynamic_session_doc =
        include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");

    for root_anchor in [
        "mod events;",
        "pub(super) unsafe fn handle_event(",
        "with_session(handle, |session| session.handle_event(event))",
    ] {
        assert!(
            session_root.contains(root_anchor),
            "session.rs should keep dynamic ABI event entry anchor `{root_anchor}`"
        );
    }

    for moved_event_anchor in [
        "fn handle_mouse_button",
        "fn handle_mouse_wheel",
        "fn handle_keyboard",
        "fn handle_ime",
        "fn handle_gamepad_axis",
        "fn sync_orbit_target_from_selection",
    ] {
        assert!(
            !session_root.contains(moved_event_anchor),
            "session.rs should not reclaim dynamic event helper `{moved_event_anchor}`"
        );
        assert!(
            session_events.contains(moved_event_anchor),
            "session/events.rs should own dynamic event helper `{moved_event_anchor}`"
        );
    }

    for events_anchor in [
        "pub(super) fn handle_event(&mut self, event: ZrRuntimeEventV1) -> ZrStatus",
        "UiAccessibilityActionRequest",
        "runtime_session_menu_action_at",
        "write_runtime_menu_action",
        "ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1",
    ] {
        assert!(
            session_events.contains(events_anchor),
            "session/events.rs should retain dynamic event dispatch anchor `{events_anchor}`"
        );
    }

    for doc_anchor in [
        "Dynamic Session Event Split",
        "session/events.rs",
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
        "dynamic session event split",
    ] {
        assert!(
            runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || dynamic_session_doc.contains(doc_anchor),
            "Dynamic session event split docs should retain `{doc_anchor}`"
        );
    }
}
