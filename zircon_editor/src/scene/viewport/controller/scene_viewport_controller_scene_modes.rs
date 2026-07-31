use crate::core::editor_message::SceneModeId;
use crate::scene::modes::{SceneModeCtx, SceneModeRegistration, SceneModeRegistry};

use super::SceneViewportController;

impl SceneViewportController {
    pub(crate) fn prepare_scene_modes(
        &self,
        registrations: impl IntoIterator<Item = SceneModeRegistration>,
    ) -> Result<SceneModeRegistry, String> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        candidate_registry(&self.state.scene_mode_registry, &registrations)
    }

    pub(crate) fn install_prepared_scene_modes(&mut self, registry: SceneModeRegistry) {
        self.state.scene_mode_registry = registry;
        self.interaction_extract.invalidate();
    }

    pub(crate) fn push_scene_mode_overlay(&mut self, mode_id: &SceneModeId) -> Result<(), String> {
        let mode = self
            .state
            .scene_mode_registry
            .create(mode_id)
            .map_err(|error| error.to_string())?;
        let state = &mut self.state;
        let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state
            .scene_modes
            .push(mode, &mut ctx)
            .map_err(|error| error.to_string())?;
        self.interaction_extract.invalidate();
        Ok(())
    }

    pub(crate) fn pop_scene_mode_overlay(&mut self) -> Option<SceneModeId> {
        let state = &mut self.state;
        let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        let popped = state.scene_modes.pop(&mut ctx);
        if popped.is_some() {
            self.interaction_extract.invalidate();
        }
        popped
    }

    pub(crate) fn update_scene_modes(&mut self) {
        let state = &mut self.state;
        let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state.scene_modes.update(&mut ctx);
        if ctx.take_overlay_invalidation() {
            self.interaction_extract.invalidate();
        }
    }

    pub(crate) fn shutdown_scene_modes(&mut self) {
        let state = &mut self.state;
        let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state.scene_modes.shutdown(&mut ctx);
        self.interaction_extract.invalidate();
    }
}

fn candidate_registry(
    current: &SceneModeRegistry,
    registrations: &[SceneModeRegistration],
) -> Result<SceneModeRegistry, String> {
    let mut candidate = current.clone();
    for registration in registrations {
        let mode_id = registration.mode_id().clone();
        candidate
            .register(registration.clone())
            .map_err(|error| error.to_string())?;
        candidate
            .create(&mode_id)
            .map_err(|error| error.to_string())?;
    }
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use zircon_runtime_interface::math::UVec2;

    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::scene::modes::{
        EditorSceneMode, InputOutcome, SceneModeActivation, SceneModeCtx, SceneModeRegistration,
        ViewportOverlayBuilder,
    };
    use crate::scene::viewport::ViewportInput;

    use super::SceneViewportController;

    struct RecordingMode {
        id: SceneModeId,
        events: Arc<Mutex<Vec<String>>>,
    }

    impl EditorSceneMode for RecordingMode {
        fn id(&self) -> &SceneModeId {
            &self.id
        }

        fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {
            self.events
                .lock()
                .unwrap()
                .push(format!("enter:{}", self.id.as_str()));
        }

        fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {
            self.events
                .lock()
                .unwrap()
                .push(format!("exit:{}", self.id.as_str()));
        }

        fn handle_input(
            &mut self,
            _input: &ViewportInput,
            _ctx: &mut SceneModeCtx<'_>,
        ) -> InputOutcome {
            InputOutcome::PassThrough
        }

        fn update(&mut self, ctx: &mut SceneModeCtx<'_>) {
            self.events
                .lock()
                .unwrap()
                .push(format!("update:{}", self.id.as_str()));
            ctx.invalidate_overlay();
        }

        fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
    }

    #[test]
    fn installed_scene_modes_run_base_overlay_update_pop_and_shutdown_lifecycle() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let registrations = ["test.scene.base", "test.scene.overlay"]
            .into_iter()
            .map(|mode_id| recording_registration(mode_id, events.clone()))
            .collect::<Vec<_>>();
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        let registry = controller.prepare_scene_modes(registrations).unwrap();
        controller.install_prepared_scene_modes(registry);
        controller
            .activate_scene_mode(SceneModeActivation::Custom(SceneModeId::new(
                "test.scene.base",
            )))
            .unwrap();
        controller
            .push_scene_mode_overlay(&SceneModeId::new("test.scene.overlay"))
            .unwrap();
        controller.update_scene_modes();
        assert_eq!(
            controller.pop_scene_mode_overlay(),
            Some(SceneModeId::new("test.scene.overlay"))
        );
        controller.shutdown_scene_modes();

        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                "enter:test.scene.base",
                "enter:test.scene.overlay",
                "update:test.scene.base",
                "update:test.scene.overlay",
                "exit:test.scene.overlay",
                "exit:test.scene.base",
            ]
        );
    }

    #[test]
    fn scene_mode_install_is_atomic_when_a_factory_returns_the_wrong_id() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let initial_len = controller.state.scene_mode_registry.len();
        let descriptor = mode_descriptor("test.scene.invalid");
        let registration = SceneModeRegistration::new(descriptor, || {
            Box::new(RecordingMode {
                id: SceneModeId::new("test.scene.wrong"),
                events: Arc::new(Mutex::new(Vec::new())),
            }) as Box<dyn EditorSceneMode>
        });

        let error = controller.prepare_scene_modes([registration]).unwrap_err();

        assert!(error.contains("test.scene.invalid"));
        assert!(error.contains("test.scene.wrong"));
        assert_eq!(controller.state.scene_mode_registry.len(), initial_len);
    }

    #[test]
    fn prepared_scene_mode_install_invokes_each_factory_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let factory_calls = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&factory_calls);
        let registration = SceneModeRegistration::new(
            mode_descriptor("test.scene.single-factory-call"),
            move || {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                Box::new(RecordingMode {
                    id: SceneModeId::new(if call == 0 {
                        "test.scene.single-factory-call"
                    } else {
                        "test.scene.unstable-factory"
                    }),
                    events: Arc::new(Mutex::new(Vec::new())),
                }) as Box<dyn EditorSceneMode>
            },
        );
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        let registry = controller.prepare_scene_modes([registration]).unwrap();
        controller.install_prepared_scene_modes(registry);

        assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
    }

    fn recording_registration(
        mode_id: &'static str,
        events: Arc<Mutex<Vec<String>>>,
    ) -> SceneModeRegistration {
        SceneModeRegistration::new(mode_descriptor(mode_id), move || {
            Box::new(RecordingMode {
                id: SceneModeId::new(mode_id),
                events: events.clone(),
            }) as Box<dyn EditorSceneMode>
        })
    }

    fn mode_descriptor(mode_id: &str) -> SceneModeDescriptor {
        SceneModeDescriptor::new(
            mode_id,
            mode_id,
            "editor.scene",
            EditorOperationPath::parse(&format!("{mode_id}.activate")).unwrap(),
        )
    }
}
