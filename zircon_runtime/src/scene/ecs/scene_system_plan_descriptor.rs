use super::scene_system_registry::{
    validate_native_system_execution_policy, validate_system_tick_policy,
};
use super::{
    RuntimeSceneSystem, SceneSystem, SceneSystemMetadata, SceneSystemTickPolicy,
    ScheduleConflictNode, ScheduleError, ScheduledSceneStep, SystemOrderingConstraint,
    SystemParamAccess, SystemSetId, SystemStage,
};

/// The graph contract for one registered external system. Executable boxes may
/// leave the registry during a callback; this descriptor stays until retirement.
#[derive(Clone, Debug)]
pub(super) struct SceneSystemPlanDescriptor {
    metadata: SceneSystemMetadata,
    access: SystemParamAccess,
    execution: SceneSystemExecution,
}

#[derive(Clone, Copy, Debug)]
enum SceneSystemExecution {
    Native {
        worker_safe: bool,
        has_deferred_commands: bool,
    },
    Runtime,
}

impl SceneSystemPlanDescriptor {
    pub(super) fn native(system: &dyn SceneSystem) -> Self {
        // Capture trait-visible fields, including overrides of metadata accessors.
        let mut metadata = SceneSystemMetadata::new(system.id(), system.stage(), system.order())
            .with_constraints(system.constraints().iter().cloned())
            .with_thread_affinity(system.thread_affinity())
            .with_tick_policy(system.tick_policy());
        for set in system.sets() {
            metadata = metadata.with_set(*set);
        }
        Self {
            metadata,
            access: system.access().clone(),
            execution: SceneSystemExecution::Native {
                worker_safe: system.supports_worker_dispatch(),
                has_deferred_commands: system.has_deferred_commands(),
            },
        }
    }

    pub(super) fn runtime(system: &dyn RuntimeSceneSystem) -> Self {
        let mut metadata = SceneSystemMetadata::new(system.id(), system.stage(), system.order())
            .with_constraints(system.constraints().iter().cloned())
            .with_tick_policy(system.tick_policy());
        for set in system.sets() {
            metadata = metadata.with_set(*set);
        }
        Self {
            metadata,
            access: system.access().clone(),
            execution: SceneSystemExecution::Runtime,
        }
    }

    pub(super) fn id(&self) -> &str {
        self.metadata.id()
    }

    pub(super) fn stage(&self) -> SystemStage {
        self.metadata.stage()
    }

    pub(super) fn order(&self) -> i32 {
        self.metadata.order()
    }

    pub(super) fn sets(&self) -> &[SystemSetId] {
        self.metadata.sets()
    }

    pub(super) fn constraints(&self) -> &[SystemOrderingConstraint] {
        self.metadata.constraints()
    }

    fn tick_policy(&self) -> SceneSystemTickPolicy {
        self.metadata.tick_policy()
    }

    pub(super) fn is_native(&self) -> bool {
        matches!(self.execution, SceneSystemExecution::Native { .. })
    }

    pub(super) fn validate_execution_policy(&self) -> Result<(), ScheduleError> {
        match self.execution {
            SceneSystemExecution::Native {
                has_deferred_commands,
                ..
            } => validate_native_system_execution_policy(
                self.id(),
                self.stage(),
                self.tick_policy(),
                has_deferred_commands,
            ),
            SceneSystemExecution::Runtime => {
                validate_system_tick_policy(self.id(), self.stage(), self.tick_policy())
            }
        }
    }

    fn needs_deferred_barrier(&self) -> bool {
        matches!(
            self.execution,
            SceneSystemExecution::Native {
                worker_safe: false,
                has_deferred_commands: true,
            }
        )
    }

    pub(super) fn step_count(&self) -> usize {
        if self.needs_deferred_barrier() {
            2
        } else {
            1
        }
    }

    pub(super) fn append_steps(&self, order: i32, steps: &mut Vec<ScheduledSceneStep>) {
        match self.execution {
            SceneSystemExecution::Native { worker_safe, .. } => {
                steps.push(ScheduledSceneStep::native(
                    self.id(),
                    self.stage(),
                    order,
                    self.tick_policy(),
                    worker_safe,
                    self.access.has_conservative_world_access(),
                ));
            }
            SceneSystemExecution::Runtime => {
                steps.push(ScheduledSceneStep::runtime(
                    self.id(),
                    self.stage(),
                    order,
                    self.tick_policy(),
                ));
            }
        }
        if self.needs_deferred_barrier() {
            steps.push(ScheduledSceneStep::apply_deferred_after(
                self.id(),
                self.stage(),
                order,
                self.tick_policy(),
            ));
        }
    }

    pub(super) fn append_conflict_nodes(&self, nodes: &mut Vec<ScheduleConflictNode>) {
        nodes.push(ScheduleConflictNode::new(
            self.id(),
            self.stage(),
            self.access.clone(),
        ));
        if self.needs_deferred_barrier() {
            nodes.push(ScheduleConflictNode::barrier(
                format!("apply_deferred:{}", self.id()),
                self.stage(),
            ));
        }
    }
}
