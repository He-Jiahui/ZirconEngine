use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundSourceDescriptor, SoundTrackId};

use super::super::super::routing::add_scaled;

pub(super) fn route_source_block(
    track_buffers: &mut HashMap<SoundTrackId, Vec<f32>>,
    output_track: SoundTrackId,
    descriptor: &SoundSourceDescriptor,
    source_buffer: &[f32],
    dry_source_buffer: &[f32],
) {
    if let Some(destination) = track_buffers.get_mut(&output_track) {
        add_scaled(destination, source_buffer, 1.0);
    }

    for send in &descriptor.sends {
        if let Some(destination) = track_buffers.get_mut(&send.target) {
            let source = if send.pre_spatial {
                dry_source_buffer
            } else {
                source_buffer
            };
            add_scaled(destination, source, send.gain);
        }
    }
}
