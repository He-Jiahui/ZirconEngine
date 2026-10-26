use super::*;

fn property_node(item: &str, field: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("property-{item}").into(),
        parent_node_id: "component-properties".into(),
        source_path: "res://ui/editor/workbench/data/component_property_row.zui".into(),
        source_node_id: "root".into(),
        instance_path: r#"[{"sourcePath":"res://ui/editor/unreal/level_editor_window.zui","sourceNodeId":"details"},{"sourcePath":"res://ui/editor/unreal/components/details.zui","sourceNodeId":"prototype"}]"#.into(),
        parent_source_path: "res://ui/editor/unreal/components/details.zui".into(),
        parent_source_node_id: "properties".into(),
        parent_instance_path: r#"[{"sourcePath":"res://ui/editor/unreal/level_editor_window.zui","sourceNodeId":"details"}]"#.into(),
        inspector_property_item_key: item.into(),
        inspector_property_field_id: field.into(),
        ..Default::default()
    }
}

fn property_receipt(node: &TemplatePaneNodeData) -> Value {
    json!({
        "sourcePath":node.source_path.as_str(),"sourceNodeId":node.source_node_id.as_str(),"instancePath":node.instance_path.as_str(),
        "parentNodeId":node.parent_node_id.as_str(),"parentSourcePath":node.parent_source_path.as_str(),
        "parentSourceNodeId":node.parent_source_node_id.as_str(),"parentInstancePath":node.parent_instance_path.as_str(),
        "itemKey":node.inspector_property_item_key.as_str(),"propertyFieldId":node.inspector_property_field_id.as_str(),
    })
}

#[test]
fn normal_repeat_paint_observes_only_matching_item_and_authored_parent() {
    let far = property_node("far-key", "camera.far_clip");
    let near = property_node("near-key", "camera.near_clip");
    let mut other_owner = far.clone();
    other_owner.node_id = "other-properties-far".into();
    other_owner.parent_source_node_id = "other-properties".into();
    let receipt = property_receipt(&far);
    let data = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::layouts::common::model_rc(vec![
            near,
            far,
            other_owner,
        ]),
        ..Default::default()
    };
    let stream = ChromeCommandStream::full_rebuild((640, 520));
    for style_receipt in [true, false] {
        let sources = if style_receipt {
            vec![]
        } else {
            vec![Some(receipt.clone())]
        };
        let styles = if style_receipt {
            vec![receipt.clone()]
        } else {
            vec![]
        };
        let observed =
            stream_observations(&stream, &data, &sources, Ok(json!({"styles":styles})));
        let nodes = observed["semanticNodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["itemKey"], "far-key");
        assert_eq!(nodes[0]["propertyFieldId"], "camera.far_clip");
        assert_eq!(nodes[0]["sourceNodeId"], "root");
        assert_eq!(nodes[0]["parentSourceNodeId"], "properties");
    }
}

#[test]
fn normal_repeat_paint_rejects_legacy_partial_and_conflicting_identity_receipts() {
    let node = property_node("current-key", "camera.far_clip");
    let receipt = property_receipt(&node);
    let legacy = json!({"sourcePath":node.source_path.as_str(),
        "sourceNodeId":node.source_node_id.as_str(),"instancePath":node.instance_path.as_str()});
    assert!(!paint_owner_matches(&legacy, &node));
    for key in [
        "itemKey",
        "propertyFieldId",
        "parentSourcePath",
        "parentSourceNodeId",
        "parentInstancePath",
    ] {
        let mut missing = receipt.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(!paint_owner_matches(&missing, &node), "missing {key}");
    }
    for key in [
        "itemKey",
        "propertyFieldId",
        "parentSourcePath",
        "parentSourceNodeId",
        "parentInstancePath",
        "parentNodeId",
    ] {
        for wrong in [json!("stale"), Value::Null, json!(17)] {
            let mut stale = receipt.clone();
            stale[key] = wrong;
            assert!(!paint_owner_matches(&stale, &node), "stale {key}");
        }
    }
    for missing in [
        "itemKey",
        "propertyFieldId",
        "parentSourcePath",
        "parentSourceNodeId",
        "parentInstancePath",
    ] {
        let mut partial = node.clone();
        match missing {
            "itemKey" => partial.inspector_property_item_key = Default::default(),
            "propertyFieldId" => partial.inspector_property_field_id = Default::default(),
            "parentSourcePath" => partial.parent_source_path = Default::default(),
            "parentSourceNodeId" => partial.parent_source_node_id = Default::default(),
            _ => partial.parent_instance_path = Default::default(),
        }
        assert!(
            !paint_owner_matches(&property_receipt(&partial), &partial),
            "partial {missing}"
        );
    }
    let data = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::layouts::common::model_rc(vec![node.clone()]),
        ..Default::default()
    };
    let mut old_item = receipt;
    old_item["itemKey"] = json!("previous-materializer-key");
    let observed = stream_observations(
        &ChromeCommandStream::full_rebuild((640, 520)),
        &data,
        &[Some(old_item)],
        Ok(json!({"styles":[legacy]})),
    );
    assert_eq!(observed["semanticNodes"], json!([]));
}

#[test]
fn normal_nonrepeat_paint_retains_legacy_authored_tuple_matching() {
    let mut node = property_node("", "");
    node.source_node_id = "nonrepeat".into();
    let legacy = json!({"sourcePath":node.source_path.as_str(),
        "sourceNodeId":node.source_node_id.as_str(),"instancePath":node.instance_path.as_str()});
    assert!(paint_owner_matches(&legacy, &node));
    let receipt = property_receipt(&node);
    assert!(paint_owner_matches(&receipt, &node));
    for key in [
        "itemKey",
        "propertyFieldId",
        "parentSourcePath",
        "parentSourceNodeId",
        "parentInstancePath",
        "parentNodeId",
    ] {
        let mut stale = receipt.clone();
        stale[key] = json!("foreign");
        assert!(!paint_owner_matches(&stale, &node), "{key}");
    }
}

#[test]
fn submitted_repeat_sources_copy_existing_parent_owner_and_match_only_current_item() {
    use crate::ui::retained_host::host_contract::{
        chrome_command_stream::{ChromeCommand, ChromeCommandLayer},
        data::{FrameRect, TemplateNodeFrameData},
        paint_frame::{HostRenderCommandSource, HostRenderSourceTable},
    };
    use std::sync::Arc;
    use zircon_runtime_interface::ui::{
        event_ui::UiNodeId,
        layout::UiFrame,
        surface::{UiArrangedNode, UiArrangedTree, UiRenderFrameCommandRef, UiSurfaceFrame},
    };
    let id = UiNodeId::new(42);
    let arranged = UiArrangedNode {
        node_id: id,
        node_path: Default::default(),
        parent: None,
        children: vec![],
        frame: UiFrame::new(12.0, 30.0, 90.0, 24.0),
        clip_frame: UiFrame::new(0.0, 0.0, 640.0, 520.0),
        z_index: 0,
        paint_order: 0,
        visibility: Default::default(),
        input_policy: Default::default(),
        pointer_events: Default::default(),
        enabled: false,
        clickable: false,
        hoverable: false,
        focusable: false,
        clip_to_bounds: false,
        control_id: Some("PhysicalPoolRow7".into()),
        slot: None,
    };
    let frame = Arc::new(UiSurfaceFrame {
        generation: 20,
        arranged_tree: Arc::new(UiArrangedTree {
            nodes: std::iter::once(arranged).collect(),
            ..Default::default()
        }),
        ..Default::default()
    });
    let before = serde_json::to_value(frame.as_ref()).unwrap();
    let mut table = HostRenderSourceTable::default();
    let key = table.register(&frame).unwrap();
    let stream = ChromeCommandStream::from_extracted_commands(
        (640, 520),
        None,
        vec![ChromeCommand {
            layer: ChromeCommandLayer::Dynamic,
            z_index: 0,
            frame: FrameRect {
                x: 12.0,
                y: 30.0,
                width: 90.0,
                height: 24.0,
            },
            clip: None,
            source: Some(HostRenderCommandSource {
                surface_key: key,
                command_ref: UiRenderFrameCommandRef::new(id, 0),
                fragment_index: 0,
            }),
            kind: ChromeCommandKind::Quad {
                color: [10, 20, 30, 255],
                corner_radius: 0.0,
            },
        }],
        table,
    );
    let node = TemplatePaneNodeData {
        source_surface_frame: Some(Arc::clone(&frame)),
        surface_node_id: Some(id),
        control_id: "PhysicalPoolRow7".into(),
        disabled: true,
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 30.0,
            width: 90.0,
            height: 24.0,
        },
        ..property_node("far-key", "camera.far_clip")
    };
    let mut data = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::layouts::common::model_rc(vec![node.clone()]),
        ..Default::default()
    };
    let sources = super::super::source_rows(&stream, &data);
    let source = sources[0].as_ref().unwrap();
    assert!(paint_owner_matches(source, &node));
    assert_eq!(source["parentNodeId"], node.parent_node_id.as_str());
    assert_eq!(source["parentSourcePath"], node.parent_source_path.as_str());
    assert_eq!(
        source["parentSourceNodeId"],
        node.parent_source_node_id.as_str()
    );
    assert_eq!(
        source["parentInstancePath"],
        node.parent_instance_path.as_str()
    );
    assert_eq!(source["propertyFieldId"], "camera.far_clip");
    assert_eq!(source["itemKey"], "far-key");
    assert_eq!(source["disabled"], true);
    let observed = stream_observations(&stream, &data, &sources, Ok(json!({"styles":[]})));
    assert_eq!(observed["semanticNodes"].as_array().unwrap().len(), 1);
    assert_eq!(observed["semanticNodes"][0]["itemKey"], "far-key");
    let mut rebound = node;
    rebound.inspector_property_item_key = "near-key".into();
    rebound.inspector_property_field_id = "camera.near_clip".into();
    data.workbench_window_nodes = crate::ui::layouts::common::model_rc(vec![rebound]);
    let stale = stream_observations(&stream, &data, &sources, Ok(json!({"styles":[]})));
    assert_eq!(stale["semanticNodes"], json!([]));
    assert_eq!(before, serde_json::to_value(frame.as_ref()).unwrap());
}

#[test]
fn normal_focus_publication_reads_captured_interaction_scope() {
    use crate::ui::retained_host::host_contract::data::{
        HostDockOverflowMenuStateData, HostMenuStateData, HostPageOverflowMenuStateData,
        HostPaneInteractionStateData, HostPresentationGeneration, HostTextInputFocusData,
        HostViewportImageSet,
    };
    use crate::ui::retained_host::host_contract::surface_hit_test::HostWorkbenchHitIndex;
    use crate::ui::retained_host::primitives::SharedString;
    use std::sync::Arc;
    let structure = Arc::new(HostWindowPresentationData::default());
    for visible in [false, true] {
        let interaction = Arc::new(HostPaneInteractionStateData {
            template_focus_observed: true,
            template_focus_visible: visible,
            focused_template_control_id: if visible {
                "FocusProbe".into()
            } else {
                Default::default()
            },
            ..HostPaneInteractionStateData::default()
        });
        let generation = HostPresentationGeneration::new(
            Arc::clone(&structure), Arc::new(HostMenuStateData::default()),
            Arc::new(HostPageOverflowMenuStateData::default()),
            Arc::new(HostDockOverflowMenuStateData::default()), interaction,
            Arc::new(HostTextInputFocusData::default()), HostViewportImageSet::default(),
            Arc::new(HostWorkbenchHitIndex::from_presentation(&structure)),
            crate::ui::retained_host::host_contract::paint_theme::capture_host_paint_theme_snapshot(),
            Arc::new(SharedString::default()), 1, 1, 1, 1, 1,
        );
        assert!(focus_publication(generation.structure()).is_none());
        {
            let _scope = generation.enter_paint_scope();
            let focus =
                focus_publication(generation.structure()).expect("captured focus observation");
            assert_eq!(focus["observed"], true);
            assert_eq!(focus["templateFocusVisible"], visible);
            assert_eq!(
                focus["focusedNativeControlId"].as_str(),
                visible.then_some("FocusProbe")
            );
            assert_eq!(focus["sourceOwnerMappingComplete"], !visible);
            assert!(
                !generation
                    .structure()
                    .pane_interaction_state
                    .template_focus_observed
            );
        }
        assert!(focus_publication(generation.structure()).is_none());
    }
}

#[test]
fn static_default_is_not_a_known_focus_loss() {
    assert!(focus_publication(&HostWindowPresentationData::default()).is_none());
}

#[test]
fn image_readiness_requires_actual_residency_not_uploaded_bytes() {
    let mut evidence = json!({"commands":[{"paint":{"kind":"image","resourceKey":"image","generation":7,"hasUploadBytes":true}}]});
    certify_image_residency(&mut evidence, |_, _| false);
    assert_eq!(evidence["gpuImageResidencyAudit"]["complete"], false);
    certify_image_residency(&mut evidence, |key, generation| {
        key == "image" && generation == 7
    });
    assert_eq!(evidence["gpuImageResidencyAudit"]["complete"], true);
}
