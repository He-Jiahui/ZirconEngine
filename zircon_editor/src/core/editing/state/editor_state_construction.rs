use zircon_math::UVec2;
use zircon_scene::{LevelSystem, NodeKind};

use crate::core::editing::asset_workspace::AssetWorkspaceState;
use crate::core::editing::history::EditorHistory;
use crate::scene::viewport::SceneViewportController;
use crate::module::DEFAULT_PROJECT_PATH;
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};

use super::editor_state::EditorState;
use super::editor_world_slot::EditorWorldSlot;

impl EditorState {
    pub fn new(world: LevelSystem, viewport_size: UVec2) -> Self {
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
            viewport_controller: SceneViewportController::new(viewport_size),
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
            self.viewport_controller.set_selected_node(selection);
        }
    }
}
