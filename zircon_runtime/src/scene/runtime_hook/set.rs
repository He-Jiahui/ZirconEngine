use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::scene::SystemStage;

use super::SceneRuntimeHookRegistration;

/// Owned, stage-indexed hook snapshot used by one scene tick.
#[derive(Debug)]
pub(crate) struct SceneRuntimeHookStagePlan {
    by_stage: [Vec<SceneRuntimeHookRegistration>; SystemStage::COUNT],
}

impl SceneRuntimeHookStagePlan {
    fn from_ordered(ordered: &[SceneRuntimeHookRegistration]) -> Self {
        let stage_hook_counts = hook_counts_by_stage(ordered);
        let mut by_stage = hook_groups_with_capacity(&stage_hook_counts);
        for hook in ordered {
            by_stage[hook.descriptor().stage.rank()].push(hook.clone());
        }
        Self { by_stage }
    }

    pub(crate) fn hooks_for_stage(&self, stage: SystemStage) -> &[SceneRuntimeHookRegistration] {
        &self.by_stage[stage.rank()]
    }
}

impl Default for SceneRuntimeHookStagePlan {
    fn default() -> Self {
        Self {
            by_stage: empty_hook_groups(),
        }
    }
}

/// Installed scene hooks in canonical order plus per-stage dispatch cache.
#[derive(Clone, Debug)]
pub(crate) struct SceneRuntimeHookSet {
    ordered: Vec<SceneRuntimeHookRegistration>,
    stage_plan: Arc<SceneRuntimeHookStagePlan>,
}

impl SceneRuntimeHookSet {
    pub(crate) fn from_ordered(ordered: Vec<SceneRuntimeHookRegistration>) -> Self {
        let stage_plan = Arc::new(SceneRuntimeHookStagePlan::from_ordered(&ordered));
        Self {
            ordered,
            stage_plan,
        }
    }

    pub(crate) fn ordered(&self) -> &[SceneRuntimeHookRegistration] {
        &self.ordered
    }

    pub(crate) fn hooks_for_stage(&self, stage: SystemStage) -> &[SceneRuntimeHookRegistration] {
        self.stage_plan.hooks_for_stage(stage)
    }

    pub(crate) fn stage_plan(&self) -> Arc<SceneRuntimeHookStagePlan> {
        Arc::clone(&self.stage_plan)
    }

    pub(crate) fn try_merge(
        &self,
        registrations: impl IntoIterator<Item = SceneRuntimeHookRegistration>,
    ) -> Result<Self, String> {
        let registrations = registrations.into_iter().collect::<Vec<_>>();
        let mut hook_ids = HashSet::with_capacity(self.ordered.len() + registrations.len());
        for current in &self.ordered {
            hook_ids.insert(current.descriptor().id.as_str());
        }
        for registration in &registrations {
            let id = registration.descriptor().id.as_str();
            if !hook_ids.insert(id) {
                return Err(id.to_string());
            }
        }
        drop(hook_ids);

        let mut ordered = Vec::with_capacity(self.ordered.len() + registrations.len());
        ordered.extend(self.ordered.iter().cloned());
        ordered.extend(registrations);
        ordered.sort_by(|left, right| {
            left.descriptor()
                .stage
                .rank()
                .cmp(&right.descriptor().stage.rank())
                .then(left.descriptor().order.cmp(&right.descriptor().order))
                .then(left.descriptor().id.cmp(&right.descriptor().id))
        });
        Ok(Self::from_ordered(ordered))
    }
}

impl Default for SceneRuntimeHookSet {
    fn default() -> Self {
        Self {
            ordered: Vec::new(),
            stage_plan: Arc::new(SceneRuntimeHookStagePlan::default()),
        }
    }
}

fn empty_hook_groups() -> [Vec<SceneRuntimeHookRegistration>; SystemStage::COUNT] {
    std::array::from_fn(|_| Vec::new())
}

fn hook_counts_by_stage(ordered: &[SceneRuntimeHookRegistration]) -> [usize; SystemStage::COUNT] {
    let mut counts = [0; SystemStage::COUNT];
    for hook in ordered {
        counts[hook.descriptor().stage.rank()] += 1;
    }
    counts
}

fn hook_groups_with_capacity(
    stage_hook_counts: &[usize; SystemStage::COUNT],
) -> [Vec<SceneRuntimeHookRegistration>; SystemStage::COUNT] {
    std::array::from_fn(|stage_index| Vec::with_capacity(stage_hook_counts[stage_index]))
}
