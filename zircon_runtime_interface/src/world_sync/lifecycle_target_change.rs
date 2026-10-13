//! A lifecycle callback may retire the logical World or its Schedule.
//!
//! Lifecycle callback 可能替换逻辑 World 或其 Schedule；该类型把取消原因送回调用方。
//!
//! 它只标识被替换的目标，不包含目标句柄，也不授权调用方继续访问旧目标。反射错误和
//! 场景提交错误使用同一原因类型，让上层可在回滚或继续批处理前停止旧目标工作。
//!
//! # 参考
//!
//! ReflectError::LifecycleTargetReplaced 携带该原因；动态场景和场景事务错误向上暴露
//! 相同分类，相关取消测试位于 Runtime 动态场景提交测试中。
use std::fmt;

use serde::{Deserialize, Serialize};

/// Transportable cancellation cause; it carries no World or Schedule authority.
///
/// 生命周期回调替换其正在操作的目标后，Runtime 用此值报告原操作应取消。序列化标签使用
/// snake_case；Display 输出与变体同名的诊断文本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleTargetChange {
    /// 回调替换了逻辑 World。
    World,
    /// 回调替换了该 World 的 Schedule。
    Schedule,
}

/// 将取消原因格式化为对应逻辑目标的名称。
impl fmt::Display for LifecycleTargetChange {
    /// 输出 World 或 Schedule 的目标名，嵌入取消错误的诊断文本。
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 保持诊断文本与目标枚举名称一致，供携带该值的错误实现 Display。
        formatter.write_str(match self {
            Self::World => "World",
            Self::Schedule => "Schedule",
        })
    }
}
