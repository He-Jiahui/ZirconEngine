//! Explicit, owner-bound time disposition for a whole-World replacement.
//! These are in-memory values. They are not archive data or a clock authority.

use std::fmt;
use std::sync::Weak;
use std::time::Duration;

use thiserror::Error;

use crate::core::framework::time::{ClockDomainStamp, TimePolicy, TimePolicyError};
use crate::scene::world_publication::WorldPublicationOwner;
use crate::scene::World;

use super::{SimulationTickId, WorldTimeState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldReplacementTimePolicy {
    Preserve,
    Reset,
    Restore,
}

/// Records the caller's purpose; it grants neither replacement nor restore authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldReplacementReason {
    ExplicitHostRequest,
    SceneTravel,
    HotReload,
    CheckpointRestore,
    LegacyCompatibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldReplacementIdentity {
    pub(in crate::scene) world_generation: u64,
    pub(in crate::scene) replacement_epoch: u64,
}

impl WorldReplacementIdentity {
    pub const fn world_generation(self) -> u64 {
        self.world_generation
    }

    pub const fn replacement_epoch(self) -> u64 {
        self.replacement_epoch
    }
}

/// A target captured from one Level owner while its World lane is held.
#[derive(Clone)]
pub struct WorldReplacementExpectation {
    pub(in crate::scene) owner: Weak<WorldPublicationOwner>,
    pub(in crate::scene) identity: WorldReplacementIdentity,
}

impl WorldReplacementExpectation {
    pub const fn identity(&self) -> WorldReplacementIdentity {
        self.identity
    }
}

impl fmt::Debug for WorldReplacementExpectation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorldReplacementExpectation")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

/// The actual retired World and its matching clocks, moved out by one successful
/// explicit replacement. It is neither a World clone nor a serializable snapshot.
/// A weak owner binding does not extend the Level's lifetime. Dropping the token
/// releases its World on the caller's stack, outside the replacement locks.
pub struct WorldReplacementCheckpoint {
    pub(in crate::scene) owner: Weak<WorldPublicationOwner>,
    pub(in crate::scene) world: World,
    pub(in crate::scene) time: WorldTimeState,
    pub(in crate::scene) identity: WorldReplacementIdentity,
}

impl WorldReplacementCheckpoint {
    pub const fn identity(&self) -> WorldReplacementIdentity {
        self.identity
    }

    pub const fn time(&self) -> WorldTimeState {
        self.time
    }

    pub fn world(&self) -> &World {
        &self.world
    }
}

impl fmt::Debug for WorldReplacementCheckpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorldReplacementCheckpoint")
            .field("identity", &self.identity)
            .field("time", &self.time)
            .finish_non_exhaustive()
    }
}

/// Restore accepts only the owned pair returned by this same Level. A copied
/// WorldTimeState cannot be combined with an unrelated caller-supplied World.
pub enum WorldReplacementRequest {
    Preserve {
        world: World,
    },
    Reset {
        world: World,
        policy: TimePolicy,
        paused: bool,
    },
    Restore {
        checkpoint: WorldReplacementCheckpoint,
    },
}

impl WorldReplacementRequest {
    pub fn preserve(world: World) -> Self {
        Self::Preserve { world }
    }

    pub fn reset(world: World, policy: TimePolicy, paused: bool) -> Self {
        Self::Reset {
            world,
            policy,
            paused,
        }
    }

    pub fn restore(checkpoint: WorldReplacementCheckpoint) -> Self {
        Self::Restore { checkpoint }
    }

    pub const fn time_policy(&self) -> WorldReplacementTimePolicy {
        match self {
            Self::Preserve { .. } => WorldReplacementTimePolicy::Preserve,
            Self::Reset { .. } => WorldReplacementTimePolicy::Reset,
            Self::Restore { .. } => WorldReplacementTimePolicy::Restore,
        }
    }

    pub(in crate::scene) fn incoming_world(&self) -> &World {
        match self {
            Self::Preserve { world } | Self::Reset { world, .. } => world,
            Self::Restore { checkpoint } => &checkpoint.world,
        }
    }

    pub(in crate::scene) fn incoming_world_mut(&mut self) -> &mut World {
        match self {
            Self::Preserve { world } | Self::Reset { world, .. } => world,
            Self::Restore { checkpoint } => &mut checkpoint.world,
        }
    }

    pub(in crate::scene) fn into_world(self) -> World {
        match self {
            Self::Preserve { world } | Self::Reset { world, .. } => world,
            Self::Restore { checkpoint } => checkpoint.world,
        }
    }

    pub(in crate::scene) fn time_disposition(&self) -> WorldReplacementTimeDisposition {
        match self {
            Self::Preserve { .. } => WorldReplacementTimeDisposition::Preserve,
            Self::Reset { policy, paused, .. } => WorldReplacementTimeDisposition::Reset {
                policy: *policy,
                paused: *paused,
            },
            Self::Restore { checkpoint } => {
                WorldReplacementTimeDisposition::Restore(checkpoint.time)
            }
        }
    }
}

impl fmt::Debug for WorldReplacementRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorldReplacementRequest")
            .field("time_policy", &self.time_policy())
            .field(
                "incoming_world_generation",
                &self.incoming_world().world_generation(),
            )
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum WorldReplacementTimeDisposition {
    Preserve,
    Reset { policy: TimePolicy, paused: bool },
    Restore(WorldTimeState),
}

/// All values describe the one committed World/time boundary, not a later read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldReplacementReceipt {
    pub(in crate::scene) previous: WorldReplacementIdentity,
    pub(in crate::scene) current: WorldReplacementIdentity,
    pub(in crate::scene) time_policy: WorldReplacementTimePolicy,
    pub(in crate::scene) reason: WorldReplacementReason,
    pub(in crate::scene) before: WorldTimeState,
    pub(in crate::scene) after: WorldTimeState,
    pub(in crate::scene) preserved_debt: Duration,
    pub(in crate::scene) discarded_debt: Duration,
    pub(in crate::scene) restored_debt: Duration,
    pub(in crate::scene) restored_checkpoint: Option<WorldReplacementIdentity>,
}

impl WorldReplacementReceipt {
    pub const fn previous(self) -> WorldReplacementIdentity {
        self.previous
    }
    pub const fn current(self) -> WorldReplacementIdentity {
        self.current
    }
    pub const fn time_policy(self) -> WorldReplacementTimePolicy {
        self.time_policy
    }
    pub const fn reason(self) -> WorldReplacementReason {
        self.reason
    }
    pub const fn before(self) -> WorldTimeState {
        self.before
    }
    pub const fn after(self) -> WorldTimeState {
        self.after
    }
    pub const fn preserved_debt(self) -> Duration {
        self.preserved_debt
    }
    pub const fn discarded_debt(self) -> Duration {
        self.discarded_debt
    }
    pub const fn restored_debt(self) -> Duration {
        self.restored_debt
    }
    pub const fn restored_checkpoint(self) -> Option<WorldReplacementIdentity> {
        self.restored_checkpoint
    }
}

#[derive(Debug)]
pub struct WorldReplacementCommit {
    pub(in crate::scene) receipt: WorldReplacementReceipt,
    pub(in crate::scene) checkpoint: WorldReplacementCheckpoint,
}

impl WorldReplacementCommit {
    pub const fn receipt(&self) -> WorldReplacementReceipt {
        self.receipt
    }

    pub fn into_parts(self) -> (WorldReplacementReceipt, WorldReplacementCheckpoint) {
        (self.receipt, self.checkpoint)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorldReplacementError {
    #[error("the replacement target belongs to a different Level owner")]
    TargetOwnerChanged,
    #[error("the replacement target changed: expected {expected:?}, found {actual:?}")]
    TargetChanged {
        expected: WorldReplacementIdentity,
        actual: WorldReplacementIdentity,
    },
    #[error("the checkpoint belongs to a different Level owner")]
    CheckpointOwnerChanged,
    #[error("fixed step {active:?} must settle before an explicit replacement")]
    FixedStepActive { active: SimulationTickId },
    #[error("{remaining} fixed-step slots remain in the accepted outer frame")]
    FixedStepBudgetPending { remaining: u32 },
    #[error("invalid replacement time policy: {0}")]
    InvalidTimePolicy(#[from] TimePolicyError),
    #[error("cannot rebase clock {clock:?} beyond epoch {floor}")]
    ClockEpochExhausted { clock: ClockDomainStamp, floor: u64 },
    #[error("world time policy generation {current} is exhausted")]
    PolicyGenerationExhausted { current: u64 },
    #[error("world replacement epoch {current} is exhausted")]
    ReplacementEpochExhausted { current: u64 },
    #[error("world generation is exhausted: current {current}, incoming {incoming}")]
    WorldGenerationExhausted { current: u64, incoming: u64 },
}

/// No live state was published. The original request, including a restore token,
/// is returned so the caller can retry or drop it after all Level guards are gone.
#[derive(Debug)]
pub struct WorldReplacementRejected {
    pub(in crate::scene) error: WorldReplacementError,
    pub(in crate::scene) request: WorldReplacementRequest,
}

impl WorldReplacementRejected {
    pub fn error(&self) -> &WorldReplacementError {
        &self.error
    }

    pub fn into_parts(self) -> (WorldReplacementError, WorldReplacementRequest) {
        (self.error, self.request)
    }
}

impl fmt::Display for WorldReplacementRejected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for WorldReplacementRejected {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
