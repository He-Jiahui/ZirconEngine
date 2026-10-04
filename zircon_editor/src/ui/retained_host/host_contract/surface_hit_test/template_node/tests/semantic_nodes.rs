use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::host_contract::data::{
    HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneCollectionRowData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::to_host_contract_workbench_window_nodes;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

use super::support::{hit_test_workbench_window_template_node, model, workbench_node};

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
