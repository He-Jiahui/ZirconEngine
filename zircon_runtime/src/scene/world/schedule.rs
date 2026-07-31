use super::World;
#[cfg(test)]
use crate::scene::ecs::ScheduledSceneStep;
use crate::scene::ecs::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, IntoSceneSystem, Schedule, ScheduleError,
    SystemParam, SystemStage,
};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

impl World {
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    pub(crate) fn register_boxed_runtime_scene_system(
        &mut self,
        system: BoxedRuntimeSceneSystem,
    ) -> Result<(), ScheduleError> {
        let mut schedule = std::mem::take(&mut self.schedule);
        let result = schedule.register_boxed_runtime_system(system);
        self.schedule = schedule;
        result
    }

    pub fn schedule_mut(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    pub fn register_native_system<P, S>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        order: i32,
        system: S,
    ) -> Result<(), ScheduleError>
    where
        P: SystemParam + 'static,
        P::State: Send,
        S: IntoSceneSystem<P>,
    {
        let mut schedule = std::mem::take(&mut self.schedule);
        let result = schedule.register_native_system::<P, S>(id, stage, order, self, system);
        self.schedule = schedule;
        result
    }

    pub(crate) fn register_boxed_native_system(
        &mut self,
        system: BoxedSceneSystem,
    ) -> Result<(), ScheduleError> {
        let mut schedule = std::mem::take(&mut self.schedule);
        let result = schedule.register_boxed_native_system(system);
        self.schedule = schedule;
        result
    }

    #[cfg(test)]
    pub(crate) fn scheduled_native_system_steps_for_stage(
        &self,
        stage: SystemStage,
    ) -> Vec<ScheduledSceneStep> {
        self.schedule.native_system_steps_for_stage(stage)
    }

    pub(crate) fn run_native_scene_system(&mut self, id: &str) -> bool {
        let mut schedule = std::mem::take(&mut self.schedule);
        let Some(mut system) = schedule.take_native_system(id) else {
            self.schedule = schedule;
            return false;
        };
        self.schedule = schedule;

        let result = catch_unwind(AssertUnwindSafe(|| system.run(self)));

        let mut schedule = std::mem::take(&mut self.schedule);
        schedule.restore_native_system(system);
        self.schedule = schedule;

        if let Err(payload) = result {
            resume_unwind(payload);
        }

        true
    }

    pub(crate) fn take_worldless_native_scene_systems(
        &mut self,
        ids: &[&str],
    ) -> Option<Vec<BoxedSceneSystem>> {
        let mut schedule = std::mem::take(&mut self.schedule);
        let mut systems = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(system) = schedule.take_native_system(id) else {
                for system in systems {
                    schedule.restore_native_system(system);
                }
                self.schedule = schedule;
                return None;
            };
            if !system.supports_worldless_execution() {
                schedule.restore_native_system(system);
                for system in systems {
                    schedule.restore_native_system(system);
                }
                self.schedule = schedule;
                return None;
            }
            systems.push(system);
        }
        self.schedule = schedule;
        Some(systems)
    }

    pub(crate) fn restore_worldless_native_scene_systems(
        &mut self,
        systems: Vec<BoxedSceneSystem>,
    ) {
        let mut schedule = std::mem::take(&mut self.schedule);
        for system in systems {
            schedule.restore_native_system(system);
        }
        self.schedule = schedule;
    }

    #[cfg(test)]
    pub(crate) fn run_native_scene_systems_for_stage(&mut self, stage: SystemStage) {
        let steps = self
            .scheduled_native_system_steps_for_stage(stage)
            .into_iter()
            .collect::<Vec<_>>();

        for step in steps {
            match step {
                ScheduledSceneStep::Native { id, .. } => {
                    self.run_native_scene_system(&id);
                }
                ScheduledSceneStep::Runtime { .. } => {}
                ScheduledSceneStep::ApplyDeferred { .. } => {
                    self.apply_deferred();
                }
            }
        }
    }
}
