use super::super::support::*;
use super::support::{control_bool, control_float, control_string, control_visibility};
use crate::ui::retained_host::popup_anchor_metrics::{
    clamp_popup_x_to_bounds, toolbar_popup_render_gap,
};
use zircon_runtime_interface::ui::tree::UiVisibility;

#[test]
fn workbench_toolbar_window_menus_open_exclusively_and_toggle_closed() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id } if action_id == "workbench.run_mode.menu.open"
    ));
    assert!(control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRunMode", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Visible)
    );
    assert!(
        bridge.control_frame("WorkbenchRunModeMenu").is_some(),
        "opened run mode menu should have a native frame"
    );

    bridge
        .dispatch_control_state("WorkbenchLayoutGrid", UiEventKind::Click)
        .expect("layout menu should dispatch")
        .expect("layout menu should expose a menu binding");
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert!(control_bool(&bridge, "WorkbenchLayoutGrid", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLayoutMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Visible)
    );

    bridge
        .dispatch_control_state("WorkbenchLayoutGrid", UiEventKind::Click)
        .expect("layout menu should dispatch when toggled closed")
        .expect("layout menu should expose a menu binding");
    assert!(!control_bool(&bridge, "WorkbenchLayoutGrid", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchLayoutMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn workbench_toolbar_window_menu_item_selection_closes_trigger_and_menu() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");

    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchRunModeMenu", "menu.item.simulate")
            .expect("run mode menu item should select"),
        Some(true)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value_text").as_deref(),
        Some("Simulate")
    );
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn workbench_toolbar_window_menus_anchor_to_toolbar_controls_across_widths() {
    for width in [900.0, 1260.0, 1672.0] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, 620.0))
                .unwrap_or_else(|error| {
                    panic!("workbench {width}px bridge should build: {error:?}")
                });
        assert_toolbar_menu_anchor(
            &mut bridge,
            "WorkbenchToolbarMenu",
            "WorkbenchToolbarMainMenu",
            ToolbarMenuAlign::Start,
        );
    }

    let mut compact = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("compact workbench bridge should build");
    assert_toolbar_menu_anchor(
        &mut compact,
        "WorkbenchModuleMore",
        "WorkbenchModuleOverflowMenu",
        ToolbarMenuAlign::Start,
    );

    for (trigger_id, menu_id) in [
        ("WorkbenchRunMode", "WorkbenchRunModeMenu"),
        ("WorkbenchLayoutGrid", "WorkbenchLayoutMenu"),
    ] {
        let mut wide = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
            .expect("wide workbench bridge should build");
        assert_toolbar_menu_anchor(&mut wide, trigger_id, menu_id, ToolbarMenuAlign::End);
    }
}

#[test]
fn workbench_toolbar_window_menu_anchor_metrics_stay_shared() {
    let source = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs",
    ))
    .expect("window menu state source should be readable");

    assert!(source.contains("popup_anchor_metrics"));
    assert!(
        !source.contains("const TOOLBAR_POPUP_EDGE_MARGIN"),
        "toolbar popup edge margin must stay in popup_anchor_metrics"
    );
    assert!(
        !source.contains("const TOOLBAR_POPUP_RENDER_GAP"),
        "toolbar popup render gap must stay in popup_anchor_metrics"
    );
}

#[derive(Clone, Copy)]
enum ToolbarMenuAlign {
    Start,
    End,
}

fn assert_toolbar_menu_anchor(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    trigger_id: &str,
    menu_id: &str,
    align: ToolbarMenuAlign,
) {
    let trigger = bridge
        .control_frame(trigger_id)
        .unwrap_or_else(|| panic!("{trigger_id} should expose a visible trigger frame"));
    let toolbar = bridge
        .control_frame("WorkbenchWindowTopToolbarRegion")
        .expect("workbench should expose the top toolbar frame");
    let root = bridge
        .control_frame("WorkbenchWindowRoot")
        .expect("workbench should expose the root frame");

    bridge
        .dispatch_control_state(trigger_id, UiEventKind::Click)
        .unwrap_or_else(|error| panic!("{trigger_id} should dispatch: {error:?}"))
        .unwrap_or_else(|| panic!("{trigger_id} should expose a click binding"));

    let menu = bridge
        .control_frame(menu_id)
        .unwrap_or_else(|| panic!("{menu_id} should open with a visible menu frame"));
    let authored_x = match align {
        ToolbarMenuAlign::Start => trigger.x,
        ToolbarMenuAlign::End => trigger.right() - menu.width,
    };
    let expected_x = clamped_toolbar_menu_x(authored_x, menu.width, root.width);
    assert_near(&format!("{menu_id} x"), menu.x, expected_x);
    assert_near(&format!("{menu_id} y"), menu.y, toolbar.bottom());
    assert_near(
        &format!("{menu_id} popup_anchor_x"),
        control_float(bridge, menu_id, "popup_anchor_x")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_anchor_x")) as f32,
        menu.x,
    );
    assert_near(
        &format!("{menu_id} popup_anchor_y"),
        control_float(bridge, menu_id, "popup_anchor_y")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_anchor_y")) as f32,
        menu.y,
    );
    assert_near(
        &format!("{menu_id} popup_offset_y"),
        control_float(bridge, menu_id, "popup_offset_y")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_offset_y")) as f32,
        -toolbar_popup_render_gap(),
    );
    assert_eq!(
        control_string(bridge, menu_id, "placement").as_deref(),
        Some("bottom-start")
    );
}

fn clamped_toolbar_menu_x(authored_x: f32, menu_width: f32, root_width: f32) -> f32 {
    clamp_popup_x_to_bounds(authored_x, 0.0, root_width, menu_width)
}

fn assert_near(label: &str, actual: f32, expected: f32) {
    const EPSILON: f32 = 0.01;
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}
