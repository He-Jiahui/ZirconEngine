//! 将渲染框架统计投影到固定诊断路径，供运行时快照、日志与开发工具共用。
//! 本层只读取一次查询所得的 RenderStats；统计生成与历史保留由各自所有者负责。
mod advanced_provider;
mod ambient_occlusion;
mod anti_alias;
mod capability;
mod dispatch;
mod graph;
mod history;
mod hybrid_gi;
mod measurement;
mod particle;
mod post_process;
mod product;
mod profile;
mod scene_submission_completion;
mod shader_variant;
mod solari;
mod virtual_geometry;
mod volumetric_fog;

use super::DiagnosticStore;
use measurement::{record_bool, record_bytes, record_count, record_microseconds};

pub(crate) use dispatch::record_render_stats_diagnostics;

#[cfg(test)]
mod tests;
