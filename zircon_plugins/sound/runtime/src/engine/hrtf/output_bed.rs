pub(crate) fn clear_non_binaural_output_channels(buffer: &mut [f32], channels: usize) {
    if channels <= 2 {
        return;
    }

    for frame in buffer.chunks_exact_mut(channels) {
        // HRTF output is a binaural front pair; surround and LFE beds must not
        // keep the dry source after spatialization has taken ownership.
        for sample in &mut frame[2..] {
            *sample = 0.0;
        }
    }
}
