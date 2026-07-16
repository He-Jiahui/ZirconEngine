use std::sync::{Arc, Mutex};

use zircon_runtime_interface::math::Vec2;

use crate::core::commands::{CommandEvalCtx, WhenClause};
use crate::core::editor_authoring_extension::ViewportToolModeDescriptor;
use crate::core::editor_message::SceneModeId;
use crate::core::editor_operation::EditorOperationPath;
use crate::scene::selection::{SelectionModel, WorldDomain};
use crate::scene::viewport::{SceneViewportSettings, ViewportInput};

use super::{
    EditorSceneMode, InputOutcome, SceneModeCtx, SceneModeRegistration, SceneModeRegistry,
    SceneModeRegistryError, SceneModeStack, ViewportOverlayBuilder,
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
    let mut stack = SceneModeStack::new(base, &mut ctx);
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
    let mut stack = SceneModeStack::new(base, &mut ctx);
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
    let mut stack = SceneModeStack::new(base, &mut ctx);

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
    let mut stack = SceneModeStack::new(base, &mut ctx);
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
fn mode_registry_creates_the_factory_bound_to_a_tool_descriptor() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let factory_events = events.clone();
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            tool_descriptor("scene.navigation"),
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
            tool_descriptor("scene.navigation"),
            recording_factory("scene.navigation", events.clone()),
        ))
        .unwrap();

    let duplicate = registry
        .register(SceneModeRegistration::new(
            tool_descriptor("scene.navigation"),
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
            tool_descriptor("scene.mismatch"),
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
    let mut stack = SceneModeStack::new(base, &mut ctx);

    let edit_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(WhenClause::SceneModeActive(SceneModeId::new("scene.select")).eval(&edit_projection));
    assert!(WhenClause::SelectionNonEmpty.eval(&edit_projection));

    stack
        .push(
            Box::new(RecordingMode::new(
                "scene.transform",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap();
    ctx.selection_mut().set_active_domain(WorldDomain::Play);
    let empty_play_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(
        WhenClause::SceneModeActive(SceneModeId::new("scene.transform"))
            .eval(&empty_play_projection)
    );
    assert!(!WhenClause::SelectionNonEmpty.eval(&empty_play_projection));

    ctx.selection_mut().select_only(WorldDomain::Play, 99);
    let selected_play_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(WhenClause::SelectionNonEmpty.eval(&selected_play_projection));
}

fn tool_descriptor(id: &str) -> ViewportToolModeDescriptor {
    ViewportToolModeDescriptor::new(
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
