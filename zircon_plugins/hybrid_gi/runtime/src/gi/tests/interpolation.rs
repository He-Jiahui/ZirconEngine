use super::*;

#[test]
fn radiance_cache_midpoint_marks_all_eight_weighted_corners() {
    let clipmap = HybridGiRadianceCacheClipmapDescriptor {
        level: 0,
        anchor: Vec3::ZERO,
        anchor_cell: [0, 0, 0],
        cell_size: 1.0,
        resolution: RADIANCE_CACHE_CLIPMAP_RESOLUTION,
    };

    let corners = radiance_probe_interpolation_corners(Vec3::ZERO, &[clipmap]);

    assert_eq!(corners.len(), RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT);
    assert!(corners.iter().all(|corner| corner.weight_q16 > 0));
    assert!(corners
        .iter()
        .any(|corner| corner.demand.probe_coord == [23, 23, 23]));
    assert!(corners
        .iter()
        .any(|corner| corner.demand.probe_coord == [24, 24, 24]));
}
