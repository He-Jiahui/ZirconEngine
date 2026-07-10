mod event;
mod pair;
mod point;
mod scan;

use zircon_runtime::core::framework::physics::{
    PhysicsSettings, PhysicsTriggerEvent, PhysicsWorldSyncState,
};

pub(crate) use pair::PhysicsTriggerPairMap;

pub(crate) fn compute_trigger_events(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
    previous: &PhysicsTriggerPairMap,
) -> (PhysicsTriggerPairMap, Vec<PhysicsTriggerEvent>) {
    scan::compute_trigger_events(sync, settings, previous)
}
