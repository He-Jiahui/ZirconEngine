use crate::core::framework::render::{RenderHybridGiPayloadSource, RenderStats};

use super::super::{record_bool, DiagnosticStore};

/// 将 GI 输入载荷来源变为稳定互斥状态序列；读取时应结合功能就绪报告判断无来源是否预期。
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let source = stats.last_hybrid_gi_payload_source;
    record_bool(
        store,
        "render.hybrid_gi.payload.source.none",
        frame_index,
        source == RenderHybridGiPayloadSource::None,
        &["render", "hybrid_gi", "payload", "source"],
    );
    record_bool(
        store,
        "render.hybrid_gi.payload.source.scene_representation",
        frame_index,
        source == RenderHybridGiPayloadSource::SceneRepresentation,
        &["render", "hybrid_gi", "payload", "source"],
    );
}
