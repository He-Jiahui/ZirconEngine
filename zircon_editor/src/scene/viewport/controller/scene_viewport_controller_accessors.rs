use std::collections::VecDeque;
use std::path::Path;

use zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal;

use crate::core::commands::CommandEvalCtx;
use crate::core::settings::{
    SettingValue, SettingsKey, SettingsPaths, SettingsPersistenceTicket, SettingsProjectLayerLoad,
    SettingsScope, SettingsStore,
};
use crate::scene::modes::{
    SceneModeActivation, SceneModeCtx, SELECT_SCENE_MODE_ID, TRANSFORM_SCENE_MODE_ID,
};
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::{
    GizmoAxis, SceneViewportChromeSettings, SceneViewportSettings, SceneViewportSnapSteps,
    ViewportState,
};
use zircon_runtime_interface::math::Vec3;

use super::SceneViewportController;

const MAX_TRACKED_SETTINGS_PERSISTENCE_TICKETS: usize = 16;

impl SceneViewportController {
    pub(crate) fn viewport(&self) -> &ViewportState {
        &self.state.viewport
    }

    pub(crate) fn hovered_axis(&self) -> Option<GizmoAxis> {
        self.state.hover.hovered_axis
    }

    pub(crate) fn settings(&self) -> &SceneViewportSettings {
        &self.state.settings
    }

    pub(crate) fn settings_mut(&mut self) -> &mut SceneViewportSettings {
        &mut self.state.settings
    }

    pub(crate) fn chrome_settings(&self) -> SceneViewportChromeSettings {
        SceneViewportChromeSettings::new(
            &self.state.settings,
            self.snap_steps(),
            self.active_scene_mode(),
        )
    }

    pub(crate) fn configure_project_settings(&mut self, project_root: &Path) {
        self.reset_project_settings(Some(project_root));
    }

    pub(crate) fn clear_project_settings(&mut self) {
        self.reset_project_settings(None);
    }

    pub(crate) fn active_scene_mode(&self) -> SceneModeActivation {
        self.scene_mode_activation(self.state.scene_modes.active_mode_id())
    }

    fn base_scene_mode(&self) -> SceneModeActivation {
        self.scene_mode_activation(self.state.scene_modes.base_mode_id())
    }

    fn scene_mode_activation(
        &self,
        mode_id: &crate::core::editor_message::SceneModeId,
    ) -> SceneModeActivation {
        match mode_id.as_str() {
            SELECT_SCENE_MODE_ID => SceneModeActivation::Select,
            TRANSFORM_SCENE_MODE_ID => {
                SceneModeActivation::Transform(self.state.settings.transform_handle)
            }
            _ => SceneModeActivation::Custom(mode_id.clone()),
        }
    }

    pub(in crate::scene::viewport::controller) fn active_transform_handle(
        &self,
    ) -> Option<crate::scene::viewport::TransformHandleKind> {
        (self.state.scene_modes.active_mode_id().as_str() == TRANSFORM_SCENE_MODE_ID)
            .then_some(self.state.settings.transform_handle)
    }

    pub(crate) fn project_command_eval_ctx(&self, context: CommandEvalCtx) -> CommandEvalCtx {
        self.state
            .scene_modes
            .project_command_eval_ctx(context, &self.state.selection)
    }

    pub(crate) fn activate_scene_mode(
        &mut self,
        activation: SceneModeActivation,
    ) -> Result<bool, String> {
        activation.validate()?;
        if self.base_scene_mode() == activation {
            return Ok(false);
        }

        let mode_id = activation.mode_id();
        let previous_handle = self.state.settings.transform_handle;
        if let Some(handle) = activation.transform_handle() {
            self.state.settings.transform_handle = handle;
        }
        if self.state.scene_modes.base_mode_id() != &mode_id {
            let mode = match self.state.scene_mode_registry.create(&mode_id) {
                Ok(mode) => mode,
                Err(error) => {
                    self.state.settings.transform_handle = previous_handle;
                    return Err(error.to_string());
                }
            };
            let replacement = {
                let state = &mut self.state;
                let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
                state.scene_modes.replace_base(mode, &mut mode_ctx)
            };
            if let Err(error) = replacement {
                self.state.settings.transform_handle = previous_handle;
                return Err(error.to_string());
            }
        }
        self.interaction_extract.invalidate();
        Ok(true)
    }

    pub(crate) fn selection(&self) -> &SelectionModel {
        &self.state.selection
    }

    pub(crate) fn selection_mut(&mut self) -> &mut SelectionModel {
        &mut self.state.selection
    }

    pub(crate) fn set_orbit_target(&mut self, target: Vec3) {
        self.state.orbit_target = target;
        self.state.orbit_controller.set_target(target);
    }

    pub(crate) fn orbit_target(&self) -> Vec3 {
        self.state.orbit_target
    }

    pub(crate) fn is_handle_drag_active(&self) -> bool {
        matches!(
            self.state.drag,
            Some(super::viewport_drag_session::ViewportDragSession::Handle { .. })
        )
    }

    pub(in crate::scene::viewport::controller) fn set_project_snap_step(
        &mut self,
        key: &str,
        step: f32,
    ) -> Result<(), String> {
        self.retry_failed_project_settings_persistence()?;
        let key = SettingsKey::parse(key).expect("the built-in viewport snap key is valid");
        let change = self
            .settings_authority
            .set(
                SettingsScope::Project,
                &key,
                SettingValue::Float(f64::from(step)),
            )
            .map_err(|error| error.to_string())?;
        let Some(change) = change else {
            return Ok(());
        };
        if let Some(store) = self.settings_store.as_ref() {
            let ticket = self
                .settings_persistence
                .submit(&change, store.clone())
                .map_err(|error| error.to_string())?;
            self.track_project_settings_persistence_ticket(ticket);
        }
        Ok(())
    }

    fn reset_project_settings(&mut self, project_root: Option<&Path>) {
        self.cancel_project_settings_persistence_tickets();
        self.settings_store = None;
        let Some(project_root) = project_root else {
            self.settings_authority.clear_project_layer();
            return;
        };
        let user_root = match SettingsPaths::user_root_from_environment() {
            Ok(user_root) => user_root,
            Err(error) => {
                let result = self
                    .settings_authority
                    .load_project_layer_from_environment(project_root);
                tracing::warn!(
                    error = %error,
                    project_settings = ?result,
                    "could not resolve the editor user settings root; the authority retained an invalid project layer"
                );
                return;
            }
        };
        let store = SettingsStore::from_roots(user_root, Some(project_root));
        match self
            .settings_authority
            .load_project_layer_from_store(&store)
        {
            SettingsProjectLayerLoad::Persisted {
                path,
                schema_version,
            } => {
                tracing::info!(
                    source = %path.display(),
                    schema_version,
                    "bound persisted project settings from the shared settings authority"
                );
            }
            SettingsProjectLayerLoad::Missing { path } => {
                tracing::info!(
                    source = %path.display(),
                    "bound missing project settings from the shared settings authority"
                );
            }
            SettingsProjectLayerLoad::Invalid { path, message } => {
                tracing::warn!(source = %path.display(), error = %message, "project settings authority retained an invalid source");
            }
        }
        self.settings_store = Some(store);
    }

    pub(in crate::scene::viewport::controller) fn retry_failed_project_settings_persistence(
        &mut self,
    ) -> Result<usize, String> {
        self.reap_project_settings_persistence_tickets();
        let Some(store) = self.settings_store.clone() else {
            return Ok(0);
        };

        let mut retained = VecDeque::new();
        let mut retried = 0;
        let mut retry_error = None;
        while let Some(ticket) = self.settings_persistence_tickets.pop_front() {
            if matches!(ticket.terminal(), Some(BoundedKeyedIoTerminal::Failed(_))) {
                match self.settings_persistence.retry(&ticket, store.clone()) {
                    Ok(retry) => {
                        retained.push_back(retry);
                        retried += 1;
                    }
                    Err(error) => {
                        retry_error.get_or_insert_with(|| error.to_string());
                        retained.push_back(ticket);
                    }
                }
            } else {
                retained.push_back(ticket);
            }
        }
        self.settings_persistence_tickets = retained;
        retry_error.map_or(Ok(retried), Err)
    }

    fn track_project_settings_persistence_ticket(&mut self, ticket: SettingsPersistenceTicket) {
        self.settings_persistence_tickets.retain(|existing| {
            let replaces_existing =
                existing.scope() == ticket.scope() && existing.key() == ticket.key();
            if replaces_existing {
                let _ = existing.cancel_before_start();
            }
            !replaces_existing
        });
        while self.settings_persistence_tickets.len() >= MAX_TRACKED_SETTINGS_PERSISTENCE_TICKETS {
            if let Some(expired) = self.settings_persistence_tickets.pop_front() {
                let _ = expired.cancel_before_start();
            }
        }
        self.settings_persistence_tickets.push_back(ticket);
    }

    fn reap_project_settings_persistence_tickets(&mut self) {
        self.settings_persistence_tickets.retain(|ticket| {
            matches!(
                ticket.terminal(),
                None | Some(BoundedKeyedIoTerminal::Failed(_))
            )
        });
    }

    fn cancel_project_settings_persistence_tickets(&mut self) {
        for ticket in self.settings_persistence_tickets.drain(..) {
            let _ = ticket.cancel_before_start();
        }
    }

    pub(in crate::scene::viewport::controller) fn snap_steps(&self) -> SceneViewportSnapSteps {
        let snap = self.settings_authority.snapshot().viewport_snap();
        SceneViewportSnapSteps {
            translate_step: snap.translate_step() as f32,
            rotate_step_deg: snap.rotate_step_degrees() as f32,
            scale_step: snap.scale_step() as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use zircon_runtime::core::runtime::tasks::{BoundedKeyedIoTerminal, BoundedKeyedIoWaitResult};
    use zircon_runtime_interface::math::UVec2;

    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::settings::{
        SettingValue, SettingsAuthority, SettingsKey, SettingsPersistenceService, SettingsScope,
        SettingsStore, VIEWPORT_TRANSLATE_STEP_KEY,
    };
    use crate::scene::modes::{
        EditorSceneMode, InputOutcome, SceneModeActivation, SceneModeCtx, SceneModeRegistration,
        SceneModeRegistry, ViewportOverlayBuilder,
    };
    use crate::scene::viewport::{TransformHandleKind, ViewportInput};

    use super::SceneViewportController;

    struct SettingsRecordingMode {
        id: SceneModeId,
        entered_handles: Arc<Mutex<Vec<TransformHandleKind>>>,
    }

    impl SettingsRecordingMode {
        fn new(id: &str, entered_handles: Arc<Mutex<Vec<TransformHandleKind>>>) -> Self {
            Self {
                id: SceneModeId::new(id),
                entered_handles,
            }
        }
    }

    impl EditorSceneMode for SettingsRecordingMode {
        fn id(&self) -> &SceneModeId {
            &self.id
        }

        fn enter(&mut self, ctx: &mut SceneModeCtx<'_>) {
            self.entered_handles
                .lock()
                .unwrap()
                .push(ctx.settings().transform_handle);
        }

        fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

        fn handle_input(
            &mut self,
            _input: &ViewportInput,
            _ctx: &mut SceneModeCtx<'_>,
        ) -> InputOutcome {
            InputOutcome::PassThrough
        }

        fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
    }

    #[test]
    fn snap_projection_reads_the_shared_authority_typed_slot() {
        let authority = Arc::new(SettingsAuthority::with_defaults());
        let persistence = SettingsPersistenceService::new(Arc::clone(&authority));
        let controller = SceneViewportController::with_settings(
            UVec2::new(1280, 720),
            authority.clone(),
            persistence,
        );
        let translate_key = SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        authority
            .set(
                SettingsScope::Project,
                &translate_key,
                SettingValue::Float(3.5),
            )
            .unwrap();

        assert_eq!(controller.snap_steps().translate_step, 3.5);
    }

    #[test]
    fn failed_settings_retry_keeps_the_original_ticket_when_the_lane_is_closed() {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-viewport-settings-retry-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos()
        ));
        let project_root = root.join("project");
        fs::create_dir_all(&project_root).unwrap();
        let store = SettingsStore::from_roots(root.join("user"), Some(&project_root));
        let authority = Arc::new(SettingsAuthority::with_defaults());
        assert!(matches!(
            authority.load_project_layer_from_store(&store),
            crate::core::settings::SettingsProjectLayerLoad::Missing { .. }
        ));
        let snap_key = SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        let change = authority
            .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
            .unwrap()
            .expect("a changed project setting must enqueue a save request");
        let persistence = SettingsPersistenceService::new(Arc::clone(&authority));

        fs::remove_dir(&project_root).unwrap();
        fs::write(
            &project_root,
            "a file blocks the project settings directory",
        )
        .unwrap();
        let failed = persistence.submit(&change, store.clone()).unwrap();
        assert!(matches!(
            failed.wait_until(Instant::now() + Duration::from_secs(5)),
            BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(_))
        ));

        let mut controller = SceneViewportController::with_settings(
            UVec2::new(1280, 720),
            authority,
            persistence.clone(),
        );
        controller.settings_store = Some(store);
        controller.settings_persistence_tickets.push_back(failed);
        let shutdown = persistence.shutdown();

        assert!(controller
            .retry_failed_project_settings_persistence()
            .is_err());
        assert_eq!(controller.settings_persistence_tickets.len(), 1);
        assert!(matches!(
            controller
                .settings_persistence_tickets
                .front()
                .and_then(|ticket| ticket.terminal()),
            Some(BoundedKeyedIoTerminal::Failed(_))
        ));

        drop(shutdown);
        fs::remove_file(&project_root).unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn activate_scene_mode_enters_modes_against_target_transform_settings() {
        let entered_handles = Arc::new(Mutex::new(Vec::new()));
        let mut registry = SceneModeRegistry::default();
        for (mode_id, operation) in [
            ("scene.select", "scene.mode.activate.select"),
            ("scene.transform", "scene.mode.activate.transform"),
        ] {
            let mode_entered_handles = entered_handles.clone();
            registry
                .register(SceneModeRegistration::new(
                    SceneModeDescriptor::new(
                        mode_id,
                        mode_id,
                        "editor.scene",
                        EditorOperationPath::parse(operation).unwrap(),
                    ),
                    move || {
                        Box::new(SettingsRecordingMode::new(
                            mode_id,
                            mode_entered_handles.clone(),
                        )) as Box<dyn EditorSceneMode>
                    },
                ))
                .unwrap();
        }

        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        controller.state.scene_mode_registry = registry;

        assert!(controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move,))
            .unwrap());
        assert_eq!(
            entered_handles.lock().unwrap().as_slice(),
            [TransformHandleKind::Move]
        );
    }

    #[test]
    fn failed_scene_mode_activation_rolls_back_transform_handle_configuration() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        controller
            .activate_scene_mode(SceneModeActivation::Select)
            .unwrap();
        controller.state.scene_mode_registry = SceneModeRegistry::default();

        let error = controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Rotate))
            .unwrap_err();

        assert!(error.contains("scene.transform"));
        assert_eq!(controller.active_scene_mode(), SceneModeActivation::Select);
        assert_eq!(
            controller.settings().transform_handle,
            TransformHandleKind::Move
        );
    }

    #[test]
    fn custom_scene_mode_activation_rejects_reserved_builtin_ids() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        let error = controller
            .activate_scene_mode(SceneModeActivation::Custom(SceneModeId::new(
                "scene.select",
            )))
            .unwrap_err();

        assert!(error.contains("reserved built-in id"));
        assert_eq!(controller.active_scene_mode(), SceneModeActivation::Select);
    }

    #[test]
    fn active_scene_mode_tracks_the_overlay_stack_top() {
        let entered_handles = Arc::new(Mutex::new(Vec::new()));
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        {
            let state = &mut controller.state;
            let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state
                .scene_modes
                .push(
                    Box::new(SettingsRecordingMode::new("test.overlay", entered_handles)),
                    &mut mode_ctx,
                )
                .unwrap();
        }

        assert_eq!(
            controller.active_scene_mode(),
            SceneModeActivation::Custom(SceneModeId::new("test.overlay"))
        );
        assert_eq!(controller.active_transform_handle(), None);
    }
}
