use super::World;
#[cfg(test)]
use crate::scene::ecs::ScheduledSceneStep;
use crate::scene::ecs::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, DeferredSystemKey, IntoSceneSystem,
    IntoWorldlessSceneSystem, Schedule, ScheduleError, SystemParam, SystemStage,
    WorldlessSystemParam,
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

    pub fn register_worldless_native_system<P, S>(
        &mut self,
        id: impl Into<String>,
        stage: SystemStage,
        order: i32,
        system: S,
    ) -> Result<(), ScheduleError>
    where
        P: WorldlessSystemParam + 'static,
        P::State: Send,
        S: IntoWorldlessSceneSystem<P>,
    {
        let mut schedule = std::mem::take(&mut self.schedule);
        let result =
            schedule.register_worldless_native_system::<P, S>(id, stage, order, self, system);
        self.schedule = schedule;
        result
    }

    pub(crate) fn register_boxed_native_system(
        &mut self,
        system: BoxedSceneSystem,
    ) -> Result<(), ScheduleError> {
        let mut schedule = std::mem::take(&mut self.schedule);
        let result = schedule.register_boxed_native_system(self, system);
        self.schedule = schedule;
        result
    }

    /// Permanently removes a native system after it has reached the schedule's
    /// quiescent boundary. A running worker system reports `SystemInFlight`
    /// instead of releasing World-bound parameter state concurrently.
    pub fn unregister_native_system(&mut self, id: &str) -> Result<bool, ScheduleError> {
        let mut schedule = std::mem::take(&mut self.schedule);
        let result = schedule.unregister_native_system(self, id);
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
        let key = schedule
            .native_system_deferred_key(id)
            .expect("registered native system must have a compiled schedule key");
        self.schedule = schedule;

        system.bind_deferred_system_key(key);
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

        let mut worker_batch = Vec::new();
        for step in steps {
            match step {
                ScheduledSceneStep::Native {
                    id,
                    stage,
                    order,
                    worker_safe,
                    ..
                } => {
                    if worker_safe {
                        let key = DeferredSystemKey::compiled(stage.rank(), order, id.clone());
                        worker_batch.push((id, key));
                    } else {
                        self.flush_test_worldless_native_batch(&mut worker_batch);
                        self.run_native_scene_system(&id);
                    }
                }
                ScheduledSceneStep::Runtime { .. } => {}
                ScheduledSceneStep::ApplyDeferred { .. } => {
                    self.flush_test_worldless_native_batch(&mut worker_batch);
                    self.apply_deferred();
                }
            }
        }
        self.flush_test_worldless_native_batch(&mut worker_batch);
        self.apply_deferred();
    }

    #[cfg(test)]
    fn flush_test_worldless_native_batch(
        &mut self,
        dispatches: &mut Vec<(String, DeferredSystemKey)>,
    ) {
        if dispatches.is_empty() {
            return;
        }
        let ids = dispatches
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>();
        let mut systems = self
            .take_worldless_native_scene_systems(&ids)
            .expect("scheduled worldless systems must remain registered");
        for (system, (_, key)) in systems.iter_mut().zip(dispatches.iter()) {
            if let Some(buffer) = system.worker_command_buffer_mut() {
                buffer.bind_compiled_key(key.clone());
            }
        }

        let run_result = catch_unwind(AssertUnwindSafe(|| {
            for system in &mut systems {
                system.run_without_world();
            }
        }));
        if let Err(payload) = run_result {
            for system in &mut systems {
                if let Some(buffer) = system.worker_command_buffer_mut() {
                    buffer.discard_pending();
                }
            }
            self.restore_worldless_native_scene_systems(systems);
            dispatches.clear();
            resume_unwind(payload);
        }

        let mut buffers = systems
            .iter_mut()
            .filter_map(|system| system.worker_command_buffer_mut())
            .collect::<Vec<_>>();
        if !buffers.is_empty() {
            self.merge_worker_command_buffers(&mut buffers)
                .expect("compiled worldless keys must remain unique");
            let apply_result = catch_unwind(AssertUnwindSafe(|| self.apply_deferred()));
            self.reclaim_worker_command_buffers(&mut buffers);
            if let Err(payload) = apply_result {
                self.restore_worldless_native_scene_systems(systems);
                dispatches.clear();
                resume_unwind(payload);
            }
        }
        self.restore_worldless_native_scene_systems(systems);
        dispatches.clear();
    }
}
