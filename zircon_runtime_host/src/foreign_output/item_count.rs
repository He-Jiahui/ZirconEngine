//! Structural item accounting for bounded runtime payloads.
//!
//! 本模块按 Interface DTO 结构统计 Runtime payload 的类型化条目。
//!
//! 计数被 `RuntimeForeignOutputState::decode_json` 用于预算验证；它只描述业务条目及其
//! JSON 子值，不负责释放 ABI 分配。World 变体必须与 Interface 的 `WorldQueryResult` 同步。

use zircon_runtime_interface::world_sync::{InvalidationBatch, WorldQueryResult};
use zircon_runtime_interface::{
    ProfileControlResponse, ProfileSnapshot, ZrRuntimeOperationResultV1,
    ZrRuntimePluginEventDeliveryBatchV1,
};

/// 递归统计 JSON 数组、对象及标量的结构项数量。
///
/// ## 参数
///
/// `value` 是要计数的 JSON 值；容器节点与其所有嵌套值都各计一项。
///
/// ## 返回值
///
/// 返回饱和累计的结构项数。调用方在反序列化后把该值交给输出预算校验；本函数不执行限额或
/// 解码耗时检查。
pub fn json_value_item_count(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(values) => values.iter().fold(1_usize, |count, value| {
            count.saturating_add(json_value_item_count(value))
        }),
        serde_json::Value::Object(values) => values.values().fold(1_usize, |count, value| {
            count.saturating_add(json_value_item_count(value))
        }),
        _ => 1,
    }
}

/// 统计操作结果外壳及成功 JSON 结果中的结构项。
///
/// ## 参数
///
/// `result` 是 Interface 的操作结果 DTO。
///
/// ## 返回值
///
/// 返回结果外壳一项，加上成功结果中递归 JSON 项数；失败结果只计外壳。反序列化后由调用方
/// 用该值检查类型化条目预算；总解码时限由独立的 elapsed 检查负责。
pub fn operation_result_item_count(result: &ZrRuntimeOperationResultV1) -> usize {
    result
        .succeeded_output()
        .map(json_value_item_count)
        .unwrap_or(1)
        .saturating_add(1)
}

/// 返回插件事件投递批次中的投递数量。
///
/// ## 参数
///
/// `batch` 是 Interface 的插件事件投递 DTO。
///
/// ## 返回值
///
/// 返回 `deliveries` 中的投递数，不包含额外批次外壳；调用方在解码后将其与类型化条目上限比较。
pub fn plugin_event_batch_item_count(batch: &ZrRuntimePluginEventDeliveryBatchV1) -> usize {
    batch.deliveries.len()
}

/// 统计 profile 响应外壳、文件、快照和各已知报告集合。
///
/// ## 参数
///
/// `response` 是 Interface 的 `ProfileControlResponse` DTO。
///
/// ## 返回值
///
/// 返回响应外壳、文件以及实现显式枚举的快照、诊断和报告集合元素数；不按未知 JSON 字段猜测
/// 类型化条目。调用方在反序列化后用此值校验 `max_items`。
pub fn profile_control_response_item_count(response: &ProfileControlResponse) -> usize {
    let mut count = 1_usize.saturating_add(response.files.len());
    if let Some(snapshot) = &response.snapshot {
        count = count.saturating_add(profile_snapshot_item_count(snapshot));
    }
    if let Some(diagnostics) = &response.runtime_diagnostics {
        count = diagnostics.diagnostic_series.iter().fold(
            count.saturating_add(diagnostics.diagnostic_series.len()),
            |count, series| {
                count
                    .saturating_add(series.subsystem_tags.len())
                    .saturating_add(series.history.len())
            },
        );
        count = count.saturating_add(profile_snapshot_item_count(&diagnostics.profile));
    }
    if response.module_composition_receipt.is_some() {
        count = count.saturating_add(1);
    }
    if let Some(report) = &response.hotspot_report {
        count = count
            .saturating_add(report.hotspots.len())
            .saturating_add(report.hints.len());
    }
    if let Some(report) = &response.counter_hotspot_report {
        count = count
            .saturating_add(report.counters.len())
            .saturating_add(report.hints.len());
    }
    if let Some(report) = &response.ui_hotspot_report {
        count = count
            .saturating_add(report.scenarios.len())
            .saturating_add(report.alerts.len());
    }
    count
}

/// 统计 `ProfileSnapshot` 的 frames、spans、counters 和 recorder-retention 集合元素数。
fn profile_snapshot_item_count(snapshot: &ProfileSnapshot) -> usize {
    snapshot
        .frames
        .len()
        .saturating_add(snapshot.spans.len())
        .saturating_add(snapshot.counters.len())
        .saturating_add(snapshot.recorder_retention.len())
}

/// 为每种 World 查询变体计算 Host 预算使用的条目数。
///
/// ## 参数
///
/// `result` 是 Interface 的 `WorldQueryResult`。
///
/// ## 返回值
///
/// 返回该变体对应的结构项数：组件行还递归计入组件 JSON 值，层级行和 Inspector 字段按行数计，
/// 变换快照、实体缺失和未修改结果各计一项。调用方在反序列化后将结果与类型化条目上限比较；
/// 解码总耗时由独立预算检查。
pub fn world_query_item_count(result: &WorldQueryResult) -> usize {
    match result {
        WorldQueryResult::ComponentRows { rows, .. } => {
            rows.iter().fold(rows.len(), |count, row| {
                row.components.values().fold(count, |count, value| {
                    count.saturating_add(json_value_item_count(value))
                })
            })
        }
        WorldQueryResult::HierarchyRows { rows, .. } => rows.len(),
        WorldQueryResult::InspectionFields { fields, .. } => fields.len(),
        WorldQueryResult::TransformSnapshot { .. } => 1,
        WorldQueryResult::EntityMissing { .. } => 1,
        WorldQueryResult::NotModified { .. } => 1,
    }
}

/// 统计失效 dirty/facts 集合并计入批次外壳。
///
/// ## 参数
///
/// `batch` 是 Interface 的 `InvalidationBatch`。
///
/// ## 返回值
///
/// 返回 dirty 项、facts 项与批次外壳的饱和总数，供解码后的类型化预算检查使用；本函数不执行
/// JSON 解析或耗时检查。
pub fn world_invalidation_item_count(batch: &InvalidationBatch) -> usize {
    batch
        .dirty
        .len()
        .saturating_add(batch.facts.len())
        .saturating_add(1)
}
