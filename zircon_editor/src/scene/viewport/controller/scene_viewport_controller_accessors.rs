use std::path::Path;

use crate::core::commands::CommandEvalCtx;
use crate::core::settings::{
    SettingValue, SettingsKey, SettingsLoad, SettingsPaths, SettingsScope, SettingsStore,
    VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
    settings_registry_with_defaults,
};
use crate::scene::modes::{
    SELECT_SCENE_MODE_ID, SceneModeActivation, SceneModeCtx, TRANSFORM_SCENE_MODE_ID,
};
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::{
    GizmoAxis, SceneViewportChromeSettings, SceneViewportSettings, SceneViewportSnapSteps,
    ViewportState,
};
use zircon_runtime_interface::math::Vec3;

use super::SceneViewportController;

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
        self.reset_settings_registry(Some(project_root));
    }

    pub(crate) fn clear_project_settings(&mut self) {
        self.reset_settings_registry(None);
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
        let key = SettingsKey::parse(key).expect("the built-in viewport snap key is valid");
        let mut registry = self.settings_registry.clone();
        registry
            .set(
                SettingsScope::Project,
                &key,
                SettingValue::Float(f64::from(step)),
            )
            .map_err(|error| error.to_string())?;
        if let Some(store) = self.settings_store.as_ref() {
            store
                .save_from(SettingsScope::Project, &registry)
                .map_err(|error| error.to_string())?;
        }
        self.settings_registry = registry;
        Ok(())
    }

    fn reset_settings_registry(&mut self, project_root: Option<&Path>) {
        let mut registry = settings_registry_with_defaults();
        let Ok(user_root) = SettingsPaths::user_root_from_environment() else {
            tracing::warn!(
                "could not resolve the editor user settings root; using setting defaults"
            );
            self.settings_registry = registry;
            self.settings_store = None;
            return;
        };
        let store = SettingsStore::from_roots(user_root, project_root);
        match store.load_into(SettingsScope::User, &mut registry) {
            Ok(SettingsLoad::Loaded {
                path,
                schema_version,
                changes,
            }) => {
                tracing::info!(
                    source = %path.display(),
                    schema_version,
                    changed_settings = changes.len(),
                    "loaded persisted editor user settings"
                );
            }
            Ok(SettingsLoad::Missing { path }) => {
                tracing::info!(
                    source = %path.display(),
                    "editor user settings are absent; using registered defaults"
                );
            }
            Err(error) => {
                tracing::warn!(error = %error, "failed to load editor user settings; using registered defaults");
            }
        }
        if project_root.is_some() {
            match store.load_into(SettingsScope::Project, &mut registry) {
                Ok(SettingsLoad::Loaded {
                    path,
                    schema_version,
                    changes,
                }) => {
                    tracing::info!(
                        source = %path.display(),
                        schema_version,
                        changed_settings = changes.len(),
                        "loaded persisted project settings"
                    );
                }
                Ok(SettingsLoad::Missing { path }) => {
                    tracing::warn!(
                        source = %path.display(),
                        "project settings are absent; using user fallback or registered defaults"
                    );
                }
                Err(error) => {
                    tracing::warn!(error = %error, "failed to load project settings; using user fallback or registered defaults");
                }
            }
        }
        self.settings_registry = registry;
        self.settings_store = project_root.map(|_| store);
    }

    pub(in crate::scene::viewport::controller) fn snap_steps(&self) -> SceneViewportSnapSteps {
        SceneViewportSnapSteps {
            translate_step: self.resolved_snap_step(VIEWPORT_TRANSLATE_STEP_KEY),
            rotate_step_deg: self.resolved_snap_step(VIEWPORT_ROTATE_STEP_DEGREES_KEY),
            scale_step: self.resolved_snap_step(VIEWPORT_SCALE_STEP_KEY),
        }
    }

    fn resolved_snap_step(&self, key: &str) -> f32 {
        let key = SettingsKey::parse(key).expect("the built-in viewport snap key is valid");
        match self
            .settings_registry
            .resolve(&key)
            .expect("the built-in viewport snap key is registered")
        {
            SettingValue::Float(value) => *value as f32,
            _ => unreachable!("the built-in viewport snap key uses a float schema"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use zircon_runtime_interface::math::UVec2;

    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::scene::modes::{
        EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeRegistration, SceneModeRegistry,
        ViewportOverlayBuilder,
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

        assert!(
            controller
                .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move,))
                .unwrap()
        );
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
