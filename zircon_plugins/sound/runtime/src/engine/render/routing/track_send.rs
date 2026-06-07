use zircon_runtime::core::framework::sound::SoundTrackSend;

pub(in crate::engine::render) fn track_send_source_buffer<'a>(
    send: &SoundTrackSend,
    raw_buffer: &'a [f32],
    processed_buffer: &'a [f32],
) -> &'a [f32] {
    if send.pre_effects {
        raw_buffer
    } else {
        processed_buffer
    }
}
