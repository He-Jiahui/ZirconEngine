use super::super::super::*;
use super::descriptor::valid_ray_traced_descriptor;

#[test]
fn ray_traced_impulse_response_rejects_invalid_occlusion_gain() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundRayTracedImpulseResponseDescriptor {
        impulse_response: SoundImpulseResponseId::new(97),
        cell_key: "bad-occlusion".to_string(),
        occlusion_gain: Some(1.5),
        ..valid_ray_traced_descriptor()
    };

    assert!(sound
        .submit_ray_traced_impulse_response(descriptor)
        .unwrap_err()
        .to_string()
        .contains("occlusion gain"));
}
