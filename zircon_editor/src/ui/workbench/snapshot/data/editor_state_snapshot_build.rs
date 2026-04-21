use crate::ui::workbench::state::EditorState;
use zircon_runtime::scene::Scene;

use super::super::{AssetSurfaceMode, EditorDataSnapshot, InspectorSnapshot, SceneEntry};

impl EditorState {
    pub fn snapshot(&self) -> EditorDataSnapshot {
        let selected = self.viewport_controller.selected_node();
        let (scene_entries, inspector) = self
            .world
            .try_with_world(|scene| {
                let inspector = selected
                    .and_then(|id| scene.find_node(id).map(|node| (id, node)))
                    .map(|(id, _node)| InspectorSnapshot {
                        id,
                        name: self.name_field.clone(),
                        parent: self.parent_field.clone(),
                        translation: self.transform_fields.clone(),
                    });
                let scene_entries = scene
                    .nodes()
                    .iter()
                    .map(|node| SceneEntry {
                        id: node.id,
                        name: node.name.clone(),
                        depth: hierarchy_depth(scene, node.id),
                        selected: selected == Some(node.id),
                    })
                    .collect();

                (scene_entries, inspector)
            })
            .unwrap_or_else(|| (Vec::new(), None));

        EditorDataSnapshot {
            scene_entries,
            inspector,
            status_line: self.status_line.clone(),
            hovered_axis: self.viewport_controller.hovered_axis(),
            viewport_size: self.viewport_controller.viewport().size,
            scene_viewport_settings: self.viewport_controller.settings().clone(),
            mesh_import_path: self.mesh_import_path.clone(),
            project_overview: self.asset_workspace.project_overview(),
            asset_activity: self
                .asset_workspace
                .build_snapshot(AssetSurfaceMode::Activity),
            asset_browser: self
                .asset_workspace
                .build_snapshot(AssetSurfaceMode::Explorer),
            project_path: self.project_path.clone(),
            session_mode: self.session_mode,
            welcome: self.welcome.clone(),
            project_open: self.project_open,
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }
}

fn hierarchy_depth(scene: &Scene, node_id: zircon_runtime::scene::NodeId) -> usize {
    let mut depth = 0;
    let mut cursor = scene.find_node(node_id).and_then(|node| node.parent);
    while let Some(parent) = cursor {
        depth += 1;
        cursor = scene.find_node(parent).and_then(|node| node.parent);
    }
    depth
}
