use super::super::super::super::*;

pub(super) fn sidechain_compressor(track: SoundTrackId, pre_effects: bool) -> SoundEffectKind {
    SoundEffectKind::Compressor(SoundCompressorEffect {
        threshold_db: -12.0,
        ratio: 2.0,
        attack_ms: 1.0,
        release_ms: 10.0,
        makeup_gain_db: 0.0,
        sidechain: Some(SoundSidechainInput { track, pre_effects }),
    })
}
