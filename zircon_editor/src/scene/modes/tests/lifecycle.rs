use super::*;

struct LifecycleMode {
    id: SceneModeId,
    events: Arc<Mutex<Vec<String>>>,
    enter_count: usize,
    fail_on_enters: Vec<usize>,
    enter_selection: Option<u64>,
    boundary_failure: Option<String>,
}

impl LifecycleMode {
    fn stable(id: &str, events: Arc<Mutex<Vec<String>>>) -> Self {
        Self::new(id, events, Vec::new())
    }

    fn failing_on(id: &str, events: Arc<Mutex<Vec<String>>>, fail_on_enters: Vec<usize>) -> Self {
        Self::new(id, events, fail_on_enters)
    }

    fn new(id: &str, events: Arc<Mutex<Vec<String>>>, fail_on_enters: Vec<usize>) -> Self {
        Self {
            id: SceneModeId::new(id),
            events,
            enter_count: 0,
            fail_on_enters,
            enter_selection: None,
            boundary_failure: None,
        }
    }

    fn with_enter_selection(mut self, entity: u64) -> Self {
        self.enter_selection = Some(entity);
        self
    }

    fn record(&self, event: &str) {
        self.events.lock().unwrap().push(event.to_owned());
    }
}

impl EditorSceneMode for LifecycleMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, ctx: &mut SceneModeCtx<'_>) {
        self.enter_count += 1;
        self.record(&format!("enter:{}", self.id.as_str()));
        if let Some(entity) = self.enter_selection {
            ctx.selection_mut().select_only(WorldDomain::Edit, entity);
        }
        if self.fail_on_enters.contains(&self.enter_count) {
            self.boundary_failure = Some(format!("enter attempt {} failed", self.enter_count));
        }
    }

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {
        self.record(&format!("exit:{}", self.id.as_str()));
    }

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::PassThrough
    }

    fn take_boundary_failure(&mut self) -> Option<String> {
        self.boundary_failure.take()
    }
}

#[test]
fn replacing_base_exits_the_old_mode_before_entering_the_new_mode() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        custom_activation("scene.old"),
        Box::new(LifecycleMode::stable("scene.old", Arc::clone(&events))),
        &mut ctx,
    )
    .unwrap();
    events.lock().unwrap().clear();

    stack
        .replace_base(
            custom_activation("scene.new"),
            Box::new(LifecycleMode::stable("scene.new", Arc::clone(&events))),
            &mut ctx,
        )
        .unwrap();

    assert_eq!(
        events.lock().unwrap().as_slice(),
        ["exit:scene.old", "enter:scene.new"]
    );
    assert_eq!(stack.base_mode_id().as_str(), "scene.new");
    assert_eq!(stack.base_activation(), &custom_activation("scene.new"));
    assert_eq!(stack.revision(), 1);
}

#[test]
fn failed_base_replacement_restores_the_old_mode_and_context() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    selection.select_only(WorldDomain::Edit, 41);
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        custom_activation("scene.old"),
        Box::new(LifecycleMode::stable("scene.old", Arc::clone(&events))),
        &mut ctx,
    )
    .unwrap();
    events.lock().unwrap().clear();
    let original_revision = ctx.selection().revision();

    let error = stack
        .replace_base(
            custom_activation("scene.new"),
            Box::new(
                LifecycleMode::failing_on("scene.new", Arc::clone(&events), vec![1])
                    .with_enter_selection(99),
            ),
            &mut ctx,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        SceneModeStackError::EnterFailure { mode_id, .. }
            if mode_id == SceneModeId::new("scene.new")
    ));
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "exit:scene.old",
            "enter:scene.new",
            "exit:scene.new",
            "enter:scene.old",
        ]
    );
    assert_eq!(ctx.selection().active_primary(), Some(41));
    assert_eq!(ctx.selection().revision(), original_revision);
    assert_eq!(stack.base_mode_id().as_str(), "scene.old");
    assert_eq!(stack.base_activation(), &custom_activation("scene.old"));
    assert_eq!(stack.revision(), 0);
}

#[test]
fn failed_base_replacement_reports_a_failed_rollback_without_committing_topology() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let mut stack = SceneModeStack::new(
        custom_activation("scene.old"),
        Box::new(LifecycleMode::failing_on(
            "scene.old",
            Arc::clone(&events),
            vec![2],
        )),
        &mut ctx,
    )
    .unwrap();
    events.lock().unwrap().clear();

    let error = stack
        .replace_base(
            custom_activation("scene.new"),
            Box::new(LifecycleMode::failing_on(
                "scene.new",
                Arc::clone(&events),
                vec![1],
            )),
            &mut ctx,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        SceneModeStackError::BaseReplacementRollbackFailure {
            replacement_mode_id,
            replacement_message,
            rollback_mode_id,
            rollback_message,
        } if replacement_mode_id == SceneModeId::new("scene.new")
            && replacement_message == "enter attempt 1 failed"
            && rollback_mode_id == SceneModeId::new("scene.old")
            && rollback_message == "enter attempt 2 failed"
    ));
    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "exit:scene.old",
            "enter:scene.new",
            "exit:scene.new",
            "enter:scene.old",
            "exit:scene.old",
        ]
    );
    assert_eq!(stack.base_mode_id().as_str(), "scene.old");
    assert_eq!(stack.base_activation(), &custom_activation("scene.old"));
    assert_eq!(stack.revision(), 0);
}
