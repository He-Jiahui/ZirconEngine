use super::*;

#[test]
fn responsive_grid_uses_columns_and_preserves_scrollable_content_extent() {
    let metrics = AssetThumbnailGridMetrics::new(420.0, 12);

    assert_eq!(metrics.columns(), 3);
    assert!(metrics.content_extent() > 600.0);
    let second = metrics.item_frame(1).expect("second thumbnail");
    let fourth = metrics.item_frame(3).expect("fourth thumbnail");
    assert!(second.x > 140.0);
    assert_eq!(fourth.x, 8.0);
    assert!(fourth.y > 150.0);
}

#[test]
fn materialized_budget_uses_the_actual_grid_columns() {
    let metrics = AssetThumbnailGridMetrics::new(128.0, 10_000);

    assert_eq!(metrics.columns(), 1);
    assert_eq!(metrics.materialized_item_budget(620.0, 2), 9);
    assert_eq!(
        AssetThumbnailGridMetrics::conservative_materialized_item_budget(620.0, 10_000, 2),
        54,
        "the pre-layout budget remains a conservative upper bound"
    );
}

#[test]
fn grid_omits_cards_when_the_viewport_cannot_fit_the_minimum_card_width() {
    let minimum_viewport_width = GRID_PADDING * 2.0 + CARD_MIN_WIDTH;
    let too_narrow = AssetThumbnailGridMetrics::new(minimum_viewport_width - 0.1, 12);
    let exact_fit = AssetThumbnailGridMetrics::new(minimum_viewport_width, 12);

    assert_eq!(too_narrow.columns(), 0);
    assert!(too_narrow.item_frame(0).is_none());
    assert_eq!(too_narrow.content_extent(), 0.0);

    assert_eq!(exact_fit.columns(), 1);
    assert_eq!(
        exact_fit.item_frame(0).expect("minimum-width card").width,
        CARD_MIN_WIDTH
    );
}

#[test]
fn grid_rejects_non_finite_or_collapsed_viewports() {
    for viewport_width in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let metrics = AssetThumbnailGridMetrics::new(viewport_width, 1);

        assert_eq!(metrics.columns(), 0);
        assert!(metrics.item_frame(0).is_none());
        assert_eq!(metrics.content_extent(), 0.0);
    }
}

#[test]
fn point_lookup_rejects_grid_gaps_and_resolves_cards_without_scanning() {
    let metrics = AssetThumbnailGridMetrics::new(420.0, 12);
    let second = metrics.item_frame(1).expect("second thumbnail");

    assert_eq!(
        metrics.item_index_at_point(UiPoint::new(
            second.x + second.width * 0.5,
            second.y + second.height * 0.5,
        )),
        Some(1)
    );
    assert_eq!(
        metrics.item_index_at_point(UiPoint::new(second.x - GRID_GAP * 0.5, second.y + 4.0)),
        None
    );
}
