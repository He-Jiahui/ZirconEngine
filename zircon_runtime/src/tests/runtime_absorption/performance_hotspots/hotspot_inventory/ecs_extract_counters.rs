//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。保存同组守卫使用的局部数据或辅助变换。
#[path = "ecs_extract_counters/asset_animation.rs"]
mod asset_animation;
#[path = "ecs_extract_counters/extract_cache.rs"]
mod extract_cache;
#[path = "ecs_extract_counters/frame_diagnostics.rs"]
mod frame_diagnostics;
#[path = "ecs_extract_counters/query_change.rs"]
mod query_change;
#[path = "ecs_extract_counters/split_layout.rs"]
mod split_layout;

use super::sources::HotspotInventorySources;

pub(super) fn assert_ecs_extract_counter_evidence(sources: &HotspotInventorySources) {
    query_change::assert_query_and_change_evidence(sources);
    extract_cache::assert_extract_evidence(sources);
    asset_animation::assert_asset_and_animation_evidence(sources);
    frame_diagnostics::assert_ecs_frame_diagnostic_aggregation(sources);
}
