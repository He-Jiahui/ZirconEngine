use crate::core::framework::render::RenderStats;

use super::super::{record_count, record_microseconds, DiagnosticStore};

/// 对照计划存活通道与实际执行通道，暴露遗漏、意外执行和重复执行。
pub(super) fn record_coverage(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let report = stats.last_graph_execution_coverage_report;
    record_count(
        store,
        "render.graph.execution.coverage.planned_live_pass_count",
        frame_index,
        report.planned_live_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.executed_pass_count",
        frame_index,
        report.executed_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.matched_planned_pass_count",
        frame_index,
        report.matched_planned_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.missing_planned_pass_count",
        frame_index,
        report.missing_planned_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.unexpected_executed_pass_count",
        frame_index,
        report.unexpected_executed_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
    record_count(
        store,
        "render.graph.execution.coverage.duplicate_executed_pass_count",
        frame_index,
        report.duplicate_executed_pass_count,
        &["render", "graph", "execution", "coverage"],
    );
}

/// 将执行阶段的分组结果与计划通道数对齐，便于定位图计划和实际调度之间的偏差。
pub(super) fn record_stage(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let report = stats.last_graph_stage_execution_report;
    record_count(
        store,
        "render.graph.execution.stage.staged_pass_count",
        frame_index,
        report.staged_pass_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.unstaged_pass_count",
        frame_index,
        report.unstaged_pass_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.unique_stage_count",
        frame_index,
        report.unique_stage_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.transition_count",
        frame_index,
        report.stage_transition_count,
        &["render", "graph", "execution", "stage"],
    );
    record_count(
        store,
        "render.graph.execution.stage.order_violation_count",
        frame_index,
        report.stage_order_violation_count,
        &["render", "graph", "execution", "stage", "order"],
    );
}

/// 汇总执行批次与阶段边界，解释相同通道数下不同的调度开销。
pub(super) fn record_batches(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let report = stats.last_graph_execution_batch_report;
    for (metric, value, tags) in [
        (
            "render.graph.execution.batch.planned_batch_count",
            report.planned_batch_count,
            &["render", "graph", "execution", "batch"][..],
        ),
        (
            "render.graph.execution.batch.planned_live_pass_count",
            report.planned_live_pass_count,
            &["render", "graph", "execution", "batch", "pass"][..],
        ),
        (
            "render.graph.execution.batch.graphics_count",
            report.graphics_batch_count,
            &["render", "graph", "execution", "batch", "graphics"][..],
        ),
        (
            "render.graph.execution.batch.async_compute_count",
            report.async_compute_batch_count,
            &["render", "graph", "execution", "batch", "async_compute"][..],
        ),
        (
            "render.graph.execution.batch.async_copy_count",
            report.async_copy_batch_count,
            &["render", "graph", "execution", "batch", "async_copy"][..],
        ),
        (
            "render.graph.execution.batch.max_passes_per_batch",
            report.max_passes_per_batch,
            &["render", "graph", "execution", "batch", "pass"][..],
        ),
        (
            "render.graph.execution.batch.queue_transition_count",
            report.queue_transition_count,
            &["render", "graph", "execution", "batch", "queue"][..],
        ),
    ] {
        record_count(store, metric, frame_index, value, tags);
    }
}

/// 区分逻辑资源、别名和实际后备资源，判断复用效果时不能把别名数当成分配数。
pub(super) fn record_aliases(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let report = &stats.last_graph_execution_alias_report;
    record_count(
        store,
        "render.graph.execution.alias.texture_logical_count",
        frame_index,
        report.texture_logical_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.texture_alias_count",
        frame_index,
        report.texture_alias_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.texture_backing_count",
        frame_index,
        report.texture_backing_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
            "backing",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.buffer_logical_count",
        frame_index,
        report.buffer_logical_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.buffer_alias_count",
        frame_index,
        report.buffer_alias_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
        ],
    );
    record_count(
        store,
        "render.graph.execution.alias.buffer_backing_count",
        frame_index,
        report.buffer_backing_count(),
        &[
            "render",
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
            "backing",
        ],
    );
}

/// 发布执行通道 CPU 总耗时与最大耗时，供比较总体成本和单点瓶颈。
pub(super) fn record_profile(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let report = &stats.last_graph_execution_profile_report;
    record_count(
        store,
        "render.graph.execution.profile.pass_count",
        frame_index,
        report.pass_count(),
        &["render", "graph", "execution", "profile"],
    );
    record_microseconds(
        store,
        "render.graph.execution.profile.cpu_elapsed_total_us",
        frame_index,
        report.total_cpu_elapsed_micros(),
        &["render", "graph", "execution", "profile", "cpu"],
    );
    record_microseconds(
        store,
        "render.graph.execution.profile.cpu_elapsed_max_us",
        frame_index,
        report.max_cpu_elapsed_micros(),
        &["render", "graph", "execution", "profile", "cpu"],
    );
}
