use zircon_runtime::core::framework::sound::{
    SoundOutputDeviceDescriptor, SoundOutputDeviceStatus, SoundOutputLatencyStatus,
};

pub(super) fn latency_status_for_descriptor(
    descriptor: &SoundOutputDeviceDescriptor,
    queued_samples: Option<usize>,
    capacity_samples: Option<usize>,
) -> SoundOutputLatencyStatus {
    let estimated_latency_frames = descriptor
        .block_size_frames
        .saturating_mul(descriptor.latency_blocks);
    let estimated_latency_seconds = if descriptor.sample_rate_hz == 0 {
        0.0
    } else {
        estimated_latency_frames as f64 / descriptor.sample_rate_hz as f64
    };
    SoundOutputLatencyStatus {
        requested_latency_blocks: descriptor.latency_blocks,
        estimated_latency_frames,
        estimated_latency_seconds,
        queued_samples,
        capacity_samples,
    }
}

pub(super) fn push_status_diagnostic(status: &mut SoundOutputDeviceStatus, diagnostic: String) {
    if !status.diagnostics.iter().any(|entry| entry == &diagnostic) {
        status.diagnostics.push(diagnostic);
    }
}
