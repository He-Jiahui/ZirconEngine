use zircon_runtime::core::framework::sound::{SoundListenerDescriptor, SoundTrackId};

pub(crate) fn active_listener_for(
    listeners: &[SoundListenerDescriptor],
    output_track: SoundTrackId,
) -> Option<&SoundListenerDescriptor> {
    listeners
        .iter()
        .filter(|listener| listener.active)
        .min_by_key(|listener| {
            let rank = if listener.mixer_target == output_track {
                0_u8
            } else if listener.mixer_target == SoundTrackId::master() {
                1
            } else {
                2
            };
            (rank, listener.id.raw())
        })
}
