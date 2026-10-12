//! Export the same immutable collection used by runtime layout to native presentation.
use zr_rhi::UiSurfaceFontSnapshot;

/// Process-owned Editor UI uses the existing process collection, like measure_text_size.
pub fn current_ui_surface_font_snapshot() -> UiSurfaceFontSnapshot {
    from_snapshot(crate::text::font::shared_font_collection_snapshot())
}

/// Core-owned callers pass an already admitted TextRuntimeContext at this boundary.
/// The guard is borrowed only during export; the returned packet owns byte leases.
pub fn ui_surface_font_snapshot_for_context(
    context: &crate::text::TextRuntimeContext,
) -> UiSurfaceFontSnapshot {
    from_snapshot(context.font_collection().collection_snapshot())
}

fn from_snapshot(snapshot: crate::text::font::FontCollectionSnapshot) -> UiSurfaceFontSnapshot {
    snapshot
        .database()
        .render_font_snapshot(snapshot.collection_id().value(), snapshot.generation())
}
