//! Observations of the command stream submitted by the normal GPU window.
//! Static presentation materialization does not call this owner.
use serde_json::{json, Value};

#[path = "../normal_present/tree_owner_focus.rs"]
mod tree_owner_focus;

use super::super::chrome_command_stream::{ChromeCommandKind, ChromeCommandStream};
use super::super::data::{HostWindowPresentationData, TemplatePaneNodeData};
use zircon_runtime::rhi::{
    UiSurfaceDrawList, UiSurfaceImageReadbackRequest, UiSurfaceRect, UiSurfaceResolvedCommandKind,
};

/// Editor attribution frozen before the source table is recycled. The RHI receives only `request`.
#[derive(Clone, Debug)]
pub(in crate::ui::retained_host::host_contract) struct NormalSceneImageSelection {
    pub(super) request: UiSurfaceImageReadbackRequest,
    pub(super) evidence: Value,
    pub(super) command_frame: UiSurfaceRect,
    pub(super) command_clip: Option<UiSurfaceRect>,
}

impl NormalSceneImageSelection {
    pub(in crate::ui::retained_host::host_contract) fn matches_draw_list(
        &self,
        draw_list: &UiSurfaceDrawList,
    ) -> bool {
        draw_list.commands.get(self.request.command_index).is_some_and(|command| {
            command.frame == self.command_frame && command.clip == self.command_clip
                && matches!(draw_list.resolved_kind(command), Some(UiSurfaceResolvedCommandKind::Image { payload })
                    if payload.resource_key == self.request.resource_key
                        && payload.resource_generation == self.request.resource_generation
                        && (payload.width, payload.height) == self.request.physical_size
                        && payload.rgba.is_none() && payload.atlas_uv.is_none())
        })
    }
}

fn scene_source_identity(node: &TemplatePaneNodeData) -> Result<Value, String> {
    let steps: Value = serde_json::from_str(node.instance_path.as_str())
        .map_err(|_| "scene source invocation is not JSON".to_owned())?;
    if node.source_path.is_empty()
        || node.source_node_id.is_empty()
        || !steps.as_array().is_some_and(|steps| {
            steps.iter().all(|step| {
                step.as_object().is_some_and(|step| {
                    step.len() == 2
                        && ["sourcePath", "sourceNodeId"].iter().all(|key| {
                            step.get(*key)
                                .and_then(Value::as_str)
                                .is_some_and(|value| !value.is_empty())
                        })
                })
            })
        })
        || !node.inspector_property_item_key.is_empty()
        || !node.inspector_property_field_id.is_empty()
    {
        return Err("scene source needs its complete non-repeated authored identity".into());
    }
    Ok(
        json!({"sourcePath":node.source_path.as_str(),"sourceNodeId":node.source_node_id.as_str(),
        "instancePath":node.instance_path.as_str()}),
    )
}

fn scene_rect(frame: &super::super::data::FrameRect) -> UiSurfaceRect {
    UiSurfaceRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

/// Resolve the same immutable images and authored document owner used by the actual painter.
/// Missing media remains unavailable; no CPU capture, neighboring document or overlay substitutes.
pub(in crate::ui::retained_host::host_contract) fn scene_image_selection(
    presentation: &HostWindowPresentationData,
    stream: &ChromeCommandStream,
    sources: &[Option<Value>],
) -> Result<Option<NormalSceneImageSelection>, String> {
    if !stream.is_full_rebuild() || sources.len() != stream.commands().len() {
        return Err("scene media needs the complete actual command stream and source table".into());
    }
    let images = super::super::data::paint_viewport_images(presentation);
    let mut selections = Vec::new();
    for document in presentation.host_scene_data.document_surfaces() {
        if document.pane.kind.as_str() != "Scene" {
            continue;
        }
        let Some(viewport) = super::super::componentized_workbench_regions::authored_level_viewport(
            presentation,
            document,
        ) else {
            continue;
        };
        let owner = scene_source_identity(viewport.media)?;
        let root = scene_source_identity(viewport.root)?;
        for node in [viewport.root, viewport.media] {
            if presentation
                .workbench_window_nodes
                .iter()
                .filter(|candidate| {
                    candidate.source_path == node.source_path
                        && candidate.source_node_id == node.source_node_id
                        && candidate.instance_path == node.instance_path
                })
                .count()
                != 1
            {
                return Err("scene authored source identity is ambiguous".into());
            }
        }
        let image = images
            .for_surface(document.surface_key.as_str(), document.pane.kind.as_str())
            .filter(|image| image.is_valid() && image.rgba().is_none())
            .ok_or_else(|| "scene has no exact external GPU image product".to_owned())?;
        let source_frame = viewport
            .media
            .source_surface_frame
            .as_ref()
            .ok_or_else(|| "scene media has no retained source surface".to_owned())?;
        if source_frame.tree_id.0.is_empty() {
            return Err("scene retained source tree identity is empty".into());
        }
        let mut matches = Vec::new();
        for (index, command) in stream.commands().iter().enumerate() {
            let ChromeCommandKind::Image { payload } = &command.kind else {
                continue;
            };
            if payload.resource_key != image.resource_key
                || payload.resource_generation != image.resource_generation
                || (payload.width, payload.height) != (image.width, image.height)
            {
                continue;
            }
            let Some((frame, reference, fragment)) = stream.resolve_command_source(index) else {
                continue;
            };
            if !std::sync::Arc::ptr_eq(frame, source_frame)
                || viewport.media.surface_node_id != Some(reference.node_id)
                || viewport.media.surface_render_command_ref != Some(reference)
            {
                continue;
            }
            let source = sources[index]
                .as_ref()
                .filter(|source| paint_owner_matches(source, viewport.media))
                .ok_or_else(|| {
                    "scene image command has no unique complete source owner".to_owned()
                })?;
            let source_ref = json!({"treeId":frame.tree_id,"generation":frame.generation,"nodeId":reference.node_id,
                "nodeCommandIndex":reference.node_command_index,"fragmentIndex":fragment,
                "authority":"surface-node-owner","primitiveIdentity":false});
            let bounds = scene_rect(&command.frame);
            let clip = command.clip.as_ref().map(scene_rect);
            if !bounds.has_finite_positive_area()
                || bounds != scene_rect(&viewport.frame)
                || !clip.is_some_and(|clip| {
                    clip.has_finite_positive_area()
                        && clip.x.max(bounds.x) < (clip.x + clip.width).min(bounds.x + bounds.width)
                        && clip.y.max(bounds.y)
                            < (clip.y + clip.height).min(bounds.y + bounds.height)
                })
                || payload.rgba.is_some()
                || payload.atlas_uv.is_some()
                || source["sourceRef"] != source_ref
            {
                return Err(
                    "scene image command bounds, clip or full-resource identity is unproven".into(),
                );
            }
            matches.push(NormalSceneImageSelection {
                request: UiSurfaceImageReadbackRequest { command_index: index, resource_key: image.resource_key.clone(),
                    resource_generation: image.resource_generation, physical_size: (image.width, image.height),
                    expected_pixel_format: None },
                evidence: json!({"schema":"dev.zircon.editor.normal-scene-image-readback","version":1,
                    "authority":"normal-actual-submitted-runtime-image-readback","owner":owner,"viewportRoot":root,
                    "surfaceKey":document.surface_key.as_str(),"paneKind":document.pane.kind.as_str(),
                    "commandIndex":index,"sourceRef":source["sourceRef"],
                    "commandBounds":{"x":bounds.x,"y":bounds.y,"width":bounds.width,"height":bounds.height},
                    "commandClip":clip.map(|clip|json!({"x":clip.x,"y":clip.y,"width":clip.width,"height":clip.height})),
                    "atlasUv":null,"resourceKey":image.resource_key,"resourceGeneration":image.resource_generation,
                    "physicalSize":[image.width,image.height],"overlayPresent":image.overlay().is_some()}),
                command_frame: bounds, command_clip: clip,
            });
        }
        if matches.len() != 1 {
            return Err(
                "scene needs one source-owned image command for its exact GPU product".into(),
            );
        }
        selections.extend(matches);
    }
    match selections.len() {
        0 => Ok(None),
        1 => Ok(selections.pop()),
        _ => Err("normal media capture has multiple authored Scene surfaces".into()),
    }
}

pub(in crate::ui::retained_host::host_contract) fn stream_observations(
    stream: &ChromeCommandStream,
    presentation: &HostWindowPresentationData,
    sources: &[Option<Value>],
    paint: Result<Value, String>,
) -> Value {
    let commands = observed_commands(stream, sources);
    let paint = paint.unwrap_or_else(|error| json!({"issues":[error],"styles":[],"assetAudit":{"complete":false,"resources":[]}}));
    let styles = paint.get("styles").and_then(Value::as_array);
    let mut nodes = Vec::new();
    for model in presentation.paint_node_models() {
        for node in model.iter() {
            if node.source_path.is_empty()
                || node.source_node_id.is_empty()
                || node.instance_path.is_empty()
            {
                continue;
            }
            // Include nodes observed by this stream, not every authored prototype.
            let painted = styles
                .is_some_and(|styles| styles.iter().any(|style| paint_owner_matches(style, &node)))
                || sources
                    .iter()
                    .flatten()
                    .any(|source| paint_owner_matches(source, &node));
            if painted {
                nodes.push(json!({"nodeId":node.node_id.as_str(),"parentNodeId":node.parent_node_id.as_str(),
                    "sourcePath":node.source_path.as_str(),"sourceNodeId":node.source_node_id.as_str(),"instancePath":node.instance_path.as_str(),
                    "parentSourcePath":node.parent_source_path.as_str(),"parentSourceNodeId":node.parent_source_node_id.as_str(),"parentInstancePath":node.parent_instance_path.as_str(),
                    "itemKey":(!node.inspector_property_item_key.is_empty()).then_some(node.inspector_property_item_key.as_str()),
                    "propertyFieldId":(!node.inspector_property_field_id.is_empty()).then_some(node.inspector_property_field_id.as_str()),
                    "controlId":node.control_id.as_str(),"role":node.role.as_str(),"disabled":node.disabled,
                    "bounds":{"x":node.frame.x,"y":node.frame.y,"width":node.frame.width,"height":node.frame.height},
                    "clip":node.has_clip_frame.then(||json!({"x":node.clip_frame.x,"y":node.clip_frame.y,"width":node.clip_frame.width,"height":node.clip_frame.height}))}));
            }
        }
    }
    if nodes
        .iter()
        .any(|node| node["controlId"] == "WorkbenchSceneTree")
    {
        let publication = tree_owner_focus::publication(presentation, sources, styles);
        for node in &mut nodes {
            if node["controlId"] == "WorkbenchSceneTree" {
                node["treeOwnerFocused"] = publication["value"].clone();
                node["treeOwnerFocusPublication"] = publication.clone();
            }
        }
    }
    let mut result = json!({"schema":"dev.zircon.editor.normal-present-evidence","version":1,
        "coordinateSpace":"physical-pixels","commandStreamFullRebuild":stream.is_full_rebuild(),
        "commands":commands,"semanticNodes":nodes,"paintEvidence":paint});
    if let Some(toolbar) = viewport_toolbar_publication(presentation, sources, styles) {
        result["viewportToolbarPublication"] = toolbar;
    }
    if let Some(focus) = focus_publication(presentation) {
        result["focusPublication"] = focus;
    }
    result
}

fn paint_owner_matches(receipt: &Value, node: &TemplatePaneNodeData) -> bool {
    let item = node.inspector_property_item_key.as_str();
    let field = node.inspector_property_field_id.as_str();
    let repeated = !item.is_empty() || !field.is_empty();
    if repeated
        && (item.is_empty()
            || field.is_empty()
            || node.parent_source_path.is_empty()
            || node.parent_source_node_id.is_empty()
            || node.parent_instance_path.is_empty())
    {
        return false;
    }
    receipt["sourcePath"].as_str() == Some(node.source_path.as_str())
        && receipt["sourceNodeId"].as_str() == Some(node.source_node_id.as_str())
        && receipt["instancePath"].as_str() == Some(node.instance_path.as_str())
        && receipt_field_matches(receipt, "itemKey", item, repeated)
        && receipt_field_matches(receipt, "propertyFieldId", field, repeated)
        && receipt_field_matches(
            receipt,
            "parentSourcePath",
            node.parent_source_path.as_str(),
            repeated,
        )
        && receipt_field_matches(
            receipt,
            "parentSourceNodeId",
            node.parent_source_node_id.as_str(),
            repeated,
        )
        && receipt_field_matches(
            receipt,
            "parentInstancePath",
            node.parent_instance_path.as_str(),
            repeated,
        )
        && receipt_field_matches(receipt, "parentNodeId", node.parent_node_id.as_str(), false)
}

fn receipt_field_matches(receipt: &Value, name: &str, expected: &str, required: bool) -> bool {
    match receipt.get(name) {
        None => !required,
        Some(Value::Null) => !required && expected.is_empty(),
        Some(Value::String(value)) => value == expected,
        _ => false,
    }
}

pub(in crate::ui::retained_host::host_contract) fn certify_image_residency(
    evidence: &mut Value,
    mut resident: impl FnMut(&str, u64) -> bool,
) {
    let mut complete = true;
    if let Some(commands) = evidence["commands"].as_array_mut() {
        for command in commands {
            if command["paint"]["kind"] != "image" {
                continue;
            }
            let key = command["paint"]["resourceKey"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            let generation = command["paint"]["generation"].as_u64().unwrap_or_default();
            let ready = !key.is_empty() && resident(&key, generation);
            command["paint"]["residentAfterSuccessfulPresent"] = json!(ready);
            complete &= ready;
        }
    } else {
        complete = false;
    }
    evidence["gpuImageResidencyAudit"] =
        json!({"complete":complete,"authority":"actual-presenter-resource-residency"});
}

pub(in crate::ui::retained_host::host_contract) fn focus_publication(
    presentation: &HostWindowPresentationData,
) -> Option<Value> {
    let interaction =
        crate::ui::retained_host::host_contract::data::paint_pane_interaction_state(presentation);
    if !interaction.template_focus_observed {
        return None;
    }
    let focused = interaction.focused_template_control_id.as_str();
    let owner_control = if focused == "HierarchyListPanel" {
        super::super::componentized_workbench_regions::authored_hierarchy(presentation)?;
        "WorkbenchSceneTree"
    } else {
        focused
    };
    let mut owners = Vec::new();
    for model in presentation.paint_node_models() {
        for node in model.iter() {
            if !owner_control.is_empty()
                && node.control_id.as_str() == owner_control
                && !node.source_path.is_empty()
                && !node.source_node_id.is_empty()
                && !node.instance_path.is_empty()
            {
                owners.push(json!({"sourcePath":node.source_path.as_str(),"sourceNodeId":node.source_node_id.as_str(),
                    "instancePath":node.instance_path.as_str(),"controlId":node.control_id.as_str()}));
            }
        }
    }
    // Ambiguous or missing mappings remain unknown; an observed empty native
    // owner is distinct from the unobserved default state above.
    let owner = match owners.as_slice() {
        [owner] => owner.clone(),
        _ => Value::Null,
    };
    Some(
        json!({"authority":"normal-native-pane-interaction","observed":true,
        "focusedNativeControlId":(!focused.is_empty()).then_some(focused),
        "templateFocusVisible":interaction.template_focus_visible,
        "sourceOwner":owner,"sourceOwnerMappingComplete":focused.is_empty() || owners.len()==1}),
    )
}

#[cfg(test)]
mod tests {
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
}

pub(in crate::ui::retained_host::host_contract) fn observed_commands(
    stream: &ChromeCommandStream,
    sources: &[Option<Value>],
) -> Vec<Value> {
    stream.commands().iter().enumerate().map(|(index, command)| {
        let style = match &command.kind {
            ChromeCommandKind::Quad { color, corner_radius } => json!({"kind":"quad", "color":color,"cornerRadius":corner_radius}),
            ChromeCommandKind::Border { color, width, corner_radius } => json!({"kind":"border","color":color,"width":width,"cornerRadius":corner_radius}),
            ChromeCommandKind::Text { text, color, size, line_height, font_request, .. } => json!({"kind":"text","text":text,"color":color,"fontSize":size,"lineHeight":line_height,"fontFamily":font_request.as_ref().map(|request| request.family.as_str()),"fontWeight":font_request.as_ref().map(|request| request.weight)}),
            ChromeCommandKind::Image { payload } => json!({"kind":"image","resourceKey":payload.resource_key,"generation":payload.resource_generation,"width":payload.width,"height":payload.height,"hasUploadBytes":payload.rgba.is_some()}),
            ChromeCommandKind::Clip => json!({"kind":"clip"}),
        };
        json!({"commandIndex":index,"source":sources.get(index).and_then(Option::as_ref),
            "bounds":{"x":command.frame.x,"y":command.frame.y,"width":command.frame.width,"height":command.frame.height},
            "clip":command.clip.as_ref().map(|clip|json!({"x":clip.x,"y":clip.y,"width":clip.width,"height":clip.height})),"paint":style})
    }).collect::<Vec<_>>()
}

// No live bridge/tree access. Only the prepared source payload and immutable
// node models actually submitted by this normal presentation are admitted.
fn viewport_toolbar_publication(
    presentation: &HostWindowPresentationData,
    sources: &[Option<Value>],
    styles: Option<&Vec<Value>>,
) -> Option<Value> {
    let (_, frame, value) = presentation.normal_viewport_toolbar.as_ref()?;
    let mut payload = (**value).clone();
    if payload["schema"] != "dev.zircon.editor.viewport-toolbar-publication"
        || payload["version"] != 1
        || payload["authority"] != "prepared-source-surface"
        || payload["observed"] != true
        || payload["coordinateSpace"] != "physical-pixels"
        || payload["surfaceGeneration"].as_u64() != Some(frame.generation)
        || payload["surfaceTreeId"] != json!(frame.tree_id)
    {
        return None;
    }
    let nodes = payload["nodes"].as_array()?;
    let models = presentation.paint_node_models();
    let root = nodes
        .iter()
        .find(|node| node["nodeId"] == payload["rootNodeId"])?;
    if root["visible"] != true {
        return None;
    }
    let mut roots = models.iter().flat_map(|model| model.iter()).filter(|node| {
        node.node_id.as_str() == root["nodeId"].as_str().unwrap_or_default()
            && node.control_id.as_str() == "UnrealViewportToolbar"
    });
    let owner = roots.next()?;
    if roots.next().is_some() || !viewport_toolbar_source_owner_matches(&owner, frame) {
        return None;
    }
    // Transparent groups and hidden children remain current source-frame
    // evidence. Visible authored controls additionally agree with submitted paint.
    for published in nodes.iter().filter(|node| node["visible"] == true) {
        let required = published["nodeId"] == payload["rootNodeId"]
            || matches!(
                published["component"].as_str(),
                Some("IconButton" | "Dropdown" | "WorkbenchDropdown" | "ScrollableBox")
            );
        let mut matches = models.iter().flat_map(|model| model.iter()).filter(|node| {
            node.source_path.as_str() == published["sourcePath"].as_str().unwrap_or_default()
                && node.source_node_id.as_str()
                    == published["sourceNodeId"].as_str().unwrap_or_default()
                && node.instance_path.as_str()
                    == published["instancePath"].as_str().unwrap_or_default()
        });
        let Some(actual) = matches.next() else {
            if required {
                return None;
            }
            // Transparent or unowned descendants have no submitted typed
            // node; their state is solely immutable prepared-source evidence.
            continue;
        };
        if matches.next().is_some()
            || published["nodeId"].as_str() != Some(actual.node_id.as_str())
            || !viewport_toolbar_source_owner_matches(&actual, frame)
        {
            return None;
        }
        for (axis, current) in [
            ("x", actual.frame.x),
            ("y", actual.frame.y),
            ("width", actual.frame.width),
            ("height", actual.frame.height),
        ] {
            if !current.is_finite()
                || (published["bounds"][axis].as_f64()? - (current as f64)).abs() > 0.001
            {
                return None;
            }
        }
        if published["parentNodeId"].as_str().unwrap_or_default() != actual.parent_node_id.as_str()
        {
            return None;
        }
        let state = &published["state"];
        if state["disabled"].as_bool() != Some(actual.disabled) {
            return None;
        }
        // Each known source bool must agree. A conflicting selected/checked
        // pair cannot conceal one value through an or_else fallback.
        for name in ["selected", "checked"] {
            if let Some(selected) = state[name].as_bool() {
                if selected != actual.selected {
                    return None;
                }
            }
        }
        if let Some(open) = state["popupOpen"].as_bool() {
            if open != actual.popup_open {
                return None;
            }
        }
        if let Some(text) = state["text"].as_str() {
            if text != actual.text.as_str() {
                return None;
            }
        }
        let is_value_control = matches!(
            published["component"].as_str(),
            Some(
                "Dropdown"
                    | "WorkbenchDropdown"
                    | "InputField"
                    | "TextField"
                    | "NumberField"
                    | "RangeField"
                    | "SearchField"
                    | "AssetField"
                    | "ObjectField"
            )
        ) || matches!(
            actual.role.as_str(),
            "input-field"
                | "number-field"
                | "range-field"
                | "slider"
                | "range-slider"
                | "combo-box"
                | "dropdown"
                | "enum-field"
                | "flags-field"
                | "search-select"
                | "asset-field"
                | "object-field"
                | "instance-field"
        );
        if is_value_control {
            if let Some(text) = state["valueText"].as_str() {
                if text != actual.value_text.as_str() {
                    return None;
                }
            }
            if let Some(value) = state["value"].as_f64() {
                if !actual.value_number.is_finite()
                    || (value - actual.value_number as f64).abs() > 0.001
                {
                    return None;
                }
            }
            // Typed paint exposes caption/valueText, not a separate string ID.
            // The ID was independently checked against current source metadata
            // before capture. If caption is absent, its painted value is the ID.
            if state["valueText"].is_null() {
                if let Some(value) = state["value"].as_str() {
                    if value != actual.value_text.as_str() {
                        return None;
                    }
                }
            }
        }
        if let Some(value) = state["value"].as_bool() {
            if value != actual.selected {
                return None;
            }
        }
    }
    let observed = nodes
        .iter()
        .filter(|node| node["visible"] == true)
        .any(|node| {
            let same = |source: &Value| {
                source["sourcePath"] == node["sourcePath"]
                    && source["sourceNodeId"] == node["sourceNodeId"]
                    && source["instancePath"] == node["instancePath"]
            };
            sources.iter().flatten().any(|source| same(source))
                || styles.is_some_and(|styles| styles.iter().any(|style| same(style)))
        });
    if !observed {
        return None;
    }
    payload["authority"] = json!("normal-prepared-source-surface");
    Some(payload)
}
fn viewport_toolbar_source_owner_matches(
    node: &super::super::TemplatePaneNodeData,
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
) -> bool {
    let Some(source) = node.source_surface_frame.as_ref() else {
        return false;
    };
    let Some(id) = node.surface_node_id else {
        return false;
    };
    if source.tree_id != frame.tree_id || source.generation > frame.generation {
        return false;
    }
    let (Some(previous), Some(current)) =
        (source.arranged_tree.get(id), frame.arranged_tree.get(id))
    else {
        return false;
    };
    // Sparse preparation legitimately retains old Arcs for unchanged nodes.
    // Admit one only when its complete arranged owner (including children,
    // parent, clip, visibility, enabled/input state and slot) is unchanged.
    previous == current
        && current.node_path.0 == node.node_id.as_str()
        && current.control_id.as_deref() == Some(node.control_id.as_str())
}
#[cfg(test)]
#[path = "../normal_present/tests/viewport_toolbar.rs"]
mod viewport_toolbar_tests;

#[cfg(test)]
mod normal_viewport_media_tests {
    use super::super::super::{
        chrome_command_stream::{ChromeCommand, ChromeCommandLayer, ChromeImagePayload},
        data::{FrameRect, HostViewportImageData, TemplateNodeFrameData},
        paint_frame::{HostRenderCommandSource, HostRenderSourceTable},
    };
    use super::*;
    use std::sync::Arc;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiTreeId},
        layout::UiFrame,
        surface::{UiArrangedNode, UiArrangedTree, UiRenderFrameCommandRef, UiSurfaceFrame},
    };

    fn fixture() -> (
        HostWindowPresentationData,
        ChromeCommandStream,
        Vec<Option<Value>>,
    ) {
        let id = UiNodeId::new(2);
        let arranged = UiArrangedNode {
            node_id: id,
            node_path: Default::default(),
            parent: None,
            children: vec![],
            frame: UiFrame::new(10.0, 20.0, 600.0, 400.0),
            clip_frame: UiFrame::new(0.0, 0.0, 800.0, 600.0),
            z_index: 0,
            paint_order: 0,
            visibility: Default::default(),
            input_policy: Default::default(),
            pointer_events: Default::default(),
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: false,
            clip_to_bounds: false,
            control_id: Some("WorkbenchViewportSurface".into()),
            slot: None,
        };
        let frame = Arc::new(UiSurfaceFrame {
            tree_id: UiTreeId::new("actual-scene-surface"),
            generation: 9,
            arranged_tree: Arc::new(UiArrangedTree {
                nodes: vec![arranged].into(),
                ..Default::default()
            }),
            ..Default::default()
        });
        let invocation = r#"[{"sourcePath":"res://ui/editor/unreal/level_editor_window.zui","sourceNodeId":"viewport"}]"#;
        let node = |name: &str, parent: &str, control: &str, authored: &str| TemplatePaneNodeData {
            node_id: name.into(),
            parent_node_id: parent.into(),
            control_id: control.into(),
            source_path: "res://ui/editor/unreal/components/viewport.zui".into(),
            source_node_id: authored.into(),
            instance_path: invocation.into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 600.0,
                height: 400.0,
            },
            ..Default::default()
        };
        let root = TemplatePaneNodeData {
            node_id: "window".into(),
            control_id: "WorkbenchWindowRoot".into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
            ..Default::default()
        };
        let content = TemplatePaneNodeData {
            node_id: "content".into(),
            parent_node_id: "window".into(),
            control_id: "UnrealLevelEditorContent".into(),
            ..Default::default()
        };
        let viewport = node(
            "viewport",
            "content",
            "WorkbenchMainBandViewportPanel",
            "root",
        );
        let media = TemplatePaneNodeData {
            source_surface_frame: Some(Arc::clone(&frame)),
            surface_node_id: Some(id),
            surface_render_command_ref: Some(UiRenderFrameCommandRef::new(id, 0)),
            ..node(
                "media",
                "viewport",
                "WorkbenchViewportSurface",
                "scene_media",
            )
        };
        let mut presentation = HostWindowPresentationData {
            workbench_window_nodes: crate::ui::layouts::common::model_rc(vec![
                root, content, viewport, media,
            ]),
            ..Default::default()
        };
        presentation.host_scene_data.document_dock.surface_key = "document:scene:current".into();
        presentation.host_scene_data.document_dock.pane.kind = "Scene".into();
        presentation.host_scene_data.document_dock.region_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        presentation.viewport_images.replace_scene_for_surface(
            "document:scene:current",
            HostViewportImageData {
                resource_key: "viewport:exact:5".into(),
                resource_generation: 5,
                width: 600,
                height: 400,
                ..Default::default()
            },
        );
        let mut table = HostRenderSourceTable::default();
        let key = table.register(&frame).unwrap();
        let stream = ChromeCommandStream::from_extracted_commands(
            (800, 600),
            None,
            vec![ChromeCommand {
                layer: ChromeCommandLayer::Viewport,
                z_index: 0,
                frame: FrameRect {
                    x: 10.0,
                    y: 20.0,
                    width: 600.0,
                    height: 400.0,
                },
                clip: Some(FrameRect {
                    x: 10.0,
                    y: 20.0,
                    width: 600.0,
                    height: 400.0,
                }),
                source: Some(HostRenderCommandSource {
                    surface_key: key,
                    command_ref: UiRenderFrameCommandRef::new(id, 0),
                    fragment_index: 0,
                }),
                kind: ChromeCommandKind::Image {
                    payload: ChromeImagePayload {
                        resource_key: "viewport:exact:5".into(),
                        resource_generation: 5,
                        width: 600,
                        height: 400,
                        upload_bytes: 0,
                        rgba: None,
                        atlas_uv: None,
                    },
                },
            }],
            table,
        );
        let sources = super::super::source_rows(&stream, &presentation);
        (presentation, stream, sources)
    }

    #[test]
    fn normal_viewport_media_joins_actual_document_surface_source_and_gpu_command() {
        let (presentation, stream, sources) = fixture();
        let before = serde_json::to_value(
            presentation
                .workbench_window_nodes
                .iter()
                .find(|node| node.node_id.as_str() == "media")
                .unwrap()
                .source_surface_frame
                .as_ref()
                .unwrap()
                .as_ref(),
        )
        .unwrap();
        let selected = scene_image_selection(&presentation, &stream, &sources)
            .unwrap()
            .unwrap();
        assert_eq!(selected.request.resource_key, "viewport:exact:5");
        assert_eq!(selected.request.physical_size, (600, 400));
        assert_eq!(selected.evidence["owner"]["sourceNodeId"], "scene_media");
        assert_eq!(selected.evidence["surfaceKey"], "document:scene:current");
        assert_eq!(selected.evidence["sourceRef"]["generation"], 9);
        assert_eq!(selected.evidence["commandBounds"]["x"], 10.0);
        assert_eq!(
            selected.evidence["owner"]["instancePath"],
            sources[0].as_ref().unwrap()["instancePath"]
        );
        let draw =
            super::super::super::chrome_command_stream::ui_surface_draw_list_from_stream(&stream);
        assert!(selected.matches_draw_list(&draw));
        assert_eq!(
            before,
            serde_json::to_value(
                presentation
                    .workbench_window_nodes
                    .iter()
                    .find(|node| node.node_id.as_str() == "media")
                    .unwrap()
                    .source_surface_frame
                    .as_ref()
                    .unwrap()
                    .as_ref()
            )
            .unwrap()
        );
    }

    #[test]
    fn normal_viewport_media_rejects_stale_or_partial_source_evidence_and_duplicates() {
        let (presentation, stream, sources) = fixture();
        for key in ["sourcePath", "sourceNodeId", "instancePath"] {
            let mut stale = sources.clone();
            stale[0].as_mut().unwrap()[key] = json!("foreign");
            assert!(
                scene_image_selection(&presentation, &stream, &stale).is_err(),
                "{key}"
            );
        }
        let mut stale = sources.clone();
        stale[0].as_mut().unwrap()["sourceRef"]["generation"] = json!(8);
        assert!(scene_image_selection(&presentation, &stream, &stale).is_err());
        let mut duplicate = stream.clone();
        duplicate.push_command_for_test(stream.commands()[0].clone());
        let repeated = super::super::source_rows(&duplicate, &presentation);
        assert!(scene_image_selection(&presentation, &duplicate, &repeated).is_err());
        let mut nodes = presentation
            .workbench_window_nodes
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut duplicate = nodes
            .iter()
            .find(|node| node.node_id.as_str() == "media")
            .unwrap()
            .clone();
        duplicate.node_id = "another-physical-media".into();
        nodes.push(duplicate);
        let data = HostWindowPresentationData {
            workbench_window_nodes: crate::ui::layouts::common::model_rc(nodes),
            ..presentation
        };
        assert!(scene_image_selection(&data, &stream, &sources).is_err());
    }

    #[test]
    fn normal_viewport_media_does_not_replace_scene_with_overlay_neighbor_or_cpu_pixels() {
        let (mut presentation, mut stream, _) = fixture();
        // Another actual command may use the same resource, but has no matching scene source.
        let mut outside = stream.commands()[0].clone();
        outside.source = None;
        stream.push_command_for_test(outside);
        let mut overlay = stream.commands()[0].clone();
        if let ChromeCommandKind::Image { payload } = &mut overlay.kind {
            payload.resource_key = "viewport:overlay".into();
        }
        stream.push_command_for_test(overlay);
        let sources = super::super::source_rows(&stream, &presentation);
        assert_eq!(
            scene_image_selection(&presentation, &stream, &sources)
                .unwrap()
                .unwrap()
                .request
                .command_index,
            0
        );
        let mut neighbor = presentation.host_scene_data.document_dock.clone();
        neighbor.surface_key = "neighbor".into();
        neighbor.region_frame.x = 900.0;
        presentation.host_scene_data.document_leaves = vec![neighbor];
        assert!(scene_image_selection(&presentation, &stream, &sources)
            .unwrap()
            .is_none());
        presentation.host_scene_data.document_leaves.clear();
        presentation.viewport_images.replace_scene_for_surface(
            "document:scene:current",
            HostViewportImageData {
                resource_key: "cpu-scene".into(),
                width: 600,
                height: 400,
                rgba: Some(Arc::from(vec![255; 600 * 400 * 4])),
                ..Default::default()
            },
        );
        assert!(scene_image_selection(&presentation, &stream, &sources).is_err());
    }

    #[test]
    fn normal_viewport_media_rejects_changed_post_conversion_command() {
        let (presentation, stream, sources) = fixture();
        let selected = scene_image_selection(&presentation, &stream, &sources)
            .unwrap()
            .unwrap();
        let draw =
            super::super::super::chrome_command_stream::ui_surface_draw_list_from_stream(&stream);
        for mutation in 0..7 {
            let mut changed = draw.clone();
            match mutation {
                0 => changed.commands[0].frame.x += 1.0,
                1 => changed.commands[0].clip = None,
                _ => {
                    if let zircon_runtime::rhi::UiSurfaceCommandKind::Image { payload } =
                        &mut changed.commands[0].kind
                    {
                        match mutation {
                            2 => payload.resource_key = "other".into(),
                            3 => payload.resource_generation += 1,
                            4 => payload.width += 1,
                            5 => payload.rgba = Some(vec![]),
                            _ => {
                                payload.atlas_uv = Some(zircon_runtime::rhi::UiSurfaceImageUvRect {
                                    min: [0.0, 0.0],
                                    max: [0.5, 1.0],
                                })
                            }
                        }
                    }
                }
            }
            assert!(!selected.matches_draw_list(&changed), "mutation {mutation}");
        }
    }
}
