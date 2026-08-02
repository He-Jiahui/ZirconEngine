use super::*;

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

    fn handle_input(&mut self, _input: &ViewportInput, ctx: &mut SceneModeCtx<'_>) -> InputOutcome {
        self.events
            .lock()
            .unwrap()
            .push(format!("input:{}", self.id.as_str()));
        ctx.selection_mut().select_only(WorldDomain::Edit, 999);
        ctx.push_input_effect(SceneModeInputEffect::PrimaryReleased);
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
fn isolated_scene_mode_enter_panic_rejects_stack_mutation_and_runs_cleanup() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let panic_events = Arc::clone(&events);
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.panicking-enter"),
            move || {
                Box::new(PanickingEnterMode {
                    id: SceneModeId::new("scene.panicking-enter"),
                    events: Arc::clone(&panic_events),
                }) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        Box::new(RecordingMode::new(
            "scene.safe-base",
            InputOutcome::PassThrough,
            Arc::clone(&events),
        )),
        &mut ctx,
    )
    .unwrap();
    events.lock().unwrap().clear();

    for replace_base in [false, true] {
        let mode = registry
            .create(&SceneModeId::new("scene.panicking-enter"))
            .unwrap();
        let error = if replace_base {
            stack.replace_base(mode, &mut ctx)
        } else {
            stack.push(mode, &mut ctx)
        }
        .unwrap_err();

        assert!(matches!(
            error,
            SceneModeStackError::EnterFailure { mode_id, .. }
                if mode_id.as_str() == "scene.panicking-enter"
        ));
        assert_eq!(stack.base_mode_id().as_str(), "scene.safe-base");
        assert_eq!(stack.active_mode_id().as_str(), "scene.safe-base");
    }
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "enter:scene.panicking-enter",
            "exit:scene.panicking-enter",
            "enter:scene.panicking-enter",
            "exit:scene.panicking-enter",
        ]
    );
}

#[test]
fn isolated_scene_mode_input_panic_invalidates_overlay_and_still_exits() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let panic_events = Arc::clone(&events);
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.panicking-input"),
            move || {
                Box::new(PanickingInputMode {
                    id: SceneModeId::new("scene.panicking-input"),
                    events: Arc::clone(&panic_events),
                }) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();
    let mut selection = SelectionModel::default();
    selection.select_only(WorldDomain::Edit, 41);
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        Box::new(RecordingMode::new(
            "scene.safe-base",
            InputOutcome::Consumed,
            Arc::clone(&events),
        )),
        &mut ctx,
    )
    .unwrap();
    stack
        .push(
            registry
                .create(&SceneModeId::new("scene.panicking-input"))
                .unwrap(),
            &mut ctx,
        )
        .unwrap();
    events.lock().unwrap().clear();

    assert_eq!(
        stack.handle_input(&ViewportInput::PointerMoved(Vec2::ZERO), &mut ctx),
        InputOutcome::Consumed
    );
    assert_eq!(ctx.selection().active_primary(), Some(41));
    assert!(ctx.take_input_effect().is_none());
    assert!(ctx.take_overlay_invalidation());
    stack.shutdown(&mut ctx);

    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "input:scene.panicking-input",
            "input:scene.safe-base",
            "exit:scene.panicking-input",
            "exit:scene.safe-base",
        ]
    );
}

#[test]
fn isolated_scene_mode_contains_inner_drop_panics_for_valid_and_mismatched_modes() {
    let mut registry = SceneModeRegistry::default();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.panicking-drop"),
            || {
                Box::new(PanickingDropMode {
                    id: SceneModeId::new("scene.panicking-drop"),
                }) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();
    registry
        .register(SceneModeRegistration::new(
            scene_mode_descriptor("scene.mismatched-panicking-drop"),
            || {
                Box::new(PanickingDropMode {
                    id: SceneModeId::new("scene.other"),
                }) as Box<dyn EditorSceneMode>
            },
        ))
        .unwrap();

    drop(
        registry
            .create(&SceneModeId::new("scene.panicking-drop"))
            .unwrap(),
    );
    let mismatch = registry
        .create(&SceneModeId::new("scene.mismatched-panicking-drop"))
        .err()
        .expect("the mismatched mode must be rejected without unwinding its destructor");
    assert!(matches!(
        mismatch,
        SceneModeRegistryError::FactoryModeIdMismatch { .. }
    ));
}
