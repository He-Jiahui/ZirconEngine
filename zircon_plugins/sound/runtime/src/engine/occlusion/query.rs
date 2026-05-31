use zircon_runtime::core::framework::sound::{SoundListenerId, SoundSourceId, SoundVolumeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SoundOcclusionQuery {
    pub(crate) source: SoundSourceId,
    pub(crate) listener: Option<SoundListenerId>,
    pub(crate) volume: Option<SoundVolumeId>,
}
