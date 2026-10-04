use std::fmt;
use std::marker::PhantomData;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

use crate::scene::ecs::{
    ChangeTick, ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError,
    WorkerCommandBuffer, WorldlessSystemParam,
};
use crate::scene::World;

/// 持有参数缓存、访问集与逐系统的变更窗口；调度系统在多次调用间复用，并在所属 World 退出时退休。
pub struct SystemState<P>
where
    P: SystemParam,
{
    state: P::State,
    access: SystemParamAccess,
    last_run: ChangeTick,
    retired: bool,
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
            retired: false,
            _marker: PhantomData,
        })
    }

    pub fn access(&self) -> &SystemParamAccess {
        &self.access
    }

    pub fn last_run(&self) -> ChangeTick {
        self.last_run
    }

    pub fn is_retired(&self) -> bool {
        self.retired
    }

    /// Releases all World-bound parameter state. It is idempotent because
    /// schedule retirement and explicit owner shutdown may meet at the same
    /// lifecycle boundary.
    pub fn retire(&mut self, world: &mut World) {
        if self.retired {
            return;
        }
        if let Some(buffer) = self.deferred_command_buffer_mut() {
            buffer.discard_pending();
        }
        P::retire_state(world, &mut self.state);
        self.retired = true;
    }

    /// Rebuilds parameter state against a new World after first constructing
    /// the replacement. A failed rebuild leaves the existing state active.
    pub fn rebind(&mut self, world: &mut World) -> Result<(), SystemParamError> {
        let mut access = SystemParamAccess::default();
        let state = P::init_state(world, &mut access)?;
        // BUG: [CR-R02-runtime_ecs_schedule_systems-0005] 活跃 EventReaderParam 跨 World rebind 且启用调试断言时，旧 lease 被交给新 World，retire 的归属检查触发 panic。证据：此调用传新 World，disconnect_reader 检查 registry 身份；同 World 已退休测试未覆盖。
        self.retire(world);
        self.state = state;
        self.access = access;
        self.last_run = ChangeTick::ZERO;
        self.retired = false;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn state(&self) -> &P::State {
        &self.state
    }

    pub fn run<R>(&mut self, world: &mut World, f: impl FnOnce(P::Item<'_>) -> R) -> R {
        assert!(
            !self.retired,
            "retired system state must be rebound before it can run"
        );
        if let Some(buffer) = self.deferred_command_buffer_mut() {
            buffer.begin_run();
            world.reclaim_worker_command_buffer(buffer);
        }
        // 参数获取与回调共享活动 tick；二者展开时先恢复 World 的活动 tick，last_run 保留原窗口以供重试。
        let this_run = world.advance_change_tick();
        let ticks = ChangeTickWindow::new(self.last_run, this_run);
        let result = {
            let mut active_tick_guard = ActiveChangeTickGuard::enter(world, this_run);
            catch_unwind(AssertUnwindSafe(|| {
                // One original grant spans all Params and the callback. Items
                // drop before this scope ends and before diagnostics/deferred merge.
                // Their access and retained-leaf contracts must compose; a short
                // scoped parent read is not a substitute for that compatibility.
                let item =
                    unsafe { P::get_param(active_tick_guard.world_ptr(), &mut self.state, ticks) };
                f(item)
            }))
        };
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
        assert!(
            !self.retired,
            "retired system state must be rebound before it can run"
        );
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

/// Restores World mutation attribution when parameter construction or callback
/// execution unwinds. The guard owns the only mutable World borrow throughout
/// the system window, so no unrelated mutation can observe the active tick.
struct ActiveChangeTickGuard<'world> {
    world: &'world mut World,
    previous_active_tick: Option<ChangeTick>,
}

impl<'world> ActiveChangeTickGuard<'world> {
    fn enter(world: &'world mut World, active_tick: ChangeTick) -> Self {
        let previous_active_tick = world.replace_active_change_tick(Some(active_tick));
        Self {
            world,
            previous_active_tick,
        }
    }

    fn world_ptr(&mut self) -> *mut World {
        self.world
    }
}

impl Drop for ActiveChangeTickGuard<'_> {
    fn drop(&mut self) {
        self.world
            .replace_active_change_tick(self.previous_active_tick);
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
            .field("retired", &self.retired)
            .finish_non_exhaustive()
    }
}
