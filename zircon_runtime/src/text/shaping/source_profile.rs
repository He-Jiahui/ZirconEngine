//! 成形结果原文的共享入口与固定的请求级观测。

use std::sync::Arc;

use crate::text::BackendShapeRequest;

/// Materializes the current exact request source at the shaped-artifact boundary. Profiling is
/// intentionally request-level: no source label, glyph event, or loop-local observer is emitted.
///
/// 为正在组装的成形结果取得原文所有权，并提供本次调用的逻辑观测。
///
/// # 参数
/// - `request`：包含原文及可选所有者的借用，本函数仅在调用期间使用这些借用。
///
/// # 返回
/// 当前原文的 `Arc<str>`，其所有权不依赖请求借用继续存活。
/// [BackendShapeRequest::shared_source_text] 会重新比较 owner 内容；符合时共享，否则复制原文。
///
/// # 观测边界
/// 四个值根据本次调用、owner 存在标志与 UTF-8 内容长度构造，是逻辑观测，
/// 不是实际分配器计量或内存预算；字节值不含 Arc 控制块、容量和分配器开销。
/// `profiling` 记录器仍受 `capture_active` 控制，`profiling-tracy` 另有独立事件通道。
/// 观测点不保证结果最终发布成功，也不保证每请求或每帧只调用一次。
/// 未启用这些 feature 时，原文所有权的获取仍会执行。
pub(in crate::text::shaping) fn materialize_source_text(
    request: BackendShapeRequest<'_>,
) -> Arc<str> {
    let reuses_owner = request.has_exact_source_owner();
    let source = request.shared_source_text();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    {
        crate::profile_counter!("runtime", "text_shape_source_materialization_count", 1);
        crate::profile_counter!(
            "runtime",
            "text_shape_source_owner_reuse_count",
            reuses_owner as usize
        );
        crate::profile_counter!(
            "runtime",
            "text_shape_source_allocation_count",
            (!reuses_owner) as usize
        );
        crate::profile_counter!(
            "runtime",
            "text_shape_source_allocation_byte_count",
            if reuses_owner { 0 } else { source.len() }
        );
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = reuses_owner;
    source
}
