use std::cmp::Ordering;

use crate::plugin::SceneRuntimeHookRegistration;
use crate::scene::ecs::{SceneSystemDescriptor, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScheduledSceneStep {
    Native {
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
        debug_assert!(internal_systems.iter().all(|system| system.stage == stage));
        debug_assert!(native_steps.iter().all(|step| step.stage() == stage));
        debug_assert!(hooks.iter().all(|hook| hook.descriptor().stage == stage));

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
            Self::ApplyDeferred { stage, .. } => *stage,
        }
    }
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
        let mut next = self
            .internal_systems
            .get(self.internal_index)
            .map(ScheduledSceneStepRef::Internal);

        if let Some(native_step) = self.native_steps.get(self.native_index) {
            let native_step = ScheduledSceneStepRef::from_native_step(native_step);
            if next
                .as_ref()
                .is_none_or(|current| compare_step_refs(&native_step, current).is_lt())
            {
                next = Some(native_step);
            }
        }

        if let Some(hook) = self.hooks.get(self.hook_index) {
            let hook_step = ScheduledSceneStepRef::Hook(hook);
            if next
                .as_ref()
                .is_none_or(|current| compare_step_refs(&hook_step, current).is_lt())
            {
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

pub(crate) enum ScheduledSceneStepRef<'a> {
    Internal(&'a SceneSystemDescriptor),
    Native {
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
            Self::Native { order, .. } | Self::ApplyDeferred { order, .. } => *order,
            Self::Hook(hook) => hook.descriptor().order,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Internal(system) => system.id.as_str(),
            Self::Native { id, .. } => id,
            Self::ApplyDeferred {
                after_system_id, ..
            } => after_system_id,
            Self::Hook(hook) => hook.descriptor().id.as_str(),
        }
    }

    fn step_rank(&self) -> u8 {
        match self {
            Self::Internal(_) | Self::Native { .. } => 0,
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
        .then(left.id().cmp(right.id()))
        .then(left.step_rank().cmp(&right.step_rank()))
}
