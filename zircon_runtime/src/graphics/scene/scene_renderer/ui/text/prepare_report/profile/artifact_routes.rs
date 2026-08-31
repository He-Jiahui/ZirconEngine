use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiResolvedGlyphArtifactRouteReport;

pub(super) fn record_resolved_glyph_artifact_route_profile(
    report: &ScreenSpaceUiResolvedGlyphArtifactRouteReport,
    post_layout_stale_artifact_batch_rejection_count: usize,
) {
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.artifact_commands",
        report.artifact_command_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.visual_only_commands",
        report.visual_only_command_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.source_isomorphic_fallback_commands",
        report.source_isomorphic_fallback_command_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.missing",
        report.missing_artifact_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.stale",
        report.stale_artifact_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.incomplete",
        report.incomplete_artifact_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.resolved_glyph_artifact_route.rejected_commands",
        report.rejected_command_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.prepare.post_layout_stale_artifact_batch_rejections",
        post_layout_stale_artifact_batch_rejection_count
    );
}
