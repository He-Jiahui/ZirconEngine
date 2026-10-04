//! 渲染历史以声源、监听器和配置三者为键，避免同一声源在不同听者或配置之间复用卷积尾音。
use zircon_runtime::core::framework::sound::{SoundListenerId, SoundSourceId};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SoundHrtfRenderStateKey {
    pub(super) source: SoundSourceId,
    pub(super) listener: SoundListenerId,
    pub(super) profile_id: String,
}

impl SoundHrtfRenderStateKey {
    pub(crate) fn new(
        source: SoundSourceId,
        listener: SoundListenerId,
        profile_id: impl Into<String>,
    ) -> Self {
        Self {
            source,
            listener,
            profile_id: profile_id.into(),
        }
    }
}
