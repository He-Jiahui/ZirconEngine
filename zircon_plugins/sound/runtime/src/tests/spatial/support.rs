use super::super::*;

pub(super) fn test_hrtf_profile(profile_id: &str) -> SoundHrtfProfileDescriptor {
    SoundHrtfProfileDescriptor {
        profile_id: profile_id.to_string(),
        display_name: "Test HRTF".to_string(),
        sample_rate_hz: 48_000,
        left_kernel: vec![0.0, 0.5],
        right_kernel: vec![1.0],
        notes: vec!["unit-test profile".to_string()],
    }
}

pub(super) fn long_tail_hrtf_profile(profile_id: &str) -> SoundHrtfProfileDescriptor {
    SoundHrtfProfileDescriptor {
        profile_id: profile_id.to_string(),
        display_name: "Long Tail Test HRTF".to_string(),
        sample_rate_hz: 48_000,
        left_kernel: vec![0.0, 0.0, 0.5],
        right_kernel: vec![1.0],
        notes: vec!["unit-test long-tail profile".to_string()],
    }
}
