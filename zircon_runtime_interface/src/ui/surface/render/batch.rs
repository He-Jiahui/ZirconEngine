//! 为诊断和工具构造排序、键合并及裁剪状态的批次计划；实际后端提交与实测计数由 Runtime RHI 独立维护。
mod clip;
mod key;
mod plan;
mod range;
mod split_reason;
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
