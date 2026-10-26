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
#[path = "tests/present.rs"]
mod tests;

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
#[path = "tests/media.rs"]
mod normal_viewport_media_tests;
