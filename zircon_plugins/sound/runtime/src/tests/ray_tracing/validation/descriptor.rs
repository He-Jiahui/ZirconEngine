use super::super::super::*;

pub(super) fn valid_ray_traced_descriptor() -> SoundRayTracedImpulseResponseDescriptor {
    SoundRayTracedImpulseResponseDescriptor {
        impulse_response: SoundImpulseResponseId::new(93),
        cell_key: "valid-cell".to_string(),
        source: None,
        listener: None,
        volume: None,
        occlusion_gain: None,
        sample_rate_hz: 48_000,
        channel_count: 1,
        rays_traced: 1,
        samples: vec![1.0],
    }
}
