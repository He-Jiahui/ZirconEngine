use zircon_runtime::core::framework::sound::{SoundEffectId, SoundTrackId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SoundEffectStateKey {
    pub(crate) track: SoundTrackId,
    pub(crate) effect: SoundEffectId,
}

impl SoundEffectStateKey {
    pub(crate) fn new(track: SoundTrackId, effect: SoundEffectId) -> Self {
        Self { track, effect }
    }
}
