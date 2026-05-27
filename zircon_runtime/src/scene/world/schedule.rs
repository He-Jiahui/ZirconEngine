use super::World;
use crate::scene::ecs::{
    IntoSceneSystem, Schedule, ScheduleError, ScheduledSceneStep, SystemParam, SystemStage,
};

impl World {
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
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

        system.run(self);

        let mut schedule = std::mem::take(&mut self.schedule);
        schedule.restore_native_system(system);
        self.schedule = schedule;

        true
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
                ScheduledSceneStep::ApplyDeferred { .. } => self.apply_deferred(),
                ScheduledSceneStep::Internal(_) | ScheduledSceneStep::Hook(_) => {}
            }
        }
    }
}
