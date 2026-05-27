use crate::plugin::SceneRuntimeHookRegistration;
use crate::scene::ecs::{SceneSystemDescriptor, SystemStage};

pub(crate) enum ScheduledSceneStep {
    Internal(SceneSystemDescriptor),
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
    Hook(SceneRuntimeHookRegistration),
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

    pub(crate) fn sorted_for_stage(
        stage: SystemStage,
        internal_systems: Vec<SceneSystemDescriptor>,
        native_steps: Vec<Self>,
        hooks: Vec<SceneRuntimeHookRegistration>,
    ) -> Vec<Self> {
        let mut steps = internal_systems
            .into_iter()
            .filter(|system| system.stage == stage)
            .map(Self::Internal)
            .chain(
                native_steps
                    .into_iter()
                    .filter(|step| step.stage() == stage),
            )
            .chain(
                hooks
                    .into_iter()
                    .filter(|hook| hook.descriptor().stage == stage)
                    .map(Self::Hook),
            )
            .collect::<Vec<_>>();
        steps.sort_by(|left, right| {
            left.order()
                .cmp(&right.order())
                .then(left.id().cmp(right.id()))
                .then(left.step_rank().cmp(&right.step_rank()))
        });
        steps
    }

    pub(crate) fn order(&self) -> i32 {
        match self {
            Self::Internal(system) => system.order,
            Self::Native { order, .. } => *order,
            Self::ApplyDeferred { order, .. } => *order,
            Self::Hook(hook) => hook.descriptor().order,
        }
    }

    pub(crate) fn id(&self) -> &str {
        match self {
            Self::Internal(system) => system.id.as_str(),
            Self::Native { id, .. } => id.as_str(),
            Self::ApplyDeferred {
                after_system_id, ..
            } => after_system_id.as_str(),
            Self::Hook(hook) => hook.descriptor().id.as_str(),
        }
    }

    fn stage(&self) -> SystemStage {
        match self {
            Self::Internal(system) => system.stage,
            Self::Native { stage, .. } => *stage,
            Self::ApplyDeferred { stage, .. } => *stage,
            Self::Hook(hook) => hook.descriptor().stage,
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
