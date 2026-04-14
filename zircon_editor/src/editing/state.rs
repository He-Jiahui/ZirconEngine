//! Mutable editor session: scene, viewport, undo, and project I/O.

use zircon_graphics::{ViewportController, ViewportFeedback, ViewportInput, ViewportState};
use zircon_manager::{
    AssetRecordKind, EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
    ResourceStatusRecord,
};
use zircon_math::UVec2;
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_scene::{LevelSystem, NodeKind, RenderSceneSnapshot, Scene};

use crate::command::{EditorCommand, NodeEditState};
use crate::editing::asset_workspace::AssetWorkspaceState;
use crate::history::EditorHistory;
use crate::intent::EditorIntent;
use crate::module::DEFAULT_PROJECT_PATH;
use crate::snapshot::{
    AssetSurfaceMode, AssetUtilityTab, AssetViewMode, EditorDataSnapshot, InspectorSnapshot,
    SceneEntry,
};
use crate::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

#[derive(Debug, Default)]
pub struct EditorWorldSlot {
    inner: Option<LevelSystem>,
}

impl EditorWorldSlot {
    #[allow(dead_code)]
    pub fn loaded(world: LevelSystem) -> Self {
        Self { inner: Some(world) }
    }

    pub fn unloaded() -> Self {
        Self { inner: None }
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.is_some()
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> Scene {
        self.inner
            .as_ref()
            .expect("editor world is not loaded")
            .snapshot()
    }

    pub fn try_snapshot(&self) -> Option<Scene> {
        self.inner.as_ref().map(LevelSystem::snapshot)
    }

    #[allow(dead_code)]
    pub fn with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> R {
        self.inner
            .as_ref()
            .expect("editor world is not loaded")
            .with_world(read)
    }

    pub fn try_with_world<R>(&self, read: impl FnOnce(&Scene) -> R) -> Option<R> {
        self.inner.as_ref().map(|world| world.with_world(read))
    }

    #[allow(dead_code)]
    pub fn with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> R {
        self.inner
            .as_ref()
            .expect("editor world is not loaded")
            .with_world_mut(write)
    }

    pub fn try_with_world_mut<R>(&self, write: impl FnOnce(&mut Scene) -> R) -> Option<R> {
        self.inner.as_ref().map(|world| world.with_world_mut(write))
    }

    pub fn replace(&mut self, world: LevelSystem) {
        self.inner = Some(world);
    }

    pub fn clear(&mut self) {
        self.inner = None;
    }
}

/// Editor shell state shared between the UI host and the scene server.
#[derive(Debug)]
pub struct EditorState {
    pub(crate) world: EditorWorldSlot,
    viewport_controller: ViewportController,
    name_field: String,
    parent_field: String,
    transform_fields: [String; 3],
    mesh_import_path: String,
    asset_workspace: AssetWorkspaceState,
    project_path: String,
    session_mode: EditorSessionMode,
    welcome: WelcomePaneSnapshot,
    project_open: bool,
    status_line: String,
    history: EditorHistory,
}

impl EditorState {
    pub fn new(world: LevelSystem, viewport_size: UVec2) -> Self {
        world.with_world_mut(|scene| scene.set_selected(None));
        Self::new_with_world(
            EditorWorldSlot::loaded(world),
            viewport_size,
            DEFAULT_PROJECT_PATH.to_string(),
            EditorSessionMode::Welcome,
            WelcomePaneSnapshot::default(),
            false,
            "Ready".to_string(),
        )
    }

    pub fn with_default_selection(world: LevelSystem, viewport_size: UVec2) -> Self {
        let mut state = Self::new(world, viewport_size);
        state.select_default_node();
        state.sync_selection_state();
        state
    }

    pub fn project(
        world: LevelSystem,
        viewport_size: UVec2,
        project_path: impl Into<String>,
    ) -> Self {
        let mut state = Self::new_with_world(
            EditorWorldSlot::loaded(world),
            viewport_size,
            project_path.into(),
            EditorSessionMode::Project,
            WelcomePaneSnapshot::default(),
            true,
            "Ready".to_string(),
        );
        state.sync_selection_state();
        state
    }

    pub fn welcome(viewport_size: UVec2, welcome: WelcomePaneSnapshot) -> Self {
        let status_line = if welcome.status_message.trim().is_empty() {
            "Ready".to_string()
        } else {
            welcome.status_message.clone()
        };
        Self::new_with_world(
            EditorWorldSlot::unloaded(),
            viewport_size,
            String::new(),
            EditorSessionMode::Welcome,
            welcome,
            false,
            status_line,
        )
    }

    fn new_with_world(
        world: EditorWorldSlot,
        viewport_size: UVec2,
        project_path: String,
        session_mode: EditorSessionMode,
        welcome: WelcomePaneSnapshot,
        project_open: bool,
        status_line: String,
    ) -> Self {
        Self {
            world,
            viewport_controller: ViewportController::new(ViewportState::new(viewport_size)),
            name_field: String::new(),
            parent_field: String::new(),
            transform_fields: Default::default(),
            mesh_import_path: String::new(),
            asset_workspace: AssetWorkspaceState::default(),
            project_path,
            session_mode,
            welcome,
            project_open,
            status_line,
            history: EditorHistory::default(),
        }
    }

    pub fn apply_intent(&mut self, intent: EditorIntent) -> Result<bool, String> {
        match intent {
            EditorIntent::CreateNode(kind) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::create_node(scene, kind))
                    .ok_or_else(no_project_open)??;
                let id = command.target_node();
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Created node {id}");
                Ok(true)
            }
            EditorIntent::DeleteNode(id) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::delete_node(scene, id))
                    .ok_or_else(no_project_open)??;
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Deleted node {id}");
                Ok(true)
            }
            EditorIntent::SelectNode(id) => {
                if self
                    .world
                    .try_with_world(|scene| scene.find_node(id).is_none())
                    .ok_or_else(no_project_open)?
                {
                    return Err(format!("Cannot select missing node {id}"));
                }
                self.world
                    .try_with_world_mut(|scene| scene.set_selected(Some(id)))
                    .ok_or_else(no_project_open)?;
                self.sync_selection_state();
                self.status_line = format!("Selected node {id}");
                Ok(true)
            }
            EditorIntent::RenameNode(id, name) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::rename_node(scene, id, name))
                    .ok_or_else(no_project_open)??;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Renamed node {id}");
                Ok(true)
            }
            EditorIntent::SetParent(id, parent) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::set_parent(scene, id, parent))
                    .ok_or_else(no_project_open)??;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = match parent {
                    Some(parent) => format!("Reparented node {id} under {parent}"),
                    None => format!("Detached node {id} to root"),
                };
                Ok(true)
            }
            EditorIntent::SetTransform(id, transform) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::set_transform(scene, id, transform))
                    .ok_or_else(no_project_open)??;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Updated transform for node {id}");
                Ok(true)
            }
            EditorIntent::ApplyInspectorChanges => self.apply_inspector_changes(),
            EditorIntent::BeginGizmoDrag => {
                let history = &mut self.history;
                self.world
                    .try_with_world(|scene| history.begin_drag(scene))
                    .ok_or_else(no_project_open)?;
                self.status_line = "Translate gizmo drag".to_string();
                Ok(false)
            }
            EditorIntent::DragGizmo => {
                self.status_line = "Dragging translate gizmo".to_string();
                Ok(false)
            }
            EditorIntent::EndGizmoDrag => {
                let history = &mut self.history;
                let command = self
                    .world
                    .try_with_world(|scene| history.end_drag(scene))
                    .ok_or_else(no_project_open)??;
                if let Some(command) = command {
                    self.history.push(command);
                    self.sync_selection_state();
                }
                self.status_line = "Gizmo drag finished".to_string();
                Ok(false)
            }
            EditorIntent::Undo => {
                let history = &mut self.history;
                let changed = self
                    .world
                    .try_with_world_mut(|scene| history.undo(scene))
                    .ok_or_else(no_project_open)??;
                if changed {
                    self.sync_selection_state();
                    self.status_line = "Undo".to_string();
                    Ok(true)
                } else {
                    self.status_line = "Nothing to undo".to_string();
                    Ok(false)
                }
            }
            EditorIntent::Redo => {
                let history = &mut self.history;
                let changed = self
                    .world
                    .try_with_world_mut(|scene| history.redo(scene))
                    .ok_or_else(no_project_open)??;
                if changed {
                    self.sync_selection_state();
                    self.status_line = "Redo".to_string();
                    Ok(true)
                } else {
                    self.status_line = "Nothing to redo".to_string();
                    Ok(false)
                }
            }
        }
    }

    pub fn snapshot(&self) -> EditorDataSnapshot {
        let (scene_entries, inspector) = self
            .world
            .try_with_world(|scene| {
                let inspector = scene
                    .selected_node()
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
                        selected: scene.selected_node() == Some(node.id),
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
            mesh_import_path: self.mesh_import_path.clone(),
            project_overview: self.asset_workspace.project_overview(),
            asset_activity: self.asset_workspace.build_snapshot(AssetSurfaceMode::Activity),
            asset_browser: self.asset_workspace.build_snapshot(AssetSurfaceMode::Explorer),
            project_path: self.project_path.clone(),
            session_mode: self.session_mode,
            welcome: self.welcome.clone(),
            project_open: self.project_open,
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }

    pub fn viewport_state(&self) -> ViewportState {
        self.viewport_controller.viewport().clone()
    }

    pub fn render_snapshot(&self) -> Option<RenderSceneSnapshot> {
        self.world.try_with_world(|scene| scene.to_render_snapshot())
    }

    pub fn handle_viewport_input(&mut self, input: ViewportInput) -> ViewportFeedback {
        let Some(feedback) = self
            .world
            .try_with_world_mut(|scene| self.viewport_controller.handle_input(scene, input))
        else {
            return ViewportFeedback::default();
        };

        if feedback.transformed_node.is_some() {
            self.sync_selection_state();
        }

        let orbit_target = self
            .world
            .try_with_world(|scene| {
                scene
                    .selected_node()
                    .and_then(|selected| scene.find_node(selected))
                    .map(|node| node.transform.translation)
            })
            .flatten();
        if let Some(target) = orbit_target {
            self.viewport_controller.set_orbit_target(target);
        }
        if let Some(axis) = feedback.hovered_axis {
            self.status_line = format!("Hover gizmo axis {:?}", axis);
        }
        feedback
    }

    pub fn update_translation_field(&mut self, axis: usize, value: String) -> bool {
        self.transform_fields[axis] = value;
        false
    }

    pub fn update_name_field(&mut self, value: String) {
        self.name_field = value;
    }

    pub fn update_parent_field(&mut self, value: String) {
        self.parent_field = value;
    }

    pub fn delete_selected(&mut self) -> Result<bool, String> {
        let selected = self
            .world
            .try_with_world(|scene| scene.selected_node())
            .ok_or_else(no_project_open)?;
        let Some(node_id) = selected else {
            self.status_line = "Nothing selected".to_string();
            return Ok(false);
        };
        self.apply_intent(EditorIntent::DeleteNode(node_id))
    }

    pub fn apply_inspector_changes(&mut self) -> Result<bool, String> {
        let selected = self
            .world
            .try_with_world(|scene| {
                scene.selected_node().and_then(|node_id| {
                    scene
                        .find_node(node_id)
                        .cloned()
                        .map(|node| (node_id, node))
                })
            })
            .ok_or_else(no_project_open)?;
        let Some((node_id, current)) = selected else {
            return Err("Nothing selected".to_string());
        };

        let parent = parse_parent_field(&self.parent_field)?;
        let parsed = [
            self.transform_fields[0].parse::<f32>(),
            self.transform_fields[1].parse::<f32>(),
            self.transform_fields[2].parse::<f32>(),
        ];
        let [Ok(x), Ok(y), Ok(z)] = parsed else {
            return Err("Transform fields must be valid numbers".to_string());
        };
        let transform = zircon_math::Transform {
            translation: zircon_math::Vec3::new(x, y, z),
            ..current.transform
        };
        let command = self
            .world
            .try_with_world_mut(|scene| {
                EditorCommand::update_node(scene, node_id, self.name_field.clone(), parent, transform)
            })
            .ok_or_else(no_project_open)??;
        let Some(command) = command else {
            return Ok(false);
        };
        self.history.push(command);
        self.sync_selection_state();
        self.status_line = format!("Applied inspector changes to node {node_id}");
        Ok(true)
    }

    pub fn set_mesh_import_path(&mut self, value: String) {
        self.mesh_import_path = value;
    }

    pub fn sync_asset_catalog(&mut self, catalog: EditorAssetCatalogSnapshotRecord) {
        self.asset_workspace.sync_catalog(catalog);
    }

    pub fn sync_asset_details(&mut self, details: Option<EditorAssetDetailsRecord>) {
        self.asset_workspace.sync_selected_details(details);
    }

    pub fn sync_asset_resources(&mut self, resources: Vec<ResourceStatusRecord>) {
        self.asset_workspace.sync_resources(resources);
    }

    pub fn select_asset_folder(&mut self, folder_id: impl Into<String>) {
        self.asset_workspace.select_folder(folder_id);
    }

    pub fn select_asset(&mut self, asset_uuid: Option<String>) {
        self.asset_workspace.select_asset(asset_uuid);
    }

    pub fn navigate_to_asset(&mut self, asset_uuid: &str) {
        self.asset_workspace.navigate_to_asset(asset_uuid);
    }

    pub fn set_asset_search_query(&mut self, query: impl Into<String>) {
        self.asset_workspace.set_search_query(query);
    }

    pub fn set_asset_kind_filter(&mut self, kind_filter: Option<AssetRecordKind>) {
        self.asset_workspace.set_kind_filter(kind_filter);
    }

    pub fn set_asset_activity_view_mode(&mut self, view_mode: AssetViewMode) {
        self.asset_workspace.set_activity_view_mode(view_mode);
    }

    pub fn set_asset_browser_view_mode(&mut self, view_mode: AssetViewMode) {
        self.asset_workspace.set_browser_view_mode(view_mode);
    }

    pub fn set_asset_activity_tab(&mut self, tab: AssetUtilityTab) {
        self.asset_workspace.set_activity_utility_tab(tab);
    }

    pub fn set_asset_browser_tab(&mut self, tab: AssetUtilityTab) {
        self.asset_workspace.set_browser_utility_tab(tab);
    }

    pub fn set_project_path(&mut self, value: String) {
        self.project_path = value;
    }

    pub fn import_mesh_asset(
        &mut self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
        display_path: impl Into<String>,
    ) -> Result<bool, String> {
        let command = self
            .world
            .try_with_world_mut(|scene| EditorCommand::import_mesh(scene, model, material))
            .ok_or_else(no_project_open)??;
        let id = command.target_node();
        self.mesh_import_path = display_path.into();
        self.history.push(command);
        self.sync_selection_state();
        self.status_line = format!("Imported mesh node {id}");
        Ok(true)
    }

    pub fn replace_world(&mut self, world: LevelSystem, project_path: impl Into<String>) {
        self.world.replace(world);
        self.project_path = project_path.into();
        self.session_mode = EditorSessionMode::Project;
        self.project_open = true;
        self.welcome = WelcomePaneSnapshot::default();
        self.history.clear();
        self.sync_selection_state();
    }

    pub fn clear_project(&mut self, welcome: WelcomePaneSnapshot) {
        self.world.clear();
        self.project_path.clear();
        self.session_mode = EditorSessionMode::Welcome;
        self.project_open = false;
        self.welcome = welcome;
        self.history.clear();
        self.sync_selection_state();
    }

    pub fn mark_project_open(&mut self) {
        self.session_mode = EditorSessionMode::Project;
        self.project_open = true;
    }

    pub fn set_session_mode(&mut self, session_mode: EditorSessionMode) {
        self.session_mode = session_mode;
    }

    pub fn set_welcome_snapshot(&mut self, welcome: WelcomePaneSnapshot) {
        self.welcome = welcome;
        if self.session_mode == EditorSessionMode::Welcome {
            self.status_line = self.welcome.status_message.clone();
        }
    }

    pub fn project_scene(&self) -> Option<Scene> {
        self.world.try_snapshot()
    }

    pub fn has_project_world(&self) -> bool {
        self.world.is_loaded()
    }

    pub fn set_status_line(&mut self, value: impl Into<String>) {
        self.status_line = value.into();
    }

    fn select_default_node(&mut self) {
        let selection = self.world.try_with_world(|scene| {
            scene
                .nodes()
                .iter()
                .find(|node| matches!(&node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .or_else(|| {
                    scene
                        .nodes()
                        .iter()
                        .find(|node| matches!(&node.kind, NodeKind::Camera))
                        .map(|node| node.id)
                })
                .or_else(|| scene.nodes().first().map(|node| node.id))
        });
        if let Some(selection) = selection {
            let _ = self
                .world
                .try_with_world_mut(|scene| scene.set_selected(selection));
        }
    }

    fn sync_selection_state(&mut self) {
        let selected_state = self
            .world
            .try_with_world(|scene| {
                scene
                    .selected_node()
                    .and_then(|selected| scene.find_node(selected))
                    .map(|node| NodeEditState {
                        name: node.name.clone(),
                        parent: node.parent,
                        transform: node.transform,
                    })
            })
            .flatten();
        if let Some(node) = selected_state {
            let translation = node.transform.translation;
            self.name_field = node.name;
            self.parent_field = node
                .parent
                .map(|value| value.to_string())
                .unwrap_or_default();
            self.transform_fields = [
                format!("{:.2}", translation.x),
                format!("{:.2}", translation.y),
                format!("{:.2}", translation.z),
            ];
            self.viewport_controller.set_orbit_target(translation);
            return;
        }

        self.name_field.clear();
        self.parent_field.clear();
        self.transform_fields = [String::new(), String::new(), String::new()];
    }
}

fn no_project_open() -> String {
    "No project open".to_string()
}

fn parse_parent_field(value: &str) -> Result<Option<zircon_scene::NodeId>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<zircon_scene::NodeId>()
        .map(Some)
        .map_err(|error| format!("Parent field must be a valid node id: {error}"))
}

fn hierarchy_depth(scene: &Scene, node_id: zircon_scene::NodeId) -> usize {
    let mut depth = 0;
    let mut cursor = scene.find_node(node_id).and_then(|node| node.parent);
    while let Some(parent) = cursor {
        depth += 1;
        cursor = scene.find_node(parent).and_then(|node| node.parent);
    }
    depth
}
