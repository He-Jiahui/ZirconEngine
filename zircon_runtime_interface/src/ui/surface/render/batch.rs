//! 为诊断和工具构造排序、键合并及裁剪状态的批次计划；实际后端提交与实测计数由 Runtime RHI 独立维护。
#[path = "batch/clip.rs"]
mod clip;
#[path = "batch/key.rs"]
mod key;
#[path = "batch/plan.rs"]
mod plan;
#[path = "batch/range.rs"]
mod range;
#[path = "batch/split_reason.rs"]
mod split_reason;
#[path = "batch/stats.rs"]
mod stats;

#[cfg(test)]
use clip::UiClipStack;

#[cfg(test)]
#[path = "batch/tests/cases.rs"]
mod tests;

pub use key::{UiBatchKey, UiBatchPrimitive, UiBatchShader, UiOpacityClass};
pub use plan::{UiBatch, UiBatchPlan};
pub use range::UiBatchRange;
pub use split_reason::UiBatchSplitReason;
pub use stats::UiBatchStats;
