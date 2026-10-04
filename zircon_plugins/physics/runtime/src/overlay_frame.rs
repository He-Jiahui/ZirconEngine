use serde::{Deserialize, Serialize};

use zircon_runtime::core::framework::physics::{PhysicsColliderSyncState, PhysicsWorldSyncState};
use zircon_runtime::core::framework::scene::SceneResource;

pub const PHYSICS_OVERLAY_FRAME_EVENT_ID: &str = "physics.events.overlay_frame";
pub const PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA: &str = "physics.events.overlay_frame.v1";

/// The canonical physics synchronization payload mirrored to an active editor overlay reader.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsOverlayFrame {
    pub owner_generation: u64,
    pub colliders: Vec<PhysicsColliderSyncState>,
}

impl PhysicsOverlayFrame {
    pub fn from_sync(owner_generation: u64, sync: &PhysicsWorldSyncState) -> Self {
        Self {
            owner_generation,
            colliders: sync.colliders.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsDebugOverlayCapture {
    pub enabled: bool,
}

impl SceneResource for PhysicsDebugOverlayCapture {}
