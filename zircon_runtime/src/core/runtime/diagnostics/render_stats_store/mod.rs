//! 将渲染框架统计投影到固定诊断路径，供运行时快照、日志与开发工具共用。
//! 本层只读取一次查询所得的 RenderStats；统计生成与历史保留由各自所有者负责。
#[path = "../../render_stats_store/advanced_provider.rs"]
mod advanced_provider;
#[path = "../../render_stats_store/ambient_occlusion.rs"]
mod ambient_occlusion;
#[path = "anti_alias.rs"]
mod anti_alias;
#[path = "capability.rs"]
mod capability;
#[path = "dispatch.rs"]
mod dispatch;
#[path = "graph/mod.rs"]
mod graph;
#[path = "history.rs"]
mod history;
#[path = "hybrid_gi.rs"]
mod hybrid_gi;
#[path = "measurement.rs"]
mod measurement;
#[path = "particle.rs"]
mod particle;
#[path = "post/mod.rs"]
mod post_process;
#[path = "product.rs"]
mod product;
#[path = "profile.rs"]
mod profile;
#[path = "../../render_stats_store/scene_submission_complete.rs"]
mod scene_submission_completion;
#[path = "shader_variant.rs"]
mod shader_variant;
#[path = "solari.rs"]
mod solari;
#[path = "geometry/mod.rs"]
mod virtual_geometry;
#[path = "volumetric_fog.rs"]
mod volumetric_fog;

use super::DiagnosticStore;

pub(super) fn record_identity_u64(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: u64,
    subsystem_tags: &[&str],
) {
    store.record_identity_u64_static(path, frame_index, value, Some("u64"), subsystem_tags);
}
use measurement::{record_bool, record_bytes, record_count, record_microseconds};

pub(crate) use dispatch::record_render_stats_diagnostics;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
