//! Live authority for terminal publication of work prepared from a level snapshot.
//!
//! A source owns an immutable snapshot for unlocked preparation and weak references to the
//! level's world owner and lifecycle. It can publish only while the original owner is alive,
//! still loaded, and still represents the captured World generation and replacement epoch.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::level_system::LevelLifecycleState;
use super::world::World;

/// The lifetime token shared by every clone of one live LevelSystem.
pub(crate) struct WorldPublicationOwner {
    pub(crate) world: Arc<Mutex<World>>,
    pub(crate) replacement_epoch: Arc<AtomicU64>,
    pub(crate) lifecycle_epoch: AtomicU64,
}

/// A snapshot plus a weak, generation checked publication authority.
///
/// The only constructor is LevelSystem::capture, which captures the
/// snapshot while holding the authoritative World mutex. Consumers may prepare from snapshot
/// without retaining that mutex, then use publish for the short terminal state transition.
pub struct WorldPublicationSource {
    pub(crate) owner: std::sync::Weak<WorldPublicationOwner>,
    pub(crate) lifecycle: std::sync::Weak<Mutex<LevelLifecycleState>>,
    snapshot: World,
    generation: u64,
    replacement_epoch: u64,
    lifecycle_epoch: u64,
}

impl Clone for WorldPublicationSource {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            lifecycle: self.lifecycle.clone(),
            snapshot: self.snapshot.clone(),
            generation: self.generation,
            replacement_epoch: self.replacement_epoch,
            lifecycle_epoch: self.lifecycle_epoch,
        }
    }
}

impl fmt::Debug for WorldPublicationSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorldPublicationSource")
            .field("generation", &self.generation)
            .field("replacement_epoch", &self.replacement_epoch)
            .field("lifecycle_epoch", &self.lifecycle_epoch)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldPublicationError {
    WorldOwnerDropped,
    LifecycleDropped,
    LevelUnloaded,
    WorldGenerationChanged { expected: u64, actual: u64 },
    WorldReplacementChanged { expected: u64, actual: u64 },
    LifecycleChanged { expected: u64, actual: u64 },
}

impl fmt::Display for WorldPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldOwnerDropped => formatter.write_str("navigation World owner was dropped"),
            Self::LifecycleDropped => formatter.write_str("navigation level lifecycle was dropped"),
            Self::LevelUnloaded => formatter.write_str("navigation level is unloaded"),
            Self::WorldGenerationChanged { expected, actual } => write!(
                formatter,
                "navigation World publication generation changed (expected {expected}, actual {actual})"
            ),
            Self::WorldReplacementChanged { expected, actual } => write!(
                formatter,
                "navigation World publication replacement epoch changed (expected {expected}, actual {actual})"
            ),
            Self::LifecycleChanged { expected, actual } => write!(
                formatter,
                "navigation level lifecycle changed (expected epoch {expected}, actual {actual})"
            ),
        }
    }
}

impl std::error::Error for WorldPublicationError {}

impl WorldPublicationSource {
    pub(crate) fn from_capture(
        owner: std::sync::Weak<WorldPublicationOwner>,
        lifecycle: std::sync::Weak<Mutex<LevelLifecycleState>>,
        snapshot: World,
        generation: u64,
        replacement_epoch: u64,
        lifecycle_epoch: u64,
    ) -> Self {
        Self {
            owner,
            lifecycle,
            snapshot,
            generation,
            replacement_epoch,
            lifecycle_epoch,
        }
    }

    /// Returns the immutable World snapshot captured by the owning LevelSystem.
    pub fn snapshot(&self) -> &World {
        &self.snapshot
    }

    /// Returns the World generation captured with the snapshot.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the replacement identity captured with the snapshot.
    pub fn replacement_epoch(&self) -> u64 {
        self.replacement_epoch
    }

    /// Runs one short terminal publication while the live World and lifecycle locks are held.
    ///
    /// The lock order is World owner, lifecycle, project asset generation (when the callback
    /// chooses to acquire it), then the navigation manager. The callback must only publish already
    /// prepared state; it must not load assets, run external callbacks, or reenter the live level.
    /// set_lifecycle takes only the lifecycle lock and advances its epoch before changing state.
    pub fn publish<T>(
        &self,
        publish: impl FnOnce(&World) -> T,
    ) -> Result<T, WorldPublicationError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(WorldPublicationError::WorldOwnerDropped)?;
        let lifecycle = self
            .lifecycle
            .upgrade()
            .ok_or(WorldPublicationError::LifecycleDropped)?;

        let world = owner
            .world
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let actual_replacement_epoch = owner.replacement_epoch.load(Ordering::Acquire);
        if actual_replacement_epoch != self.replacement_epoch {
            return Err(WorldPublicationError::WorldReplacementChanged {
                expected: self.replacement_epoch,
                actual: actual_replacement_epoch,
            });
        }
        let actual_generation = world.world_generation();
        if actual_generation != self.generation {
            return Err(WorldPublicationError::WorldGenerationChanged {
                expected: self.generation,
                actual: actual_generation,
            });
        }

        let lifecycle = lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *lifecycle != LevelLifecycleState::Loaded {
            return Err(WorldPublicationError::LevelUnloaded);
        }

        // A level may unload and reload without changing its World. That ABA transition must
        // still reject work prepared during an earlier loaded lifetime.
        let actual_lifecycle_epoch = owner.lifecycle_epoch.load(Ordering::Acquire);
        if actual_lifecycle_epoch != self.lifecycle_epoch {
            return Err(WorldPublicationError::LifecycleChanged {
                expected: self.lifecycle_epoch,
                actual: actual_lifecycle_epoch,
            });
        }
        Ok(publish(&world))
    }
}

#[cfg(test)]
#[path = "world_publication/tests/cases.rs"]
mod tests;
