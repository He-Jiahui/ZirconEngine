use zircon_runtime::core::framework::sound::{SoundError, SoundMixBlock, SoundMixRenderManager};

use super::super::DefaultSoundManager;

impl SoundMixRenderManager for DefaultSoundManager {
    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError> {
        self.render_mix_impl(frames)
    }
}
