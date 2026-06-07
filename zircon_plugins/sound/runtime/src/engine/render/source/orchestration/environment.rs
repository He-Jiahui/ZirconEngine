use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundSourceDescriptor, SoundSourceId, SoundTrackId};

use crate::engine::source_environment::{apply_source_environment, hrtf_tail_pending_for_source};
use crate::engine::{SoundHrtfRenderState, SoundHrtfRenderStateKey};
use crate::SoundConfig;

use super::snapshot::SourceRenderSnapshot;

pub(super) fn apply_source_environment_block(
    source_buffer: &mut [f32],
    channels: usize,
    config: &SoundConfig,
    source_id: SoundSourceId,
    descriptor: &SoundSourceDescriptor,
    output_track: SoundTrackId,
    snapshot: &SourceRenderSnapshot,
    hrtf_states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
) {
    apply_source_environment(
        source_buffer,
        channels,
        config.sample_rate_hz,
        source_id,
        descriptor,
        snapshot.active_listener(output_track),
        config.default_spatial_scale,
        &snapshot.volumes,
        &snapshot.impulse_responses,
        &snapshot.ray_traced_impulse_responses,
        &snapshot.hrtf_profiles,
        hrtf_states,
        &snapshot.ray_tracing,
    );
}

pub(super) fn source_has_pending_tail(
    source_id: SoundSourceId,
    output_track: SoundTrackId,
    snapshot: &SourceRenderSnapshot,
    hrtf_states: &HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
) -> bool {
    hrtf_tail_pending_for_source(
        hrtf_states,
        source_id,
        snapshot.active_listener(output_track),
        &snapshot.hrtf_profiles,
    )
}
