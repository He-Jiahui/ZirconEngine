use std::sync::Arc;

use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::PipelineKey;

use super::{MeshDraw, MeshDrawGeometrySource};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshDrawQueuePhase {
    Opaque,
    AlphaMask,
    Transparent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshDrawQueueProfile {
    phase: MeshDrawQueuePhase,
    geometry_source: MeshDrawGeometrySource,
    mobility: Mobility,
    uses_indirect_draw: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshDrawBatchKey {
    geometry_source: MeshDrawGeometrySource,
    mesh: usize,
    texture: usize,
    material_uniform: usize,
    pipeline_key: PipelineKey,
    first_index: u32,
    draw_index_count: u32,
}

impl MeshDrawQueuePhase {
    pub(crate) fn from_pipeline_flags(is_transparent: bool, is_alpha_mask: bool) -> Self {
        if is_transparent {
            Self::Transparent
        } else if is_alpha_mask {
            Self::AlphaMask
        } else {
            Self::Opaque
        }
    }
}

impl MeshDrawQueueProfile {
    pub(crate) fn new(
        phase: MeshDrawQueuePhase,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        uses_indirect_draw: bool,
    ) -> Self {
        Self {
            phase,
            geometry_source,
            mobility,
            uses_indirect_draw,
        }
    }

    pub(crate) fn phase(self) -> MeshDrawQueuePhase {
        self.phase
    }

    pub(crate) fn geometry_source(self) -> MeshDrawGeometrySource {
        self.geometry_source
    }

    pub(crate) fn early_z_eligible(self) -> bool {
        !matches!(self.phase, MeshDrawQueuePhase::Transparent)
    }

    pub(crate) fn static_batch_eligible(self) -> bool {
        self.direct_prepared_non_transparent() && self.mobility == Mobility::Static
    }

    pub(crate) fn dynamic_batch_eligible(self) -> bool {
        self.direct_prepared_non_transparent() && self.mobility == Mobility::Dynamic
    }

    pub(crate) fn gpu_instancing_eligible(self) -> bool {
        self.direct_prepared_non_transparent()
    }

    pub(crate) fn uses_indirect_draw(self) -> bool {
        self.uses_indirect_draw
    }

    fn direct_prepared_non_transparent(self) -> bool {
        self.geometry_source == MeshDrawGeometrySource::Prepared
            && !self.uses_indirect_draw
            && self.early_z_eligible()
    }
}

impl MeshDraw {
    pub(crate) fn queue_profile(&self) -> MeshDrawQueueProfile {
        MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::from_pipeline_flags(self.is_transparent(), self.is_alpha_mask()),
            self.geometry_source,
            self.mobility,
            self.uses_indirect_draw(),
        )
    }

    pub(crate) fn batch_key(&self) -> MeshDrawBatchKey {
        MeshDrawBatchKey {
            geometry_source: self.geometry_source,
            mesh: Arc::as_ptr(&self.mesh) as usize,
            texture: Arc::as_ptr(&self.texture) as usize,
            material_uniform: Arc::as_ptr(&self.material_uniform) as usize,
            pipeline_key: self.pipeline_key.clone(),
            first_index: self.first_index,
            draw_index_count: self.draw_index_count,
        }
    }
}
