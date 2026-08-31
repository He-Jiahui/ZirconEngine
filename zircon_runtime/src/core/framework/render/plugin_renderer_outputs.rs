mod hybrid_gi;
mod particles;
mod virtual_geometry;

pub use self::hybrid_gi::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiGlobalSdfStats,
    RenderHybridGiProbeTraceDiagnosticRecord, RenderHybridGiRadianceCacheGpuStage,
    RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
    RenderHybridGiScenePrepareSample, RenderHybridGiSurfaceCachePageRecord,
    RenderHybridGiTraceCostCounters, RenderHybridGiTraceFallbackReason,
    RenderHybridGiTraceIntersectionSource, RenderHybridGiTraceLightingSource,
    RenderHybridGiTraceTileRecord, RenderHybridGiVoxelCellDominantNodeRecord,
    RenderHybridGiVoxelCellRecord, RenderHybridGiVoxelCellSampleRecord,
    RenderHybridGiVoxelClipmapRecord, RenderHybridGiVoxelOccupancyMaskRecord,
    RENDER_HYBRID_GI_PROBE_TRACE_DIAGNOSTIC_WORD_COUNT,
    RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
};
pub use self::particles::RenderParticleGpuReadbackOutputs;
pub use self::virtual_geometry::{
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryPageAssignmentRecord,
    RenderVirtualGeometryPageReplacementRecord, RenderVirtualGeometryReadbackOutputs,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPluginRendererOutputs {
    pub virtual_geometry: RenderVirtualGeometryReadbackOutputs,
    pub hybrid_gi: RenderHybridGiReadbackOutputs,
    pub particles: RenderParticleGpuReadbackOutputs,
}

impl RenderPluginRendererOutputs {
    pub fn is_empty(&self) -> bool {
        self.virtual_geometry.is_empty() && self.hybrid_gi.is_empty() && self.particles.is_empty()
    }
}

#[cfg(test)]
mod tests;
