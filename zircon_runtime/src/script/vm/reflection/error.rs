use thiserror::Error;
use zircon_runtime_interface::reflect::ReflectError;

use crate::core::CoreError;
use crate::scene::{SceneError, WorldRuntimeExtensionError};
use crate::script::PluginSlotId;

/// Typed failures produced while projecting and publishing VM reflection schemas.
#[derive(Debug, Error)]
pub enum VmReflectionError {
    /// The Core runtime became unavailable while synchronizing managed Worlds.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// A shared reflection registration failed validation.
    #[error(transparent)]
    Reflect(#[from] ReflectError),
    /// A World rejected the projected VM component registration.
    #[error(transparent)]
    Scene(#[from] SceneError),
    /// The scene extension plan could not accept the catalog callback.
    #[error(transparent)]
    WorldExtension(#[from] WorldRuntimeExtensionError),
    /// Another loaded VM package slot already owns the fully-qualified type path.
    #[error(
        "VM reflected type `{type_path}` is owned by slot {owner_slot:?}, not slot {requesting_slot:?}"
    )]
    TypePathOwnedByAnotherSlot {
        /// Fully-qualified reflected type path.
        type_path: String,
        /// Slot that first published the type path.
        owner_slot: PluginSlotId,
        /// Slot attempting to replace the existing owner.
        requesting_slot: PluginSlotId,
    },
    /// A stale package generation attempted to replace a newer reflected schema.
    #[error(
        "VM reflection generation regressed for slot {slot:?}: current={current_generation}, requested={requested_generation}"
    )]
    GenerationRegression {
        /// Package slot whose generation regressed.
        slot: PluginSlotId,
        /// Newest generation already published for the slot.
        current_generation: u32,
        /// Stale generation supplied by the caller.
        requested_generation: u32,
    },
    /// An existing generation was reused with different reflected metadata.
    #[error(
        "VM reflection generation {generation} for slot {slot:?} is immutable and cannot publish a different schema"
    )]
    GenerationConflict {
        /// Package slot whose published generation was reused.
        slot: PluginSlotId,
        /// Equal generation carrying conflicting metadata.
        generation: u32,
    },
    /// A package schema declared a reflected namespace different from its trusted manifest owner.
    #[error(
        "VM reflected type `{type_path}` declares package owner `{declared_owner}`, expected `{expected_owner}`"
    )]
    PackageOwnerMismatch {
        /// Fully-qualified reflected type path supplied by the package schema.
        type_path: String,
        /// Package owner read from the trusted package manifest.
        expected_owner: String,
        /// Owner self-reported by the reflected registration.
        declared_owner: String,
    },
    /// A hot-reload attempted to change the trusted manifest owner of an existing slot.
    #[error(
        "VM reflection slot {slot:?} is owned by package `{current_owner}`, not `{requested_owner}`"
    )]
    SlotOwnerConflict {
        /// Package slot whose trusted owner changed.
        slot: PluginSlotId,
        /// Trusted owner already committed for the slot.
        current_owner: String,
        /// Owner supplied by the newer package generation.
        requested_owner: String,
    },
    /// A prepared schema was based on an older committed catalog epoch.
    #[error(
        "VM reflection prepared generation is stale: base epoch={base_epoch}, committed epoch={committed_epoch}"
    )]
    PreparedGenerationStale {
        /// Committed epoch observed while the candidate was prepared.
        base_epoch: u64,
        /// Committed epoch observed when commit was attempted.
        committed_epoch: u64,
    },
    /// A prepared generation capability was presented to a different catalog instance.
    #[error("VM reflection prepared generation belongs to another catalog")]
    ForeignPreparedGeneration,
    /// The process-wide candidate epoch exhausted its monotonic identifier space.
    #[error("VM reflection candidate epoch exhausted")]
    CandidateEpochExhausted,
    /// The process-wide reflection revision exhausted its monotonic identifier space.
    #[error("VM reflection catalog revision exhausted")]
    RevisionExhausted,
}
