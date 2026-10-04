//! 插件公开的蒙皮策略和调色板类型；渲染器的实际制品与绑定路径需在调用端确认。
// TODO: [CR-PLUGIN-ANIMATION-0005] 确认这些公开 GPU 决策与双缓冲 API 的运行时所有者；目前全第一方搜索仅见契约测试，渲染器另建调色板；下一步核对渲染集成规划。
mod decision;
mod double_buffer;
mod error;
mod palette;

pub use decision::AnimationGpuSkinningDecision;
pub use double_buffer::SkinningPaletteDoubleBuffer;
pub use error::SkinningPaletteError;
pub use palette::{SkinningPalette, MAX_SKIN_JOINTS};
