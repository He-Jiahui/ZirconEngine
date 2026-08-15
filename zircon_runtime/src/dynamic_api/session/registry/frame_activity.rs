use std::sync::{Mutex, MutexGuard};

use super::frame_demand::FrameDemandAccumulator;
use super::{RuntimeFrameDemand, RuntimeWakeRegistration};

/// Session-scoped frame activity shared by synchronous producers and async observers.
pub(in crate::dynamic_api::session) struct RuntimeFrameActivity {
    demand: Mutex<FrameDemandAccumulator>,
    wake: RuntimeWakeRegistration,
}

impl RuntimeFrameActivity {
    pub(in crate::dynamic_api::session) fn new(wake: RuntimeWakeRegistration) -> Self {
        Self {
            demand: Mutex::new(FrameDemandAccumulator::default()),
            wake,
        }
    }

    pub(in crate::dynamic_api::session) fn begin_tick(&self) {
        self.lock_demand().consume();
    }

    pub(in crate::dynamic_api::session) fn request_frame(&self, demand: RuntimeFrameDemand) {
        self.lock_demand().merge(demand);
    }

    pub(in crate::dynamic_api::session) fn consume_frame_demand(&self) -> RuntimeFrameDemand {
        self.lock_demand().consume()
    }

    pub(in crate::dynamic_api::session) fn wake_registration(&self) -> RuntimeWakeRegistration {
        self.wake.clone()
    }

    pub(in crate::dynamic_api::session) fn disable_wake_entries(&self) {
        self.wake.disable_new_entries();
    }

    pub(in crate::dynamic_api::session) fn wait_for_wake_callbacks(&self) {
        self.wake.wait_for_callbacks();
    }

    pub(in crate::dynamic_api::session) fn wake_callback_active_on_current_thread(&self) -> bool {
        self.wake.callback_active_on_current_thread()
    }

    fn lock_demand(&self) -> MutexGuard<'_, FrameDemandAccumulator> {
        self.demand
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
