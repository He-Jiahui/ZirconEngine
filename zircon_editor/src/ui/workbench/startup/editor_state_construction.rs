use std::path::Path;
use std::sync::Arc;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::UVec2;

use crate::core::context::EditorContext;
#[cfg(test)]
use crate::core::context::EditorContextBuilder;
use crate::core::editing::authoring_world::{AuthoringWorldSeed, EditorAuthoringWorld};
#[cfg(test)]
use crate::core::jobs::test_job_scheduler;
use crate::scene::viewport::SceneViewportController;
use crate::ui::workbench::project::AssetWorkspaceState;
use crate::ui::workbench::state::EditorState;

use super::{EditorSessionMode, WelcomePaneSnapshot};

const DEFAULT_PROJECT_PATH: &str = "sandbox-project";

impl EditorState {
    #[cfg(test)]
    pub fn new(world: impl Into<AuthoringWorldSeed>, viewport_size: UVec2) -> Self {
        Self::new_with_context(
            world,
            viewport_size,
            EditorContextBuilder::new(test_job_scheduler()).build(),
        )
    }

    pub fn new_with_context(
        world: impl Into<AuthoringWorldSeed>,
        viewport_size: UVec2,
        context: Arc<EditorContext>,
    ) -> Self {
        Self::new_with_world(
            EditorAuthoringWorld::loaded(context.gateway(), world)
                .expect("editor context must accept an authoring gateway"),
            viewport_size,
            DEFAULT_PROJECT_PATH.to_string(),
            EditorSessionMode::Welcome,
            WelcomePaneSnapshot::default(),
            false,
            "Ready".to_string(),
            context,
        )
    }

    #[cfg(test)]
    pub fn with_default_selection(
        world: impl Into<AuthoringWorldSeed>,
        viewport_size: UVec2,
    ) -> Self {
        Self::with_default_selection_with_context(
            world,
            viewport_size,
            EditorContextBuilder::new(test_job_scheduler()).build(),
        )
    }

    pub fn with_default_selection_with_context(
        world: impl Into<AuthoringWorldSeed>,
        viewport_size: UVec2,
        context: Arc<EditorContext>,
    ) -> Self {
        let mut state = Self::new_with_context(world, viewport_size, context);
        state.select_default_node();
        state.sync_selection_state();
        state
    }

    #[cfg(test)]
    pub fn project(
        world: impl Into<AuthoringWorldSeed>,
        viewport_size: UVec2,
        project_path: impl Into<String>,
    ) -> Self {
        Self::project_with_context(
            world,
            viewport_size,
            project_path,
            EditorContextBuilder::new(test_job_scheduler()).build(),
        )
    }

    pub fn project_with_context(
        world: impl Into<AuthoringWorldSeed>,
        viewport_size: UVec2,
        project_path: impl Into<String>,
        context: Arc<EditorContext>,
    ) -> Self {
        let mut state = Self::new_with_world(
            EditorAuthoringWorld::loaded(context.gateway(), world)
                .expect("editor context must accept an authoring gateway"),
            viewport_size,
            project_path.into(),
            EditorSessionMode::Project,
            WelcomePaneSnapshot::default(),
            true,
            "Ready".to_string(),
            context,
        );
        state.select_default_node();
        state.sync_selection_state();
        state
    }

    #[cfg(test)]
    pub fn welcome(viewport_size: UVec2, welcome: WelcomePaneSnapshot) -> Self {
        Self::welcome_with_context(
            viewport_size,
            welcome,
            EditorContextBuilder::new(test_job_scheduler()).build(),
        )
    }

    pub fn welcome_with_context(
        viewport_size: UVec2,
        welcome: WelcomePaneSnapshot,
        context: Arc<EditorContext>,
    ) -> Self {
        let status_line = if welcome.status_message.trim().is_empty() {
            "Ready".to_string()
        } else {
            welcome.status_message.clone()
        };
        Self::new_with_world(
            EditorAuthoringWorld::unloaded(context.gateway())
                .expect("editor context must accept a detached authoring gateway"),
            viewport_size,
            String::new(),
            EditorSessionMode::Welcome,
            welcome,
            false,
            status_line,
            context,
        )
    }

    fn new_with_world(
        world: EditorAuthoringWorld,
        viewport_size: UVec2,
        project_path: String,
        session_mode: EditorSessionMode,
        welcome: WelcomePaneSnapshot,
        project_open: bool,
        status_line: String,
        context: Arc<EditorContext>,
    ) -> Self {
        let console_history =
            crate::ui::workbench::state::console_history::EditorConsoleHistory::new(&status_line);
        let viewport_controller = SceneViewportController::with_settings(
            viewport_size,
            Arc::clone(context.settings()),
            context.settings_persistence().clone(),
        );
        let mut state = Self {
            context,
            world,
            viewport_controller,
            name_field: String::new(),
            parent_field: String::new(),
            transform_fields: Default::default(),
            scale_fields: Default::default(),
            inspector_dynamic_fields: Default::default(),
            mesh_import_path: String::new(),
            asset_workspace: AssetWorkspaceState::default(),
            project_path,
            session_mode,
            welcome,
            project_open,
            status_line,
            console_history,
            status_task_progress: None,
            bridge_diagnostics: Default::default(),
            scene_entry_projection_cache: Default::default(),
            gizmo_transaction: None,
            play_session: None,
            #[cfg(test)]
            fail_next_transaction_selection_sync: false,
        };
        if state.project_open {
            state
                .viewport_controller
                .configure_project_settings(Path::new(&state.project_path));
        }
        state
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
        if let Some(Some(selection)) = selection {
            self.viewport_controller
                .selection_mut()
                .select_only_active(selection);
        }
    }
}
