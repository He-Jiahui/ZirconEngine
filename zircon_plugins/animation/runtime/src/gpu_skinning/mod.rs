mod decision;
mod double_buffer;
mod error;
mod palette;

pub use decision::AnimationGpuSkinningDecision;
pub use double_buffer::SkinningPaletteDoubleBuffer;
pub use error::SkinningPaletteError;
pub use palette::{SkinningPalette, MAX_SKIN_JOINTS};
