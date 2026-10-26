//! Publish the UiSurface owner-focus value consumed by ordinary native TreeRows.
//! Normal present builds this stream and its paint evidence from the same immutable
//! presentation. The ordinary row clone keeps the prototype's owner-focus value;
//! native pane pointer focus and keyboard focus-visible are separate contracts.
use serde_json::{json, Value};

use super::super::super::data::{HostWindowPresentationData, TemplatePaneNodeData};

fn identity(node: &TemplatePaneNodeData) -> Option<Value> {
    (!node.source_path.is_empty()
        && !node.source_node_id.is_empty()
        && !node.instance_path.is_empty())
    .then(|| {
        json!({"sourcePath":node.source_path.as_str(),
            "sourceNodeId":node.source_node_id.as_str(),
            "instancePath":node.instance_path.as_str(),
            "controlId":node.control_id.as_str()})
    })
}

fn same_source(value: &Value, node: &TemplatePaneNodeData) -> bool {
    value["sourcePath"].as_str() == Some(node.source_path.as_str())
        && value["sourceNodeId"].as_str() == Some(node.source_node_id.as_str())
        && value["instancePath"].as_str() == Some(node.instance_path.as_str())
}

pub(super) fn publication(
    presentation: &HostWindowPresentationData,
    sources: &[Option<Value>],
    styles: Option<&Vec<Value>>,
) -> Value {
    let mut result = json!({
        "authority":"normal-template-tree-row-painter-input",
        "sourceAuthority":"UiSurface.focus/tree_selection_owner",
        "value":null,"observed":false,"sourceOwner":null,"originComponent":null,
        "sourceOwnerMappingComplete":false,"sourceFrame":null,
        "paintedNativeControlId":null,"submittedOwnerCommandIndices":[],
        "unavailableReason":"missing-or-ambiguous-tree-owner"
    });
    let models = presentation.paint_node_models();
    let owners = models
        .iter()
        .flat_map(|model| model.iter())
        .filter(|node| node.control_id.as_str() == "WorkbenchSceneTree")
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return result;
    };
    let Some(owner_identity) = identity(owner) else {
        return result;
    };
    result["sourceOwner"] = owner_identity;
    result["unavailableReason"] = json!("authored-hierarchy-unavailable");
    let Some(hierarchy) =
        crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(presentation)
    else { return result; };
    let prototype = hierarchy.prototype;
    let Some(origin) = identity(prototype) else {
        return result;
    };
    result["originComponent"] = origin;
    result["unavailableReason"] = json!("ambiguous-row-prototype-or-owner");
    // A different/ambiguous tree or prototype cannot borrow this owner's focus.
    if prototype.parent_node_id != owner.node_id
        || models
            .iter()
            .flat_map(|model| model.iter())
            .filter(|node| node.node_id == prototype.node_id || same_node_source(node, prototype))
            .count()
            != 1
    {
        return result;
    }
    result["unavailableReason"] = json!("missing-source-surface-frame");
    let (Some(owner_frame), Some(owner_id), Some(row_id)) = (
        owner.source_surface_frame.as_ref(),
        owner.surface_node_id,
        prototype.surface_node_id,
    ) else {
        return result;
    };
    result["unavailableReason"] = json!("source-surface-frame-or-focus-mismatch");
    let (Some(actual_owner), Some(actual_row)) = (
        owner_frame.arranged_tree.get(owner_id),
        owner_frame.arranged_tree.get(row_id),
    ) else {
        return result;
    };
    // The submitted owner's immutable surface proves state ownership. A row
    // prototype need not have an independent replayable primitive; sparse
    // projection also retains unchanged nodes' outer frames. Neither condition
    // changes the focus value consumed by the actual native row painter.
    if actual_owner.node_path.0 != owner.node_id.as_str()
        || actual_owner.control_id.as_deref() != Some(owner.control_id.as_str())
        || actual_row.node_path.0 != prototype.node_id.as_str()
        || actual_row.control_id.as_deref() != Some(prototype.control_id.as_str())
        || actual_row.parent != Some(owner_id)
        || !actual_owner.children.contains(&row_id)
        || !actual_owner.is_render_visible()
        || !actual_row.is_render_visible()
        || actual_owner.enabled == owner.disabled
        || actual_row.enabled == prototype.disabled
        || prototype.parent_source_path != owner.source_path
        || prototype.parent_source_node_id != owner.source_node_id
        || prototype.parent_instance_path != owner.instance_path
        || owner.tree_owner_focused != prototype.tree_owner_focused
        || prototype
            .source_surface_frame
            .as_ref()
            .is_some_and(|row_frame| {
                row_frame.tree_id != owner_frame.tree_id
                    || row_frame.arranged_tree.get(row_id) != Some(actual_row)
                    || row_frame.focus_state != owner_frame.focus_state
            })
    {
        return result;
    }
    result["unavailableReason"] = json!("missing-or-ambiguous-painted-prototype");
    let Some(styles) = styles else {
        return result;
    };
    let rows = styles
        .iter()
        .filter(|style| same_source(style, prototype))
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return result;
    };
    result["unavailableReason"] = json!("painted-prototype-is-not-a-current-native-entity");
    let Some(entity_id) = row["controlId"]
        .as_str()
        .and_then(|control| control.strip_prefix("NativeHierarchyEntity:"))
    else {
        return result;
    };
    if row["nodeId"].as_str() != Some(format!("{}/entity/{entity_id}", prototype.node_id).as_str())
        || !hierarchy
            .pane
            .hierarchy
            .hierarchy_nodes
            .iter()
            .any(|entity| entity.id.as_str() == entity_id)
    {
        return result;
    }
    result["unavailableReason"] = json!("submitted-owner-command-not-observed-in-source-frame");
    // The owner itself must also be in this exact submitted source frame.
    // This is owner-command admission, not a claim of row primitive identity.
    let indices = sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            let source = source.as_ref()?;
            (same_source(source, owner)
                && source["controlId"].as_str() == Some(owner.control_id.as_str())
                && source["sourceRef"]["authority"] == "surface-node-owner"
                && source["sourceRef"]["treeId"].as_str() == Some(owner_frame.tree_id.0.as_str())
                && source["sourceRef"]["generation"].as_u64() == Some(owner_frame.generation)
                && source["sourceRef"]["nodeId"].as_u64() == Some(owner_id.0))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if indices.is_empty() {
        return result;
    }
    result["sourceFrame"] = json!({"treeId":owner_frame.tree_id,
        "generation":owner_frame.generation,"ownerNodeId":owner_id,"rowPrototypeNodeId":row_id});
    result["paintedNativeControlId"] = row["controlId"].clone();
    result["submittedOwnerCommandIndices"] = json!(indices);
    result["sourceOwnerMappingComplete"] = json!(true);
    result["observed"] = json!(prototype.tree_owner_focused.is_some());
    result["value"] = json!(prototype.tree_owner_focused);
    result["unavailableReason"] = if prototype.tree_owner_focused.is_some() {
        Value::Null
    } else {
        json!("owner-focus-not-observed")
    };
    result
}

fn same_node_source(left: &TemplatePaneNodeData, right: &TemplatePaneNodeData) -> bool {
    left.source_path == right.source_path
        && left.source_node_id == right.source_node_id
        && left.instance_path == right.instance_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::env_lock;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::chrome_command_stream::build_chrome_command_stream;
    use crate::ui::retained_host::host_contract::data::{
        FrameRect, HostWindowLayoutData, SceneNodeData,
    };
    use crate::ui::retained_host::host_contract::paint_template_nodes::PaintEvidenceScope;
    use crate::ui::template_runtime::EditorUiHostRuntime;
    use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

    fn fixture(focused: bool) -> HostWindowPresentationData {
        fixture_with_entities(focused, 1)
    }

    fn fixture_with_entities(focused: bool, entity_count: usize) -> HostWindowPresentationData {
        assert!((1..=3).contains(&entity_count));
        let mut runtime = EditorUiHostRuntime::default();
        let document = crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
        runtime
            .load_builtin_host_templates_for_document_ids(&[document])
            .unwrap();
        let projection = runtime.project_document(document).unwrap();
        let runtime = std::sync::Arc::new(runtime);
        let mut bridge = crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(
            std::sync::Arc::clone(&runtime), UiSize::new(1280.0, 800.0),
        ).unwrap();
        let entries =
            crate::ui::workbench::snapshot::SceneEntries::from_hierarchy_rows_at_generation(
                (1..=entity_count)
                    .map(
                        |entity| zircon_runtime::scene::WorldInspectionHierarchyRow {
                            entity: entity as _,
                            parent: None,
                            depth: 0,
                            display_name: ["Camera", "Sun", "Cube"][entity - 1].into(),
                            kind: "Entity".into(),
                            subtree_hash: 0,
                            active_in_hierarchy: true,
                            has_children: false,
                        },
                    )
                    .collect::<Vec<_>>(),
                [1],
                1,
            );
        bridge.resync_scene_hierarchy(&entries).unwrap();
        assert_eq!(
            bridge.scene_entity_for_control("WorkbenchSceneRootItem"),
            Some(1)
        );
        let mut surface = bridge.surface().clone();
        let owner = surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some("WorkbenchSceneTree")
            })
            .unwrap()
            .node_id;
        if focused {
            surface.focus_node(owner).unwrap();
        } else {
            surface.clear_focus();
        }
        surface.compute_layout(UiSize::new(1280.0, 800.0)).unwrap();
        let retained = runtime
            .build_retained_host_projection_with_surface(&projection, &surface)
            .unwrap();
        // Match the mounted host's chrome/pane route. A zero layout instead
        // paints the authored row prototype through the chrome fallback before
        // the native hierarchy rows, so it cannot prove a current entity draw.
        let frames = bridge.layout_frames();
        let frame_rect = |frame: Option<UiFrame>| {
            let frame = frame.unwrap_or_default();
            FrameRect {
                x: frame.x,
                y: frame.y,
                width: frame.width,
                height: frame.height,
            }
        };
        let host_layout = HostWindowLayoutData {
            authoritative: true,
            center_band_frame: frame_rect(frames.center_band_frame),
            status_bar_frame: frame_rect(frames.status_bar_frame),
            left_region_frame: frame_rect(frames.left_region_frame),
            document_region_frame: frame_rect(frames.document_region_frame),
            right_region_frame: frame_rect(frames.right_region_frame),
            bottom_region_frame: frame_rect(frames.bottom_region_frame),
            left_splitter_frame: frame_rect(frames.left_resize_splitter_frame),
            right_splitter_frame: frame_rect(frames.right_resize_splitter_frame),
            bottom_splitter_frame: frame_rect(frames.bottom_resize_splitter_frame),
            viewport_content_frame: frame_rect(frames.viewport_content_frame),
        };
        assert!(host_layout.center_band_frame.width > 0.0);
        assert!(host_layout.center_band_frame.height > 0.0);
        assert!(host_layout.status_bar_frame.width > 0.0);
        assert!(host_layout.status_bar_frame.height > 0.0);
        let mut result = HostWindowPresentationData {
            host_layout: host_layout.clone(),
            workbench_window_nodes:
                crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                    &retained,
                )),
            ..HostWindowPresentationData::default()
        };
        result.host_scene_data.layout = host_layout;
        result.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
        result
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes = model_rc(
            (1..=entity_count)
                .map(|entity| SceneNodeData {
                    id: entity.to_string().into(),
                    name: ["Camera", "Sun", "Cube"][entity - 1].into(),
                    selected: entity == 1,
                    ..Default::default()
                })
                .collect(),
        );
        // Deliberately differs from UiSurface focus for the unfocused case.
        result.pane_interaction_state.template_focus_observed = true;
        result.pane_interaction_state.focused_template_control_id = "HierarchyListPanel".into();
        result.pane_interaction_state.template_focus_visible = false;
        result
    }

    fn evidence(presentation: &HostWindowPresentationData, damage: Option<&FrameRect>) -> Value {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let scope = PaintEvidenceScope::begin_for_normal_present(repo).unwrap();
        let stream = build_chrome_command_stream(presentation, (1280, 800), damage, false);
        let sources = super::super::super::source_rows(&stream, presentation);
        let paint = scope.finish().unwrap();
        super::super::stream_observations(&stream, presentation, &sources, Ok(paint))
    }

    fn published(value: &Value) -> &Value {
        &value["semanticNodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["controlId"] == "WorkbenchSceneTree")
            .unwrap()["treeOwnerFocusPublication"]
    }

    #[test]
    fn tree_owner_focus_publication_tracks_three_actual_native_entities() {
        let _guard = env_lock();
        for focused in [false, true] {
            let value = evidence(&fixture_with_entities(focused, 3), None);
            let owner = published(&value);
            assert_eq!(owner["observed"], true, "{owner}");
            assert_eq!(owner["value"], focused);
            assert_eq!(owner["sourceOwnerMappingComplete"], true);
            assert_eq!(owner["paintedNativeControlId"], "NativeHierarchyEntity:1");
            assert!(owner["unavailableReason"].is_null());
        }
    }

    #[test]
    fn tree_owner_focus_publication_uses_actual_surface_input_without_pointer_or_ring_inference() {
        let _guard = env_lock();
        for focused in [false, true] {
            let presentation = fixture(focused);
            let before = presentation
                .host_scene_data
                .left_dock
                .pane
                .hierarchy
                .hierarchy_nodes
                .get(0)
                .unwrap()
                .clone();
            let value = evidence(&presentation, None);
            let owner = published(&value);
            assert_eq!(owner["observed"], true);
            assert_eq!(owner["value"], focused);
            assert_eq!(owner["sourceOwnerMappingComplete"], true);
            assert_eq!(
                owner["sourceAuthority"],
                "UiSurface.focus/tree_selection_owner"
            );
            assert_eq!(
                value["focusPublication"]["focusedNativeControlId"],
                "HierarchyListPanel"
            );
            assert_eq!(value["focusPublication"]["templateFocusVisible"], false);
            let after = presentation
                .host_scene_data
                .left_dock
                .pane
                .hierarchy
                .hierarchy_nodes
                .get(0)
                .unwrap();
            assert_eq!(after.id, before.id);
            assert_eq!(after.selected, before.selected);
        }
    }

    #[test]
    fn tree_owner_focus_publication_preserves_unknown_and_rejects_ambiguous_or_unpainted_rows() {
        let _guard = env_lock();
        let mut presentation = fixture(true);
        let original = presentation
            .workbench_window_nodes
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut unknown = original.clone();
        for node in &mut unknown {
            if node.control_id.as_str() == "WorkbenchSceneTree"
                || node.control_id.as_str() == "WorkbenchSceneRootItem"
            {
                node.tree_owner_focused = None;
            }
        }
        presentation.workbench_window_nodes = model_rc(unknown);
        let value = evidence(&presentation, None);
        assert!(published(&value)["value"].is_null());
        assert_eq!(published(&value)["observed"], false);
        assert_eq!(published(&value)["sourceOwnerMappingComplete"], true);

        let mut duplicate = original.clone();
        duplicate.push(
            original
                .iter()
                .find(|node| node.control_id.as_str() == "WorkbenchSceneTree")
                .unwrap()
                .clone(),
        );
        presentation.workbench_window_nodes = model_rc(duplicate);
        let value = evidence(&presentation, None);
        assert!(published(&value)["value"].is_null());
        assert_eq!(published(&value)["sourceOwnerMappingComplete"], false);

        let mut fallback = fixture(true);
        fallback.host_layout = HostWindowLayoutData::default();
        fallback.host_scene_data.layout = HostWindowLayoutData::default();
        let value = evidence(&fallback, None);
        assert!(published(&value)["value"].is_null());
        assert_eq!(published(&value)["observed"], false);
        assert_eq!(published(&value)["sourceOwnerMappingComplete"], false);
        assert_eq!(
            published(&value)["unavailableReason"],
            "painted-prototype-is-not-a-current-native-entity"
        );

        presentation.workbench_window_nodes = model_rc(original);
        presentation
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes = model_rc(Vec::new());
        let value = evidence(&presentation, None);
        assert!(published(&value)["value"].is_null());
        assert_eq!(published(&value)["sourceOwnerMappingComplete"], false);
    }

    #[test]
    fn tree_owner_focus_publication_accepts_prototype_without_independent_primitive_anchor() {
        let _guard = env_lock();
        for focused in [false, true] {
            let mut presentation = fixture_with_entities(focused, 3);
            let mut nodes = presentation
                .workbench_window_nodes
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            let row = nodes
                .iter_mut()
                .find(|node| node.control_id.as_str() == "WorkbenchSceneRootItem")
                .unwrap();
            row.source_surface_frame = None;
            row.surface_render_command_ref = None;
            presentation.workbench_window_nodes = model_rc(nodes);
            let value = evidence(&presentation, None);
            assert_eq!(published(&value)["value"], focused, "{}", published(&value));
            assert_eq!(published(&value)["observed"], true);
            assert_eq!(published(&value)["sourceOwnerMappingComplete"], true);
        }
    }

    #[test]
    fn tree_owner_focus_publication_accepts_cached_row_frame_with_unchanged_arranged_and_focus_state(
    ) {
        let _guard = env_lock();
        for focused in [false, true] {
            let mut presentation = fixture_with_entities(focused, 3);
            let mut nodes = presentation
                .workbench_window_nodes
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            let row = nodes
                .iter_mut()
                .find(|node| node.control_id.as_str() == "WorkbenchSceneRootItem")
                .unwrap();
            let mut cached = (**row.source_surface_frame.as_ref().unwrap()).clone();
            cached.generation = cached.generation.saturating_sub(1);
            row.source_surface_frame = Some(std::sync::Arc::new(cached));
            presentation.workbench_window_nodes = model_rc(nodes);
            let value = evidence(&presentation, None);
            assert_eq!(published(&value)["value"], focused, "{}", published(&value));
            assert_eq!(published(&value)["sourceOwnerMappingComplete"], true);
        }
    }

    #[test]
    fn tree_owner_focus_publication_rejects_wrong_prototype_identity_parent_or_focus_state() {
        let _guard = env_lock();
        for mutation in 0..10 {
            let mut presentation = fixture(true);
            let mut nodes = presentation
                .workbench_window_nodes
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            let row = nodes
                .iter_mut()
                .find(|node| node.control_id.as_str() == "WorkbenchSceneRootItem")
                .unwrap();
            match mutation {
                0 => row.surface_node_id = None,
                1 => row.node_id = "wrong-runtime-row".into(),
                2 => row.parent_node_id = "wrong-runtime-parent".into(),
                3 => {
                    row.parent_source_path = "res://ui/editor/unreal/components/details.zui".into()
                }
                4 => row.parent_source_node_id = "foreign-tree".into(),
                5 => row.parent_instance_path = "[]".into(),
                6 => row.disabled = true,
                7 => row.tree_owner_focused = Some(false),
                8 => {
                    let mut foreign = (**row.source_surface_frame.as_ref().unwrap()).clone();
                    foreign.tree_id.0.push_str(".foreign");
                    row.source_surface_frame = Some(std::sync::Arc::new(foreign));
                }
                _ => {
                    let mut stale = (**row.source_surface_frame.as_ref().unwrap()).clone();
                    let mut focus = (*stale.focus_state).clone();
                    focus.focused = None;
                    stale.focus_state = std::sync::Arc::new(focus);
                    row.source_surface_frame = Some(std::sync::Arc::new(stale));
                }
            }
            presentation.workbench_window_nodes = model_rc(nodes);
            let value = evidence(&presentation, None);
            assert!(
                published(&value)["value"].is_null(),
                "mutation {mutation}: {}",
                published(&value)
            );
            assert_eq!(published(&value)["sourceOwnerMappingComplete"], false);
        }
    }

    #[test]
    fn tree_owner_focus_publication_rejects_missing_submitted_owner_source_frame() {
        let _guard = env_lock();
        let mut presentation = fixture(true);
        let mut nodes = presentation
            .workbench_window_nodes
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let row = nodes
            .iter_mut()
            .find(|node| node.control_id.as_str() == "WorkbenchSceneTree")
            .unwrap();
        row.source_surface_frame = None;
        presentation.workbench_window_nodes = model_rc(nodes);
        let value = evidence(&presentation, None);
        assert!(published(&value)["value"].is_null());
        assert_eq!(published(&value)["observed"], false);
        assert_eq!(published(&value)["sourceOwnerMappingComplete"], false);
    }
}
