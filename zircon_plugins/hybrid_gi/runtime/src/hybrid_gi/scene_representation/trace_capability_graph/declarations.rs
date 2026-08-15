#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) enum HybridGiTraceDomain {
    Screen,
    WorldProbe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) enum HybridGiIntersectionBackend {
    SurfaceCacheHzb,
    GlobalSdf,
    VoxelClipmap,
    HardwareRayTracing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) enum HybridGiLightingSource {
    SurfaceCache,
    ProbeLineage,
    VoxelRadiance,
    NeutralAmbient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) enum HybridGiTraceSource {
    SurfaceCache,
    GlobalSdf,
    VoxelClipmap,
    HardwareRayTracing,
    Miss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) enum HybridGiTraceFallbackReason {
    ScreenDataUnavailable,
    HardwareRayTracingUnavailable,
    GlobalSdfUnavailable,
    IntersectionMiss,
    LightingUnavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiTraceCostCounters {
    pub(in crate::hybrid_gi) texture_samples: u32,
    pub(in crate::hybrid_gi) page_tests: u32,
    pub(in crate::hybrid_gi) sdf_steps: u32,
    pub(in crate::hybrid_gi) voxel_candidates: u32,
    pub(in crate::hybrid_gi) hardware_rays: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiTraceCapabilities {
    pub(in crate::hybrid_gi) surface_cache_hzb: bool,
    pub(in crate::hybrid_gi) global_sdf: bool,
    pub(in crate::hybrid_gi) voxel_clipmap: bool,
    pub(in crate::hybrid_gi) hardware_ray_tracing: bool,
    pub(in crate::hybrid_gi) probe_lineage_lighting: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiTraceRequest {
    pub(in crate::hybrid_gi) domain: HybridGiTraceDomain,
    pub(in crate::hybrid_gi) prefer_hardware_ray_tracing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiTraceResult {
    pub(in crate::hybrid_gi) domain: HybridGiTraceDomain,
    pub(in crate::hybrid_gi) intersection_backend: Option<HybridGiIntersectionBackend>,
    pub(in crate::hybrid_gi) lighting_source: HybridGiLightingSource,
    pub(in crate::hybrid_gi) source: HybridGiTraceSource,
    pub(in crate::hybrid_gi) distance: f32,
    pub(in crate::hybrid_gi) confidence: f32,
    pub(in crate::hybrid_gi) fallback_reason: Option<HybridGiTraceFallbackReason>,
    pub(in crate::hybrid_gi) cost: HybridGiTraceCostCounters,
}
