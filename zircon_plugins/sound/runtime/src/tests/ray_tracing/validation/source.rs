use super::super::super::*;
use super::descriptor::valid_ray_traced_descriptor;

#[test]
fn ray_traced_impulse_response_rejects_missing_source() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundRayTracedImpulseResponseDescriptor {
        impulse_response: SoundImpulseResponseId::new(94),
        cell_key: "missing-source".to_string(),
        source: Some(SoundSourceId::new(404)),
        ..valid_ray_traced_descriptor()
    };

    assert!(matches!(
        sound
            .submit_ray_traced_impulse_response(descriptor)
            .unwrap_err(),
        SoundError::UnknownSource { .. }
    ));
}
