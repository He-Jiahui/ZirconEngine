use crate::core::editor_message::SceneModeId;
use crate::core::extension::ContributionTicket;
use crate::core::tools::{AcquireOutcome, ToolDefinitionId, ToolOwnerGeneration};
use crate::scene::modes::{
    SceneModeActivation, SceneModeCtx, SceneModeRegistration, SceneModeRegistry,
    SceneModeRegistryError,
};

use super::{
    scene_viewport_controller::SceneToolIdentity, SceneViewportController,
    SceneViewportControllerError,
};

pub(crate) struct PreparedSceneModeContributionRetirement {
    ticket: ContributionTicket,
    registry: SceneModeRegistry,
}

impl SceneViewportController {
    fn scene_tool_id(
        &mut self,
    ) -> Result<crate::core::tools::ToolInstanceId, SceneViewportControllerError> {
        match &self.scene_tool_identity {
            SceneToolIdentity::Allocated(tool_id) => return Ok(tool_id.clone()),
            SceneToolIdentity::Failed(error) => return Err(error.clone().into()),
            SceneToolIdentity::Pending => {}
        }
        let definition = ToolDefinitionId::parse("editor.scene.viewport")?;
        match self
            .tool_scheduler
            .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
        {
            Ok(tool_id) => {
                self.scene_tool_identity = SceneToolIdentity::Allocated(tool_id.clone());
                Ok(tool_id)
            }
            Err(error) => {
                self.scene_tool_identity = SceneToolIdentity::Failed(error.clone());
                Err(error.into())
            }
        }
    }

    pub(super) fn ensure_scene_tool_lease(&mut self) -> Result<(), SceneViewportControllerError> {
        if self.scene_tool_lease.is_some() {
            return Ok(());
        }
        let tool = self.scene_tool_id()?;
        let resources = self.scene_tool_resources.clone();
        let report = self.tool_scheduler.acquire(tool, resources)?;
        match report.outcome() {
            AcquireOutcome::Acquired { lease } | AcquireOutcome::AlreadyHeld { lease } => {
                self.scene_tool_lease = Some(lease.clone());
                Ok(())
            }
            AcquireOutcome::Queued { request, position }
            | AcquireOutcome::AlreadyQueued { request, position } => {
                let _ = self.tool_scheduler.withdraw(request.id());
                Err(SceneViewportControllerError::SceneToolQueued {
                    position: *position,
                })
            }
            AcquireOutcome::Denied { reason, .. } => {
                Err(SceneViewportControllerError::SceneToolDenied {
                    reason: reason.clone(),
                })
            }
        }
    }

    pub(super) fn release_scene_tool_lease(&mut self) {
        if let Some(lease) = self.scene_tool_lease.take() {
            let _ = self.tool_scheduler.release(lease.id());
        }
    }

    pub(crate) fn prepare_scene_modes(
        &self,
        registrations: impl IntoIterator<Item = SceneModeRegistration>,
    ) -> Result<SceneModeRegistry, SceneModeRegistryError> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        candidate_registry(&self.state.scene_mode_registry, &registrations)
    }

    pub(crate) fn install_prepared_scene_modes(&mut self, registry: SceneModeRegistry) {
        self.state.scene_mode_registry = registry;
        self.interaction_extract.invalidate();
    }

    pub(crate) fn push_scene_mode_overlay(
        &mut self,
        mode_id: &SceneModeId,
    ) -> Result<(), SceneViewportControllerError> {
        let activation = SceneModeActivation::Custom(mode_id.clone());
        activation.validate()?;
        let acquired_now = self.scene_tool_lease.is_none();
        self.ensure_scene_tool_lease()?;
        let (mode, contribution_ticket) = match self
            .state
            .scene_mode_registry
            .create_with_contribution(mode_id)
        {
            Ok(created) => created,
            Err(error) => {
                if acquired_now {
                    self.release_scene_tool_lease();
                }
                return Err(error.into());
            }
        };
        let result = {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state.scene_modes.push_overlay_with_contribution(
                activation,
                mode,
                contribution_ticket,
                &mut ctx,
            )
        };
        if let Err(error) = result {
            if acquired_now {
                self.release_scene_tool_lease();
            }
            return Err(error.into());
        }
        self.interaction_extract.invalidate();
        Ok(())
    }

    pub(crate) fn pop_scene_mode_overlay(&mut self) -> Option<SceneModeId> {
        let popped = {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state.scene_modes.pop(&mut ctx)
        };
        if popped.is_some() {
            if !self.state.scene_modes.requires_exclusive_tool() {
                self.release_scene_tool_lease();
            }
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
        {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            state.scene_modes.shutdown(&mut ctx);
        }
        self.release_scene_tool_lease();
        self.interaction_extract.invalidate();
    }

    pub(crate) fn prepare_scene_mode_contribution_retirement(
        &self,
        ticket: ContributionTicket,
    ) -> PreparedSceneModeContributionRetirement {
        let (registry_candidate, _) = self.state.scene_mode_registry.without_contribution(ticket);
        PreparedSceneModeContributionRetirement {
            ticket,
            registry: registry_candidate,
        }
    }

    pub(crate) fn install_prepared_scene_mode_contribution_retirement(
        &mut self,
        prepared: PreparedSceneModeContributionRetirement,
    ) -> (
        Vec<Box<dyn crate::scene::modes::EditorSceneMode>>,
        Option<String>,
    ) {
        let retirement = {
            let state = &mut self.state;
            let mut ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
            let retirement = state
                .scene_modes
                .retire_contribution_to_builtin_select(prepared.ticket, &mut ctx);
            state.scene_mode_registry = prepared.registry;
            retirement
        };
        if !self.state.scene_modes.requires_exclusive_tool() {
            self.release_scene_tool_lease();
        }
        self.interaction_extract.invalidate();
        retirement
    }
}

fn candidate_registry(
    current: &SceneModeRegistry,
    registrations: &[SceneModeRegistration],
) -> Result<SceneModeRegistry, SceneModeRegistryError> {
    let mut candidate = current.clone();
    for registration in registrations {
        let mode_id = registration.mode_id().clone();
        candidate.register(registration.clone())?;
        candidate.create(&mode_id)?;
    }
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use zircon_runtime_interface::math::UVec2;

    use crate::core::context::ToolSchedulerService;
    use crate::core::editor_authoring_extension::SceneModeDescriptor;
    use crate::core::editor_event::ViewInstanceId;
    use crate::core::editor_message::SceneModeId;
    use crate::core::editor_message::SharedEditorMessageBus;
    use crate::core::editor_operation::EditorOperationPath;
    use crate::core::extension::{
        ContributionBatch, ContributionSource, ContributionStore, ContributionTicket,
        PluginContributionId,
    };
    use crate::core::tools::{ToolInstanceId, ToolQueueLimits, ToolResourceKey, ToolResourceSet};
    use crate::scene::modes::{
        EditorSceneMode, InputOutcome, SceneModeActivation, SceneModeCtx, SceneModeRegistration,
        SceneModeRegistryError, SceneModeStackError, ViewportOverlayBuilder,
    };
    use crate::scene::viewport::{SceneViewportControllerError, ViewportInput};

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

        assert!(matches!(
            error,
            SceneModeRegistryError::FactoryModeIdMismatch {
                registered_mode_id,
                produced_mode_id,
            } if registered_mode_id == SceneModeId::new("test.scene.invalid")
                && produced_mode_id == SceneModeId::new("test.scene.wrong")
        ));
        assert_eq!(controller.state.scene_mode_registry.len(), initial_len);
    }

    #[test]
    fn pushing_an_unknown_overlay_keeps_the_registry_error_typed() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));

        let error = controller
            .push_scene_mode_overlay(&SceneModeId::new("test.scene.missing"))
            .unwrap_err();

        assert!(matches!(
            error,
            SceneViewportControllerError::SceneModeRegistry(
                SceneModeRegistryError::UnknownMode { mode_id }
            ) if mode_id == SceneModeId::new("test.scene.missing")
        ));
    }

    #[test]
    fn pushing_a_duplicate_overlay_keeps_the_stack_error_typed() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let registry = controller
            .prepare_scene_modes([recording_registration("test.scene.overlay", events)])
            .unwrap();
        controller.install_prepared_scene_modes(registry);
        let mode_id = SceneModeId::new("test.scene.overlay");
        controller.push_scene_mode_overlay(&mode_id).unwrap();

        let error = controller.push_scene_mode_overlay(&mode_id).unwrap_err();

        assert!(matches!(
            error,
            SceneViewportControllerError::SceneModeStack(SceneModeStackError::DuplicateMode {
                mode_id: actual,
            }) if actual == mode_id
        ));
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

    #[test]
    fn ticket_retirement_falls_back_from_owned_base_and_preserves_other_overlay() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut store = ContributionStore::default();
        let (weather_ticket, weather_source) = plugin_owner(&mut store, "weather");
        let (lighting_ticket, lighting_source) = plugin_owner(&mut store, "lighting");
        let weather = recording_registration("plugin.weather.mode", Arc::clone(&events))
            .bind_contribution_owner(weather_ticket, weather_source, "weather")
            .unwrap();
        let lighting = recording_registration("plugin.lighting.mode", Arc::clone(&events))
            .bind_contribution_owner(lighting_ticket, lighting_source, "lighting")
            .unwrap();
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let registry = controller.prepare_scene_modes([weather, lighting]).unwrap();
        controller.install_prepared_scene_modes(registry);
        controller
            .activate_scene_mode(SceneModeActivation::Custom(SceneModeId::new(
                "plugin.weather.mode",
            )))
            .unwrap();
        controller
            .push_scene_mode_overlay(&SceneModeId::new("plugin.lighting.mode"))
            .unwrap();

        let prepared = controller.prepare_scene_mode_contribution_retirement(weather_ticket);
        let (retired_modes, cleanup_error) =
            controller.install_prepared_scene_mode_contribution_retirement(prepared);

        assert_eq!(
            controller.state.scene_modes.base_activation(),
            &SceneModeActivation::Select
        );
        assert_eq!(
            controller.active_scene_mode(),
            SceneModeActivation::Custom(SceneModeId::new("plugin.lighting.mode"))
        );
        assert!(matches!(
            controller
                .state
                .scene_mode_registry
                .create(&SceneModeId::new("plugin.weather.mode")),
            Err(SceneModeRegistryError::UnknownMode { mode_id })
                if mode_id == SceneModeId::new("plugin.weather.mode")
        ));
        assert!(controller
            .state
            .scene_mode_registry
            .create(&SceneModeId::new("plugin.lighting.mode"))
            .is_ok());
        assert!(cleanup_error.is_none());
        assert_eq!(retired_modes.len(), 1);
        assert!(events
            .lock()
            .unwrap()
            .contains(&"exit:plugin.weather.mode".to_string()));
        drop(retired_modes);
    }

    #[test]
    fn prepared_ticket_retirement_removes_owned_modes_activated_before_install() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut store = ContributionStore::default();
        let (weather_ticket, weather_source) = plugin_owner(&mut store, "weather");
        let weather = recording_registration("plugin.weather.mode", Arc::clone(&events))
            .bind_contribution_owner(weather_ticket, weather_source, "weather")
            .unwrap();
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let registry = controller.prepare_scene_modes([weather]).unwrap();
        controller.install_prepared_scene_modes(registry);

        let prepared = controller.prepare_scene_mode_contribution_retirement(weather_ticket);
        controller
            .push_scene_mode_overlay(&SceneModeId::new("plugin.weather.mode"))
            .unwrap();
        let (retired_modes, cleanup_error) =
            controller.install_prepared_scene_mode_contribution_retirement(prepared);

        assert_eq!(controller.active_scene_mode(), SceneModeActivation::Select);
        assert_eq!(retired_modes.len(), 1);
        assert!(cleanup_error.is_none());
        assert_eq!(
            events.lock().unwrap().as_slice(),
            ["enter:plugin.weather.mode", "exit:plugin.weather.mode"]
        );
        drop(retired_modes);
    }

    #[test]
    fn scene_controller_acquires_and_releases_its_atomic_tool_set() {
        let bus = SharedEditorMessageBus::default();
        let scheduler = ToolSchedulerService::with_queue_limits(bus, ToolQueueLimits::new(1, 1));
        let viewport_id = ViewInstanceId::new("test.scene#1");
        let viewport_resource = ToolResourceKey::viewport_input(viewport_id.clone());
        let scene_mode_resource = ToolResourceKey::scene_mode_slot(viewport_id.clone());
        let mut controller = SceneViewportController::with_settings_and_tools(
            UVec2::new(1280, 720),
            std::sync::Arc::new(
                crate::core::settings::SettingsMutationCoordinator::in_memory_with_defaults(),
            ),
            scheduler.clone(),
            viewport_id,
        );
        controller
            .activate_scene_mode(SceneModeActivation::Transform(
                crate::scene::viewport::TransformHandleKind::Move,
            ))
            .unwrap();
        let scene_tool = controller
            .scene_tool_identity
            .allocated()
            .expect("activating a scene mode should allocate the controller tool identity")
            .clone();
        assert_eq!(
            scheduler
                .holder(&viewport_resource)
                .map(|lease| lease.instance().clone()),
            Some(scene_tool.clone())
        );
        controller.shutdown_scene_modes();
        assert_eq!(scheduler.holder(&viewport_resource), None);
        assert_eq!(scheduler.holder(&scene_mode_resource), None);
    }

    #[test]
    fn scene_viewport_instances_cannot_release_each_others_tool_claims() {
        let scheduler = ToolSchedulerService::with_queue_limits(
            SharedEditorMessageBus::default(),
            ToolQueueLimits::new(1, 1),
        );
        let settings = || {
            std::sync::Arc::new(
                crate::core::settings::SettingsMutationCoordinator::in_memory_with_defaults(),
            )
        };
        let viewport_id = ViewInstanceId::new("test.scene#1");
        let viewport_resource = ToolResourceKey::viewport_input(viewport_id.clone());
        let mut first = SceneViewportController::with_settings_and_tools(
            UVec2::new(1280, 720),
            settings(),
            scheduler.clone(),
            viewport_id.clone(),
        );
        let mut second = SceneViewportController::with_settings_and_tools(
            UVec2::new(1280, 720),
            settings(),
            scheduler.clone(),
            viewport_id,
        );
        first
            .activate_scene_mode(SceneModeActivation::Transform(
                crate::scene::viewport::TransformHandleKind::Move,
            ))
            .unwrap();
        let first_id = first
            .scene_tool_identity
            .allocated()
            .expect("the first controller should allocate its tool identity")
            .clone();
        let error = second
            .activate_scene_mode(SceneModeActivation::Transform(
                crate::scene::viewport::TransformHandleKind::Move,
            ))
            .unwrap_err();
        let second_id = second
            .scene_tool_identity
            .allocated()
            .expect("the second controller should allocate its tool identity")
            .clone();
        assert_ne!(first_id, second_id);
        assert!(first_id.as_str().starts_with("editor.scene.viewport."));
        assert!(second_id.as_str().starts_with("editor.scene.viewport."));
        assert!(matches!(
            error,
            SceneViewportControllerError::SceneToolQueued { position: 1 }
        ));
        assert_eq!(
            scheduler
                .holder(&viewport_resource)
                .map(|lease| lease.instance().clone()),
            Some(first_id.clone())
        );

        drop(second);

        assert_eq!(
            scheduler
                .holder(&viewport_resource)
                .map(|lease| lease.instance().clone()),
            Some(first_id)
        );
        first.shutdown_scene_modes();
        assert_eq!(scheduler.holder(&viewport_resource), None);
    }

    #[test]
    fn denied_scene_tool_set_keeps_the_existing_mode_stack_unchanged() {
        let bus = SharedEditorMessageBus::default();
        let scheduler = ToolSchedulerService::with_queue_limits(bus, ToolQueueLimits::new(0, 0));
        let blocker =
            ToolInstanceId::for_test("modal.asset_picker", ToolOwnerGeneration::BUILTIN).unwrap();
        let viewport_id = ViewInstanceId::new("test.scene#1");
        let resources = ToolResourceSet::new([
            ToolResourceKey::viewport_input(viewport_id.clone()),
            ToolResourceKey::scene_mode_slot(viewport_id.clone()),
        ])
        .unwrap();
        assert!(matches!(
            scheduler.acquire(blocker, resources).unwrap().outcome(),
            crate::core::tools::AcquireOutcome::Acquired { .. }
        ));
        let mut controller = SceneViewportController::with_settings_and_tools(
            UVec2::new(1280, 720),
            std::sync::Arc::new(
                crate::core::settings::SettingsMutationCoordinator::in_memory_with_defaults(),
            ),
            scheduler,
            viewport_id,
        );

        let error = controller
            .activate_scene_mode(SceneModeActivation::Transform(
                crate::scene::viewport::TransformHandleKind::Move,
            ))
            .unwrap_err();
        assert!(matches!(
            error,
            SceneViewportControllerError::SceneToolDenied { .. }
        ));
        assert_eq!(controller.active_scene_mode(), SceneModeActivation::Select);
    }

    fn plugin_owner(
        store: &mut ContributionStore,
        plugin_id: &str,
    ) -> (ContributionTicket, ContributionSource) {
        let source = ContributionSource::Plugin(
            PluginContributionId::parse(plugin_id).expect("plugin id should be valid"),
        );
        let ticket = store
            .contribute(source.clone(), ContributionBatch::default())
            .expect("an empty contribution batch should allocate a ticket");
        (ticket, source)
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
