use super::*;

#[test]
fn screen_route_keeps_bounded_global_sdf_and_voxel_fallbacks() {
    let route = HybridGiTraceCapabilityGraph.select(
        HybridGiTraceRequest {
            domain: HybridGiTraceDomain::Screen,
            prefer_hardware_ray_tracing: false,
        },
        HybridGiTraceCapabilities {
            surface_cache_hzb: true,
            global_sdf: true,
            voxel_clipmap: true,
            probe_lineage_lighting: true,
            ..Default::default()
        },
    );

    assert!(route.allows(HybridGiIntersectionBackend::SurfaceCacheHzb));
    assert!(route.allows(HybridGiIntersectionBackend::GlobalSdf));
    assert!(route.allows(HybridGiIntersectionBackend::VoxelClipmap));
    assert!(!route.allows(HybridGiIntersectionBackend::HardwareRayTracing));
    assert_eq!(
        route.lighting_source_for(HybridGiIntersectionBackend::SurfaceCacheHzb),
        HybridGiLightingSource::SurfaceCache
    );
    assert_eq!(
        route.lighting_source_for(HybridGiIntersectionBackend::GlobalSdf),
        HybridGiLightingSource::ProbeLineage
    );
    assert_eq!(
        route.lighting_source_for(HybridGiIntersectionBackend::VoxelClipmap),
        HybridGiLightingSource::VoxelRadiance
    );
}

#[test]
fn optional_hardware_rt_absence_records_a_typed_forward_fallback() {
    let route = HybridGiTraceCapabilityGraph.select(
        HybridGiTraceRequest {
            domain: HybridGiTraceDomain::WorldProbe,
            prefer_hardware_ray_tracing: true,
        },
        HybridGiTraceCapabilities {
            global_sdf: true,
            probe_lineage_lighting: true,
            ..Default::default()
        },
    );
    let result = route.record_hit(
        HybridGiIntersectionBackend::GlobalSdf,
        3.5,
        0.8,
        HybridGiTraceCostCounters {
            page_tests: 4,
            sdf_steps: 6,
            ..Default::default()
        },
    );

    assert_eq!(result.source, HybridGiTraceSource::GlobalSdf);
    assert_eq!(result.lighting_source, HybridGiLightingSource::ProbeLineage);
    assert_eq!(result.distance, 3.5);
    assert_eq!(result.confidence, 0.8);
    assert_eq!(
        result.fallback_reason,
        Some(HybridGiTraceFallbackReason::HardwareRayTracingUnavailable)
    );
    assert_eq!(result.cost.page_tests, 4);
    assert_eq!(result.cost.sdf_steps, 6);
}

#[test]
fn unavailable_intersection_records_a_complete_miss_result() {
    let route = HybridGiTraceCapabilityGraph.select(
        HybridGiTraceRequest {
            domain: HybridGiTraceDomain::WorldProbe,
            prefer_hardware_ray_tracing: false,
        },
        HybridGiTraceCapabilities::default(),
    );
    let result = route.record_miss(HybridGiTraceCostCounters::default());

    assert_eq!(result.source, HybridGiTraceSource::Miss);
    assert!(result.distance.is_infinite());
    assert_eq!(result.confidence, 0.0);
    assert!(result.fallback_reason.is_some());
}

#[test]
fn unadmitted_backend_hit_uses_the_route_miss_lighting_source() {
    let route = HybridGiTraceCapabilityGraph.select(
        HybridGiTraceRequest {
            domain: HybridGiTraceDomain::Screen,
            prefer_hardware_ray_tracing: false,
        },
        HybridGiTraceCapabilities {
            surface_cache_hzb: true,
            ..Default::default()
        },
    );
    let result = route.record_hit(
        HybridGiIntersectionBackend::GlobalSdf,
        2.0,
        1.0,
        HybridGiTraceCostCounters::default(),
    );

    assert_eq!(result.intersection_backend, None);
    assert_eq!(result.source, HybridGiTraceSource::Miss);
    assert_eq!(result.lighting_source, HybridGiLightingSource::SurfaceCache);
    assert!(result.distance.is_infinite());
    assert_eq!(result.confidence, 0.0);
    assert_eq!(
        result.fallback_reason,
        Some(HybridGiTraceFallbackReason::IntersectionMiss)
    );
}
