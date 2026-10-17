//! 在一个窗口输入批次全部成功后收集结果与宿主元数据。
//! 汇总拥有自己的请求副本，不会清空逐项结果或持续追踪表面后续状态。

use zircon_runtime_interface::ui::dispatch::{
    UiDispatchEffect, UiDispatchHostRequest, UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;

/// 批次结果及构造时的宿主请求、重绘需求快照。
/// # 生命周期与约束
/// 管理器成功消费整个批次后创建；宿主接收并执行请求，逐项结果仍保留原请求。
/// 此类型可由公开字段自行构造，默认值为空且不主动同步表面状态。
/// # 参考
/// 见输入管理器的窗口批次入口及本模块的元数据收集辅助函数。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiInputDispatchOutcome {
    /// 按输入顺序保留的逐项结果。
    pub results: Vec<UiInputDispatchResult>,
    /// 按结果顺序克隆的请求，不去重，也不从逐项结果移出。
    pub host_requests: Vec<UiDispatchHostRequest>,
    /// 构造时表面已有重绘需求，或结果包含已应用重绘效果时为真。
    pub redraw_requested: bool,
}

/// 借用结果，按顺序构造请求副本并合并初始重绘需求。
/// 一旦已知需要重绘，就不再扫描后续结果的效果列表；结果及其请求保持原样。
fn collect_dispatch_metadata(
    results: &[UiInputDispatchResult],
    initial_redraw_requested: bool,
) -> (Vec<UiDispatchHostRequest>, bool) {
    let mut host_requests = Vec::new();
    let mut redraw_requested = initial_redraw_requested;
    let mut remaining_results = results.len();
    for result in results {
        if host_requests.is_empty() && !result.host_requests.is_empty() {
            // 首个非空请求处按剩余结果数给容量提示；任意批次仍可能需要继续扩容。
            host_requests.reserve(remaining_results.max(result.host_requests.len()));
        }
        host_requests.extend(result.host_requests.iter().cloned());
        if !redraw_requested
            && result
                .applied_effects
                .iter()
                .any(|applied| matches!(applied.effect, UiDispatchEffect::DirtyRedraw { .. }))
        {
            redraw_requested = true;
        }
        remaining_results = remaining_results.saturating_sub(1);
    }
    (host_requests, redraw_requested)
}

impl UiInputDispatchOutcome {
    /// 接收逐项结果所有权，并在此刻读取表面的重绘标志；以后两者不会自动联动。
    pub(crate) fn from_results(surface: &UiSurface, results: Vec<UiInputDispatchResult>) -> Self {
        let (host_requests, redraw_requested) =
            collect_dispatch_metadata(&results, surface.window_state.redraw_requested);

        Self {
            results,
            host_requests,
            redraw_requested,
        }
    }
}

#[cfg(test)]
#[path = "../outcome/tests/single_pass_metadata.rs"]
mod single_pass_metadata_tests;

#[cfg(test)]
#[path = "../outcome/tests/host_request_capacity.rs"]
mod host_request_capacity_tests;
