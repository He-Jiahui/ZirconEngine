use std::rc::Rc;

use super::host_value_toml::{notification_text_copy_count, reset_notification_text_copy_count};
use super::*;
use crate::ui::retained_host::{
    callback_dispatch::{
        BuiltinWorkbenchWindowTemplateSurfaceBridge, WORKBENCH_CONTEXT_MENU_CONTROL_ID,
    },
    WorkbenchContextMenuRequestData,
};
use crate::ui::template_runtime::RetainedUiHostComponentKind;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

fn inspector_repeat_parent_projection() -> RetainedUiHostProjection {
    let mut parent = test_host_node("Panel", "panel", None, []);
    parent.node_id = "component-properties".into();
    parent.control_id = Some("ComponentProperties".into());
    parent.source_path = Some("res://ui/editor/unreal/components/details.zui".into());
    parent.source_node_id = Some("properties".into());
    parent.instance_path = Some(
        serde_json::from_value(serde_json::json!([{
            "sourcePath":"res://ui/editor/unreal/level_editor_window.zui",
            "sourceNodeId":"details"
        }]))
        .unwrap(),
    );
    let mut row = test_host_node(
        "FieldRow",
        "property-row",
        Some("Far Clip"),
        [
            ("inspector_property_item_key", "12"),
            ("inspector_property_field_id", "camera.far_clip"),
        ],
    );
    row.node_id = "component-property-slot-02".into();
    row.control_id = Some("WorkbenchComponentPropertyVirtualRow02".into());
    row.parent_id = Some(parent.node_id.clone());
    row.source_path = Some("res://ui/editor/workbench/data/component_property_row.zui".into());
    row.source_node_id = Some("root".into());
    row.instance_path = parent.instance_path.clone();
    RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "inspector-real-parent".into(),
        nodes: vec![parent, row],
    }
}

#[test]
fn inspector_repeat_projection_copies_real_parent_source_through_semantic_and_geometry_patches() {
    let mut projection = inspector_repeat_parent_projection();
    let initial = to_host_contract_workbench_window_nodes(Some(&projection));
    let row = initial.get(1).unwrap();
    let owner = &projection.nodes[0];
    assert_eq!(row.parent_node_id.as_str(), owner.node_id);
    assert_eq!(
        row.parent_source_path.as_str(),
        owner.source_path.as_deref().unwrap()
    );
    assert_eq!(
        row.parent_source_node_id.as_str(),
        owner.source_node_id.as_deref().unwrap()
    );
    assert_eq!(
        row.parent_instance_path.as_str(),
        serde_json::to_string(owner.instance_path.as_ref().unwrap()).unwrap()
    );
    assert_eq!(row.source_node_id.as_str(), "root");
    assert_eq!(row.inspector_property_item_key.as_str(), "12");
    let mut changed = projection.nodes[1].clone();
    changed.text = Some("Far Clip Updated".into());
    let sparse = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        &projection.document_id,
        &[changed],
        &initial,
        None,
        1.25,
    )
    .unwrap();
    let patched = sparse.get(1).unwrap();
    assert_eq!(patched.text.as_str(), "Far Clip Updated");
    assert_eq!(patched.parent_node_id, row.parent_node_id);
    assert_eq!(patched.parent_source_path, row.parent_source_path);
    assert_eq!(patched.parent_source_node_id, row.parent_source_node_id);
    assert_eq!(patched.parent_instance_path, row.parent_instance_path);
    projection.nodes[1].frame.x = 33.0;
    let geometry = build_host_contract_workbench_window_geometry_patch_at_mount_and_scale(
        &projection,
        &[1],
        &sparse,
        None,
        1.25,
    )
    .unwrap();
    let moved = geometry.nodes.get(1).unwrap();
    assert_eq!(moved.frame.x, 41.25);
    assert_eq!(moved.parent_source_path, row.parent_source_path);
    assert_eq!(moved.parent_source_node_id, row.parent_source_node_id);
    assert_eq!(moved.parent_instance_path, row.parent_instance_path);
    assert_eq!(moved.inspector_property_item_key.as_str(), "12");
}

#[test]
fn inspector_repeat_projection_does_not_invent_missing_parent_source() {
    let mut projection = inspector_repeat_parent_projection();
    projection.nodes[0].source_node_id = None;
    let nodes = to_host_contract_workbench_window_nodes(Some(&projection));
    let row = nodes.get(1).unwrap();
    assert!(row.parent_source_node_id.is_empty());
    assert_eq!(row.source_node_id.as_str(), "root");
    projection.nodes.remove(0);
    let nodes = to_host_contract_workbench_window_nodes(Some(&projection));
    let row = nodes.get(0).unwrap();
    assert!(row.parent_node_id.is_empty());
    assert!(row.parent_source_path.is_empty());
    assert!(row.parent_source_node_id.is_empty());
    assert!(row.parent_instance_path.is_empty());
    assert_eq!(row.inspector_property_item_key.as_str(), "12");
}

#[test]
fn workbench_projection_uses_a_memoized_node_index() {
    let source = include_str!("../../windows.rs");

    assert!(source.contains("mod node_index;"));
    assert!(source.contains("ProjectionNodeIndex::new"));
    assert!(!source.contains("fn host_projection_node_render_visible("));
    assert!(!source.contains("fn projected_parent_node_id("));
}

#[test]
fn workbench_projection_keeps_property_and_style_domains_in_leaf_owners() {
    let source = include_str!("../../windows.rs");

    for module in ["defaults", "properties", "selection_style", "status_right"] {
        assert!(
            source.contains(&format!("mod {module};")),
            "window projection must retain the {module} leaf owner"
        );
    }
    for root_implementation in [
        "fn normalized_percent(",
        "fn default_workbench_surface_variant(",
        "fn normalize_workbench_selection_control_style_values(",
        "fn inherited_status_right_numeric_property(",
    ] {
        assert!(
            !source.contains(root_implementation),
            "window projection root must not reclaim {root_implementation}"
        );
    }
}

#[test]
fn workbench_projection_memoizes_collapsed_ancestor_visibility() {
    let mut parent = test_host_node("Panel", "panel", None, []);
    parent.node_id = "collapsed-parent".to_string();
    parent.properties.insert(
        "visibility".to_string(),
        RetainedUiHostValue::String("Collapsed".to_string()),
    );
    let mut child = test_host_node("Label", "label", Some("hidden"), []);
    child.node_id = "hidden-child".to_string();
    child.parent_id = Some(parent.node_id.clone());

    let node_index = ProjectionNodeIndex::new([&parent, &child]);

    assert!(!node_index.render_visible(&parent));
    assert!(!node_index.render_visible(&child));
}

#[test]
fn product_bottom_drawer_projection_keeps_geometry_anchors_and_filters_retired_showcase() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let contains_control = |control_id: &str| {
        nodes
            .iter()
            .any(|node| node.control_id.as_str() == control_id)
    };

    for control_id in [
        "BottomDrawerShellRoot",
        "BottomDrawerHeaderRoot",
        "BottomDrawerContentRoot",
    ] {
        assert!(
            contains_control(control_id),
            "host geometry anchor `{control_id}` must remain projected"
        );
    }
    for control_id in [
        "WorkbenchComponentDrawer",
        "WorkbenchDrawerTabComponents",
        "WorkbenchDrawerTabConsole",
        "WorkbenchComponentDrawerBody",
        "WorkbenchComponentDrawerConsoleBody",
        "WorkbenchConsoleInfo",
        "WorkbenchConsoleWarning",
        "WorkbenchConsoleOk",
    ] {
        assert!(
            !contains_control(control_id),
            "retired product showcase control `{control_id}` must not enter native paint or hit indices"
        );
    }
}

#[test]
fn workbench_projection_treats_parent_cycles_as_not_render_visible() {
    let mut first = test_host_node("Panel", "panel", None, []);
    first.node_id = "cycle-first".to_string();
    first.parent_id = Some("cycle-second".to_string());
    let mut second = test_host_node("Panel", "panel", None, []);
    second.node_id = "cycle-second".to_string();
    second.parent_id = Some("cycle-first".to_string());

    let node_index = ProjectionNodeIndex::new([&first, &second]);

    assert!(!node_index.render_visible(&first));
    assert!(!node_index.render_visible(&second));
}

#[test]
fn status_right_inheritance_rejects_parent_cycles() {
    let mut status = test_host_node("Label", "label", None, []);
    status.node_id = "status-grid".to_string();
    status.control_id = Some("WorkbenchStatusGrid".to_string());
    status.parent_id = Some("cycle-first".to_string());
    let mut first = test_host_node("Panel", "panel", None, []);
    first.node_id = "cycle-first".to_string();
    first.parent_id = Some("cycle-second".to_string());
    let mut second = test_host_node("Panel", "panel", None, []);
    second.node_id = "cycle-second".to_string();
    second.parent_id = Some(first.node_id.clone());

    let node_index = ProjectionNodeIndex::new([&status, &first, &second]);

    assert_eq!(
        status_right::inherited_status_right_numeric_property(
            &status,
            &node_index,
            "status_right_offset_y",
        ),
        Some(-0.5),
    );
}

#[test]
fn mounted_workbench_projection_translates_frames_clips_and_popup_anchors() {
    let mut node = test_host_node("Panel", "panel", None, []);
    node.frame = UiFrame::new(8.0, 12.0, 80.0, 40.0);
    node.clip_frame = Some(UiFrame::new(10.0, 14.0, 60.0, 20.0));
    node.has_popup_anchor = true;
    node.popup_anchor_x = 24.0;
    node.popup_anchor_y = 36.0;
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "mounted-workbench-test".to_string(),
        nodes: vec![node],
    };

    let mounted = to_host_contract_workbench_window_nodes_with_previous_at_mount(
        Some(&projection),
        None,
        Some(UiFrame::new(0.0, 57.0, 320.0, 180.0)),
    );
    let node = mounted.get(0).expect("mounted workbench node");

    assert_eq!(node.frame.y, 69.0);
    assert_eq!(node.clip_frame.y, 71.0);
    assert_eq!(node.popup_anchor_x, 24.0);
    assert_eq!(node.popup_anchor_y, 93.0);
}

#[test]
fn scaled_full_and_sparse_workbench_projection_share_one_physical_boundary() {
    let mut node = test_host_node("Button", "button", Some("Run"), []);
    node.node_id = "run-button".to_string();
    node.control_id = Some("WorkbenchRunButton".to_string());
    node.frame = UiFrame::new(8.0, 12.0, 80.0, 24.0);
    node.properties
        .insert("font_size".to_string(), RetainedUiHostValue::Float(12.0));
    node.properties
        .insert("corner_radius".to_string(), RetainedUiHostValue::Float(3.0));
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "scaled-full-and-sparse".to_string(),
        nodes: vec![node.clone()],
    };
    let mount = Some(UiFrame::new(100.0, 50.0, 640.0, 360.0));

    let initial = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        mount,
        2.0,
    );
    let initial_node = initial.get(0).expect("scaled initial row");
    assert_eq!(initial_node.frame.x, 116.0);
    assert_eq!(initial_node.frame.y, 74.0);
    assert_eq!(initial_node.frame.width, 160.0);
    assert_eq!(initial_node.font_size, 12.0);
    assert_eq!(initial_node.corner_radius, 6.0);

    node.text = Some("Run Project".to_string());
    let patched = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        "scaled-full-and-sparse",
        &[node],
        &initial,
        mount,
        2.0,
    )
    .expect("scaled sparse row patch");
    let patched_node = patched.get(0).expect("scaled patched row");
    assert_eq!(patched_node.frame.x, initial_node.frame.x);
    assert_eq!(patched_node.frame.y, initial_node.frame.y);
    assert_eq!(patched_node.frame.width, initial_node.frame.width);
    assert_eq!(patched_node.font_size, initial_node.font_size);
    assert_eq!(patched_node.corner_radius, initial_node.corner_radius);
    assert_eq!(patched_node.text.as_str(), "Run Project");
}

#[test]
fn scaled_projection_preserves_slider_semantics_and_scales_visual_size_aliases() {
    let mut slider = test_host_node("Slider", "slider", None, []);
    slider.node_id = "range-slider".to_string();
    slider.control_id = Some("WorkbenchInputRangeSlider".to_string());
    slider.properties.extend([
        ("range_min".to_string(), RetainedUiHostValue::Float(28.0)),
        (
            "step_tick_count".to_string(),
            RetainedUiHostValue::Float(5.0),
        ),
        ("value".to_string(), RetainedUiHostValue::Float(0.6)),
    ]);
    let mut visual = test_host_node("StatusMark", "status-mark", None, []);
    visual.node_id = "status-mark".to_string();
    visual.control_id = Some("WorkbenchStatusMark".to_string());
    visual.properties.extend([
        ("dot_size".to_string(), RetainedUiHostValue::Float(7.0)),
        (
            "layout_second_cell_offset_x".to_string(),
            RetainedUiHostValue::Float(12.0),
        ),
        (
            "layout_third_cell_offset_x".to_string(),
            RetainedUiHostValue::Float(15.0),
        ),
    ]);
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "scaled-semantic-and-visual-aliases".to_string(),
        nodes: vec![slider.clone(), visual.clone()],
    };

    let full = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        None,
        2.0,
    );
    let full_slider = full.get(0).expect("scaled slider row");
    let full_visual = full.get(1).expect("scaled visual row");
    assert_eq!(full_slider.layout_second_cell_offset_x, 28.0);
    assert_eq!(full_slider.layout_third_cell_offset_x, 5.0);
    assert_eq!(full_slider.value_number, 0.6);
    assert_eq!(full_visual.layout_second_cell_offset_x, 24.0);
    assert_eq!(full_visual.layout_third_cell_offset_x, 30.0);
    assert_eq!(full_visual.value_number, 14.0);

    slider.text = Some("Range".to_string());
    visual.text = Some("Ready".to_string());
    let sparse = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        "scaled-semantic-and-visual-aliases",
        &[slider, visual],
        &full,
        None,
        2.0,
    )
    .expect("scaled sparse aliases");
    let sparse_slider = sparse.get(0).expect("sparse slider row");
    let sparse_visual = sparse.get(1).expect("sparse visual row");
    assert_eq!(sparse_slider.layout_second_cell_offset_x, 28.0);
    assert_eq!(sparse_slider.layout_third_cell_offset_x, 5.0);
    assert_eq!(sparse_slider.value_number, 0.6);
    assert_eq!(sparse_visual.layout_second_cell_offset_x, 24.0);
    assert_eq!(sparse_visual.layout_third_cell_offset_x, 30.0);
    assert_eq!(sparse_visual.value_number, 14.0);
}

#[test]
fn sparse_workbench_projection_patch_reuses_unchanged_host_rows() {
    let mut first = test_host_node("WorkbenchTreeRow", "tree-row", Some("World"), []);
    first.node_id = "scene-world".to_string();
    first.control_id = Some("WorkbenchSceneRootItem".to_string());
    let mut second = test_host_node("WorkbenchTreeRow", "tree-row", Some("Camera"), []);
    second.node_id = "scene-camera".to_string();
    second.control_id = Some("WorkbenchSceneEnvironmentItem".to_string());
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "sparse-scene-patch".to_string(),
        nodes: vec![first, second.clone()],
    };
    let initial = to_host_contract_workbench_window_nodes(Some(&projection));

    second.text = Some("Gameplay Camera".to_string());
    let patch = build_host_contract_workbench_window_node_patch_at_mount_and_scale(
        "sparse-scene-patch",
        &[second],
        &initial,
        Some(UiFrame::new(0.0, 57.0, 320.0, 180.0)),
        1.0,
    )
    .expect("a same-document scene row should publish a sparse host-model overlay");
    let patched = patch.nodes;

    assert_eq!(patch.changed_rows, vec![1]);
    assert!(initial.shares_row_with(&patched, 0));
    assert!(!initial.shares_row_with(&patched, 1));
    assert_eq!(
        patched.get(1).map(|node| node.text.as_str()),
        Some("Gameplay Camera")
    );
    assert_eq!(patched.get(1).map(|node| node.frame.y), Some(57.0));
}

#[test]
fn geometry_only_workbench_projection_patch_preserves_semantics_and_unmodified_rows() {
    let mut first = test_host_node("WorkbenchTreeRow", "tree-row", Some("World"), []);
    first.node_id = "scene-world".to_string();
    first.control_id = Some("WorkbenchSceneRootItem".to_string());
    let mut second = test_host_node("WorkbenchTreeRow", "tree-row", Some("Camera"), []);
    second.node_id = "scene-camera".to_string();
    second.control_id = Some("WorkbenchSceneEnvironmentItem".to_string());
    second.frame = UiFrame::new(20.0, 30.0, 120.0, 24.0);
    second.clip_frame = Some(UiFrame::new(20.0, 30.0, 100.0, 24.0));
    let mut projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "geometry-only-scene-patch".to_string(),
        nodes: vec![first, second],
    };
    let mount = Some(UiFrame::new(10.0, 40.0, 640.0, 360.0));
    let initial = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        mount,
        2.0,
    );

    projection.nodes[1].frame = UiFrame::new(50.0, 60.0, 160.0, 32.0);
    projection.nodes[1].clip_frame = Some(UiFrame::new(55.0, 65.0, 140.0, 28.0));
    projection.nodes[1].z_index = 17;
    projection.nodes[1].text = Some("must not replace retained semantics".to_string());
    let patch = build_host_contract_workbench_window_geometry_patch_at_mount_and_scale(
        &projection,
        &[1],
        &initial,
        mount,
        2.0,
    )
    .expect("stable workbench rows should accept a geometry-only patch");

    assert_eq!(patch.changed_rows, vec![1]);
    assert!(initial.shares_row_with(&patch.nodes, 0));
    assert!(!initial.shares_row_with(&patch.nodes, 1));
    let patched = patch.nodes.get(1).expect("patched geometry row");
    assert_eq!(patched.text.as_str(), "Camera");
    assert_eq!(patched.frame.x, 110.0);
    assert_eq!(patched.frame.y, 160.0);
    assert_eq!(patched.frame.width, 320.0);
    assert_eq!(patched.clip_frame.x, 120.0);
    assert_eq!(patched.clip_frame.y, 170.0);
    assert_eq!(patched.z_index, 17);
}

#[test]
fn mounted_context_menu_round_trips_the_global_pointer_anchor() {
    let mount_frame = UiFrame::new(11.0, 57.0, 640.0, 400.0);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        mount_frame.width,
        mount_frame.height,
    ))
    .expect("mounted workbench template should project");
    bridge
        .recompute_layout_at_mount(mount_frame)
        .expect("workbench should recompute inside the host mount");
    let owner = bridge
        .host_projection()
        .node_by_control_id("WorkbenchToolbarSave")
        .expect("authored Save owner");
    assert!(!owner.disabled);
    let mut current = Some(owner.surface_node_id.expect("actual runtime owner"));
    while let Some(id) = current {
        let node = bridge.surface().tree.nodes.get(&id).unwrap();
        assert!(node.state_flags.enabled);
        assert!(node.is_render_visible());
        current = node.parent;
    }
    assert!(bridge.control_frame("WorkbenchToolbarSave").is_some());
    let initial_menu = bridge
        .host_projection()
        .node_by_control_id(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("UE context menu must be mounted");
    assert_eq!(
        bridge
            .host_projection()
            .nodes
            .iter()
            .filter(|node| node.control_id.as_deref() == Some(WORKBENCH_CONTEXT_MENU_CONTROL_ID))
            .count(),
        1
    );
    let assert_source = |actual: Option<&str>, relative: &str| {
        let checkout_path = format!("zircon_editor/assets/{relative}");
        let bundle_path = format!("res://{relative}");
        assert!(
            actual == Some(checkout_path.as_str()) || actual == Some(bundle_path.as_str()),
            "unexpected authored source: {actual:?}"
        );
    };
    assert_source(
        initial_menu.source_path.as_deref(),
        "ui/editor/workbench/primitives/feedback/context_menu.zui",
    );
    assert_eq!(initial_menu.source_node_id.as_deref(), Some("root"));
    let instance_path = initial_menu.instance_path.as_ref().unwrap();
    assert_eq!(instance_path.len(), 2);
    for (step, (source_path, source_node_id)) in instance_path.iter().zip([
        ("ui/editor/unreal/level_editor_window.zui", "menu_popups"),
        (
            "ui/editor/unreal/components/menu_popups.zui",
            "context_popup",
        ),
    ]) {
        assert_source(Some(step.source_path.as_str()), source_path);
        assert_eq!(step.source_node_id, source_node_id);
    }
    assert!(!initial_menu.popup_open);
    assert!(!initial_menu.has_popup_anchor);
    for property in ["menu_items", "options", "disabled_options"] {
        assert_eq!(
            initial_menu.properties.get(property),
            Some(&RetainedUiHostValue::Array(Vec::new()))
        );
    }
    let global_anchor = (128.0, 157.0);

    assert!(bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "WorkbenchToolbarSave".into(),
            target_action_id: "file.project.save".into(),
            target_dispatch_kind: "workbench".into(),
            target_role: "icon-button".into(),
            target_value_text: "Save Current Level".into(),
            target_path: "workbench://control/WorkbenchToolbarSave".into(),
            popup_anchor_x: global_anchor.0,
            popup_anchor_y: global_anchor.1,
            menu_items: vec!["Open|icon=folder".into(), "Delete|danger,icon=trash".into()],
        })
        .expect("mounted context menu should open"));

    let local_projection = bridge.host_projection();
    let local_menu = local_projection
        .node_by_control_id(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("local context menu projection");
    assert!(local_menu.popup_open);
    assert!(local_menu.has_popup_anchor);
    let context_menu_node_id = local_menu
        .surface_node_id
        .expect("actual context menu node");
    assert_eq!(
        bridge
            .surface()
            .input
            .popup_anchor_points
            .get(&context_menu_node_id)
            .map(|point| (point.x, point.y)),
        Some((
            global_anchor.0 - mount_frame.x,
            global_anchor.1 - mount_frame.y
        ))
    );
    assert_eq!(
        local_menu.popup_anchor_x,
        f64::from(global_anchor.0 - mount_frame.x)
    );
    assert_eq!(
        local_menu.popup_anchor_y,
        f64::from(global_anchor.1 - mount_frame.y)
    );

    let mounted = to_host_contract_workbench_window_nodes_with_previous_at_mount(
        Some(local_projection),
        None,
        Some(mount_frame),
    );
    let mounted_menu = (0..mounted.row_count())
        .filter_map(|row| mounted.row_data(row))
        .find(|node| node.control_id.as_str() == WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("mounted context menu projection");
    assert_eq!(mounted_menu.popup_anchor_x, global_anchor.0);
    assert_eq!(mounted_menu.popup_anchor_y, global_anchor.1);
    assert!(bridge
        .close_popup(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("context menu should close"));
    assert!(!bridge.control_bool_for_test(WORKBENCH_CONTEXT_MENU_CONTROL_ID, "popup_open"));
    assert!(!bridge.control_bool_for_test(WORKBENCH_CONTEXT_MENU_CONTROL_ID, "open"));
    let closed_menu = bridge
        .host_projection()
        .node_by_control_id(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .unwrap();
    assert!(!closed_menu.popup_open);
    assert!(!closed_menu.has_popup_anchor);
    assert!(!closed_menu.properties.contains_key("popup_anchor_x"));
    assert!(!closed_menu.properties.contains_key("popup_anchor_y"));
    assert!(!bridge
        .surface()
        .input
        .popup_anchor_points
        .contains_key(&context_menu_node_id));
}

#[test]
fn notification_rows_are_reused_only_while_the_complete_cache_key_matches() {
    let mut notification = test_host_node("NotificationCenter", "notification-center", None, []);
    notification.properties.extend([
        (
            "notification_generation".to_string(),
            RetainedUiHostValue::Integer(7),
        ),
        ("unread_count".to_string(), RetainedUiHostValue::Integer(2)),
        (
            "overflow_count".to_string(),
            RetainedUiHostValue::Integer(3),
        ),
        (
            "selected_notification_id".to_string(),
            RetainedUiHostValue::String("row-0".to_string()),
        ),
        ("focused_index".to_string(), RetainedUiHostValue::Integer(0)),
        ("visible_limit".to_string(), RetainedUiHostValue::Integer(2)),
        (
            "notifications".to_string(),
            RetainedUiHostValue::Array(vec![
                RetainedUiHostValue::String("row-0|title=First".to_string()),
                RetainedUiHostValue::String("row-1|title=Second".to_string()),
                RetainedUiHostValue::String("row-2|title=Offscreen".to_string()),
            ]),
        ),
    ]);
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "notification-cache-test".to_string(),
        nodes: vec![notification],
    };

    let first = to_host_contract_workbench_window_nodes(Some(&projection));
    reset_notification_text_copy_count();
    let second =
        to_host_contract_workbench_window_nodes_with_previous(Some(&projection), Some(&first));
    let first_node = first.get(0).expect("first projected notification node");
    let second_node = second.get(0).expect("reused projected notification node");

    assert!(first_node.options.shares_values_with(&second_node.options));
    assert!(Rc::ptr_eq(
        &first_node.options_text,
        &second_node.options_text
    ));
    assert!(first_node
        .structured_options
        .shares_values_with(&second_node.structured_options));
    assert_eq!(notification_text_copy_count(), 0);

    let mut other_document_projection = projection.clone();
    other_document_projection.document_id = "other-workbench-document".to_string();
    reset_notification_text_copy_count();
    let other_document = to_host_contract_workbench_window_nodes_with_previous(
        Some(&other_document_projection),
        Some(&second),
    );
    let other_document_node = other_document
        .get(0)
        .expect("other-document projected notification node");
    assert!(!second_node
        .structured_options
        .shares_values_with(&other_document_node.structured_options));
    assert_eq!(notification_text_copy_count(), 3);

    let mut changed_projection = projection;
    changed_projection.nodes[0].properties.insert(
        "notification_generation".to_string(),
        RetainedUiHostValue::Integer(8),
    );
    reset_notification_text_copy_count();
    let changed = to_host_contract_workbench_window_nodes_with_previous(
        Some(&changed_projection),
        Some(&second),
    );
    let changed_node = changed.get(0).expect("changed projected notification node");

    assert!(!second_node
        .options
        .shares_values_with(&changed_node.options));
    assert!(!second_node
        .structured_options
        .shares_values_with(&changed_node.structured_options));
    assert_eq!(notification_text_copy_count(), 3);
}

#[test]
fn workbench_button_text_prefers_authored_label_over_value_render_text() {
    let node = test_host_node(
        "Button",
        "button",
        Some("thumbnail"),
        [("text", "Thumb"), ("value", "thumbnail")],
    );

    assert_eq!(projected_workbench_text(&node, "button"), "Thumb");
}

#[test]
fn workbench_input_text_keeps_rendered_value_display_semantics() {
    let node = test_host_node(
        "TextField",
        "input-field",
        Some("albedo"),
        [("text", "Search"), ("value", "albedo")],
    );

    assert_eq!(projected_workbench_text(&node, "input-field"), "albedo");
}

#[test]
fn search_field_projection_preserves_placeholder_and_text_input_identity() {
    let node = test_host_node(
        "SearchField",
        "search-field",
        None,
        [("placeholder", "Search samples"), ("query", "")],
    );
    let node_index = ProjectionNodeIndex::new([&node]);

    let projected = to_host_contract_workbench_window_node(&node, &node_index)
        .expect("SearchField should project into the native host contract");

    assert_eq!(projected.role.as_str(), "SearchField");
    assert_eq!(projected.component_role.as_str(), "search-field");
    assert_eq!(projected.text.as_str(), "Search samples");
    assert_eq!(projected.surface_variant.as_str(), "inset");
    assert_eq!(projected.border_width, 1.0);
    assert_eq!(projected.corner_radius, 5.0);
}

#[test]
fn workbench_segmented_control_projects_selected_value_text() {
    let node = test_host_node(
        "SegmentedControl",
        "segmented-control",
        None,
        [("value", "grid")],
    );

    assert_eq!(
        projected_workbench_value_text(&node, "segmented-control", &BTreeMap::new()),
        "grid"
    );
}

#[test]
fn workbench_property_row_projects_authored_value_into_runtime_text_contract() {
    let node = test_host_node(
        "PropertyRow",
        "property-row",
        Some("Horizontal"),
        [("value", "Speed 0 - 620")],
    );
    let node_index = ProjectionNodeIndex::new([&node]);

    assert_eq!(
        projected_workbench_value_text(&node, "property-row", &BTreeMap::new()),
        "Speed 0 - 620"
    );
    let projected = to_host_contract_workbench_window_node(&node, &node_index)
        .expect("PropertyRow should project into the native host contract");
    assert_eq!(projected.text.as_str(), "Horizontal");
    assert_eq!(projected.value_text.as_str(), "Speed 0 - 620");
}

#[test]
fn workbench_node_projection_preserves_retained_parent_identity() {
    let mut parent = test_host_node("Panel", "panel", None, []);
    parent.node_id = "generated-parent-17".to_string();
    let mut node = test_host_node("Panel", "panel", None, []);
    node.parent_id = Some(parent.node_id.clone());
    let node_index = ProjectionNodeIndex::new([&node, &parent]);

    let projected = to_host_contract_workbench_window_node(&node, &node_index)
        .expect("a controlled retained node should project into the host contract");

    assert_eq!(projected.parent_node_id.as_str(), "generated-parent-17");
}

#[test]
fn workbench_node_projection_skips_control_less_component_expansion_parents() {
    let mut host = test_host_node("Panel", "panel", None, []);
    host.node_id = "extension-workspaces-host".to_string();
    let mut component_expansion = test_host_node("VerticalGroup", "layout", None, []);
    component_expansion.node_id = "generated-layout-wrapper".to_string();
    component_expansion.control_id = None;
    component_expansion.parent_id = Some(host.node_id.clone());
    let mut leaf = test_host_node("Label", "label", None, []);
    leaf.node_id = "generated-caption".to_string();
    leaf.parent_id = Some(component_expansion.node_id.clone());
    let node_index = ProjectionNodeIndex::new([&host, &component_expansion, &leaf]);

    let projected = to_host_contract_workbench_window_node(&leaf, &node_index)
        .expect("a controlled leaf should project into the host contract");

    assert_eq!(
        projected.parent_node_id.as_str(),
        "extension-workspaces-host"
    );
}

#[test]
fn agent_chat_projection_routes_messages_and_composer_text_to_native_fields() {
    let mut agent = test_host_node("AgentChat", "mui-x-agent-chat", None, []);
    agent.properties.insert(
        "messages".to_string(),
        RetainedUiHostValue::Array(vec![
            RetainedUiHostValue::String("user|Question".to_string()),
            RetainedUiHostValue::String("agent|Answer".to_string()),
        ]),
    );
    let mut composer = test_host_node("ChatComposer", "mui-x-chat-composer", None, []);
    composer.properties.insert(
        "composer_text".to_string(),
        RetainedUiHostValue::String("Continue the review".to_string()),
    );
    let node_index = ProjectionNodeIndex::new([&agent, &composer]);

    let projected_agent = to_host_contract_workbench_window_node(&agent, &node_index)
        .expect("AgentChat should project into the native host contract");
    let projected_composer = to_host_contract_workbench_window_node(&composer, &node_index)
        .expect("ChatComposer should project into the native host contract");

    let messages = projected_agent
        .collection_items
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    assert_eq!(messages, ["user|Question", "agent|Answer"]);
    assert_eq!(
        projected_composer.value_text.as_str(),
        "Continue the review"
    );
}

fn test_host_node<const N: usize>(
    component: &str,
    component_role: &str,
    text: Option<&str>,
    properties: [(&str, &str); N],
) -> RetainedUiHostNodeModel {
    RetainedUiHostNodeModel {
        source_path: None,
        source_node_id: None,
        instance_path: None,
        source_surface_frame: None,
        node_id: "test-node".to_string(),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        kind: RetainedUiHostComponentKind::from_component(component),
        component: component.to_string(),
        control_id: Some("test-control".to_string()),
        frame: UiFrame::new(0.0, 0.0, 100.0, 24.0),
        clip_frame: None,
        z_index: 0,
        text: text.map(str::to_string),
        icon: None,
        component_role: Some(component_role.to_string()),
        value_text: None,
        validation_level: None,
        validation_message: None,
        popup_open: false,
        has_popup_anchor: false,
        popup_anchor_x: 0.0,
        popup_anchor_y: 0.0,
        selection_state: None,
        options_text: None,
        options: Vec::new(),
        collection_items: Vec::new(),
        menu_items: Vec::new(),
        accepted_drag_payloads: Vec::new(),
        drop_source_summary: None,
        checked: false,
        expanded: false,
        focused: false,
        focus_visible: false,
        focus_visible_known: false,
        tree_owner_focused: None,
        hovered: false,
        pressed: false,
        dragging: false,
        drop_hovered: false,
        active_drag_target: false,
        disabled: false,
        properties: properties
            .into_iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    RetainedUiHostValue::String(value.to_string()),
                )
            })
            .collect(),
        style_tokens: BTreeMap::new(),
        routes: Vec::new(),
    }
}

#[test]
fn full_and_sparse_projection_preserve_valid_logical_paint_padding_presence() {
    let mut node = test_host_node("Button", "button", Some("File"), []);
    node.node_id = "authored-menu".into();
    node.properties.extend([
        (
            "layout_padding_left".into(),
            RetainedUiHostValue::Float(0.0),
        ),
        (
            "layout_padding_right".into(),
            RetainedUiHostValue::Float(12.0),
        ),
        (
            "layout_padding_top".into(),
            RetainedUiHostValue::Float(-1.0),
        ),
    ]);
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "authored-menu-insets".into(),
        nodes: vec![node.clone()],
    };
    let full = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        None,
        1.5,
    );
    let first = full.get(0).unwrap();
    assert_eq!(
        first.authored_field_padding,
        [Some(0.0), Some(12.0), None, None]
    );
    assert_eq!(first.layout_padding_right, 18.0);
    node.properties.insert(
        "layout_padding_right".into(),
        RetainedUiHostValue::Float(7.0),
    );
    let sparse = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        "authored-menu-insets",
        &[node],
        &full,
        None,
        1.5,
    )
    .unwrap();
    let patched = sparse.get(0).unwrap();
    assert_eq!(
        patched.authored_field_padding,
        [Some(0.0), Some(7.0), None, None]
    );
    assert_eq!(patched.layout_padding_right, 10.5);
}

#[test]
fn labeled_field_projection_keeps_authored_name_and_logical_width_at_dpi() {
    let mut node = test_host_node(
        "InputField",
        "input-field",
        Some("200"),
        [("text", "Far"), ("value_text", "200")],
    );
    node.properties.insert(
        "layout_label_width".into(),
        RetainedUiHostValue::Float(92.0),
    );
    assert_eq!(projected_workbench_text(&node, "input-field"), "Far");
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "labeled-property".into(),
        nodes: vec![node.clone()],
    };
    let nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        None,
        1.5,
    );
    let data = nodes.get(0).unwrap();
    assert_eq!(data.text.as_str(), "Far");
    assert_eq!(data.value_text.as_str(), "200");
    assert_eq!(
        data.authored_field_label_width,
        Some(92.0),
        "authored label width remains logical"
    );
    node.properties
        .insert("layout_label_width".into(), RetainedUiHostValue::Float(0.0));
    assert_eq!(
        projected_workbench_text(&node, "input-field"),
        "200",
        "plain fields keep existing display semantics"
    );
}

#[test]
fn icon_button_with_toggle_name_preserves_resolved_selected_colors() {
    let mut node = test_host_node(
        "IconButton",
        "icon-button",
        None,
        [
            ("background_color", "#0070E0"),
            ("border_color", "#0070E0"),
            ("foreground_color", "#FFFFFF"),
        ],
    );
    node.control_id = Some("ToggleTranslateSnap".into());
    node.checked = true;
    node.properties
        .insert("selected".into(), RetainedUiHostValue::Bool(true));
    node.frame = UiFrame::new(8.0, 8.0, 32.0, 32.0);
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "semantic-selected-icon".into(),
        nodes: vec![node.clone()],
    };
    let full = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        None,
        1.25,
    );
    let sparse = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        &projection.document_id,
        &[node],
        &full,
        None,
        1.25,
    )
    .unwrap();
    for nodes in [&full, &sparse] {
        let selected = nodes.get(0).unwrap();
        assert!(selected.checked && selected.selected);
        assert_eq!(
            selected.button_style.element.background_color,
            Some(test_style_rgba([0, 112, 224, 255]))
        );
        assert_eq!(
            selected.button_style.element.border_color,
            Some(test_style_rgba([0, 112, 224, 255]))
        );
        assert_eq!(
            selected.button_style.element.foreground_color,
            Some(test_style_rgba([255, 255, 255, 255]))
        );
        let pixels = host_contract::paint_template_nodes_for_test(72, 72, (*nodes).clone());
        let center = ((30 * 72 + 30) * 4) as usize;
        assert_eq!(&pixels[center..center + 4], &[0, 112, 224, 255]);
    }
}

#[test]
fn toggle_role_preserves_legacy_selected_fallback_without_name_inference() {
    for (component, role) in [
        ("Toggle", "toggle"),
        ("Switch", "switch"),
        ("WorkbenchToggle", ""),
        ("WorkbenchSwitch", ""),
    ] {
        let mut node = test_host_node(
            component,
            role,
            None,
            [
                ("background_color", "#0070E0"),
                ("border_color", "#0070E0"),
                ("foreground_color", "#FFFFFF"),
            ],
        );
        node.control_id = Some("InputSelection".into());
        node.checked = true;
        let projection = RetainedUiHostProjection {
            source_surface_frame: None,
            document_id: "legacy-selection-fallback".into(),
            nodes: vec![node],
        };
        let nodes = to_host_contract_workbench_window_nodes(Some(&projection));
        let selected = nodes.get(0).unwrap();
        assert_eq!(
            selected.button_style.element.background_color,
            Some(test_style_rgba([23, 57, 66, 255])),
            "{component}/{role} keeps the existing true Toggle fallback"
        );
        assert_eq!(
            selected.button_style.element.border_color,
            Some(test_style_rgba([65, 75, 84, 255]))
        );
        assert_eq!(
            selected.button_style.element.foreground_color,
            Some(test_style_rgba([164, 174, 180, 255]))
        );
    }
}

#[test]
fn dropdown_projection_mounts_default_and_explicit_zero_border_once() {
    for (authored_width, physical_width) in [(None, 1.25), (Some(0.0), 0.0), (Some(1.0), 1.25)] {
        let mut node = test_host_node("Dropdown", "dropdown", Some("Perspective"), []);
        node.control_id = Some("ProjectionChooser".into());
        if let Some(width) = authored_width {
            node.properties
                .insert("border_width".into(), RetainedUiHostValue::Float(width));
        }
        let projection = RetainedUiHostProjection {
            source_surface_frame: None,
            document_id: "dropdown-border-mount".into(),
            nodes: vec![node],
        };
        let nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            Some(&projection),
            None,
            None,
            1.25,
        );
        let mounted = nodes.get(0).unwrap();
        assert_eq!(mounted.border_width, physical_width);
        assert_eq!(mounted.frame.width, 125.0);
        assert_eq!(mounted.frame.height, 30.0);
        let commands = host_contract::template_node_command_summary_for_test(mounted);
        assert_eq!(commands.image_frames.len(), 1);
    }
}

#[test]
fn disabled_field_projection_preserves_state_colors_through_full_sparse_and_paint_paths() {
    let mut node = test_host_node(
        "SearchField",
        "search-field",
        Some("Search"),
        [
            ("background_color", "#0F0F0F"),
            ("foreground_color", "#C0C0C0"),
            ("focus_border_color", "#0070E0"),
            ("disabled_background_color", "#242424"),
            ("disabled_foreground_color", "#808080"),
        ],
    );
    node.disabled = true;
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "disabled-authored-field".into(),
        nodes: vec![node.clone()],
    };
    let full = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
        Some(&projection),
        None,
        None,
        1.25,
    );
    let field = full.get(0).unwrap();
    assert!(field.disabled && field.hovered && field.focused && field.pressed);
    assert_eq!(
        field.authored_field_colors.get("focus_border_color"),
        Some(&test_style_rgba([0, 112, 224, 255]))
    );
    assert_eq!(
        field.authored_field_colors.get("disabled_foreground_color"),
        Some(&test_style_rgba([128, 128, 128, 255]))
    );
    let pixels = host_contract::paint_template_nodes_for_test(140, 40, full.clone());
    let background_pixel = ((15 * 140 + 108) * 4) as usize;
    assert_eq!(
        &pixels[background_pixel..background_pixel + 4],
        &[36, 36, 36, 255]
    );
    node.properties.insert(
        "disabled_background_color".into(),
        RetainedUiHostValue::String("#162A3E".into()),
    );
    node.properties.insert(
        "disabled_foreground_color".into(),
        RetainedUiHostValue::String("#10203080".into()),
    );
    let sparse = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
        &projection.document_id,
        &[node],
        &full,
        None,
        1.25,
    )
    .unwrap();
    let field = sparse.get(0).unwrap();
    assert_eq!(
        field.authored_field_colors.get("disabled_foreground_color"),
        Some(&test_style_rgba([16, 32, 48, 128]))
    );
    let previous_frame = &full.get(0).unwrap().frame;
    assert_eq!(field.frame.x, previous_frame.x);
    assert_eq!(field.frame.y, previous_frame.y);
    assert_eq!(field.frame.width, previous_frame.width);
    assert_eq!(field.frame.height, previous_frame.height);
    let pixels = host_contract::paint_template_nodes_for_test(140, 40, sparse);
    assert_eq!(
        &pixels[background_pixel..background_pixel + 4],
        &[22, 42, 62, 255]
    );
}

#[test]
fn disabled_field_projection_keeps_transparent_values_and_absent_state_colors_distinct() {
    let mut node = test_host_node(
        "InputField",
        "input-field",
        None,
        [("disabled_background_color", "transparent")],
    );
    node.disabled = true;
    let projection = RetainedUiHostProjection {
        source_surface_frame: None,
        document_id: "disabled-color-presence".into(),
        nodes: vec![node],
    };
    let declared = to_host_contract_workbench_window_nodes(Some(&projection));
    assert_eq!(
        declared.get(0).unwrap().authored_field_colors,
        BTreeMap::from([(
            "disabled_background_color".into(),
            UiStyleColor::Transparent
        )])
    );
    let mut legacy = projection;
    legacy.nodes[0].properties.clear();
    let missing = to_host_contract_workbench_window_nodes(Some(&legacy));
    assert!(missing.get(0).unwrap().authored_field_colors.is_empty());
}

#[test]
fn layout_only_group_and_space_projection_has_no_implicit_paint() {
    for (component, expected_role) in [
        ("HorizontalGroup", "horizontal-group"),
        ("VerticalGroup", "vertical-group"),
        ("Space", "space"),
        ("ScrollableBox", "scrollable-box"),
    ] {
        for role in [None, Some(String::new()), Some(expected_role.into())] {
            let has_explicit_role = role.as_deref().is_some_and(|value| !value.is_empty());
            let mut node = test_host_node(component, "", None, []);
            node.component_role = role;
            node.frame = UiFrame::new(8.0, 12.0, 100.0, 24.0);
            node.clip_frame = Some(UiFrame::new(10.0, 14.0, 80.0, 20.0));
            let projection = RetainedUiHostProjection {
                source_surface_frame: None,
                document_id: "layout-without-paint".into(),
                nodes: vec![node],
            };
            let nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
                Some(&projection),
                None,
                None,
                1.25,
            );
            let layout = nodes.get(0).unwrap();
            let resolved_role = if component == "ScrollableBox" && !has_explicit_role {
                ""
            } else {
                expected_role
            };
            assert_eq!(layout.component_role.as_str(), resolved_role);
            assert!(layout.surface_variant.is_empty());
            assert_eq!(layout.corner_radius, 0.0);
            assert_eq!(layout.border_width, 0.0);
            assert!(layout.button_style.element.background_color.is_none());
            assert!(layout.button_style.element.border_color.is_none());
            assert_eq!(layout.frame.x, 10.0);
            assert_eq!(layout.frame.y, 15.0);
            assert_eq!(layout.frame.width, 125.0);
            assert_eq!(layout.frame.height, 30.0);
            assert_eq!(layout.clip_frame.x, 12.5);
            assert_eq!(layout.clip_frame.y, 17.5);
            assert_eq!(layout.clip_frame.width, 100.0);
            assert_eq!(layout.clip_frame.height, 25.0);
            if component != "ScrollableBox" || has_explicit_role {
                let pixels = host_contract::paint_template_nodes_for_test(140, 50, nodes.clone());
                assert!(pixels.chunks_exact(4).all(|pixel| pixel == [0, 0, 0, 255]));
            }
        }
    }
}

#[test]
fn group_and_space_projection_preserves_explicit_paint_without_changing_layout() {
    for (component, role) in [
        ("HorizontalGroup", "horizontal-group"),
        ("VerticalGroup", "vertical-group"),
        ("Space", "space"),
        ("ScrollableBox", "scrollable-box"),
    ] {
        let mut node = test_host_node(
            component,
            "",
            None,
            [
                ("background_color", "#123456"),
                ("border_color", "#ABCDEF"),
                ("surface_variant", "panel"),
            ],
        );
        node.component_role = Some(role.into());
        node.properties
            .insert("border_width".into(), RetainedUiHostValue::Float(1.0));
        node.properties
            .insert("corner_radius".into(), RetainedUiHostValue::Float(3.0));
        node.frame = UiFrame::new(8.0, 12.0, 100.0, 24.0);
        node.clip_frame = Some(UiFrame::new(10.0, 14.0, 80.0, 20.0));
        let projection = RetainedUiHostProjection {
            source_surface_frame: None,
            document_id: "explicit-layout-paint".into(),
            nodes: vec![node],
        };
        let nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            Some(&projection),
            None,
            None,
            1.25,
        );
        let painted = nodes.get(0).unwrap();
        assert_eq!(painted.surface_variant.as_str(), "panel");
        assert_eq!(painted.border_width, 1.25);
        assert_eq!(painted.corner_radius, 3.75);
        assert_eq!(
            painted.button_style.element.background_color,
            Some(test_style_rgba([18, 52, 86, 255]))
        );
        assert_eq!(
            painted.button_style.element.border_color,
            Some(test_style_rgba([171, 205, 239, 255]))
        );
        assert_eq!(painted.frame.x, 10.0);
        assert_eq!(painted.frame.y, 15.0);
        assert_eq!(painted.frame.width, 125.0);
        assert_eq!(painted.frame.height, 30.0);
        assert_eq!(painted.clip_frame.x, 12.5);
        assert_eq!(painted.clip_frame.y, 17.5);
        assert_eq!(painted.clip_frame.width, 100.0);
        assert_eq!(painted.clip_frame.height, 25.0);
    }
}

fn test_style_rgba(color: [u8; 4]) -> UiStyleColor {
    UiStyleColor::Rgba(UiRgbaColor::from_u8(color[0], color[1], color[2], color[3]))
}
