use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::core::framework::render::RenderWorldSnapshotHandle;
use crate::graphics::scene::render_scene::{RenderSceneGeneration, RenderSceneReadView};

use super::{GpuSceneJournalConsumer, GpuSceneJournalConsumerError, GpuSceneJournalResidentWrite};

#[derive(Debug)]
pub(crate) struct GpuSceneJournalReprojectionPlan<'scene> {
    world: RenderWorldSnapshotHandle,
    generation: RenderSceneGeneration,
    slot_high_water: usize,
    resident_count: usize,
    direct_slot_validation_count: usize,
    resident_writes: Vec<GpuSceneJournalResidentWrite<'scene>>,
}

impl<'scene> GpuSceneJournalReprojectionPlan<'scene> {
    pub(crate) const fn world(&self) -> RenderWorldSnapshotHandle {
        self.world
    }

    pub(crate) const fn generation(&self) -> RenderSceneGeneration {
        self.generation
    }

    pub(crate) const fn slot_high_water(&self) -> usize {
        self.slot_high_water
    }

    pub(crate) const fn resident_count(&self) -> usize {
        self.resident_count
    }

    pub(crate) fn full_resident_write_count(&self) -> usize {
        self.resident_writes.len()
    }

    pub(crate) fn instance_transform_write_count(&self) -> usize {
        self.resident_writes.len()
    }

    pub(crate) fn local_bounds_write_count(&self) -> usize {
        self.resident_writes.len()
    }

    pub(crate) const fn direct_slot_validation_count(&self) -> usize {
        self.direct_slot_validation_count
    }

    pub(crate) const fn stable_key_lookup_count(&self) -> usize {
        0
    }

    pub(crate) fn resident_writes(&self) -> &[GpuSceneJournalResidentWrite<'scene>] {
        &self.resident_writes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuSceneJournalReprojectionPreflightError {
    WorldChanged {
        expected_world: RenderWorldSnapshotHandle,
        scene_world: RenderWorldSnapshotHandle,
    },
    GenerationChanged {
        applied_generation: RenderSceneGeneration,
        scene_generation: RenderSceneGeneration,
    },
    SlotHighWaterChanged {
        resident_slot_high_water: usize,
        scene_slot_high_water: usize,
    },
    ResidentCountChanged {
        resident_count: usize,
        scene_live_primitive_count: usize,
    },
    Resident(GpuSceneJournalConsumerError),
}

impl fmt::Display for GpuSceneJournalReprojectionPreflightError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldChanged {
                expected_world,
                scene_world,
            } => write!(
                formatter,
                "GPUScene reprojection expected world {} but received world {}",
                expected_world.raw(),
                scene_world.raw()
            ),
            Self::GenerationChanged {
                applied_generation,
                scene_generation,
            } => write!(
                formatter,
                "GPUScene reprojection residency is at generation {} but scene is at generation {}",
                applied_generation.get(),
                scene_generation.get()
            ),
            Self::SlotHighWaterChanged {
                resident_slot_high_water,
                scene_slot_high_water,
            } => write!(
                formatter,
                "GPUScene reprojection residency slot high-water {resident_slot_high_water} does not match scene slot high-water {scene_slot_high_water}"
            ),
            Self::ResidentCountChanged {
                resident_count,
                scene_live_primitive_count,
            } => write!(
                formatter,
                "GPUScene reprojection resident count {resident_count} does not match scene live primitive count {scene_live_primitive_count}"
            ),
            Self::Resident(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl Error for GpuSceneJournalReprojectionPreflightError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resident(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GpuSceneJournalReprojectionError<StageError> {
    Preflight(GpuSceneJournalReprojectionPreflightError),
    Staging(StageError),
}

impl<StageError> fmt::Display for GpuSceneJournalReprojectionError<StageError>
where
    StageError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preflight(error) => {
                write!(formatter, "GPUScene reprojection preflight failed: {error}")
            }
            Self::Staging(error) => {
                write!(formatter, "GPUScene reprojection staging failed: {error}")
            }
        }
    }
}

impl<StageError> Error for GpuSceneJournalReprojectionError<StageError>
where
    StageError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Preflight(error) => Some(error),
            Self::Staging(error) => Some(error),
        }
    }
}

impl GpuSceneJournalConsumer {
    /// Rebuilds device-owned rows from the current CPU scene without changing
    /// the accepted scene generation. Staging must be externally atomic.
    pub(crate) fn reproject_with_staging<'scene, StageOutput, StageError>(
        &self,
        scene: &'scene RenderSceneReadView<'_>,
        stage: impl FnOnce(&GpuSceneJournalReprojectionPlan<'scene>) -> Result<StageOutput, StageError>,
    ) -> Result<StageOutput, GpuSceneJournalReprojectionError<StageError>> {
        let plan = self
            .preflight_reprojection(scene)
            .map_err(GpuSceneJournalReprojectionError::Preflight)?;
        stage(&plan).map_err(GpuSceneJournalReprojectionError::Staging)
    }

    fn preflight_reprojection<'scene>(
        &self,
        scene: &'scene RenderSceneReadView<'_>,
    ) -> Result<GpuSceneJournalReprojectionPlan<'scene>, GpuSceneJournalReprojectionPreflightError>
    {
        if scene.world() != self.cursor.world() {
            return Err(GpuSceneJournalReprojectionPreflightError::WorldChanged {
                expected_world: self.cursor.world(),
                scene_world: scene.world(),
            });
        }
        if scene.generation() != self.cursor.applied_generation() {
            return Err(
                GpuSceneJournalReprojectionPreflightError::GenerationChanged {
                    applied_generation: self.cursor.applied_generation(),
                    scene_generation: scene.generation(),
                },
            );
        }

        let scene_storage = scene.storage_stats();
        let scene_slot_high_water = scene_storage.handle_slot_high_water();
        if scene_slot_high_water != self.slots.len() {
            return Err(
                GpuSceneJournalReprojectionPreflightError::SlotHighWaterChanged {
                    resident_slot_high_water: self.slots.len(),
                    scene_slot_high_water,
                },
            );
        }
        let scene_live_primitive_count = scene_storage.live_primitive_count();
        if scene_live_primitive_count != self.resident_count {
            return Err(
                GpuSceneJournalReprojectionPreflightError::ResidentCountChanged {
                    resident_count: self.resident_count,
                    scene_live_primitive_count,
                },
            );
        }

        let projected = BTreeMap::new();
        let mut resident_writes = Vec::with_capacity(scene_live_primitive_count);
        for (handle, primitive) in scene.iter() {
            self.require_live_slot(&projected, handle, primitive.stable_instance_key())
                .map_err(GpuSceneJournalReprojectionPreflightError::Resident)?;
            resident_writes.push(GpuSceneJournalResidentWrite::full(handle, primitive));
        }
        resident_writes.sort_unstable_by_key(|write| write.handle().slot());

        Ok(GpuSceneJournalReprojectionPlan {
            world: scene.world(),
            generation: scene.generation(),
            slot_high_water: scene_slot_high_water,
            resident_count: scene_live_primitive_count,
            direct_slot_validation_count: resident_writes.len(),
            resident_writes,
        })
    }
}
