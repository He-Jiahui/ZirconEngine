use super::{SceneSystemDescriptor, SceneSystemRegistry, ScheduledSceneStep, SystemStage};

/// One tick's schedule snapshot, grouped by stage to avoid repeated stage scans.
#[derive(Clone, Debug)]
pub(crate) struct SceneScheduleStagePlan {
    stages: Vec<SystemStage>,
    internal_systems_by_stage: [Vec<SceneSystemDescriptor>; SystemStage::COUNT],
    native_steps_by_stage: [Vec<ScheduledSceneStep>; SystemStage::COUNT],
}

impl SceneScheduleStagePlan {
    pub(crate) fn from_registry(stages: &[SystemStage], registry: &SceneSystemRegistry) -> Self {
        let systems = registry.systems();
        let internal_system_counts = internal_system_counts_by_stage(systems);
        let mut internal_systems_by_stage =
            internal_system_groups_with_capacity(&internal_system_counts);
        for system in systems {
            internal_systems_by_stage[system.stage.rank()].push(system.clone());
        }

        Self {
            stages: stages.to_vec(),
            internal_systems_by_stage,
            native_steps_by_stage: registry.native_system_steps_by_stage(),
        }
    }

    pub(crate) fn stages(&self) -> &[SystemStage] {
        &self.stages
    }

    pub(crate) fn internal_systems_for_stage(
        &self,
        stage: SystemStage,
    ) -> &[SceneSystemDescriptor] {
        &self.internal_systems_by_stage[stage.rank()]
    }

    pub(crate) fn native_steps_for_stage(&self, stage: SystemStage) -> &[ScheduledSceneStep] {
        &self.native_steps_by_stage[stage.rank()]
    }
}

fn internal_system_counts_by_stage(
    systems: &[SceneSystemDescriptor],
) -> [usize; SystemStage::COUNT] {
    let mut counts = [0_usize; SystemStage::COUNT];
    for system in systems {
        counts[system.stage.rank()] += 1;
    }
    counts
}

fn internal_system_groups_with_capacity(
    internal_system_counts: &[usize; SystemStage::COUNT],
) -> [Vec<SceneSystemDescriptor>; SystemStage::COUNT] {
    std::array::from_fn(|stage_index| Vec::with_capacity(internal_system_counts[stage_index]))
}
