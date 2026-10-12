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
        "paintedNativeControlId":null,"submittedOwnerCommandIndices":[]
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
    let Some(hierarchy) =
        crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(presentation)
    else { return result; };
    let prototype = hierarchy.prototype;
    let Some(origin) = identity(prototype) else {
        return result;
    };
    result["originComponent"] = origin;
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
    let (Some(owner_frame), Some(row_frame), Some(owner_id), Some(row_id)) = (
        owner.source_surface_frame.as_ref(),
        prototype.source_surface_frame.as_ref(),
        owner.surface_node_id,
        prototype.surface_node_id,
    ) else {
        return result;
    };
    if !std::sync::Arc::ptr_eq(owner_frame, row_frame)
        || owner_frame.arranged_tree.get(owner_id).is_none()
        || row_frame.arranged_tree.get(row_id).is_none()
        || owner.tree_owner_focused != prototype.tree_owner_focused
    {
        return result;
    }
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
    result["sourceFrame"] = json!({"treeId":row_frame.tree_id,
        "generation":row_frame.generation,"ownerNodeId":owner_id,"rowPrototypeNodeId":row_id});
    result["paintedNativeControlId"] = row["controlId"].clone();
    result["submittedOwnerCommandIndices"] = json!(indices);
    result["sourceOwnerMappingComplete"] = json!(true);
    result["observed"] = json!(prototype.tree_owner_focused.is_some());
    result["value"] = json!(prototype.tree_owner_focused);
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
    use crate::ui::retained_host::host_contract::data::{FrameRect, SceneNodeData};
    use crate::ui::retained_host::host_contract::paint_template_nodes::PaintEvidenceScope;
    use crate::ui::template_runtime::EditorUiHostRuntime;
    use zircon_runtime_interface::ui::layout::UiSize;

    fn fixture(focused: bool) -> HostWindowPresentationData {
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
        let entries = crate::ui::workbench::snapshot::SceneEntries::from_hierarchy_rows_at_generation(
            vec![zircon_runtime::scene::WorldInspectionHierarchyRow {
                entity: 1, parent: None, depth: 0, display_name: "Camera".into(),
                kind: "Entity".into(), subtree_hash: 0, active_in_hierarchy: true,
                has_children: false,
            }], [1], 1,
        );
        bridge.resync_scene_hierarchy(&entries).unwrap();
        assert_eq!(bridge.scene_entity_for_control("WorkbenchSceneRootItem"), Some(1));
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
        let mut result = HostWindowPresentationData {
            workbench_window_nodes:
                crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                    &retained,
                )),
            ..HostWindowPresentationData::default()
        };
        result.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
        result
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes = model_rc(vec![SceneNodeData {
            id: "1".into(),
            name: "Camera".into(),
            selected: true,
            ..Default::default()
        }]);
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
    fn tree_owner_focus_publication_rejects_missing_source_frame() {
        let _guard = env_lock();
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
        row.source_surface_frame = None;
        presentation.workbench_window_nodes = model_rc(nodes);
        let value = evidence(&presentation, None);
        assert!(published(&value)["value"].is_null());
        assert_eq!(published(&value)["observed"], false);
        assert_eq!(published(&value)["sourceOwnerMappingComplete"], false);
    }
}
