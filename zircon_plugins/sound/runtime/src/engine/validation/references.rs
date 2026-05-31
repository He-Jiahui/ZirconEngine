use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectKind, SoundError, SoundTrackId,
};

pub(super) fn validate_effect_references(
    track: SoundTrackId,
    effect: &SoundEffectDescriptor,
    track_ids: &HashSet<SoundTrackId>,
) -> Result<(), SoundError> {
    if let SoundEffectKind::Compressor(compressor) = &effect.kind {
        if let Some(sidechain) = compressor.sidechain {
            if !track_ids.contains(&sidechain.track) {
                return Err(SoundError::UnknownTrack {
                    track: sidechain.track,
                });
            }
            if !sidechain.pre_effects && sidechain.track == track {
                return Err(SoundError::InvalidMixerGraph(
                    "post-effect sidechain cannot read from the same track".to_string(),
                ));
            }
        }
    }
    Ok(())
}
