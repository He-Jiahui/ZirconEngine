use std::cmp::Ordering;

use crate::scene::ecs::{SceneSystemDescriptor, SystemStage};
use crate::scene::SceneRuntimeHookRegistration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScheduledSceneStep {
    Native {
        id: String,
        stage: SystemStage,
        order: i32,
    },
    Runtime {
        id: String,
        stage: SystemStage,
        order: i32,
    },
    ApplyDeferred {
        after_system_id: String,
        stage: SystemStage,
        order: i32,
    },
}

impl ScheduledSceneStep {
    pub(crate) fn native(id: impl Into<String>, stage: SystemStage, order: i32) -> Self {
        Self::Native {
            id: id.into(),
            stage,
            order,
        }
    }

    pub(crate) fn runtime(id: impl Into<String>, stage: SystemStage, order: i32) -> Self {
        Self::Runtime {
            id: id.into(),
            stage,
            order,
        }
    }

    pub(crate) fn apply_deferred_after(
        after_system_id: impl Into<String>,
        stage: SystemStage,
        order: i32,
    ) -> Self {
        Self::ApplyDeferred {
            after_system_id: after_system_id.into(),
            stage,
            order,
        }
    }

    pub(crate) fn iter_sorted_for_stage<'a>(
        stage: SystemStage,
        internal_systems: &'a [SceneSystemDescriptor],
        native_steps: &'a [Self],
        hooks: &'a [SceneRuntimeHookRegistration],
    ) -> SortedScheduledSceneSteps<'a> {
        debug_assert!(internal_systems_match_stage(stage, internal_systems));
        debug_assert!(native_steps_match_stage(stage, native_steps));
        debug_assert!(hooks_match_stage(stage, hooks));

        SortedScheduledSceneSteps {
            internal_systems,
            native_steps,
            hooks,
            internal_index: 0,
            native_index: 0,
            hook_index: 0,
        }
    }

    fn stage(&self) -> SystemStage {
        match self {
            Self::Native { stage, .. } => *stage,
            Self::Runtime { stage, .. } => *stage,
            Self::ApplyDeferred { stage, .. } => *stage,
        }
    }
}

fn internal_systems_match_stage(
    stage: SystemStage,
    internal_systems: &[SceneSystemDescriptor],
) -> bool {
    for system in internal_systems {
        if system.stage != stage {
            return false;
        }
    }
    true
}

fn native_steps_match_stage(stage: SystemStage, native_steps: &[ScheduledSceneStep]) -> bool {
    for step in native_steps {
        if step.stage() != stage {
            return false;
        }
    }
    true
}

fn hooks_match_stage(stage: SystemStage, hooks: &[SceneRuntimeHookRegistration]) -> bool {
    for hook in hooks {
        if hook.descriptor().stage != stage {
            return false;
        }
    }
    true
}

pub(crate) struct SortedScheduledSceneSteps<'a> {
    internal_systems: &'a [SceneSystemDescriptor],
    native_steps: &'a [ScheduledSceneStep],
    hooks: &'a [SceneRuntimeHookRegistration],
    internal_index: usize,
    native_index: usize,
    hook_index: usize,
}

impl<'a> Iterator for SortedScheduledSceneSteps<'a> {
    type Item = ScheduledSceneStepRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = match self.internal_systems.get(self.internal_index) {
            Some(system) => Some(ScheduledSceneStepRef::Internal(system)),
            None => None,
        };

        if let Some(native_step) = self.native_steps.get(self.native_index) {
            let native_step = ScheduledSceneStepRef::from_native_step(native_step);
            if should_replace_next_step(&native_step, &next) {
                next = Some(native_step);
            }
        }

        if let Some(hook) = self.hooks.get(self.hook_index) {
            let hook_step = ScheduledSceneStepRef::Hook(hook);
            if should_replace_next_step(&hook_step, &next) {
                next = Some(hook_step);
            }
        }

        match next {
            Some(ScheduledSceneStepRef::Internal(system)) => {
                self.internal_index += 1;
                Some(ScheduledSceneStepRef::Internal(system))
            }
            Some(ScheduledSceneStepRef::Native { id, stage, order }) => {
                self.native_index += 1;
                Some(ScheduledSceneStepRef::Native { id, stage, order })
            }
            Some(ScheduledSceneStepRef::Runtime { id, stage, order }) => {
                self.native_index += 1;
                Some(ScheduledSceneStepRef::Runtime { id, stage, order })
            }
            Some(ScheduledSceneStepRef::ApplyDeferred {
                after_system_id,
                stage,
                order,
            }) => {
                self.native_index += 1;
                Some(ScheduledSceneStepRef::ApplyDeferred {
                    after_system_id,
                    stage,
                    order,
                })
            }
            Some(ScheduledSceneStepRef::Hook(hook)) => {
                self.hook_index += 1;
                Some(ScheduledSceneStepRef::Hook(hook))
            }
            None => None,
        }
    }
}

fn should_replace_next_step(
    candidate: &ScheduledSceneStepRef<'_>,
    current: &Option<ScheduledSceneStepRef<'_>>,
) -> bool {
    match current {
        Some(current) => compare_step_refs(candidate, current).is_lt(),
        None => true,
    }
}

pub(crate) enum ScheduledSceneStepRef<'a> {
    Internal(&'a SceneSystemDescriptor),
    Native {
        id: &'a str,
        stage: SystemStage,
        order: i32,
    },
    Runtime {
        id: &'a str,
        stage: SystemStage,
        order: i32,
    },
    ApplyDeferred {
        after_system_id: &'a str,
        stage: SystemStage,
        order: i32,
    },
    Hook(&'a SceneRuntimeHookRegistration),
}

impl<'a> ScheduledSceneStepRef<'a> {
    fn from_native_step(step: &'a ScheduledSceneStep) -> Self {
        match step {
            ScheduledSceneStep::Native { id, stage, order } => Self::Native {
                id: id.as_str(),
                stage: *stage,
                order: *order,
            },
            ScheduledSceneStep::Runtime { id, stage, order } => Self::Runtime {
                id: id.as_str(),
                stage: *stage,
                order: *order,
            },
            ScheduledSceneStep::ApplyDeferred {
                after_system_id,
                stage,
                order,
            } => Self::ApplyDeferred {
                after_system_id: after_system_id.as_str(),
                stage: *stage,
                order: *order,
            },
        }
    }

    fn order(&self) -> i32 {
        match self {
            Self::Internal(system) => system.order,
            Self::Native { order, .. }
            | Self::Runtime { order, .. }
            | Self::ApplyDeferred { order, .. } => *order,
            Self::Hook(hook) => hook.descriptor().order,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Internal(system) => system.id.as_str(),
            Self::Native { id, .. } | Self::Runtime { id, .. } => id,
            Self::ApplyDeferred {
                after_system_id, ..
            } => after_system_id,
            Self::Hook(hook) => hook.descriptor().id.as_str(),
        }
    }

    fn step_rank(&self) -> u8 {
        match self {
            Self::Internal(_) | Self::Native { .. } | Self::Runtime { .. } => 0,
            Self::ApplyDeferred { .. } => 1,
            Self::Hook(_) => 2,
        }
    }
}

fn compare_step_refs(
    left: &ScheduledSceneStepRef<'_>,
    right: &ScheduledSceneStepRef<'_>,
) -> Ordering {
    left.order()
        .cmp(&right.order())
        .then(left.before_hook_rank().cmp(&right.before_hook_rank()))
        .then(left.id().cmp(right.id()))
        .then(left.step_rank().cmp(&right.step_rank()))
}

impl<'a> ScheduledSceneStepRef<'a> {
    fn before_hook_rank(&self) -> u8 {
        match self {
            Self::Hook(_) => 1,
            _ => 0,
        }
    }
}
