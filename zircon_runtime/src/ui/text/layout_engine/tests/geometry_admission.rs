use super::admit_resolved_layout_publication;
use crate::core::framework::text::TextLayoutError;
use crate::text::font::{runtime_default_font_database_for_test, FontCollectionService};
use crate::text::{SharedTextLayoutSession, TextLayoutGeometryBudget, TextLayoutGeometryOwner};
use zircon_runtime_interface::ui::surface::{UiResolvedTextLayout, UiTextRange};

#[test]
fn resolved_layout_publication_rejects_geometry_before_artifact_build() {
    let budget = TextLayoutGeometryBudget::new(64.0, 128.0).expect("valid budget");
    let font_collection =
        FontCollectionService::from_database(runtime_default_font_database_for_test());
    let mut provider = SharedTextLayoutSession::new_with_font_collection_and_geometry_budget(
        font_collection,
        budget,
    );
    let layout = UiResolvedTextLayout {
        measured_width: 65.0,
        measured_height: 16.0,
        font_size: 12.0,
        line_height: 16.0,
        source_range: UiTextRange { start: 7, end: 19 },
        ..UiResolvedTextLayout::default()
    };

    assert_eq!(
        admit_resolved_layout_publication(&layout, &mut provider),
        Err(TextLayoutError::GeometryTooLarge)
    );
    let receipt = provider
        .geometry_report()
        .last_rejection
        .expect("publication rejection receipt");
    assert_eq!(
        receipt.owner,
        TextLayoutGeometryOwner::ResolvedLayoutPublication
    );
    assert_eq!(receipt.source_range, Some((7, 19)));
    assert_eq!(receipt.attempted_extent, 65.0);
    assert_eq!(receipt.admitted_extent, 64.0);
}
