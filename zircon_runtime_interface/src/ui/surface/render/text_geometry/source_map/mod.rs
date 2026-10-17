//! 文本命中、选择和 IME 几何共享同一份源字节到视觉簇映射，避免各入口重复解释双向文本。
//!
//! 视觉字节偏移与命中使用的簇边界索引分别由各入口说明，不能互换。
//! 每个映射借用解析行并拥有簇表与惰性索引；这里只组织共享投影及其公开类型。
#[path = "../../source_map/boundary_bias.rs"]
mod boundary_bias;
#[path = "../../source_map/cluster.rs"]
mod cluster;
#[path = "line.rs"]
mod line;
#[path = "../../source_map/visual_span.rs"]
mod visual_span;

pub use boundary_bias::UiTextVisualBoundaryBias;
pub use line::UiTextLineSourceMap;
pub use visual_span::UiTextVisualSpan;
