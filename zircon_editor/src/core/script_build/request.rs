use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScriptBuildRequestId(u64);

impl ScriptBuildRequestId {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScriptBuildGeneration(u64);

impl ScriptBuildGeneration {
    pub const fn get(self) -> u64 {
        self.0
    }

    const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptBuildTrigger {
    Watch,
    Command,
    Play,
}

impl ScriptBuildTrigger {
    const fn precedence(self) -> u8 {
        match self {
            Self::Watch => 0,
            Self::Command => 1,
            Self::Play => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptBuildStep {
    CompileModules(Vec<PathBuf>),
    ValidateLedger,
    RefreshBindings,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptBuildRequest {
    id: ScriptBuildRequestId,
    generation: ScriptBuildGeneration,
    trigger: ScriptBuildTrigger,
    steps: Vec<ScriptBuildStep>,
    play_after_build: bool,
}

impl ScriptBuildRequest {
    pub fn id(&self) -> ScriptBuildRequestId {
        self.id
    }

    pub fn trigger(&self) -> ScriptBuildTrigger {
        self.trigger
    }

    pub fn generation(&self) -> ScriptBuildGeneration {
        self.generation
    }

    pub fn changed_paths(&self) -> &[PathBuf] {
        match &self.steps[0] {
            ScriptBuildStep::CompileModules(changed_paths) => changed_paths,
            ScriptBuildStep::ValidateLedger | ScriptBuildStep::RefreshBindings => {
                unreachable!("the first script build step is always CompileModules")
            }
        }
    }

    pub fn play_after_build(&self) -> bool {
        self.play_after_build
    }

    pub(super) fn new(
        id: ScriptBuildRequestId,
        trigger: ScriptBuildTrigger,
        changed_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            id,
            generation: ScriptBuildGeneration::new(id.get()),
            trigger,
            steps: vec![
                ScriptBuildStep::CompileModules(changed_paths),
                ScriptBuildStep::ValidateLedger,
                ScriptBuildStep::RefreshBindings,
            ],
            play_after_build: trigger == ScriptBuildTrigger::Play,
        }
    }

    pub(super) fn step(&self, index: usize) -> Option<&ScriptBuildStep> {
        self.steps.get(index)
    }

    pub(super) fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub(super) fn promote_trigger(&mut self, trigger: ScriptBuildTrigger) {
        if trigger.precedence() > self.trigger.precedence() {
            self.trigger = trigger;
        }
        self.play_after_build |= trigger == ScriptBuildTrigger::Play;
    }

    pub(super) fn replace_changed_paths(&mut self, changed_paths: Vec<PathBuf>) {
        self.steps[0] = ScriptBuildStep::CompileModules(changed_paths);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScriptBuildStepDispatch {
    request_id: ScriptBuildRequestId,
    generation: ScriptBuildGeneration,
    trigger: ScriptBuildTrigger,
    step_index: usize,
    step: ScriptBuildStep,
    play_after_build: bool,
}

impl ScriptBuildStepDispatch {
    pub fn request_id(&self) -> ScriptBuildRequestId {
        self.request_id
    }

    pub fn trigger(&self) -> ScriptBuildTrigger {
        self.trigger
    }

    pub fn generation(&self) -> ScriptBuildGeneration {
        self.generation
    }

    pub fn step_index(&self) -> usize {
        self.step_index
    }

    pub fn step(&self) -> &ScriptBuildStep {
        &self.step
    }

    pub fn play_after_build(&self) -> bool {
        self.play_after_build
    }

    pub(super) fn new(request: &ScriptBuildRequest, step_index: usize) -> Self {
        Self {
            request_id: request.id(),
            generation: request.generation(),
            trigger: request.trigger(),
            step_index,
            step: request
                .step(step_index)
                .expect("active script build step index is in range")
                .clone(),
            play_after_build: request.play_after_build(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptBuildOutcome {
    Succeeded,
    Failed { summary: String },
    Cancelled { reason: String },
}

impl ScriptBuildOutcome {
    pub fn is_succeeded(&self) -> bool {
        matches!(self, Self::Succeeded)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptBuildCompletion {
    request_id: ScriptBuildRequestId,
    generation: ScriptBuildGeneration,
    completed_step_index: usize,
    outcome: ScriptBuildOutcome,
    request_completed: bool,
    dropped_queued_request_count: usize,
    dropped_watch_path_count: usize,
    resume_play: bool,
}

impl ScriptBuildCompletion {
    pub fn request_id(&self) -> ScriptBuildRequestId {
        self.request_id
    }

    pub fn outcome(&self) -> &ScriptBuildOutcome {
        &self.outcome
    }

    pub fn generation(&self) -> ScriptBuildGeneration {
        self.generation
    }

    pub fn completed_step_index(&self) -> usize {
        self.completed_step_index
    }

    pub fn request_completed(&self) -> bool {
        self.request_completed
    }

    pub fn dropped_queued_request_count(&self) -> usize {
        self.dropped_queued_request_count
    }

    pub fn dropped_watch_path_count(&self) -> usize {
        self.dropped_watch_path_count
    }

    pub fn resume_play(&self) -> bool {
        self.resume_play
    }

    pub(super) fn new(
        request_id: ScriptBuildRequestId,
        generation: ScriptBuildGeneration,
        completed_step_index: usize,
        outcome: ScriptBuildOutcome,
        request_completed: bool,
        dropped_queued_request_count: usize,
        dropped_watch_path_count: usize,
        resume_play: bool,
    ) -> Self {
        Self {
            request_id,
            generation,
            completed_step_index,
            outcome,
            request_completed,
            dropped_queued_request_count,
            dropped_watch_path_count,
            resume_play,
        }
    }
}
