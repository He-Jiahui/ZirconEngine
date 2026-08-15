use super::declarations::{
    HybridGiIntersectionBackend, HybridGiLightingSource, HybridGiTraceCapabilities,
    HybridGiTraceCostCounters, HybridGiTraceDomain, HybridGiTraceFallbackReason,
    HybridGiTraceRequest, HybridGiTraceResult, HybridGiTraceSource,
};

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::hybrid_gi) struct HybridGiTraceCapabilityGraph;

impl HybridGiTraceCapabilityGraph {
    pub(in crate::hybrid_gi) fn select(
        self,
        request: HybridGiTraceRequest,
        capabilities: HybridGiTraceCapabilities,
    ) -> HybridGiTraceRoute {
        let mut backends = [None; 4];
        let mut count = 0;
        let mut fallback_reason = None;

        if request.domain == HybridGiTraceDomain::Screen {
            push_backend(
                &mut backends,
                &mut count,
                capabilities.surface_cache_hzb,
                HybridGiIntersectionBackend::SurfaceCacheHzb,
            );
            if !capabilities.surface_cache_hzb {
                fallback_reason = Some(HybridGiTraceFallbackReason::ScreenDataUnavailable);
            }
        }
        if request.prefer_hardware_ray_tracing {
            push_backend(
                &mut backends,
                &mut count,
                capabilities.hardware_ray_tracing,
                HybridGiIntersectionBackend::HardwareRayTracing,
            );
            if !capabilities.hardware_ray_tracing && fallback_reason.is_none() {
                fallback_reason = Some(HybridGiTraceFallbackReason::HardwareRayTracingUnavailable);
            }
        }
        push_backend(
            &mut backends,
            &mut count,
            capabilities.global_sdf,
            HybridGiIntersectionBackend::GlobalSdf,
        );
        push_backend(
            &mut backends,
            &mut count,
            capabilities.voxel_clipmap,
            HybridGiIntersectionBackend::VoxelClipmap,
        );
        if count == 0 && fallback_reason.is_none() {
            fallback_reason = Some(HybridGiTraceFallbackReason::GlobalSdfUnavailable);
        }

        HybridGiTraceRoute {
            domain: request.domain,
            backends,
            backend_count: count,
            miss_lighting_source: if capabilities.surface_cache_hzb {
                HybridGiLightingSource::SurfaceCache
            } else if capabilities.probe_lineage_lighting {
                HybridGiLightingSource::ProbeLineage
            } else if capabilities.voxel_clipmap {
                HybridGiLightingSource::VoxelRadiance
            } else {
                HybridGiLightingSource::NeutralAmbient
            },
            probe_lineage_lighting: capabilities.probe_lineage_lighting,
            fallback_reason,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiTraceRoute {
    domain: HybridGiTraceDomain,
    backends: [Option<HybridGiIntersectionBackend>; 4],
    backend_count: usize,
    miss_lighting_source: HybridGiLightingSource,
    probe_lineage_lighting: bool,
    fallback_reason: Option<HybridGiTraceFallbackReason>,
}

impl HybridGiTraceRoute {
    pub(in crate::hybrid_gi) fn allows(self, backend: HybridGiIntersectionBackend) -> bool {
        self.backends[..self.backend_count].contains(&Some(backend))
    }

    pub(in crate::hybrid_gi) fn lighting_source_for(
        self,
        backend: HybridGiIntersectionBackend,
    ) -> HybridGiLightingSource {
        match backend {
            HybridGiIntersectionBackend::SurfaceCacheHzb => HybridGiLightingSource::SurfaceCache,
            HybridGiIntersectionBackend::GlobalSdf
            | HybridGiIntersectionBackend::HardwareRayTracing => {
                if self.probe_lineage_lighting {
                    HybridGiLightingSource::ProbeLineage
                } else {
                    HybridGiLightingSource::NeutralAmbient
                }
            }
            HybridGiIntersectionBackend::VoxelClipmap => HybridGiLightingSource::VoxelRadiance,
        }
    }

    pub(in crate::hybrid_gi) fn fallback_reason(self) -> Option<HybridGiTraceFallbackReason> {
        self.fallback_reason
    }

    pub(in crate::hybrid_gi) fn record_hit(
        self,
        backend: HybridGiIntersectionBackend,
        distance: f32,
        confidence: f32,
        cost: HybridGiTraceCostCounters,
    ) -> HybridGiTraceResult {
        let admitted = self.allows(backend);
        HybridGiTraceResult {
            domain: self.domain,
            intersection_backend: admitted.then_some(backend),
            lighting_source: if admitted {
                self.lighting_source_for(backend)
            } else {
                self.miss_lighting_source
            },
            source: if admitted {
                source_for_backend(backend)
            } else {
                HybridGiTraceSource::Miss
            },
            distance: if admitted && distance.is_finite() {
                distance.max(0.0)
            } else {
                f32::INFINITY
            },
            confidence: if admitted && confidence.is_finite() {
                confidence.clamp(0.0, 1.0)
            } else {
                0.0
            },
            fallback_reason: if admitted {
                self.fallback_reason
            } else {
                Some(HybridGiTraceFallbackReason::IntersectionMiss)
            },
            cost,
        }
    }

    pub(in crate::hybrid_gi) fn record_miss(
        self,
        cost: HybridGiTraceCostCounters,
    ) -> HybridGiTraceResult {
        HybridGiTraceResult {
            domain: self.domain,
            intersection_backend: None,
            lighting_source: self.miss_lighting_source,
            source: HybridGiTraceSource::Miss,
            distance: f32::INFINITY,
            confidence: 0.0,
            fallback_reason: Some(
                self.fallback_reason
                    .unwrap_or(HybridGiTraceFallbackReason::IntersectionMiss),
            ),
            cost,
        }
    }
}

fn push_backend(
    backends: &mut [Option<HybridGiIntersectionBackend>; 4],
    count: &mut usize,
    available: bool,
    backend: HybridGiIntersectionBackend,
) {
    if available && *count < backends.len() {
        backends[*count] = Some(backend);
        *count += 1;
    }
}

fn source_for_backend(backend: HybridGiIntersectionBackend) -> HybridGiTraceSource {
    match backend {
        HybridGiIntersectionBackend::SurfaceCacheHzb => HybridGiTraceSource::SurfaceCache,
        HybridGiIntersectionBackend::GlobalSdf => HybridGiTraceSource::GlobalSdf,
        HybridGiIntersectionBackend::VoxelClipmap => HybridGiTraceSource::VoxelClipmap,
        HybridGiIntersectionBackend::HardwareRayTracing => HybridGiTraceSource::HardwareRayTracing,
    }
}
