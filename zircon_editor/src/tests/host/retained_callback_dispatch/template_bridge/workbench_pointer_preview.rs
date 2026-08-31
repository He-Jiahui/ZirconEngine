use super::super::support::*;
use super::support::{
    control_bool, control_center, control_component_pressed, control_float, control_string,
    control_visibility, render_background_for_control,
};
use crate::ui::retained_host::HostInvalidationMask;
use zircon_runtime_interface::ui::dispatch::UiInputTimestamp;
use zircon_runtime_interface::ui::tree::UiVisibility;

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
fn componentized_workbench_pointer_hover_delays_labeled_icon_tooltip_and_hides_it_on_leave() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_icon_tooltip");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let tool_frame = bridge
        .control_frame("WorkbenchToolMove")
        .expect("move tool should have a frame");

    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Collapsed)
    );
    assert!(!control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));

    let hover_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(
            UiPointerEventKind::Move,
            UiPoint::new(
                tool_frame.x + tool_frame.width * 0.5,
                tool_frame.y + tool_frame.height * 0.5,
            ),
        ),
    )
    .expect("hovering a labeled icon button should arm its tooltip")
    .unwrap();

    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Collapsed)
    );
    assert!(!control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));
    assert!(!bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(149_999))
        .expect("tooltip should remain pending before the Runtime deadline"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Collapsed)
    );
    assert!(bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(150_000))
        .expect("Runtime tooltip deadline should publish the tooltip"));

    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Visible)
    );
    assert!(control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));
    assert_eq!(
        bridge
            .control_frame("WorkbenchIconTooltip")
            .expect("short icon tooltip should have a projected frame")
            .width,
        96.0,
        "short icon hints should keep the authored compact floor"
    );
    assert_eq!(
        control_float(&bridge, "WorkbenchIconTooltip", "transition_progress"),
        Some(0.0),
        "the Runtime show boundary should begin the intro at zero opacity"
    );
    assert!(bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(200_000))
        .expect("mid-intro Runtime tick should publish paint progress"));
    assert_eq!(
        control_float(&bridge, "WorkbenchIconTooltip", "transition_progress"),
        Some(0.5)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchIconTooltip", "transition_status").as_deref(),
        Some("entering")
    );
    assert!(bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(250_000))
        .expect("intro deadline should publish the fully entered state"));
    assert_eq!(
        control_float(&bridge, "WorkbenchIconTooltip", "transition_progress"),
        Some(1.0)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchIconTooltip", "transition_status").as_deref(),
        Some("entered")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchIconTooltip", "text").as_deref(),
        Some("Move")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchIconTooltip", "label_text").as_deref(),
        Some("")
    );
    let tooltip_anchor = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some("WorkbenchIconTooltip"))
            .and_then(|metadata| metadata.widget.popup_anchor.control_id())
    });
    assert_eq!(tooltip_anchor, Some("WorkbenchToolMove"));
    assert_eq!(
        control_string(&bridge, "WorkbenchIconTooltip", "placement").as_deref(),
        Some("bottom")
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
    .expect("leaving a labeled icon button should hide its tooltip")
    .unwrap();

    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Collapsed)
    );
    assert!(!control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));
    assert!(leave_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!leave_effects.render_dirty);
    assert!(!leave_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn ultra_toolbar_density_collapsed_command_keeps_explicit_action_tooltip() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_ultra_toolbar_command_tooltip");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0)).unwrap();
    let compile_frame = bridge
        .control_frame("WorkbenchModuleCompile")
        .expect("ultra toolbar compile command should remain reachable");
    assert_eq!(compile_frame.width, 34.0);

    dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(
            UiPointerEventKind::Move,
            UiPoint::new(
                compile_frame.x + compile_frame.width * 0.5,
                compile_frame.y + compile_frame.height * 0.5,
            ),
        ),
    )
    .expect("hovering an icon-only module command should arm its explicit tooltip")
    .unwrap();
    assert!(bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(150_000))
        .expect("Runtime tooltip deadline should publish the explicit tooltip"));

    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Visible)
    );
    assert!(control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));
    let tooltip_width = bridge
        .control_frame("WorkbenchIconTooltip")
        .expect("explicit action tooltip should have a projected frame")
        .width;
    assert!(
        tooltip_width > 96.0,
        "longer explicit hints should expand beyond the compact floor"
    );
    assert!(
        tooltip_width <= 420.0,
        "tooltip measurement must stay within the logical shell width"
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchIconTooltip", "text").as_deref(),
        Some("Compile Current Module")
    );

    bridge
        .recompute_layout(UiSize::new(120.0, 360.0))
        .expect("a visible tooltip should survive narrow-shell reflow");
    bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(151_000))
        .expect("the current candidate should remeasure after shell reflow");
    let resized_tooltip_width = bridge
        .control_frame("WorkbenchIconTooltip")
        .expect("the remeasured tooltip should keep a projected frame")
        .width;
    assert!(
        resized_tooltip_width < tooltip_width,
        "the current tooltip must not retain its wider pre-resize extent"
    );
    assert!(
        (96.0..=104.0).contains(&resized_tooltip_width),
        "the narrow-shell tooltip should preserve its compact floor and 8px edge insets"
    );
}

#[test]
fn componentized_workbench_pointer_press_hides_an_open_icon_tooltip() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_icon_tooltip_press");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let tool_move_center = control_center(&bridge, "WorkbenchToolMove");

    dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, tool_move_center),
    )
    .expect("hovering a labeled icon button should arm its tooltip")
    .unwrap();
    assert!(bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(150_000))
        .expect("Runtime tooltip deadline should publish the tooltip"));
    assert!(control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, tool_move_center)
            .with_button(UiPointerButton::Primary),
    )
    .expect("pressing an icon button should hide its open tooltip")
    .unwrap();

    assert_eq!(
        control_visibility(&bridge, "WorkbenchIconTooltip"),
        Some(UiVisibility::Collapsed)
    );
    assert!(!control_bool(&bridge, "WorkbenchIconTooltip", "popup_open"));
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let generation_before_release = bridge.surface().invalidation_generations().generation;
    dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, tool_move_center)
            .with_button(UiPointerButton::Primary),
    )
    .expect("releasing an icon button should dispatch its activation")
    .unwrap();
    assert_eq!(
        bridge.surface().invalidation_generations().generation - generation_before_release,
        1,
        "one pointer event must publish release and activation state in one invalidation commit"
    );
}

#[test]
fn componentized_workbench_search_clear_action_clears_query_and_restores_results() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_search_clear_action");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchAbilityBlendSpaceButton", UiEventKind::Click)
        .expect("Blend Space opener should dispatch")
        .expect("Blend Space opener should bind");
    bridge
        .mutate_control_property_for_test(
            "WorkbenchExtensionBlendSpaceSearch",
            "query",
            UiValue::String("strafe".to_string()),
        )
        .expect("search query should update");
    bridge
        .dispatch_control_state("WorkbenchExtensionBlendSpaceSearch", UiEventKind::Change)
        .expect("search change should dispatch")
        .expect("search change should bind");

    let search_frame = bridge
        .control_frame("WorkbenchExtensionBlendSpaceSearch")
        .expect("Blend Space search should have a frame");
    let clear_frame = crate::ui::retained_host::search_field_clear_action_frame(search_frame)
        .expect("search clear action should fit its field");
    let clear_point = UiPoint::new(
        clear_frame.x + clear_frame.width * 0.5,
        clear_frame.y + clear_frame.height * 0.5,
    );

    dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, clear_point)
            .with_button(UiPointerButton::Primary),
    )
    .expect("pressing the clear action should preserve search focus")
    .unwrap();
    let clear_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, clear_point)
            .with_button(UiPointerButton::Primary),
    )
    .expect("releasing the clear action should reset the search")
    .unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchExtensionBlendSpaceSearch", "query").as_deref(),
        Some("")
    );
    for control_id in [
        "WorkbenchExtensionBlendSpaceIdleRunRow",
        "WorkbenchExtensionBlendSpaceStrafeRow",
        "WorkbenchExtensionBlendSpaceSprintRow",
    ] {
        assert!(bridge.control_frame(control_id).is_some());
    }
    assert!(clear_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
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
        EditorEvent::Viewport(EditorViewportEvent::ActivateSceneMode {
            mode: SceneModeActivation::Transform(TransformHandleKind::Move)
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

    let generation_before_press = bridge.surface().invalidation_generations().generation;
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
    assert_eq!(
        bridge.surface().invalidation_generations().generation - generation_before_press,
        1,
        "one pointer event must publish press and range feedback in one invalidation commit"
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

#[test]
fn componentized_workbench_pointer_drag_preserves_the_range_slider_upper_endpoint() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_range_slider_preview");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let slider_frame = bridge
        .control_frame("WorkbenchInputRangeSlider")
        .expect("WorkbenchInputRangeSlider should have a frame");
    let slider_point = |fraction: f32| {
        UiPoint::new(
            slider_frame.x + slider_frame.width * fraction,
            slider_frame.y + slider_frame.height * 0.5,
        )
    };

    assert_float_eq(
        control_float(&bridge, "WorkbenchInputRangeSlider", "range_min").unwrap(),
        20.0,
    );
    assert_float_eq(
        control_float(&bridge, "WorkbenchInputRangeSlider", "value").unwrap(),
        80.0,
    );

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, slider_point(0.25))
            .with_button(UiPointerButton::Primary),
    )
    .expect("pressing the lower range endpoint should request paint-only feedback")
    .unwrap();

    assert_float_eq(
        control_float(&bridge, "WorkbenchInputRangeSlider", "range_min").unwrap(),
        25.0,
    );
    assert_float_eq(
        control_float(&bridge, "WorkbenchInputRangeSlider", "value").unwrap(),
        80.0,
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
        UiPointerEvent::new(UiPointerEventKind::Move, slider_point(0.30)),
    )
    .expect("dragging the lower range endpoint should preserve the upper endpoint")
    .unwrap();

    assert_float_eq(
        control_float(&bridge, "WorkbenchInputRangeSlider", "range_min").unwrap(),
        30.0,
    );
    assert_float_eq(
        control_float(&bridge, "WorkbenchInputRangeSlider", "value").unwrap(),
        80.0,
    );
    assert!(drag_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!drag_effects.render_dirty);
    assert!(!drag_effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());
}

fn assert_float_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "expected {expected}, got {actual}"
    );
}
