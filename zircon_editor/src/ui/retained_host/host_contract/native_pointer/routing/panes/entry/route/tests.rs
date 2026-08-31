use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::console_output::{ConsoleOutputPaintMetadata, ConsoleOutputViewport};
use crate::ui::retained_host::host_contract::data::{
    ConsolePaneData, FloatingWindowData, FrameRect, HostBottomDockSurfaceData,
    HostPaneInteractionStateData, HostWindowPresentationData, PaneData, TemplateNodeFrameData,
    TemplatePaneCollectionRowData, TemplatePaneMenuItemData, TemplatePaneNodeData,
    TemplatePaneOptionData, TemplateV2PaneData,
};
use crate::ui::retained_host::host_contract::surface_hit_test::rebuild_pane_template_hit_artifacts;
use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::super::super::PanePointerTarget;
use super::{route_pointer_scroll_to_pane, route_pointer_to_pane};

#[test]
fn floating_console_occludes_local_scroll_route_outside_its_output_viewport() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 60.0,
        },
        20.0,
        0,
        1,
    )
    .expect("console output metadata");
    let console_nodes = ModelRc::with_metadata(vec![TemplatePaneNodeData::default()], metadata);
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.bottom_dock = HostBottomDockSurfaceData {
        region_frame: frame(0.0, 0.0, 400.0, 300.0),
        content_frame: frame(0.0, 0.0, 400.0, 300.0),
        pane: PaneData {
            kind: "Hierarchy".into(),
            ..PaneData::default()
        },
        ..HostBottomDockSurfaceData::default()
    };
    presentation.host_scene_data.floating_layer.floating_windows =
        model_rc(vec![FloatingWindowData {
            window_id: "floating-console".into(),
            frame: frame(50.0, 50.0, 200.0, 150.0),
            header_frame: frame(0.0, 0.0, 200.0, 30.0),
            active_pane: PaneData {
                kind: "Console".into(),
                console: ConsolePaneData {
                    nodes: console_nodes,
                    output: "message".into(),
                },
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        }]);

    let route = route_pointer_scroll_to_pane(
        &presentation,
        &HostPaneInteractionStateData::default(),
        55.0,
        90.0,
    );

    assert!(route.is_none(), "floating content must consume the miss");
    assert!(!matches!(
        route.map(|route| route.target),
        Some(PanePointerTarget::Hierarchy)
    ));
}

#[test]
fn viewport_routes_borrow_dock_and_floating_surface_identity() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.bottom_dock = HostBottomDockSurfaceData {
        region_frame: frame(0.0, 0.0, 400.0, 300.0),
        content_frame: frame(0.0, 0.0, 400.0, 300.0),
        surface_key: "bottom-scene".into(),
        pane: PaneData {
            kind: "Scene".into(),
            ..PaneData::default()
        },
        ..HostBottomDockSurfaceData::default()
    };
    let interaction = HostPaneInteractionStateData::default();

    let dock_route = route_pointer_to_pane(&presentation, &interaction, 120.0, 100.0)
        .expect("docked viewport route");
    let PanePointerTarget::SceneViewport(dock_surface_key) = dock_route.target else {
        panic!("docked viewport target");
    };
    assert!(std::ptr::eq(
        dock_surface_key,
        presentation
            .host_scene_data
            .bottom_dock
            .surface_key
            .as_str()
    ));

    presentation.host_scene_data.floating_layer.floating_windows =
        model_rc(vec![FloatingWindowData {
            window_id: "floating-scene".into(),
            frame: frame(50.0, 50.0, 200.0, 150.0),
            header_frame: frame(0.0, 0.0, 200.0, 30.0),
            active_pane: PaneData {
                kind: "Scene".into(),
                ..PaneData::default()
            },
            ..FloatingWindowData::default()
        }]);

    let floating_route = route_pointer_to_pane(&presentation, &interaction, 100.0, 100.0)
        .expect("floating viewport route");
    let PanePointerTarget::SceneViewport(floating_surface_key) = floating_route.target else {
        panic!("floating viewport target");
    };
    let floating = presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .get(0)
        .expect("floating source");
    assert!(std::ptr::eq(
        floating_surface_key,
        floating.window_id.as_str()
    ));
}

#[test]
fn pane_pointer_route_borrows_generation_owned_targets_and_materializes_only_for_activation() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 8.0,
            y: 40.0,
            width: 240.0,
            height: 36.0,
        },
        40.0,
        1,
        3,
    )
    .expect("console output metadata");
    let line = |id: &str, y: f32| TemplatePaneNodeData {
        node_id: id.into(),
        control_id: id.into(),
        role: "Label".into(),
        dispatch_kind: "activity_log_jump".into(),
        action_id: format!("open-{id}"),
        binding_id: format!("binding-{id}"),
        value_text: format!("value-{id}"),
        frame: TemplateNodeFrameData {
            x: 72.0,
            y,
            width: 176.0,
            height: 18.0,
        },
        ..TemplatePaneNodeData::default()
    };
    let nodes = ModelRc::with_metadata(
        vec![
            TemplatePaneNodeData::default(),
            line("line-1", 40.0),
            line("line-2", 58.0),
            line("line-3", 76.0),
        ],
        metadata,
    );
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.bottom_dock = HostBottomDockSurfaceData {
        region_frame: frame(0.0, 0.0, 300.0, 150.0),
        content_frame: frame(0.0, 0.0, 300.0, 150.0),
        pane: PaneData {
            id: "editor.console#1".into(),
            kind: "Console".into(),
            console: ConsolePaneData {
                nodes,
                output: "three rows".into(),
            },
            ..PaneData::default()
        },
        ..HostBottomDockSurfaceData::default()
    };
    let interaction = HostPaneInteractionStateData {
        console_scroll_px: 18.0,
        ..HostPaneInteractionStateData::default()
    };

    let route = route_pointer_to_pane(&presentation, &interaction, 180.0, 49.0)
        .expect("scrolled console click route");

    let PanePointerTarget::TemplateNode(hit) = &route.target else {
        panic!("template node route");
    };
    let pane = &presentation.host_scene_data.bottom_dock.pane;
    let source = pane.console.nodes.get(2).expect("line-2 source node");

    assert_eq!(hit.control_id, "line-2");
    assert!(std::ptr::eq(hit.pane_id, pane.id.as_str()));
    assert!(std::ptr::eq(hit.control_id, source.control_id.as_str()));
    assert!(std::ptr::eq(hit.action_id, source.action_id.as_str()));
    assert!(std::ptr::eq(hit.binding_id, source.binding_id.as_str()));
    assert!(std::ptr::eq(
        hit.dispatch_kind,
        source.dispatch_kind.as_str()
    ));
    assert!(std::ptr::eq(hit.value_text, source.value_text.as_str()));

    let owned = hit.to_owned_hit();
    assert_eq!(owned.pane_id, pane.id);
    assert_eq!(owned.control_id, source.control_id);
    assert_eq!(owned.action_id, source.action_id);
    assert_eq!(owned.binding_id, source.binding_id);
    assert_eq!(owned.value_text, source.value_text);
}

#[test]
fn pane_pointer_route_borrows_table_menu_and_option_payloads() {
    let table_presentation = template_presentation(vec![TemplatePaneNodeData {
        node_id: "rows".into(),
        control_id: "GenericRows".into(),
        role: "Table".into(),
        component_role: "table".into(),
        action_id: "rows.select".into(),
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 20.0,
            width: 180.0,
            height: 80.0,
        },
        collection_rows: model_rc(vec![
            table_row(3, "41", "Ground"),
            table_row(9, "73", "Roof"),
        ]),
        ..TemplatePaneNodeData::default()
    }]);
    let table_route = route_pointer_to_pane(
        &table_presentation,
        &HostPaneInteractionStateData::default(),
        24.0,
        88.0,
    )
    .expect("table row route");
    let PanePointerTarget::TemplateNode(table_hit) = &table_route.target else {
        panic!("table template target");
    };
    let table_node = table_presentation
        .host_scene_data
        .bottom_dock
        .pane
        .template_v2
        .nodes
        .get(0)
        .expect("table node");
    let selected_row = table_node.collection_rows.get(1).expect("selected row");
    assert_eq!(table_hit.table_row_source_index, Some(9));
    assert!(std::ptr::eq(
        table_hit.table_row_identity_kind,
        selected_row.identity_kind.as_str()
    ));
    assert!(std::ptr::eq(
        table_hit.table_row_identity_text,
        selected_row.identity_text.as_str()
    ));
    let table_owned = table_hit.to_owned_hit();
    assert_eq!(table_owned.table_row_source_index, Some(9));
    assert_eq!(
        table_owned.table_row_identity_kind,
        selected_row.identity_kind
    );
    assert_eq!(
        table_owned.table_row_identity_text,
        selected_row.identity_text
    );

    let menu_presentation = template_presentation(vec![TemplatePaneNodeData {
        node_id: "menu".into(),
        control_id: "PanePopupMenu".into(),
        role: "Menu".into(),
        component_role: "menu".into(),
        action_id: "menu.open".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 20.0,
            width: 140.0,
            height: 120.0,
        },
        structured_menu_items: model_rc(vec![TemplatePaneMenuItemData {
            action_id: "delete".into(),
            label: "Delete".into(),
            ..TemplatePaneMenuItemData::default()
        }]),
        ..TemplatePaneNodeData::default()
    }]);
    let menu_route = route_pointer_to_pane(
        &menu_presentation,
        &HostPaneInteractionStateData::default(),
        24.0,
        30.0,
    )
    .expect("menu row route");
    let PanePointerTarget::TemplateNode(menu_hit) = &menu_route.target else {
        panic!("menu template target");
    };
    let menu_node = menu_presentation
        .host_scene_data
        .bottom_dock
        .pane
        .template_v2
        .nodes
        .get(0)
        .expect("menu node");
    let menu_item = menu_node.structured_menu_items.get(0).expect("menu item");
    assert!(std::ptr::eq(
        menu_hit.action_id,
        menu_item.action_id.as_str()
    ));
    assert!(std::ptr::eq(menu_hit.value_text, menu_item.label.as_str()));
    let menu_owned = menu_hit.to_owned_hit();
    assert_eq!(menu_owned.action_id, "menu.item.delete");
    assert_eq!(menu_owned.value_text, menu_item.label);

    let option_presentation = template_presentation(vec![TemplatePaneNodeData {
        node_id: "dropdown".into(),
        control_id: "PaneDropdown".into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        dispatch_kind: "asset:browser".into(),
        edit_action_id: "pane.dropdown.select".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 20.0,
            width: 120.0,
            height: 32.0,
        },
        structured_options: model_rc(vec![
            option("dropdown", false),
            option("option_a", false),
            option("option_b", true),
        ]),
        ..TemplatePaneNodeData::default()
    }]);
    let option_route = route_pointer_to_pane(
        &option_presentation,
        &HostPaneInteractionStateData::default(),
        24.0,
        96.0,
    )
    .expect("option row route");
    let PanePointerTarget::TemplateNode(option_hit) = &option_route.target else {
        panic!("option template target");
    };
    let option_node = option_presentation
        .host_scene_data
        .bottom_dock
        .pane
        .template_v2
        .nodes
        .get(0)
        .expect("option node");
    let selected_option = option_node
        .structured_options
        .get(1)
        .expect("selected option");
    assert!(std::ptr::eq(
        option_hit.action_id,
        option_node.edit_action_id.as_str()
    ));
    assert!(std::ptr::eq(
        option_hit.value_text,
        selected_option.id.as_str()
    ));
    let option_owned = option_hit.to_owned_hit();
    assert_eq!(option_owned.dispatch_kind, "asset:browser");
    assert_eq!(option_owned.action_id, option_node.edit_action_id);
    assert_eq!(option_owned.value_text, selected_option.id);
}

fn template_presentation(nodes: Vec<TemplatePaneNodeData>) -> HostWindowPresentationData {
    let mut pane = PaneData {
        id: "editor.template#1".into(),
        kind: "TemplateV2".into(),
        template_v2: TemplateV2PaneData {
            nodes: model_rc(nodes),
        },
        ..PaneData::default()
    };
    rebuild_pane_template_hit_artifacts(&mut pane, UiSize::new(200.0, 200.0));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.bottom_dock = HostBottomDockSurfaceData {
        region_frame: frame(0.0, 0.0, 200.0, 200.0),
        content_frame: frame(0.0, 0.0, 200.0, 200.0),
        pane,
        ..HostBottomDockSurfaceData::default()
    };
    presentation
}

fn table_row(source_index: i32, identity_text: &str, label: &str) -> TemplatePaneCollectionRowData {
    TemplatePaneCollectionRowData {
        source_index,
        row_identity_field: "surface_entity".into(),
        identity_kind: "integer".into(),
        identity_text: identity_text.into(),
        label: label.into(),
    }
}

fn option(id: &str, disabled: bool) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: id.into(),
        disabled,
        ..TemplatePaneOptionData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
