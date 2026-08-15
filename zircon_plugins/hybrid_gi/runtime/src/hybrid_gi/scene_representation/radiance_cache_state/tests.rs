use super::*;

fn sample(
    radiance_rgb: [u8; 3],
    confidence_q8: u8,
    source: HybridGiRadianceCacheSource,
) -> HybridGiRadianceCacheSample {
    HybridGiRadianceCacheSample {
        radiance_rgb,
        confidence_q8,
        source,
    }
}

#[test]
fn radiance_cache_interpolates_corner_samples_instead_of_selecting_one() {
    let mut interpolation = HybridGiRadianceCacheInterpolationAccumulator::default();
    interpolation.add(
        sample(
            [10, 20, 30],
            u8::MAX,
            HybridGiRadianceCacheSource::SurfaceCache,
        ),
        3,
    );
    interpolation.add(
        sample(
            [70, 80, 90],
            u8::MAX,
            HybridGiRadianceCacheSource::VoxelFallback,
        ),
        1,
    );

    assert_eq!(
        interpolation.finish(),
        sample(
            [25, 35, 45],
            u8::MAX,
            HybridGiRadianceCacheSource::SurfaceCache
        )
    );
}

#[test]
fn radiance_cache_missing_corner_lowers_confidence_without_inventing_radiance() {
    let mut interpolation = HybridGiRadianceCacheInterpolationAccumulator::default();
    interpolation.add(
        sample(
            [100, 40, 20],
            u8::MAX,
            HybridGiRadianceCacheSource::SurfaceCache,
        ),
        1,
    );
    interpolation.add(HybridGiRadianceCacheSample::MISSING, 1);

    assert_eq!(
        interpolation.finish(),
        sample(
            [100, 40, 20],
            128,
            HybridGiRadianceCacheSource::SurfaceCache
        )
    );
}

#[test]
fn radiance_cache_reads_current_resident_corners_with_trilinear_weights() {
    let clipmap = HybridGiRadianceCacheClipmapDescriptor {
        level: 0,
        anchor: Vec3::ZERO,
        anchor_cell: [0, 0, 0],
        cell_size: 1.0,
        resolution: RADIANCE_CACHE_CLIPMAP_RESOLUTION,
    };
    let corners = radiance_probe_interpolation_corners(Vec3::ZERO, &[clipmap]);
    let mut state = HybridGiRadianceCacheState::default();
    state.clipmaps = vec![clipmap];
    state.generation = 1;
    state.input_revision = Some(HybridGiRadianceCacheInputRevision {
        surface_cache_revision: 0,
        voxel_scene_revision: 0,
        participation_epoch: 1,
    });
    state.update_report.mark_stable_generation(1);
    for (slot, corner) in corners.into_iter().enumerate() {
        let radiance = if corner.demand.probe_coord[0] == 23 {
            [10, 20, 30]
        } else {
            [70, 80, 90]
        };
        state.resident_probes.insert(
            corner.demand,
            HybridGiRadianceCacheResidentProbe {
                slot: slot as u32,
                last_used_frame: 1,
                last_traced_frame: 1,
                generation: 1,
                participation_epoch: 1,
                sample: sample(radiance, u8::MAX, HybridGiRadianceCacheSource::SurfaceCache),
            },
        );
    }

    assert_eq!(
        state.probe_current_sample(&HybridGiScreenProbeDescriptor::for_test(Vec3::ZERO)),
        Some(sample(
            [40, 50, 60],
            u8::MAX,
            HybridGiRadianceCacheSource::SurfaceCache
        ))
    );
    assert_eq!(
        state.update_stage(),
        HybridGiRadianceCacheUpdateStage::Complete
    );
    assert_eq!(state.update_counts(), (0, 0, 0, 0, 0));
}
