use std::fmt;
use std::marker::PhantomData;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

use crate::scene::ecs::{
    ChangeTick, ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError,
    WorkerCommandBuffer, WorldlessSystemParam,
};
use crate::scene::World;

pub struct SystemState<P>
where
    P: SystemParam,
{
    state: P::State,
    access: SystemParamAccess,
    last_run: ChangeTick,
    _marker: PhantomData<fn() -> P>,
}

impl<P> SystemState<P>
where
    P: SystemParam,
{
    pub fn new(world: &mut World) -> Result<Self, SystemParamError> {
        let mut access = SystemParamAccess::default();
        let state = P::init_state(world, &mut access)?;
        Ok(Self {
            state,
            access,
            last_run: ChangeTick::ZERO,
            _marker: PhantomData,
        })
    }

    pub fn access(&self) -> &SystemParamAccess {
        &self.access
    }

    pub fn last_run(&self) -> ChangeTick {
        self.last_run
    }

    #[cfg(test)]
    pub(crate) fn state(&self) -> &P::State {
        &self.state
    }

    pub fn run<R>(&mut self, world: &mut World, f: impl FnOnce(P::Item<'_>) -> R) -> R {
        if let Some(buffer) = self.deferred_command_buffer_mut() {
            buffer.begin_run();
            world.reclaim_worker_command_buffer(buffer);
        }
        let this_run = world.advance_change_tick();
        let previous_active_tick = world.replace_active_change_tick(Some(this_run));
        let ticks = ChangeTickWindow::new(self.last_run, this_run);
        let item = unsafe { P::get_param(world as *mut World, &mut self.state, ticks) };
        let result = catch_unwind(AssertUnwindSafe(|| f(item)));
        world.replace_active_change_tick(previous_active_tick);
        match result {
            Ok(result) => {
                P::record_performance_diagnostics(world, &mut self.state);
                self.last_run = this_run;
                if let Some(buffer) = self.deferred_command_buffer_mut() {
                    world.merge_worker_command_buffer(buffer);
                }
                result
            }
            Err(payload) => {
                if let Some(buffer) = self.deferred_command_buffer_mut() {
                    buffer.discard_pending();
                }
                resume_unwind(payload);
            }
        }
    }

    pub(crate) fn run_without_world<R>(&mut self, f: impl FnOnce(P::Item<'_>) -> R) -> R
    where
        P: WorldlessSystemParam,
    {
        if let Some(buffer) = self.deferred_command_buffer_mut() {
            buffer.begin_run();
        }
        let item = P::get_param_without_world(&mut self.state);
        f(item)
    }

    pub(crate) fn deferred_command_buffer_mut(&mut self) -> Option<&mut WorkerCommandBuffer> {
        P::deferred_command_buffer_mut(&mut self.state)
    }
}

impl<P> fmt::Debug for SystemState<P>
where
    P: SystemParam,
    P::State: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemState")
            .field("state", &self.state)
            .field("access", &self.access)
            .field("last_run", &self.last_run)
            .finish_non_exhaustive()
    }
}
