use crate::core::framework::render::{RenderHybridGiPayloadSource, RenderStats};

use super::super::{record_bool, DiagnosticStore};

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
