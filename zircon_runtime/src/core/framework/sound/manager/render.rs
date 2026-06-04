use super::super::{SoundError, SoundMixBlock};

pub trait SoundMixRenderManager {
    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError>;
}
