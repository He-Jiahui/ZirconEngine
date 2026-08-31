use thiserror::Error;
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::template::{
    UiCompiledBindingGeneration, UiCompiledBindingProgram,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiBindingQuiescenceReceipt {
    pub tree_id: UiTreeId,
    pub old_generation: UiCompiledBindingGeneration,
    pub published_generation: UiCompiledBindingGeneration,
    pub retired_binding_count: usize,
    pub published_binding_count: usize,
    pub state_entries_migrated: usize,
    pub state_entries_reset: usize,
    pub old_generation_retired: bool,
    pub old_generation_quiescent: bool,
    pub stale_handles_rejected: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct UiBindingReloadTransaction {
    tree_id: UiTreeId,
    old_generation: UiCompiledBindingGeneration,
    published_generation: UiCompiledBindingGeneration,
    old_binding_count: usize,
    published_binding_count: usize,
    retires_old_generation: bool,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingReloadPrepareError {
    #[error("existing binding program for {tree_id:?} is malformed")]
    MalformedPrevious { tree_id: UiTreeId },
    #[error("replacement binding program for {tree_id:?} is malformed")]
    MalformedReplacement { tree_id: UiTreeId },
    #[error(
        "binding reload for {tree_id:?} cannot replace compiled generation {old_generation:?} with an invalid generation"
    )]
    InvalidReplacementGeneration {
        tree_id: UiTreeId,
        old_generation: UiCompiledBindingGeneration,
    },
    #[error("binding reload for {tree_id:?} removed root asset identity {old_asset}")]
    MissingReplacementRootAsset {
        tree_id: UiTreeId,
        old_asset: String,
    },
    #[error(
        "binding reload for {tree_id:?} changed root asset identity from {old_asset} to {new_asset}"
    )]
    RootAssetMismatch {
        tree_id: UiTreeId,
        old_asset: String,
        new_asset: String,
    },
    #[error("binding reload for {tree_id:?} reused generation {generation:?} for different IR")]
    GenerationCollision {
        tree_id: UiTreeId,
        generation: UiCompiledBindingGeneration,
    },
}

impl UiBindingReloadTransaction {
    pub(crate) fn prepare(
        tree_id: UiTreeId,
        previous: &UiCompiledBindingProgram,
        replacement: &UiCompiledBindingProgram,
    ) -> Result<Self, UiBindingReloadPrepareError> {
        if !previous.is_well_formed()
            || (!previous.generation().is_invalid() && previous.asset_id().is_none())
        {
            return Err(UiBindingReloadPrepareError::MalformedPrevious { tree_id });
        }
        if !replacement.is_well_formed() {
            return Err(UiBindingReloadPrepareError::MalformedReplacement { tree_id });
        }
        if !previous.generation().is_invalid() && replacement.generation().is_invalid() {
            return Err(UiBindingReloadPrepareError::InvalidReplacementGeneration {
                tree_id,
                old_generation: previous.generation(),
            });
        }
        if let Some(old_asset) = previous.asset_id() {
            let Some(new_asset) = replacement.asset_id() else {
                return Err(UiBindingReloadPrepareError::MissingReplacementRootAsset {
                    tree_id,
                    old_asset: old_asset.to_string(),
                });
            };
            if old_asset != new_asset {
                return Err(UiBindingReloadPrepareError::RootAssetMismatch {
                    tree_id,
                    old_asset: old_asset.to_string(),
                    new_asset: new_asset.to_string(),
                });
            }
        }
        if previous.generation() == replacement.generation() && previous != replacement {
            return Err(UiBindingReloadPrepareError::GenerationCollision {
                tree_id,
                generation: previous.generation(),
            });
        }

        let retires_old_generation = !previous.generation().is_invalid()
            && previous.generation() != replacement.generation();
        Ok(Self {
            tree_id,
            old_generation: previous.generation(),
            published_generation: replacement.generation(),
            old_binding_count: previous.binding_count(),
            published_binding_count: replacement.binding_count(),
            retires_old_generation,
        })
    }

    pub(crate) fn publish(
        self,
        published: &UiCompiledBindingProgram,
        state_entries_migrated: usize,
        state_entries_reset: usize,
    ) -> UiBindingQuiescenceReceipt {
        let published_matches_prepared = published.is_well_formed()
            && published.generation() == self.published_generation
            && published.binding_count() == self.published_binding_count;
        let stale_handles_rejected =
            self.retires_old_generation && published.generation() != self.old_generation;

        UiBindingQuiescenceReceipt {
            tree_id: self.tree_id,
            old_generation: self.old_generation,
            published_generation: published.generation(),
            retired_binding_count: if self.retires_old_generation {
                self.old_binding_count
            } else {
                0
            },
            published_binding_count: published.binding_count(),
            state_entries_migrated,
            state_entries_reset,
            old_generation_retired: self.retires_old_generation,
            old_generation_quiescent: self.retires_old_generation
                && published_matches_prepared
                && stale_handles_rejected,
            stale_handles_rejected,
        }
    }
}
