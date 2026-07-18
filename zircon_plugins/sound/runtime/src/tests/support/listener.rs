use zircon_runtime::core::framework::sound::{
    SoundListenerDescriptor, SoundListenerId, SoundTrackId,
};

pub(in crate::tests) fn test_listener() -> SoundListenerDescriptor {
    SoundListenerDescriptor {
        id: SoundListenerId::new(1),
        active: true,
        position: [0.0, 0.0, 0.0],
        forward: [0.0, 0.0, 1.0],
        up: [0.0, 1.0, 0.0],
        left_ear_offset: [-0.08, 0.0, 0.0],
        right_ear_offset: [0.08, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        hrtf_profile: None,
        doppler_tracking: true,
        mixer_target: SoundTrackId::master(),
    }
}
