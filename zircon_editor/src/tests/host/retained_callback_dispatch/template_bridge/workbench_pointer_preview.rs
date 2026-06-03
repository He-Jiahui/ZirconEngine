use super::super::support::*;
use super::support::{
    control_bool, control_center, control_component_pressed, control_float,
    render_background_for_control,
};
use crate::ui::retained_host::HostInvalidationMask;

#[test]
fn componentized_workbench_pointer_hover_updates_icon_button_preview_without_authored_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_hover_preview");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(!control_bool(&bridge, "WorkbenchToolMove", "hovered"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#171c20")
    );

    let tool_move_center = control_center(&bridge, "WorkbenchToolMove");
    let hover_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, tool_move_center),
    )
    .expect("hovering a componentized icon button should request paint-only feedback")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolMove", "hovered"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#20262b")
    );
    assert!(hover_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!hover_effects.render_dirty);
    assert!(!hover_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let leave_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(-8.0, -8.0)),
    )
    .expect("leaving a hovered componentized icon button should request paint-only feedback")
    .unwrap();

    assert!(!control_bool(&bridge, "WorkbenchToolMove", "hovered"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#171c20")
    );
    assert!(leave_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!leave_effects.render_dirty);
    assert!(!leave_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn componentized_workbench_pointer_press_updates_icon_button_preview_before_release_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_press_preview");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let tool_move_center = control_center(&bridge, "WorkbenchToolMove");

    assert!(!control_component_pressed(&bridge, "WorkbenchToolMove"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#171c20")
    );

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, tool_move_center)
            .with_button(UiPointerButton::Primary),
    )
    .expect("pressing a componentized icon button should request paint-only feedback")
    .unwrap();

    assert!(control_component_pressed(&bridge, "WorkbenchToolMove"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#12383d")
    );
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let release_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, tool_move_center)
            .with_button(UiPointerButton::Primary),
    )
    .expect("releasing a componentized icon button should dispatch the authored binding")
    .unwrap();

    assert!(!control_component_pressed(&bridge, "WorkbenchToolMove"));
    assert!(control_bool(&bridge, "WorkbenchToolMove", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#12383d")
    );
    assert!(release_effects.render_dirty);
    assert!(release_effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetTool {
            tool: SceneViewportTool::Move
        })
    );
}

#[test]
fn componentized_workbench_pointer_drag_updates_slider_value_without_authored_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_slider_preview");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let slider_frame = bridge
        .control_frame("WorkbenchInputSlider")
        .expect("WorkbenchInputSlider should have a frame");
    let slider_point = |fraction: f32| {
        UiPoint::new(
            slider_frame.x + slider_frame.width * fraction,
            slider_frame.y + slider_frame.height * 0.5,
        )
    };

    assert_float_eq(
        control_float(&bridge, "WorkbenchInputSlider", "value").unwrap(),
        75.0,
    );
    assert!(!control_component_pressed(&bridge, "WorkbenchInputSlider"));

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, slider_point(0.25))
            .with_button(UiPointerButton::Primary),
    )
    .expect("pressing a componentized slider should request paint-only feedback")
    .unwrap();

    assert!(control_component_pressed(&bridge, "WorkbenchInputSlider"));
    assert_float_eq(
        control_float(&bridge, "WorkbenchInputSlider", "value").unwrap(),
        25.0,
    );
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let drag_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, slider_point(0.80)),
    )
    .expect("dragging a componentized slider should request paint-only feedback")
    .unwrap();

    assert!(control_component_pressed(&bridge, "WorkbenchInputSlider"));
    assert_float_eq(
        control_float(&bridge, "WorkbenchInputSlider", "value").unwrap(),
        80.0,
    );
    assert!(drag_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!drag_effects.render_dirty);
    assert!(!drag_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let release_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, slider_point(0.80))
            .with_button(UiPointerButton::Primary),
    )
    .expect("releasing a componentized slider should clear pressed feedback")
    .unwrap();

    assert!(!control_component_pressed(&bridge, "WorkbenchInputSlider"));
    assert_float_eq(
        control_float(&bridge, "WorkbenchInputSlider", "value").unwrap(),
        80.0,
    );
    assert!(release_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!release_effects.render_dirty);
    assert!(!release_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());
}

fn assert_float_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "expected {expected}, got {actual}"
    );
}
