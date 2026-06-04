use super::super::*;

#[test]
fn ray_tracing_convolution_status_is_visible_and_validated() {
    let sound = DefaultSoundManager::default();

    sound
        .set_ray_tracing_convolution_status(
            SoundRayTracingConvolutionStatus::WaitingForGeometryProvider,
        )
        .unwrap();
    assert_eq!(
        sound.mixer_snapshot().unwrap().ray_tracing,
        SoundRayTracingConvolutionStatus::WaitingForGeometryProvider
    );

    sound
        .set_ray_tracing_convolution_status(SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 2,
            rays_per_update: 64,
        })
        .unwrap();
    assert_eq!(
        sound.mixer_snapshot().unwrap().ray_tracing,
        SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 2,
            rays_per_update: 64,
        }
    );
    assert!(sound
        .set_ray_tracing_convolution_status(SoundRayTracingConvolutionStatus::RayTraced {
            cached_cells: 2,
            rays_per_update: 0,
        })
        .unwrap_err()
        .to_string()
        .contains("ray"));
}
