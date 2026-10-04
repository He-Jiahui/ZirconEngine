//! 文本命中、选择和 IME 几何共享同一份源字节到视觉簇映射，避免各入口重复解释双向文本。
mod boundary_bias;
mod cluster;
mod line;
mod visual_span;

pub use boundary_bias::UiTextVisualBoundaryBias;
pub use line::UiTextLineSourceMap;
pub use visual_span::UiTextVisualSpan;
