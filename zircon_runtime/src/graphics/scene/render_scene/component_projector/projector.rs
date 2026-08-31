use crate::core::framework::render::{
    RenderComponentChangeArtifact, RenderComponentProjectionMode, RenderComponentSourceWorldId,
    RenderFrameExtract, RenderWorldSnapshotHandle,
};

use super::super::{RenderScene, RenderSceneChangeJournal, RenderSceneReadView};
use super::projection::build_delta;
use super::{RenderSceneComponentProjectionError, RenderSceneGeometryResolver};

pub(crate) struct RenderSceneComponentProjector {
    bound_world: Option<RenderComponentSourceWorldId>,
    applied_journal_generation: u64,
    scene: RenderScene,
}

impl RenderSceneComponentProjector {
    pub(crate) fn new(world: RenderWorldSnapshotHandle) -> Self {
        Self {
            bound_world: None,
            applied_journal_generation: 0,
            scene: RenderScene::new(world),
        }
    }

    pub(crate) fn read(&self) -> RenderSceneReadView<'_> {
        self.scene.read()
    }

    pub(crate) fn project_frame(
        &mut self,
        frame: &RenderFrameExtract,
        resolver: &mut impl RenderSceneGeometryResolver,
    ) -> Result<Option<RenderSceneChangeJournal>, RenderSceneComponentProjectionError> {
        let expected_world = self.scene.read().world();
        if frame.world != expected_world {
            return Err(RenderSceneComponentProjectionError::FrameWorldMismatch {
                expected: expected_world,
                incoming: frame.world,
            });
        }
        let Some(artifact) = frame.geometry.scene_changes.as_deref() else {
            return Ok(None);
        };
        self.project(artifact, resolver)
    }

    pub(crate) fn project(
        &mut self,
        artifact: &RenderComponentChangeArtifact,
        resolver: &mut impl RenderSceneGeometryResolver,
    ) -> Result<Option<RenderSceneChangeJournal>, RenderSceneComponentProjectionError> {
        if self.bound_world == Some(artifact.world())
            && self.applied_journal_generation == artifact.journal_generation()
        {
            return Ok(None);
        }
        self.validate_artifact(artifact)?;
        let delta = build_delta(&self.scene, artifact, resolver)?;
        let journal = self.scene.apply_delta(delta)?;
        self.bound_world = Some(artifact.world());
        self.applied_journal_generation = artifact.journal_generation();
        Ok(Some(journal))
    }

    fn validate_artifact(
        &self,
        artifact: &RenderComponentChangeArtifact,
    ) -> Result<(), RenderSceneComponentProjectionError> {
        let Some(bound_world) = self.bound_world else {
            return if matches!(artifact.mode(), RenderComponentProjectionMode::Full(_)) {
                Ok(())
            } else {
                Err(RenderSceneComponentProjectionError::IncrementalWithoutBaseline)
            };
        };
        if bound_world != artifact.world() {
            return Err(RenderSceneComponentProjectionError::WorldMismatch {
                expected: bound_world,
                incoming: artifact.world(),
            });
        }
        if artifact.journal_generation() < self.applied_journal_generation {
            return Err(RenderSceneComponentProjectionError::StaleArtifact {
                applied_generation: self.applied_journal_generation,
                incoming_generation: artifact.journal_generation(),
            });
        }
        if artifact.journal_generation() != self.applied_journal_generation.saturating_add(1)
            && !matches!(artifact.mode(), RenderComponentProjectionMode::Full(_))
        {
            return Err(RenderSceneComponentProjectionError::JournalDiscontinuity {
                applied_generation: self.applied_journal_generation,
                incoming_generation: artifact.journal_generation(),
            });
        }
        Ok(())
    }
}
