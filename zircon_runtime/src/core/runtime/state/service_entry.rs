use std::sync::Arc;
use std::thread::ThreadId;

use super::super::descriptors::{PluginFactory, RegistryName, ServiceFactory};
use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, LifecycleState, StartupMode};

const UNASSIGNED_SERVICE_INDEX: u32 = 0;
const INITIAL_SERVICE_GENERATION: u32 = 1;

#[derive(Clone)]
pub(crate) enum ServiceEntryFactory {
    Service(ServiceFactory),
    Plugin(PluginFactory),
}

pub(crate) struct ServiceEntry {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) startup_mode: StartupMode,
    // Dependencies are immutable after registration; sharing the canonical name
    // slice keeps resolution from rebuilding a Vec while holding the service lock.
    pub(crate) dependencies: Arc<[RegistryName]>,
    pub(crate) factory: ServiceEntryFactory,
    pub(crate) lifecycle: LifecycleState,
    pub(crate) initialization_owner: Option<ThreadId>,
    pub(crate) instance: Option<ServiceObject>,
    pub(crate) admission_open: bool,
    pub(crate) in_flight_calls: usize,
}

impl ServiceEntry {
    pub(crate) fn unassigned_index() -> u32 {
        UNASSIGNED_SERVICE_INDEX
    }

    pub(crate) fn initial_generation() -> u32 {
        INITIAL_SERVICE_GENERATION
    }

    pub(crate) fn assign_index(&mut self, index: u32) {
        debug_assert_ne!(index, UNASSIGNED_SERVICE_INDEX);
        debug_assert_eq!(self.index, UNASSIGNED_SERVICE_INDEX);
        self.index = index;
    }

    pub(crate) fn invalidate_for_unload(&mut self) {
        debug_assert!(!self.admission_open);
        debug_assert_eq!(self.in_flight_calls, 0);
        self.instance = None;
        self.initialization_owner = None;
        self.admission_open = false;
        self.generation = next_service_generation(self.generation);
        self.lifecycle = LifecycleState::Unloaded;
    }

    pub(crate) fn open_admission(&mut self) {
        debug_assert_eq!(self.lifecycle, LifecycleState::Running);
        debug_assert!(self.instance.is_some());
        self.admission_open = true;
    }

    pub(crate) fn close_admission(&mut self) {
        self.admission_open = false;
    }

    pub(crate) fn enter_call(&mut self, service_name: &str) -> Result<(), CoreError> {
        if !self.admission_open || self.lifecycle != LifecycleState::Running {
            return Err(CoreError::ServiceUnavailable(service_name.to_owned()));
        }
        self.in_flight_calls = self
            .in_flight_calls
            .checked_add(1)
            .ok_or_else(|| CoreError::ServiceUnavailable(service_name.to_owned()))?;
        Ok(())
    }

    pub(crate) fn leave_call(&mut self) -> bool {
        debug_assert!(self.in_flight_calls > 0);
        if self.in_flight_calls == 0 {
            return false;
        }
        self.in_flight_calls -= 1;
        self.in_flight_calls == 0
    }

    pub(crate) fn prepare_for_reactivation(&mut self) {
        debug_assert_eq!(self.lifecycle, LifecycleState::Unloaded);
        debug_assert!(self.instance.is_none());
        debug_assert!(self.initialization_owner.is_none());
        debug_assert_eq!(self.in_flight_calls, 0);
        self.admission_open = false;
        self.lifecycle = LifecycleState::Registered;
    }

    pub(crate) fn reset_after_failed_reactivation(&mut self) {
        if self.instance.take().is_some() {
            self.generation = next_service_generation(self.generation);
        }
        self.initialization_owner = None;
        self.admission_open = false;
        self.lifecycle = LifecycleState::Unloaded;
    }

    pub(crate) fn reset_after_failed_activation(&mut self) {
        if self.instance.take().is_some() {
            self.generation = next_service_generation(self.generation);
        }
        self.initialization_owner = None;
        self.admission_open = false;
        self.lifecycle = LifecycleState::Registered;
    }
}

fn next_service_generation(current: u32) -> u32 {
    let next = current.wrapping_add(1);
    if next == UNASSIGNED_SERVICE_INDEX {
        INITIAL_SERVICE_GENERATION
    } else {
        next
    }
}
