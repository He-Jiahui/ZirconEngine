use super::super::super::*;
use super::descriptor::valid_ray_traced_descriptor;

#[test]
fn ray_traced_impulse_response_rejects_zero_rays() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundRayTracedImpulseResponseDescriptor {
        impulse_response: SoundImpulseResponseId::new(95),
        cell_key: "bad-rays".to_string(),
        rays_traced: 0,
        ..valid_ray_traced_descriptor()
    };

    assert!(sound
        .submit_ray_traced_impulse_response(descriptor)
        .unwrap_err()
        .to_string()
        .contains("ray-traced impulse response"));
}
