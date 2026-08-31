use std::sync::{Arc, Mutex};

use zircon_runtime_interface::math::Vec2;

use crate::core::commands::{CommandEvalCtx, CommandEvalSnapshotHandle, WhenClause};
use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::play::PlayInstanceId;
use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::{SelectionModel, SelectionMutation, WorldDomain};
use crate::scene::viewport::{SceneViewportSettings, TransformHandleKind, ViewportInput};

use super::{
    builtin_scene_mode_registry, EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeInputEffect,
    SceneModeRegistration, SceneModeRegistry, SceneModeRegistryError, SceneModeStack,
    SceneModeStackError, ViewportOverlayBuilder,
};

mod input_profiling;
mod isolation;
mod lifecycle;

fn play_domain() -> WorldDomain {
    WorldDomain::Play(PlayInstanceId::for_test(1))
}

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

fn custom_activation(mode_id: &str) -> SceneModeActivation {
    SceneModeActivation::Custom(SceneModeId::new(mode_id))
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
        ctx.push_input_effect(SceneModeInputEffect::TransformPrimaryPressed {
            position: Vec2::ZERO,
            selection_mutation: SelectionMutation::Replace,
        });
        InputOutcome::PassThrough
    }

    fn update(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

struct PassThroughSelectionMutationMode {
    id: SceneModeId,
}

impl PassThroughSelectionMutationMode {
    fn new(id: &str) -> Self {
        Self {
            id: SceneModeId::new(id),
        }
    }
}

impl EditorSceneMode for PassThroughSelectionMutationMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(&mut self, _input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        ctx.selection_mut().select_only(WorldDomain::Edit, 42);
        InputOutcome::PassThrough
    }

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
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    stack
        .push_overlay(
            custom_activation("scene.navigation"),
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
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    stack
        .push_overlay(
            custom_activation("scene.navigation"),
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
fn pass_through_mode_selection_mutations_do_not_leak_to_the_next_mode() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::Consumed,
        events,
    ));
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(PassThroughSelectionMutationMode::new("scene.navigation")),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(
        stack.handle_input(&ViewportInput::PointerMoved(Vec2::ZERO), &mut ctx),
        InputOutcome::Consumed
    );
    assert!(ctx.selection().active_items().is_empty());
    assert_eq!(ctx.selection().active_primary(), None);
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
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    stack
        .push_overlay(
            custom_activation("scene.transform-overlay"),
            Box::new(RecordingMode::new(
                "scene.transform-overlay",
                InputOutcome::Consumed,
                events.clone(),
            )),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(stack.active_mode_id().as_str(), "scene.transform-overlay");
    assert_eq!(
        stack.pop(&mut ctx).unwrap().as_str(),
        "scene.transform-overlay"
    );
    assert_eq!(stack.active_mode_id().as_str(), "scene.select");
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "enter:scene.select",
            "enter:scene.transform-overlay",
            "exit:scene.transform-overlay",
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
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::PassThrough,
                Arc::clone(&events),
            )),
            &mut ctx,
        )
        .unwrap();

    let error = stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap_err();

    assert_eq!(error.mode_id().as_str(), "scene.navigation");
}

#[test]
fn stack_rejects_builtin_or_reserved_builtin_overlays() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        SceneModeActivation::Select,
        Box::new(RecordingMode::new(
            "scene.select",
            InputOutcome::PassThrough,
            Arc::clone(&events),
        )),
        &mut ctx,
    )
    .unwrap();

    for (activation, mode_id) in [
        (
            SceneModeActivation::Transform(TransformHandleKind::Move),
            "scene.transform",
        ),
        (
            SceneModeActivation::Custom(SceneModeId::new("scene.select")),
            "scene.select",
        ),
    ] {
        let error = stack
            .push_overlay(
                activation,
                Box::new(RecordingMode::new(
                    mode_id,
                    InputOutcome::PassThrough,
                    Arc::clone(&events),
                )),
                &mut ctx,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            SceneModeStackError::BuiltInOverlay { mode_id: actual } if actual.as_str() == mode_id
        ));
    }
}

#[test]
fn stack_is_the_single_owner_of_transform_activation() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        SceneModeActivation::Transform(TransformHandleKind::Move),
        Box::new(RecordingMode::new(
            "scene.transform",
            InputOutcome::PassThrough,
            Arc::clone(&events),
        )),
        &mut ctx,
    )
    .unwrap();
    events.lock().unwrap().clear();

    assert_eq!(
        stack.active_activation(),
        SceneModeActivation::Transform(TransformHandleKind::Move)
    );
    assert_eq!(
        stack.base_transform_handle(),
        Some(TransformHandleKind::Move)
    );

    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::PassThrough,
                Arc::clone(&events),
            )),
            &mut ctx,
        )
        .unwrap();
    assert_eq!(
        stack.active_activation(),
        custom_activation("scene.navigation")
    );
    assert_eq!(
        stack.base_transform_handle(),
        Some(TransformHandleKind::Move)
    );
    assert_eq!(
        stack.pop(&mut ctx),
        Some(SceneModeId::new("scene.navigation"))
    );
    events.lock().unwrap().clear();

    stack
        .replace_base(
            SceneModeActivation::Transform(TransformHandleKind::Rotate),
            Box::new(RecordingMode::new(
                "scene.transform",
                InputOutcome::PassThrough,
                Arc::clone(&events),
            )),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(
        stack.active_activation(),
        SceneModeActivation::Transform(TransformHandleKind::Rotate)
    );
    assert_eq!(
        stack.base_transform_handle(),
        Some(TransformHandleKind::Rotate)
    );
    assert_eq!(
        events.lock().unwrap().as_slice(),
        ["exit:scene.transform", "enter:scene.transform"]
    );
}

#[test]
fn stack_rejects_an_activation_that_does_not_match_its_mode_instance() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);

    let error = SceneModeStack::new(
        SceneModeActivation::Select,
        Box::new(RecordingMode::new(
            "scene.transform",
            InputOutcome::PassThrough,
            events,
        )),
        &mut ctx,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        SceneModeStackError::ActivationModeIdMismatch {
            activation_mode_id,
            mode_id,
        } if activation_mode_id == SceneModeId::new("scene.select")
            && mode_id == SceneModeId::new("scene.transform")
    ));
}

#[test]
fn stack_rejects_reserved_builtin_custom_activations_for_base_modes() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);

    let new_error = SceneModeStack::new(
        SceneModeActivation::Custom(SceneModeId::new("scene.select")),
        Box::new(RecordingMode::new(
            "scene.select",
            InputOutcome::PassThrough,
            Arc::clone(&events),
        )),
        &mut ctx,
    )
    .unwrap_err();
    assert!(matches!(
        new_error,
        SceneModeStackError::ReservedBuiltinActivation { mode_id }
            if mode_id == SceneModeId::new("scene.select")
    ));

    let mut stack = SceneModeStack::new(
        SceneModeActivation::Select,
        Box::new(RecordingMode::new(
            "scene.select",
            InputOutcome::PassThrough,
            Arc::clone(&events),
        )),
        &mut ctx,
    )
    .unwrap();
    let replace_error = stack
        .replace_base(
            SceneModeActivation::Custom(SceneModeId::new("scene.transform")),
            Box::new(RecordingMode::new(
                "scene.transform",
                InputOutcome::PassThrough,
                events,
            )),
            &mut ctx,
        )
        .unwrap_err();
    assert!(matches!(
        replace_error,
        SceneModeStackError::ReservedBuiltinActivation { mode_id }
            if mode_id == SceneModeId::new("scene.transform")
    ));
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
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    stack
        .push_overlay(
            custom_activation("scene.transform-overlay"),
            Box::new(RecordingMode::new(
                "scene.transform-overlay",
                InputOutcome::PassThrough,
                events.clone(),
            )),
            &mut ctx,
        )
        .unwrap();
    stack
        .push_overlay(
            custom_activation("scene.navigation"),
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
            "input:scene.transform-overlay",
            "input:scene.select",
        ]
    );

    events.lock().unwrap().clear();
    stack.update(&mut ctx);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "update:scene.select",
            "update:scene.transform-overlay",
            "update:scene.navigation",
        ]
    );

    events.lock().unwrap().clear();
    stack.build_overlay(&mut ViewportOverlayBuilder::default());
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "overlay:scene.select",
            "overlay:scene.transform-overlay",
            "overlay:scene.navigation",
        ]
    );

    events.lock().unwrap().clear();
    stack.shutdown(&mut ctx);
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "exit:scene.navigation",
            "exit:scene.transform-overlay",
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
        Some(SceneModeInputEffect::SelectionPrimaryPressed {
            position: primary,
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
        Some(SceneModeInputEffect::TransformPrimaryPressed {
            position: primary,
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
    selection.extend(WorldDomain::Edit, [11, 12]);
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::PassThrough,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();

    let edit_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(WhenClause::SceneModeActive(SceneModeId::new("scene.select")).eval(&edit_projection));
    assert!(WhenClause::SelectionNonEmpty.eval(&edit_projection));

    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap();
    ctx.selection_mut()
        .activate_play_domain(PlayInstanceId::for_test(1));
    ctx.selection_mut().clear_active();
    let empty_play_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(
        WhenClause::SceneModeActive(SceneModeId::new("scene.navigation"))
            .eval(&empty_play_projection)
    );
    assert!(!WhenClause::SelectionNonEmpty.eval(&empty_play_projection));

    ctx.selection_mut().select_only(play_domain(), 99);
    let selected_play_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(WhenClause::SelectionNonEmpty.eval(&selected_play_projection));
}

#[test]
fn command_eval_generation_tracks_selection_identity_and_mode_topology() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    selection.activate_play_domain(PlayInstanceId::for_test(1));
    selection.set_active_domain(WorldDomain::Edit);
    selection.select_only(WorldDomain::Edit, 11);
    selection.select_only(play_domain(), 29);
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::PassThrough,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    let snapshot = CommandEvalSnapshotHandle::default();

    let initial = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(initial));
    assert_eq!(snapshot.generation(), 1);

    // Edit and Play have the same count and per-domain generation, so the domain itself must
    // remain part of the command snapshot identity.
    assert!(ctx.selection_mut().set_active_domain(play_domain()));
    let same_count_play =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(same_count_play));
    assert_eq!(snapshot.generation(), 2);

    assert!(ctx.selection_mut().set_active_domain(WorldDomain::Edit));
    let same_count_edit =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(same_count_edit));
    assert_eq!(snapshot.generation(), 3);

    // A Play mutation must not invalidate the active Edit command snapshot.
    ctx.selection_mut().select_only(play_domain(), 30);
    let inactive_play_mutation =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(!snapshot.replace(inactive_play_mutation));
    assert_eq!(snapshot.generation(), 3);

    assert!(ctx.selection_mut().set_active_domain(play_domain()));
    let changed_play_domain =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(changed_play_domain));
    assert_eq!(snapshot.generation(), 4);

    let equivalent = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(!snapshot.replace(equivalent));
    assert_eq!(snapshot.generation(), 4);

    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap();
    let pushed = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(pushed));
    assert_eq!(snapshot.generation(), 5);

    assert_eq!(
        stack.pop(&mut ctx),
        Some(SceneModeId::new("scene.navigation"))
    );
    let popped = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(popped));
    assert_eq!(snapshot.generation(), 6);
}

fn scene_mode_descriptor(id: &str) -> SceneModeDescriptor {
    SceneModeDescriptor::new(
        id,
        "Navigation",
        "editor.scene",
        EditorOperationPath::parse("scene.mode.activate.navigation").unwrap(),
    )
}

fn recording_factory(
    id: &'static str,
    events: Arc<Mutex<Vec<String>>>,
) -> impl Fn() -> Box<dyn EditorSceneMode> + Send + Sync + 'static {
    move || {
        Box::new(RecordingMode::new(
            id,
            InputOutcome::PassThrough,
            events.clone(),
        ))
    }
}
