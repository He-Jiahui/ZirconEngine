use super::super::super::*;
use super::descriptor::valid_ray_traced_descriptor;

#[test]
fn ray_traced_impulse_response_rejects_empty_cell_key() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundRayTracedImpulseResponseDescriptor {
        cell_key: String::new(),
        ..valid_ray_traced_descriptor()
    };

    assert!(sound
        .submit_ray_traced_impulse_response(descriptor)
        .unwrap_err()
        .to_string()
        .contains("ray-traced impulse response"));
}
