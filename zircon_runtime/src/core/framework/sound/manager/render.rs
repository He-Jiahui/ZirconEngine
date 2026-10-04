use super::super::{SoundError, SoundMixBlock};

// TODO: [CR-FRAMEWORK-SOUND-0002] 确认手动渲染是否仍属公开契约；当前 DefaultSoundManager 始终返回 UnsupportedAdvancedFeature。
/// 离线拉取混音块的扩展点；当前插件实现未开放此能力。
pub trait SoundMixRenderManager {
    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError>;
}
