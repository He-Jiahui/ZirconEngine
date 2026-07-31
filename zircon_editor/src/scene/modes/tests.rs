use std::sync::{Arc, Mutex};

use zircon_runtime_interface::math::Vec2;

use crate::core::commands::{CommandEvalCtx, WhenClause};
use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::core::editor_operation::EditorOperationPath;
use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::{SelectionModel, SelectionMutation, WorldDomain};
use crate::scene::viewport::{SceneViewportSettings, TransformHandleKind, ViewportInput};

use super::{
    EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeInputEffect, SceneModeRegistration,
    SceneModeRegistry, SceneModeRegistryError, SceneModeStack, SceneModeStackError,
    ViewportOverlayBuilder, builtin_scene_mode_registry,
};

struct RecordingMode {
    id: SceneModeId,
    input_outcome: InputOutcome,
    events: Arc<Mutex<Vec<String>>>,
}

impl RecordingMode {
    fn new(id: &str, input_outcome: InputOutcome, events: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            id: SceneModeId::new(id),
            input_outcome,
            events,
        }
    }

    fn record(&self, event: &str) {
        self.events.lock().unwrap().push(event.to_owned());
    }
}

impl EditorSceneMode for RecordingMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {
        self.record(&format!("enter:{}", self.id.as_str()));
    }

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {
        self.record(&format!("exit:{}", self.id.as_str()));
    }

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        self.record(&format!("input:{}", self.id.as_str()));
        self.input_outcome
    }

    fn update(&mut self, _ctx: &mut SceneModeCtx<'_>) {
        self.record(&format!("update:{}", self.id.as_str()));
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {
        self.record(&format!("overlay:{}", self.id.as_str()));
    }
}

struct PanickingExitMode {
    id: SceneModeId,
    events: Arc<Mutex<Vec<String>>>,
}

impl EditorSceneMode for PanickingExitMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {
        self.events
            .lock()
            .unwrap()
            .push(format!("exit:{}", self.id.as_str()));
        panic!("scene mode exit panic");
    }

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::PassThrough
    }
}

struct PanickingEnterMode {
    id: SceneModeId,
    events: Arc<Mutex<Vec<String>>>,
}

impl EditorSceneMode for PanickingEnterMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {
        self.events
            .lock()
            .unwrap()
            .push(format!("enter:{}", self.id.as_str()));
        panic!("scene mode enter panic");
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
}

struct PanickingInputMode {
    id: SceneModeId,
    events: Arc<Mutex<Vec<String>>>,
}

impl EditorSceneMode for PanickingInputMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

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
        self.events
            .lock()
            .unwrap()
            .push(format!("input:{}", self.id.as_str()));
        panic!("scene mode input panic");
    }
}

struct PanickingDropMode {
    id: SceneModeId,
}

impl EditorSceneMode for PanickingDropMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::PassThrough
    }
}

impl Drop for PanickingDropMode {
    fn drop(&mut self) {
        panic!("scene mode drop panic");
    }
}

struct PassThroughEffectMode {
    id: SceneModeId,
}

impl PassThroughEffectMode {
    fn new(id: &str) -> Self {
        Self {
            id: SceneModeId::new(id),
        }
    }
}

impl EditorSceneMode for PassThroughEffectMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(&mut self, _input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        ctx.push_input_effect(SceneModeInputEffect::PrimaryPressed {
            position: Vec2::ZERO,
            allow_handle_drag: true,
            selection_mutation: SelectionMutation::Replace,
        });
        InputOutcome::PassThrough
    }

    fn update(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

#[test]
fn mode_stack_routes_input_from_top_until_consumed() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::Consumed,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(base, &mut ctx).unwrap();
    stack
        .push(
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::PassThrough,
                events.clone(),
            )),
            &mut ctx,
        )
        .unwrap();

    let outcome = stack.handle_input(&ViewportInput::PointerMoved(Vec2::ZERO), &mut ctx);

    assert_eq!(outcome, InputOutcome::Consumed);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "enter:scene.select",
            "enter:scene.navigation",
            "input:scene.navigation",
            "input:scene.select",
        ]
    );
}

#[test]
fn pass_through_mode_effects_do_not_leak_to_the_consuming_mode() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::Consumed,
        events,
    ));
    let mut stack = SceneModeStack::new(base, &mut ctx).unwrap();
    stack
        .push(
            Box::new(PassThroughEffectMode::new("scene.navigation")),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(
        stack.handle_input(
            &ViewportInput::LeftPressed {
                position: Vec2::ZERO,
                selection_mutation: SelectionMutation::Replace,
            },
            &mut ctx,
        ),
        InputOutcome::Consumed
    );
    assert!(ctx.take_input_effect().is_none());
}

#[test]
fn pop_exits_overlay_and_restores_base_mode_identity() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::PassThrough,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(base, &mut ctx).unwrap();
    stack
        .push(
            Box::new(RecordingMode::new(
                "scene.transform",
                InputOutcome::Consumed,
                events.clone(),
            )),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(stack.active_mode_id().as_str(), "scene.transform");
    assert_eq!(stack.pop(&mut ctx).unwrap().as_str(), "scene.transform");
    assert_eq!(stack.active_mode_id().as_str(), "scene.select");
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "enter:scene.select",
            "enter:scene.transform",
            "exit:scene.transform",
        ]
    );
}

#[test]
fn stack_rejects_duplicate_active_mode_ids() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::PassThrough,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(base, &mut ctx).unwrap();

    let error = stack
        .push(
            Box::new(RecordingMode::new(
                "scene.select",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap_err();

    assert_eq!(error.mode_id().as_str(), "scene.select");
}

#[test]
fn mode_stack_preserves_full_routing_update_overlay_and_shutdown_order() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::Consumed,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(base, &mut ctx).unwrap();
    stack
        .push(
            Box::new(RecordingMode::new(
                "scene.transform",
                InputOutcome::PassThrough,
                events.clone(),
            )),
            &mut ctx,
        )
        .unwrap();
    stack
        .push(
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::PassThrough,
                events.clone(),
            )),
            &mut ctx,
        )
        .unwrap();
    events.lock().unwrap().clear();

    assert_eq!(
        stack.handle_input(&ViewportInput::PointerMoved(Vec2::ZERO), &mut ctx),
        InputOutcome::Consumed
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "input:scene.navigation",
            "input:scene.transform",
            "input:scene.select",
        ]
    );

    events.lock().unwrap().clear();
    stack.update(&mut ctx);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "update:scene.select",
            "update:scene.transform",
            "update:scene.navigation",
        ]
    );

    events.lock().unwrap().clear();
    stack.build_overlay(&mut ViewportOverlayBuilder::default());
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "overlay:scene.select",
            "overlay:scene.transform",
            "overlay:scene.navigation",
        ]
    );

    events.lock().unwrap().clear();
    stack.shutdown(&mut ctx);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "exit:scene.navigation",
            "exit:scene.transform",
            "exit:scene.select",
        ]
    );
}

#[test]
fn mode_registry_creates_the_factory_bound_to_a_scene_mode_descriptor() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let factory_events = events.clone();
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.navigation"),
            move || {
                Box::new(RecordingMode::new(
                    "scene.navigation",
                    InputOutcome::PassThrough,
                    factory_events.clone(),
                )) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();

    let mode_id = SceneModeId::new("scene.navigation");
    let mode = registry.create(&mode_id).unwrap();

    assert_eq!(mode.id(), &mode_id);
    assert_eq!(
        registry.descriptor(&mode_id).unwrap().display_name(),
        "Navigation"
    );
}

#[test]
fn mode_registry_rejects_duplicate_unknown_and_mismatched_factories() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.navigation"),
            recording_factory("scene.navigation", events.clone()),
        ))
        .unwrap();

    let duplicate = registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.navigation"),
            recording_factory("scene.navigation", events.clone()),
        ))
        .unwrap_err();
    assert!(matches!(
        duplicate,
        SceneModeRegistryError::DuplicateMode { mode_id }
            if mode_id.as_str() == "scene.navigation"
    ));

    let unknown = registry
        .create(&SceneModeId::new("scene.unknown"))
        .err()
        .expect("unknown mode should be rejected");
    assert!(matches!(
        unknown,
        SceneModeRegistryError::UnknownMode { mode_id }
            if mode_id.as_str() == "scene.unknown"
    ));

    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.mismatch"),
            recording_factory("scene.other", events),
        ))
        .unwrap();
    let mismatch = registry
        .create(&SceneModeId::new("scene.mismatch"))
        .err()
        .expect("factory id mismatch should be rejected");
    assert!(matches!(
        mismatch,
        SceneModeRegistryError::FactoryModeIdMismatch {
            registered_mode_id,
            produced_mode_id,
        } if registered_mode_id.as_str() == "scene.mismatch"
            && produced_mode_id.as_str() == "scene.other"
    ));
}

#[test]
fn isolated_scene_mode_exit_panic_does_not_skip_remaining_shutdown() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.safe-base"),
            recording_factory("scene.safe-base", Arc::clone(&events)),
        ))
        .unwrap();
    let panic_events = Arc::clone(&events);
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.panicking-overlay"),
            move || {
                Box::new(PanickingExitMode {
                    id: SceneModeId::new("scene.panicking-overlay"),
                    events: Arc::clone(&panic_events),
                }) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        registry
            .create(&SceneModeId::new("scene.safe-base"))
            .unwrap(),
        &mut ctx,
    )
    .unwrap();
    stack
        .push(
            registry
                .create(&SceneModeId::new("scene.panicking-overlay"))
                .unwrap(),
            &mut ctx,
        )
        .unwrap();
    events.lock().unwrap().clear();

    stack.shutdown(&mut ctx);

    assert_eq!(
        events.lock().unwrap().as_slice(),
        ["exit:scene.panicking-overlay", "exit:scene.safe-base"]
    );
}

#[test]
fn builtin_registry_maps_scene_mode_activations_to_their_registered_modes() {
    let registry = builtin_scene_mode_registry();

    assert_eq!(registry.len(), 2);
    for (activation, expected_mode) in [
        (SceneModeActivation::Select, "scene.select"),
        (
            SceneModeActivation::Transform(TransformHandleKind::Move),
            "scene.transform",
        ),
        (
            SceneModeActivation::Transform(TransformHandleKind::Rotate),
            "scene.transform",
        ),
        (
            SceneModeActivation::Transform(TransformHandleKind::Scale),
            "scene.transform",
        ),
    ] {
        let mode_id = activation.mode_id();
        let mode = registry
            .create(&mode_id)
            .expect("every built-in scene mode must resolve through the registry");

        assert_eq!(mode.id().as_str(), expected_mode);
    }

    assert_eq!(
        registry
            .descriptor(&SceneModeId::new("scene.select"))
            .expect("the select mode must have a descriptor")
            .display_name(),
        "Select"
    );
    assert_eq!(
        registry
            .descriptor(&SceneModeId::new("scene.transform"))
            .expect("the transform mode must have a descriptor")
            .display_name(),
        "Transform"
    );
}

#[test]
fn scene_mode_activation_keeps_handle_kind_inside_transform_mode() {
    let select = SceneModeActivation::Select;
    let rotate = SceneModeActivation::Transform(TransformHandleKind::Rotate);

    assert_eq!(select.mode_id().as_str(), "scene.select");
    assert_eq!(select.transform_handle(), None);
    assert_eq!(rotate.mode_id().as_str(), "scene.transform");
    assert_eq!(rotate.transform_handle(), Some(TransformHandleKind::Rotate));
}

#[test]
fn builtin_modes_consume_primary_pointer_input_through_typed_effects() {
    let registry = builtin_scene_mode_registry();
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let primary = Vec2::new(12.0, 24.0);

    let mut select = registry
        .create(&SceneModeId::new("scene.select"))
        .expect("the select mode must resolve");
    let mut select_ctx = SceneModeCtx::new(&mut selection, &settings);
    assert_eq!(
        select.handle_input(
            &ViewportInput::LeftPressed {
                position: primary,
                selection_mutation: SelectionMutation::Extend,
            },
            &mut select_ctx,
        ),
        InputOutcome::Consumed
    );
    assert_eq!(
        select_ctx.take_input_effect(),
        Some(SceneModeInputEffect::PrimaryPressed {
            position: primary,
            allow_handle_drag: false,
            selection_mutation: SelectionMutation::Extend,
        })
    );

    let mut transform = registry
        .create(&SceneModeId::new("scene.transform"))
        .expect("the transform mode must resolve");
    let mut transform_ctx = SceneModeCtx::new(&mut selection, &settings);
    assert_eq!(
        transform.handle_input(
            &ViewportInput::LeftPressed {
                position: primary,
                selection_mutation: SelectionMutation::Toggle,
            },
            &mut transform_ctx,
        ),
        InputOutcome::Consumed
    );
    assert_eq!(
        transform_ctx.take_input_effect(),
        Some(SceneModeInputEffect::PrimaryPressed {
            position: primary,
            allow_handle_drag: true,
            selection_mutation: SelectionMutation::Toggle,
        })
    );
}

#[test]
fn scene_mode_input_effect_carrier_is_inline() {
    let source = include_str!("scene_mode_ctx.rs");

    assert!(source.contains("input_effect: Option<SceneModeInputEffect>"));
    assert!(!source.contains("Vec<SceneModeInputEffect>"));
}

#[test]
fn command_eval_projection_uses_active_mode_and_world_selection() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    selection.