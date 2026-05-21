use std::fmt;
use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTick, ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError,
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

    pub(crate) fn state(&self) -> &P::State {
        &self.state
    }

    pub fn run<R>(&mut self, world: &mut World, f: impl FnOnce(P::Item<'_>) -> R) -> R {
        let this_run = world.advance_change_tick();
        let previous_active_tick = world.replace_active_change_tick(Some(this_run));
        let ticks = ChangeTickWindow::new(self.last_run, this_run);
        let item = unsafe { P::get_param(world as *mut World, &mut self.state, ticks) };
        let result = f(item);
        world.replace_active_change_tick(previous_active_tick);
        self.last_run = this_run;
        result
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
