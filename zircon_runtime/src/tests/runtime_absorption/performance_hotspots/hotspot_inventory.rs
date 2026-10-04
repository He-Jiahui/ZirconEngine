//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。以结果断言检查当前接口或源码快照对应的边界。
#[path = "hotspot_inventory/ecs_extract_counters.rs"]
mod ecs_extract_counters;
#[path = "hotspot_inventory/evidence_gate_docs.rs"]
mod evidence_gate_docs;
#[path = "hotspot_inventory/profiling_trace_render.rs"]
mod profiling_trace_render;
#[path = "hotspot_inventory/sources.rs"]
mod sources;
#[path = "hotspot_inventory/split_layout.rs"]
mod split_layout;

#[test]
fn runtime_07_hotspot_inventory_requires_counted_evidence_before_m2() {
    let sources = sources::HotspotInventorySources::load();

    evidence_gate_docs::assert_evidence_gate_docs(&sources);
    ecs_extract_counters::assert_ecs_extract_counter_evidence(&sources);
    profiling_trace_render::assert_profiling_trace_and_render_diversion(&sources);
}
