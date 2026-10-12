#[path = "action_guard.rs"]
mod action_guard;
#[path = "allocation_registry.rs"]
mod allocation_registry;
#[path = "frame_activity.rs"]
mod frame_activity;
#[path = "frame_demand.rs"]
mod frame_demand;
#[path = "session_owner.rs"]
mod session_owner;
#[path = "session_slot.rs"]
mod session_slot;
#[path = "session_store.rs"]
mod session_store;
#[path = "wake_registration.rs"]
mod wake_registration;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(super) use allocation_registry::{
    register_runtime_allocation, register_runtime_allocation_in_action, release_runtime_allocation,
    RuntimeAllocationKind,
};
pub(super) use frame_activity::RuntimeFrameActivity;
pub(super) use frame_demand::{RuntimeFrameDemand, MAX_RUNTIME_FRAME_DEMAND_DELAY};
pub(super) use session_store::{
    destroy_session_slot, try_create_linked_session, try_create_session_with_wake, with_session,
    with_session_activity, with_session_activity_result_finalized, with_session_owned,
    with_session_result_committed, with_session_result_finalized, SessionRegistryInsertError,
};
#[cfg(test)]
pub(super) use session_store::{
    insert_session, insert_session_with_wake, poison_registry_lock_for_test, session_is_closing,
    try_insert_session, try_insert_session_with_wake,
};
pub(super) use wake_registration::RuntimeWakeRegistration;

#[cfg(test)]
pub(super) use session_owner::runtime::startup_observation::{
    observe_runtime_startup_for_test, RuntimeStartupObservation,
};
#[cfg(test)]
pub(super) use session_owner::OwnerShutdownReceipt;
#[cfg(test)]
pub(super) use session_store::session_handle_is_published_for_test;
