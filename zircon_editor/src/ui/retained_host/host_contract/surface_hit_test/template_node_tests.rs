use std::rc::Rc;

use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::console_output::{ConsoleOutputPaintMetadata, ConsoleOutputViewport};
use crate::ui::retained_host::host_contract::data::{
    ConsolePaneData, FrameRect, HostWindowPresentationData, PaneData, TemplateNodeFrameData,
    TemplatePaneCollectionRowData, TemplatePaneMenuItemData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::host_contract::template_geometry::template_nodes_bounds;
use crate::ui::retained_host::host_contract::template_popup_layout::template_option_row_frame_within;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::to_host_contract_workbench_window_nodes;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::layout::UiSize;

use super::surface_frame_builder::{
    reset_template_surface_frame_build_count, template_surface_frame_build_count,
};
use super::{
    hit_test_pane_template_node, hit_test_workbench_window_template_node_with_index,
    rebuild_pane_template_hit_artifacts, HostWorkbenchHitIndex, TemplateNodePointerHit,
};

#[test]
fn pane_hit_test_skips_the_full_popup_scan_when_no_popup_is_open() {
    let nodes = (0..10_000)
        .map(|row| TemplatePaneNodeData {
            node_id: format!("node-{row}").into(),
            control_id: format!("Control{row}").into(),
            action_id: format!("action.{row}").into(),
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: row as f32 * 28.0,
                width: 120.0,
                height: 20.0,
            },
            ..TemplatePaneNodeData::default()
        })
        .collect();
    let mut pane = PaneData {
        id: "editor.scale#1".into(),
        kind: "TemplateV2".into(),
        template_v2: crate::ui::retained_host::host_contract::data::TemplateV2PaneData {
            nodes: model(nodes),
        },
        ..PaneData::default()
    };
    rebuild_pane_template_hit_artifacts(&mut pane, UiSize::new(160.0, 280_000.0));
    let index = pane
        .body_template_hit_index
        .as_ref()
        .expect("pane hit index should be built")
        .clone();
    let body = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 280_000.0,
    };

    let hit = hit_test_pane_template_node(&pane, &body, 16.0, 279_980.0, 0.0)
        .expect("the final pane control should remain hit-testable");

    assert_eq!(hit.control_id.as_str(), "Control9999");
    assert_eq!(index.query_count_for_test(), 1);
    assert_eq!(index.last_popup_candidate_visit_count_for_test(), 0);
}

fn hit_test_workbench_window_template_node(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    hit_test_workbench_window_template_node_with_index(presentation, &index, x, y)
}

#[test]
fn console_pane_hit_test_uses_scrolled_line_geometry() {
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
    let nodes = ModelRc::with_metadata(
        vec![
            TemplatePaneNodeData {
                node_id: "source-filter".into(),
                control_id: "ConsoleSourceAll".into(),
                role: "Button".into(),
                action_id: "workbench.console.source.all".into(),
                frame: TemplateNodeFrameData {
                    x: 8.0,
                    y: 8.0,
                    width: 120.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            console_jump_node("line-1", "workbench.activity_log.jump.1", 40.0),
            console_jump_node("line-2", "workbench.activity_log.jump.2", 58.0),
            console_jump_node("line-3", "workbench.activity_log.jump.3", 76.0),
        ],
        metadata,
    );
    let pane = PaneData {
        id: "editor.console#1".into(),
        kind: "Console".into(),
        console: ConsolePaneData {
            nodes,
            status_text: "three activity rows".into(),
        },
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 260.0,
        height: 100.0,
    };

    let hit = hit_test_pane_template_node(&pane, &body, 120.0, 99.0, 18.0)
        .expect("the second row should move into the first visible slot");

    assert_eq!(hit.action_id.as_str(), "workbench.activity_log.jump.2");
    assert_eq!(hit.frame.y, 90.0);

    assert!(
        hit_test_pane_template_node(&pane, &body, 200.0, 130.0, 18.0).is_none(),
        "a raw log row below the clipped viewport must not remain dispatchable"
    );
    let header = hit_test_pane_template_node(&pane, &body, 120.0, 65.0, 18.0)
        .expect("non-log controls outside the output viewport must remain dispatchable");
    assert_eq!(header.control_id.as_str(), "ConsoleSourceAll");
}

#[test]
fn console_popup_rows_take_priority_over_scrolled_log_rows() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 8.0,
            y: 40.0,
            width: 240.0,
            height: 36.0,
        },
        40.0,
        1,
        1,
    )
    .expect("console output metadata");
    let nodes = ModelRc::with_metadata(
        vec![
            TemplatePaneNodeData {
                node_id: "source-filter".into(),
                control_id: "ConsoleSourceFilter".into(),
                role: "Dropdown".into(),
                component_role: "dropdown".into(),
                edit_action_id: "workbench.console.source.select".into(),
                popup_open: true,
                structured_options: model(vec![option("all", false), option("runtime", false)]),
                frame: TemplateNodeFrameData {
                    x: 8.0,
                    y: 8.0,
                    width: 120.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            console_jump_node("line-2", "workbench.activity_log.jump.2", 58.0),
        ],
        metadata,
    );
    let pane = PaneData {
        id: "editor.console#1".into(),
        kind: "Console".into(),
        console: ConsolePaneData {
            nodes,
            status_text: "one activity row".into(),
        },
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 260.0,
        height: 100.0,
    };

    let hit = hit_test_pane_template_node(&pane, &body, 120.0, 99.0, 18.0)
        .expect("the open source popup should cover the scrolled log row");

    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(hit.value_text.as_str(), "all");
    assert_eq!(hit.action_id.as_str(), "workbench.console.source.select");
}

#[test]
fn workbench_hit_test_does_not_rebuild_a_template_surface() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "button".into(),
            control_id: "WorkbenchButton".into(),
            role: "Button".into(),
            action_id: "workbench.button.click".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 120.0,
                height: 32.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };
    reset_template_surface_frame_build_count();
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    for _ in 0..1_000 {
        let hit =
            hit_test_workbench_window_template_node_with_index(&presentation, &index, 24.0, 30.0)
                .expect("workbench button should remain hit-testable");
        assert_eq!(hit.control_id.as_str(), "WorkbenchButton");
    }

    assert_eq!(
        template_surface_frame_build_count(),
        0,
        "pointer routing must consume committed node geometry without rebuilding a UiSurface"
    );
    assert_eq!(index.last_candidate_visit_count_for_test(), 1);
}

#[test]
fn workbench_paint_index_limits_single_region_damage_to_nearby_rows() {
    let nodes = (0..100)
        .map(|row| TemplatePaneNodeData {
            node_id: format!("row-{row}"),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: row as f32 * 100.0,
                width: 24.0,
                height: 20.0,
            },
            ..TemplatePaneNodeData::default()
        })
        .collect();
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(nodes),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let rows = index.paint_rows_for_clip(&FrameRect {
        x: 0.0,
        y: 5_000.0,
        width: 24.0,
        height: 20.0,
    });

    assert!(rows.contains(&50));
    assert!(rows.len() <= 2, "single-cell damage visited {rows:?}");
}

#[test]
fn workbench_paint_index_returns_candidates_in_stable_z_order() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "front".into(),
                z_index: 2,
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 24.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "back".into(),
                z_index: 0,
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 24.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "middle".into(),
                z_index: 1,
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 24.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    assert_eq!(
        index.paint_rows_for_clip(&FrameRect {
            x: 0.0,
            y: 0.0,
            width: 24.0,
            height: 20.0,
        }),
        vec![1, 2, 0]
    );
}

#[test]
fn workbench_hit_test_preserves_reverse_paint_order_and_clip_frames() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "back".into(),
                control_id: "BackButton".into(),
                action_id: "back.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "front".into(),
                control_id: "FrontButton".into(),
                action_id: "front.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                has_clip_frame: true,
                clip_frame: TemplateNodeFrameData {
                    x: 50.0,
                    y: 10.0,
                    width: 60.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };

    let clipped_front = hit_test_workbench_window_template_node(&presentation, 20.0, 20.0)
        .expect("the back node should remain hittable outside the front clip");
    let visible_front = hit_test_workbench_window_template_node(&presentation, 60.0, 20.0)
        .expect("the front node should be hittable inside its clip");

    assert_eq!(clipped_front.control_id.as_str(), "BackButton");
    assert_eq!(visible_front.control_id.as_str(), "FrontButton");
}

#[test]
fn indexed_workbench_hit_test_preserves_reverse_paint_order_and_clip_frames() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "back".into(),
                control_id: "BackButton".into(),
                action_id: "back.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "front".into(),
                control_id: "FrontButton".into(),
                action_id: "front.click".into(),
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 80.0,
                },
                has_clip_frame: true,
                clip_frame: TemplateNodeFrameData {
                    x: 50.0,
                    y: 10.0,
                    width: 60.0,
                    height: 80.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let clipped_front =
        hit_test_workbench_window_template_node_with_index(&presentation, &index, 20.0, 20.0)
            .expect("the indexed back node should remain hittable outside the front clip");
    let visible_front =
        hit_test_workbench_window_template_node_with_index(&presentation, &index, 60.0, 20.0)
            .expect("the indexed front node should be hittable inside its clip");

    assert_eq!(clipped_front.control_id.as_str(), "BackButton");
    assert_eq!(visible_front.control_id.as_str(), "FrontButton");
}

#[test]
fn indexed_workbench_hit_test_visits_only_the_point_bucket() {
    let nodes = (0..10_000)
        .map(|row| TemplatePaneNodeData {
            node_id: format!("node-{row}").into(),
            control_id: format!("Control{row}").into(),
            action_id: format!("action.{row}").into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: row as f32 * 32.0,
                width: 120.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        })
        .collect();
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(nodes),
        ..HostWindowPresentationData::default()
    };
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let hit = hit_test_workbench_window_template_node_with_index(
        &presentation,
        &index,
        20.0,
        9_999.0 * 32.0 + 12.0,
    )
    .expect("the final indexed node should remain hittable");

    assert_eq!(hit.control_id.as_str(), "Control9999");
    assert!(
        index.last_candidate_visit_count_for_test() <= 2,
        "a stable hit should not scan the 10k-node model"
    );
}

#[test]
fn workbench_hit_test_routes_open_dropdown_option_rows() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "dropdown".into(),
            control_id: "WorkbenchInputDropdown".into(),
            role: "Dropdown".into(),
            component_role: "dropdown".into(),
            edit_action_id: "component_lab.input_dropdown.select".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 120.0,
                height: 32.0,
            },
            structured_options: model(vec![
                option("dropdown", false),
                option("option_a", false),
                option("option_b", true),
            ]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 96.0)
        .expect("open dropdown option row should be hit-tested");

    assert_eq!(hit.control_id.as_str(), "WorkbenchInputDropdown");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(
        hit.action_id.as_str(),
        "component_lab.input_dropdown.select"
    );
    assert_eq!(hit.value_text.as_str(), "option_a");
    assert_eq!(
        hit.frame.y,
        expected_option_row_frame(&presentation, "WorkbenchInputDropdown", 1).y
    );
}

#[test]
fn workbench_hit_test_routes_componentized_text_input_center() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
            bridge.host_projection(),
        )),
        ..HostWindowPresentationData::default()
    };
    let input = workbench_node(&presentation, "WorkbenchInputText");
    let hit = hit_test_workbench_window_template_node(
        &presentation,
        input.frame.x + input.frame.width * 0.5,
        input.frame.y + input.frame.height * 0.5,
    )
    .expect("input center should hit a componentized workbench node");

    assert_eq!(
        hit.control_id.as_str(),
        "WorkbenchInputText",
        "input center routed to {} with kind {} and role {}",
        hit.control_id,
        hit.dispatch_kind,
        hit.component_role
    );
    assert_eq!(hit.edit_action_id.as_str(), "component_lab.input_text.edit");
}

#[test]
fn text_field_family_without_legacy_input_role_is_hit_tested() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "text".into(),
            control_id: "GenericTextField".into(),
            role: "TextField".into(),
            component_category: "input".into(),
            component_role: "text-field".into(),
            component_layout_role: "leaf".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 12.0,
                width: 120.0,
                height: 28.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 20.0)
        .expect("TextInput component family should enter the template hit surface");

    assert_eq!(hit.control_id.as_str(), "GenericTextField");
    assert_eq!(
        hit.component_family,
        Some(TemplateComponentFamily::TextInput)
    );
}

#[test]
fn workbench_hit_test_preserves_the_selected_table_row_identity() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "rows".into(),
            control_id: "GenericRows".into(),
            role: "Table".into(),
            component_role: "table".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 180.0,
                height: 80.0,
            },
            collection_rows: model(vec![
                TemplatePaneCollectionRowData {
                    source_index: 3,
                    row_identity_field: "surface_entity".into(),
                    identity_kind: "integer".into(),
                    identity_text: "41".into(),
                    label: "Ground".into(),
                },
                TemplatePaneCollectionRowData {
                    source_index: 9,
                    row_identity_field: "surface_entity".into(),
                    identity_kind: "integer".into(),
                    identity_text: "73".into(),
                    label: "Roof".into(),
                },
            ]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 88.0)
        .expect("second table row should be hit-tested");

    assert_eq!(hit.table_row_source_index, Some(9));
    assert_eq!(hit.table_row_identity_kind.as_str(), "integer");
    assert_eq!(hit.table_row_identity_text.as_str(), "73");
}

#[test]
fn workbench_hit_test_uses_the_declared_virtualized_row_extent() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "rows".into(),
            control_id: "GenericRows".into(),
            role: "Table".into(),
            component_role: "table".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 180.0,
                height: 100.0,
            },
            collection_rows: model(vec![
                TemplatePaneCollectionRowData {
                    source_index: 3,
                    row_identity_field: "surface_entity".into(),
                    identity_kind: "integer".into(),
                    identity_text: "41".into(),
                    label: "Ground".into(),
                },
                TemplatePaneCollectionRowData {
                    source_index: 9,
                    row_identity_field: "surface_entity".into(),
                    identity_kind: "integer".into(),
                    identity_text: "73".into(),
                    label: "Roof".into(),
                },
            ]),
            virtualization_enabled: true,
            virtualization_item_extent: 40.0,
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 65.0)
        .expect("the second declared virtualized row should be hit-tested");

    assert_eq!(hit.table_row_source_index, Some(9));
    assert_eq!(hit.table_row_identity_text.as_str(), "73");
}

#[test]
fn workbench_hit_test_ignores_decorative_viewport_scene_layers() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleScene", UiEventKind::Click)
        .expect("scene module state dispatch should succeed")
        .expect("scene module should expose a preview binding");
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
            bridge.host_projection(),
        )),
        ..HostWindowPresentationData::default()
    };
    let scene_layer = workbench_node(&presentation, "WorkbenchViewportFloorGrateRight");
    let x = scene_layer.frame.x + scene_layer.frame.width * 0.5;
    let y = scene_layer.frame.y + scene_layer.frame.height * 0.5;

    let hit = hit_test_workbench_window_template_node(&presentation, x, y);

    assert!(
        hit.is_none(),
        "decorative viewport scene layer should not capture pointer hit, routed to {:?}",
        hit.as_ref().map(|hit| hit.control_id.to_string())
    );
}

#[test]
fn workbench_hit_test_routes_dropdown_option_rows_above_control_when_bottom_clipped() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "root".into(),
                control_id: "WorkbenchRoot".into(),
                role: "Panel".into(),
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 160.0,
                    height: 160.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "dropdown".into(),
                control_id: "WorkbenchInputDropdown".into(),
                role: "Dropdown".into(),
                component_role: "dropdown".into(),
                edit_action_id: "component_lab.input_dropdown.select".into(),
                popup_open: true,
                frame: TemplateNodeFrameData {
                    x: 20.0,
                    y: 120.0,
                    width: 100.0,
                    height: 28.0,
                },
                structured_options: model(vec![
                    option("dropdown", false),
                    option("option_a", false),
                    option("option_b", false),
                ]),
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 28.0, 74.0)
        .expect("clipped dropdown option row should be hit-tested above the control");

    assert_eq!(hit.control_id.as_str(), "WorkbenchInputDropdown");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(hit.value_text.as_str(), "option_a");
    assert_eq!(
        hit.frame.y,
        expected_option_row_frame(&presentation, "WorkbenchInputDropdown", 1).y
    );
}

#[test]
fn workbench_hit_test_routes_open_popup_menu_rows() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "popup".into(),
            control_id: "WorkbenchPopupMenu".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 140.0,
                height: 120.0,
            },
            structured_menu_items: model(vec![
                menu_item("New", false, false),
                menu_item("Open", false, false),
                menu_item("Save", false, false),
                menu_item("", true, true),
                menu_item("Delete", false, false),
            ]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 128.0)
        .expect("open popup menu item row should be hit-tested");

    assert_eq!(hit.control_id.as_str(), "WorkbenchPopupMenu");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_menu_item");
    assert_eq!(hit.action_id.as_str(), "menu.item.delete");
    assert_eq!(hit.value_text.as_str(), "Delete");
    assert_eq!(hit.frame.y, 116.0);
}

#[test]
fn workbench_hit_test_blocks_popup_menu_separator_row() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "popup".into(),
            control_id: "WorkbenchPopupMenu".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            action_id: "workbench.component.menu.open".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 140.0,
                height: 120.0,
            },
            structured_menu_items: model(vec![
                menu_item("New", false, false),
                menu_item("Open", false, false),
                menu_item("Save", false, false),
                menu_item("", true, true),
                menu_item("Delete", false, false),
            ]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    assert!(
        hit_test_workbench_window_template_node(&presentation, 24.0, 104.0).is_none(),
        "separator rows should block parent/underlay hit fallback while staying inside the popup"
    );
}

fn option(id: &str, disabled: bool) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: id.into(),
        disabled,
        ..TemplatePaneOptionData::default()
    }
}

fn console_jump_node(control_id: &str, action_id: &str, y: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        role: "Label".into(),
        dispatch_kind: "activity_log_jump".into(),
        action_id: action_id.into(),
        frame: TemplateNodeFrameData {
            x: 72.0,
            y,
            width: 176.0,
            height: 18.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn menu_item(action_id: &str, disabled: bool, separator: bool) -> TemplatePaneMenuItemData {
    TemplatePaneMenuItemData {
        action_id: action_id.into(),
        label: action_id.into(),
        disabled,
        separator,
        ..TemplatePaneMenuItemData::default()
    }
}

fn workbench_node(
    presentation: &HostWindowPresentationData,
    control_id: &str,
) -> TemplatePaneNodeData {
    (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

fn expected_option_row_frame(
    presentation: &HostWindowPresentationData,
    control_id: &str,
    row: usize,
) -> FrameRect {
    let node = workbench_node(presentation, control_id);
    let origin = workbench_template_origin(presentation);
    let control_frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    template_option_row_frame_within(
        &node,
        &control_frame,
        node.structured_options.row_count(),
        row,
        &origin,
    )
    .unwrap_or_else(|| panic!("{control_id} option row {row} should project to a popup frame"))
}

fn workbench_template_origin(presentation: &HostWindowPresentationData) -> FrameRect {
    let bounds = template_nodes_bounds(&presentation.workbench_window_nodes)
        .expect("workbench template should expose non-empty bounds");
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: bounds.width.max(bounds.x + bounds.width).max(1.0),
        height: bounds.height.max(bounds.y + bounds.height).max(1.0),
    }
}

fn model<T: Clone>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}
