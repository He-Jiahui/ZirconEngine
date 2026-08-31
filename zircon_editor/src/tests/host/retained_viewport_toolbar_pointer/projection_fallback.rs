use std::sync::Arc;

use crate::core::editor_event::{EditorEvent, EditorViewportEvent};
use crate::scene::viewport::{DisplayMode, PivotMode};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_viewport_toolbar_pointer_click, BuiltinViewportToolbarTemplateBridge,
};
use crate::ui::retained_host::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout, ViewportToolbarPointerBridge,
};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_viewport_toolbar_pointer_click_falls_back_to_surface_projection_when_control_rect_is_empty(
) {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_viewport_toolbar_projection_fallback");
    let mut template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    template_bridge
        .recompute_layout(UiSize::new(1280.0, 28.0))
        .expect("viewport toolbar layout should compute");
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));

    let dispatched = dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "scene.main",
        "SetDisplayMode",
        0.0,
        0.0,
        0.0,
        0.0,
        UiPoint::new(300.0, 10.0),
    )
    .expect("shared viewport toolbar route should fall back to projected control frame");

    assert_eq!(
        dispatched.pointer.route, None,
        "projection control ids should dispatch through template bindings, not legacy toolbar routes"
    );
    let effects = dispatched
        .effects
        .expect("projection-backed click should dispatch into the runtime");
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetDisplayMode {
            mode: DisplayMode::Shaded,
        })
    );
}

#[test]
fn viewport_toolbar_surface_frame_includes_projected_route_controls_without_action_list() {
    let _guard = env_lock().lock().unwrap();

    let mut template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    template_bridge
        .recompute_layout(UiSize::new(1280.0, 28.0))
        .expect("viewport toolbar layout should compute");

    let surface_frame = template_bridge.surface_frame_for_projection_controls(
        "scene.main",
        UiSize::new(1280.0, 28.0),
        |projection_control_id| Some(projection_control_id.to_string()),
    );
    let control_ids = surface_frame
        .arranged_tree
        .nodes
        .iter()
        .filter_map(|node| node.control_id.as_deref())
        .collect::<Vec<_>>();

    for required in [
        "ActivateSceneMode",
        "SetTransformSpace",
        "SetPivotMode",
        "SetDisplayMode",
        "SetGridMode",
        "SetTranslateSnap",
        "SetPreviewLighting",
        "FrameSelection",
        "EnterPlayMode",
        "SetProjectionMode",
        "AlignView",
    ] {
        assert!(
            control_ids.contains(&required),
            "toolbar surface frame should include projected `{required}` button"
        );
    }

    let snap_node = surface_frame
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("SetTranslateSnap"))
        .expect("projected translate snap button should be arranged");
    assert_eq!(
        Some(snap_node.frame),
        template_bridge.control_frame_for_control("SetTranslateSnap")
    );
    assert_eq!(
        surface_frame.hit_grid.entries.len(),
        control_ids.len(),
        "hit entries should be derived from the same projected route-bearing controls"
    );
}

#[test]
fn projected_pivot_control_cycles_the_authoritative_viewport_mode() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_viewport_pivot_mode_cycle");
    let mut template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    template_bridge
        .recompute_layout(UiSize::new(1280.0, 28.0))
        .expect("viewport toolbar layout should compute");
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));

    assert_eq!(
        harness.runtime.scene_viewport_settings().pivot_mode,
        PivotMode::Centroid
    );
    dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "scene.main",
        "SetPivotMode",
        0.0,
        0.0,
        0.0,
        0.0,
        UiPoint::new(76.0, 10.0),
    )
    .expect("projected pivot control should dispatch its cycle route");

    assert_eq!(
        harness.runtime.scene_viewport_settings().pivot_mode,
        PivotMode::Primary
    );
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetPivotMode {
            mode: PivotMode::Primary,
        })
    );
}

#[test]
fn viewport_toolbar_surface_frame_reuses_an_exact_projection_signature() {
    let _guard = env_lock().lock().unwrap();

    let mut template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    let surface_size = UiSize::new(1280.0, 28.0);
    template_bridge
        .recompute_layout(surface_size)
        .expect("viewport toolbar layout should compute");

    let first = template_bridge.surface_frame_for_projection_controls(
        "scene.main",
        surface_size,
        |projection_control_id| Some(projection_control_id.to_string()),
    );
    let repeated = template_bridge.surface_frame_for_projection_controls(
        "scene.main",
        surface_size,
        |projection_control_id| Some(projection_control_id.to_string()),
    );

    assert!(
        Arc::ptr_eq(&first, &repeated),
        "an unchanged projection must reuse the published hit-test frame"
    );
    let prelayout_reuse = template_bridge
        .surface_frame_from_cached_layout_for_projection_controls(
            "scene.main",
            surface_size,
            |projection_control_id| Some(projection_control_id.to_string()),
        )
        .expect("an exact signature should be reusable before another layout pass");
    assert!(Arc::ptr_eq(&first, &prelayout_reuse));
    let recomputes_before_remap = template_bridge.layout_recompute_count();

    let remapped = template_bridge
        .surface_frame_from_cached_layout_for_projection_controls(
            "scene.main",
            surface_size,
            |projection_control_id| {
                Some(match projection_control_id {
                    "SetDisplayMode" => "display.remapped".to_string(),
                    _ => projection_control_id.to_string(),
                })
            },
        )
        .expect("same-size route remapping should reuse the cached layout");
    assert!(
        !Arc::ptr_eq(&first, &remapped),
        "a changed hit-control mapping must publish a new frame"
    );
    assert!(remapped
        .hit_grid
        .entries
        .iter()
        .any(|entry| entry.control_id.as_deref() == Some("display.remapped")));
    assert_eq!(
        template_bridge.layout_recompute_count(),
        recomputes_before_remap,
        "route identity changes must not trigger template layout"
    );

    let repeated_remap = template_bridge
        .surface_frame_from_cached_layout_for_projection_controls(
            "scene.main",
            surface_size,
            |projection_control_id| {
                Some(match projection_control_id {
                    "SetDisplayMode" => "display.remapped".to_string(),
                    _ => projection_control_id.to_string(),
                })
            },
        )
        .expect("the remapped signature should be retained");
    assert!(Arc::ptr_eq(&remapped, &repeated_remap));

    let without_display_mode = template_bridge.surface_frame_for_projection_controls(
        "scene.partial",
        surface_size,
        |projection_control_id| {
            (projection_control_id != "SetDisplayMode").then(|| projection_control_id.to_string())
        },
    );
    assert!(without_display_mode
        .hit_grid
        .entries
        .iter()
        .all(|entry| entry.control_id.as_deref() != Some("SetDisplayMode")));
    let with_display_mode = template_bridge
        .surface_frame_from_cached_layout_for_projection_controls(
            "scene.partial",
            surface_size,
            |projection_control_id| Some(projection_control_id.to_string()),
        )
        .expect("a previously omitted mapping should reproject cached geometry");
    assert!(!Arc::ptr_eq(&without_display_mode, &with_display_mode));
    assert!(with_display_mode
        .hit_grid
        .entries
        .iter()
        .any(|entry| entry.control_id.as_deref() == Some("SetDisplayMode")));
    assert_eq!(
        template_bridge.layout_recompute_count(),
        recomputes_before_remap
    );
}
