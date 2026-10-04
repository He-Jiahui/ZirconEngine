use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};

use super::RenderSceneComponentProjector;

/// Owns one persistent CPU render scene for each source-world lineage seen by a renderer.
#[derive(Default)]
pub(crate) struct RenderSceneRegistry {
    projectors: HashMap<u64, RenderSceneComponentProjector>,
}

impl RenderSceneRegistry {
    pub(crate) fn projector_for_frame(
        &mut self,
        frame: &RenderFrameExtract,
    ) -> &mut RenderSceneComponentProjector {
        self.projector_for_world(frame.world)
    }

    fn projector_for_world(
        &mut self,
        world: RenderWorldSnapshotHandle,
    ) -> &mut RenderSceneComponentProjector {
        match self.projectors.entry(world.raw()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(RenderSceneComponentProjector::new(world)),
        }
    }

    /// Removes a world only after its complete resource-release frontier is accepted.
    pub(crate) fn release_world_with_staging<StageOutput, StageError>(
        &mut self,
        world: RenderWorldSnapshotHandle,
        stage: impl FnOnce(
            &[super::RenderSceneResourceReferenceDelta],
        ) -> Result<StageOutput, StageError>,
    ) -> Result<Option<StageOutput>, StageError> {
        let Some(projector) = self.projectors.get(&world.raw()) else {
            return Ok(None);
        };
        let release_deltas = projector.resource_release_deltas();
        let staged = stage(&release_deltas)?;
        self.projectors.remove(&world.raw());
        Ok(Some(staged))
    }
}

#[cfg(test)]
#[path = "tests/registry.rs"]
mod tests;
