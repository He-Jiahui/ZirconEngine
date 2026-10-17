use super::*;
use crate::scene::viewport::SceneViewportChromeSettings;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

fn resized(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, size: UiSize) {
    bridge
        .template_surface
        .recompute_layout(bridge.runtime.as_ref(), size)
        .unwrap();
    bridge.refresh_unreal_viewport_toolbar_overflow().unwrap();
}

#[test]
fn measured_prefix_reserves_the_spacer_gap_and_more_without_shrinking_groups() {
    let widths = [100.0, 40.0, 60.0, 112.0];
    assert_eq!(inline_prefix(&widths, 320.0, 28.0, 2.0), 4);
    assert_eq!(inline_prefix(&widths, 319.0, 28.0, 2.0), 3);
    assert_eq!(inline_prefix(&widths, 230.0, 28.0, 2.0), 2);
    assert_eq!(inline_prefix(&widths, 29.0, 28.0, 2.0), 0);
    assert_eq!(inline_prefix(&widths, 900.0, 28.0, 2.0), 4);
}

#[test]
fn mounted_source_groups_move_once_keep_incarnation_and_restore_original_slots() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    let contract = bridge.viewport_toolbar_overflow.contract.as_ref().unwrap();
    assert_eq!(
        bridge.viewport_toolbar_overflow.inline_count,
        contract.groups.len()
    );
    let original = contract
        .groups
        .iter()
        .map(|group| {
            let node = bridge
                .template_surface
                .surface
                .tree
                .node(group.node_id)
                .unwrap();
            (
                group.node_id,
                bridge
                    .template_surface
                    .surface
                    .tree
                    .node_incarnation(group.node_id),
                node.paint_order,
                node.template_metadata.clone(),
                group.original_slot.clone(),
            )
        })
        .collect::<Vec<_>>();
    let source_children = contract.original_row_children.clone();
    let binding = bridge
        .binding_for_control("SetScaleSnap", UiEventKind::Change)
        .unwrap()
        .native_binding();
    resized(&mut bridge, UiSize::new(900.0, 620.0));
    let contract = bridge.viewport_toolbar_overflow.contract.as_ref().unwrap();
    assert!(bridge.viewport_toolbar_overflow.inline_count < contract.groups.len());
    for (index, (node_id, incarnation, paint_order, source, _)) in original.iter().enumerate() {
        let tree = &bridge.template_surface.surface.tree;
        let node = tree.node(*node_id).unwrap();
        let expected_parent = if index < bridge.viewport_toolbar_overflow.inline_count {
            contract.row
        } else {
            contract.popup
        };
        assert_eq!(node.parent, Some(expected_parent));
        assert_eq!(
            tree.node(expected_parent)
                .unwrap()
                .children
                .iter()
                .filter(|child| **child == *node_id)
                .count(),
            1
        );
        assert_eq!(
            tree.layout_slots()
                .iter()
                .filter(|slot| slot.child_id == *node_id)
                .count(),
            1
        );
        assert_eq!(tree.node_incarnation(*node_id), *incarnation);
        assert_eq!(node.paint_order, *paint_order);
        assert_eq!(&node.template_metadata, source);
    }
    assert_eq!(
        bridge
            .binding_for_control("SetScaleSnap", UiEventKind::Change)
            .unwrap()
            .native_binding(),
        binding
    );
    resized(&mut bridge, UiSize::new(4096.0, 900.0));
    let contract = bridge.viewport_toolbar_overflow.contract.as_ref().unwrap();
    assert_eq!(
        bridge.viewport_toolbar_overflow.inline_count,
        contract.groups.len()
    );
    assert_eq!(
        bridge
            .template_surface
            .surface
            .tree
            .node(contract.row)
            .unwrap()
            .children,
        source_children
    );
    assert!(bridge
        .template_surface
        .surface
        .tree
        .node(contract.popup)
        .unwrap()
        .children
        .is_empty());
    for (node_id, _, _, _, slot) in original {
        assert_eq!(
            bridge
                .template_surface
                .surface
                .tree
                .layout_slots()
                .iter()
                .find(|live| live.child_id == node_id)
                .unwrap(),
            &slot
        );
    }
}

#[test]
fn more_keeps_original_dropdown_admission_and_closing_preserves_snap_checked_states() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let mut chrome = SceneViewportChromeSettings::default();
    chrome.translate_snap_enabled = false;
    chrome.rotate_snap_enabled = true;
    chrome.scale_snap_enabled = false;
    bridge.sync_unreal_viewport_toolbar_state(&chrome).unwrap();
    assert!(!bridge
        .apply_unreal_overflow_action("SetScaleSnap", OPEN_ACTION)
        .unwrap());
    assert!(bridge
        .apply_unreal_overflow_action(MORE, OPEN_ACTION)
        .unwrap());
    assert!(bridge.control_bool(OVERFLOW_CONTROL, "popup_open"));
    assert!(bridge
        .select_dropdown_option("SetScaleSnap", "0.25")
        .unwrap());
    assert_eq!(
        bridge.control_string("SetScaleSnap", "value").as_deref(),
        Some("0.25")
    );
    assert!(!bridge
        .select_dropdown_option("SetScaleSnap", "not-an-option")
        .unwrap());
    assert_eq!(
        bridge.control_string("SetScaleSnap", "value").as_deref(),
        Some("0.25")
    );
    assert!(bridge.close_popup(OVERFLOW_CONTROL).unwrap());
    assert!(!bridge.control_bool(OVERFLOW_CONTROL, "popup_open"));
    assert!(!bridge.control_bool("ToggleTranslateSnap", "checked"));
    assert!(bridge.control_bool("ToggleRotateSnap", "checked"));
    assert!(!bridge.control_bool("ToggleScaleSnap", "checked"));
}

#[test]
fn overflow_scroll_uses_the_existing_source_pointer_route_once_and_clamps_at_extent() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    bridge
        .apply_unreal_overflow_action(MORE, OPEN_ACTION)
        .unwrap();
    let width = bridge
        .template_surface
        .control_frame(OVERFLOW_CONTROL)
        .unwrap()
        .width;
    // Exercise the real arranged scroll container at a bounded popup extent,
    // independently of platform font metrics or the outer window's minimum size.
    bridge
        .set_fixed_control_extent(OVERFLOW_CONTROL, UiSize::new(width, 32.0))
        .unwrap();
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    let popup = bridge.control_node_id(OVERFLOW_CONTROL).unwrap();
    let before = bridge
        .template_surface
        .surface
        .tree
        .node(popup)
        .unwrap()
        .scroll_state
        .unwrap();
    assert!(before.content_extent > before.viewport_extent);
    // The source route converts physical pointer coordinates once. Its wheel
    // scalar is already the Runtime compatibility unit, independent of DPI.
    bridge.presentation_scale_factor = 1.5;
    let frame = bridge
        .template_surface
        .control_frame(OVERFLOW_CONTROL)
        .unwrap();
    let point = UiPoint::new(
        (frame.x + frame.width / 2.0) * 1.5,
        (frame.y + frame.height / 2.0) * 1.5,
    );
    let scroll =
        |delta| UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta);
    let route = bridge.route_pointer_event(scroll(3.0)).unwrap();
    assert!(
        route.routing_root_to_leaf().contains(&popup)
            || route.hit_path.root_to_leaf.contains(&popup)
    );
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    assert_eq!(
        bridge
            .template_surface
            .surface
            .tree
            .node(popup)
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        before.offset + 3.0
    );
    bridge.route_pointer_event(scroll(10000.0)).unwrap();
    bridge
        .template_surface
        .refresh_after_state_change(bridge.runtime.as_ref())
        .unwrap();
    let bottom = bridge
        .template_surface
        .surface
        .tree
        .node(popup)
        .unwrap()
        .scroll_state
        .unwrap();
    assert_eq!(
        bottom.offset,
        (bottom.content_extent - bottom.viewport_extent).max(0.0)
    );
    bridge.route_pointer_event(scroll(10000.0)).unwrap();
    assert_eq!(
        bridge
            .template_surface
            .surface
            .tree
            .node(popup)
            .unwrap()
            .scroll_state
            .unwrap(),
        bottom
    );
    bridge.close_popup(OVERFLOW_CONTROL).unwrap();
    let closed_state = bridge
        .template_surface
        .surface
        .tree
        .node(popup)
        .unwrap()
        .scroll_state;
    bridge.route_pointer_event(scroll(-10000.0)).unwrap();
    assert_eq!(
        bridge
            .template_surface
            .surface
            .tree
            .node(popup)
            .unwrap()
            .scroll_state,
        closed_state
    );
}

#[test]
fn repeated_same_geometry_and_chrome_does_not_rearrange_overflow() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0)).unwrap();
    let before = bridge.template_surface.layout_pass_count();
    let parents = bridge
        .viewport_toolbar_overflow
        .contract
        .as_ref()
        .unwrap()
        .groups
        .iter()
        .map(|group| {
            bridge
                .template_surface
                .surface
                .tree
                .node(group.node_id)
                .unwrap()
                .parent
        })
        .collect::<Vec<_>>();
    bridge.refresh_unreal_viewport_toolbar_overflow().unwrap();
    bridge.refresh_unreal_viewport_toolbar_overflow().unwrap();
    assert_eq!(bridge.template_surface.layout_pass_count(), before);
    let after = bridge
        .viewport_toolbar_overflow
        .contract
        .as_ref()
        .unwrap()
        .groups
        .iter()
        .map(|group| {
            bridge
                .template_surface
                .surface
                .tree
                .node(group.node_id)
                .unwrap()
                .parent
        })
        .collect::<Vec<_>>();
    assert_eq!(parents, after);
}
#[test]
fn dedicated_overflow_marker_preserves_registered_role_and_rejects_missing_authored_marker() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    assert_eq!(
        bridge.viewport_toolbar_overflow.inline_count,
        bridge
            .viewport_toolbar_overflow
            .contract
            .as_ref()
            .unwrap()
            .groups
            .len()
    );
    assert_eq!(
        bridge
            .control_string(OVERFLOW_CONTROL, "toolbar_overflow_role")
            .as_deref(),
        Some("toolbar-overflow")
    );
    let registered = bridge
        .control_string(OVERFLOW_CONTROL, "component_role")
        .unwrap();
    assert_ne!(registered, "toolbar-overflow");
    assert!(bridge.capture_unreal_overflow_contract().unwrap().is_some());
    resized(&mut bridge, UiSize::new(900.0, 620.0));
    assert!(
        bridge.viewport_toolbar_overflow.inline_count
            < bridge
                .viewport_toolbar_overflow
                .contract
                .as_ref()
                .unwrap()
                .groups
                .len()
    );
    assert!(bridge
        .apply_unreal_overflow_action(MORE, OPEN_ACTION)
        .unwrap());
    let nodes = crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
        bridge.host_projection(),
    ));
    let popup = nodes
        .iter()
        .find(|node| node.control_id.as_str() == OVERFLOW_CONTROL)
        .unwrap();
    assert_eq!(popup.toolbar_overflow_role.as_str(), "toolbar-overflow");
    assert_eq!(popup.component_role.as_str(), registered.as_str());
    assert!(bridge.close_popup(OVERFLOW_CONTROL).unwrap());
    resized(&mut bridge, UiSize::new(4096.0, 900.0));
    assert_eq!(
        bridge.viewport_toolbar_overflow.inline_count,
        bridge
            .viewport_toolbar_overflow
            .contract
            .as_ref()
            .unwrap()
            .groups
            .len()
    );
    assert!(bridge.capture_unreal_overflow_contract().unwrap().is_some());
    // A copied descriptor role cannot substitute for this authored contract.
    bridge
        .mutate_control_property(
            OVERFLOW_CONTROL,
            "toolbar_overflow_role",
            UiValue::String("wrong-owner".into()),
        )
        .unwrap();
    bridge
        .mutate_control_property(
            OVERFLOW_CONTROL,
            "component_role",
            UiValue::String("toolbar-overflow".into()),
        )
        .unwrap();
    assert!(bridge.capture_unreal_overflow_contract().is_err());
}

#[test]
fn dedicated_overflow_marker_keeps_exact_more_and_popup_source_ancestry() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(4096.0, 900.0)).unwrap();
    assert_eq!(
        bridge.viewport_toolbar_overflow.inline_count,
        bridge
            .viewport_toolbar_overflow
            .contract
            .as_ref()
            .unwrap()
            .groups
            .len()
    );
    let toolbar = bridge.control_node_id(TOOLBAR).unwrap();
    let row = bridge.control_node_id("UnrealViewportToolbarRow").unwrap();
    let popup = bridge.control_node_id(OVERFLOW_CONTROL).unwrap();
    let more = bridge.control_node_id(MORE).unwrap();
    assert!(bridge.capture_unreal_overflow_contract().unwrap().is_some());
    bridge
        .template_surface
        .surface
        .tree
        .node_mut(more)
        .unwrap()
        .parent = Some(toolbar);
    assert!(bridge.capture_unreal_overflow_contract().is_err());
    bridge
        .template_surface
        .surface
        .tree
        .node_mut(more)
        .unwrap()
        .parent = Some(row);
    bridge
        .template_surface
        .surface
        .tree
        .node_mut(popup)
        .unwrap()
        .parent = Some(row);
    assert!(bridge.capture_unreal_overflow_contract().is_err());
    bridge
        .template_surface
        .surface
        .tree
        .node_mut(popup)
        .unwrap()
        .parent = Some(toolbar);
    assert!(bridge.capture_unreal_overflow_contract().unwrap().is_some());
}
