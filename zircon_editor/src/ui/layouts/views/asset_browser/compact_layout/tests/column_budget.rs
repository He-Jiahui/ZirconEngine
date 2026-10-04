use super::*;

const SOURCES_WIDTH: f32 = 280.0;
const DETAILS_WIDTH: f32 = 340.0;
const PANEL_GAP: f32 = 6.0;

#[test]
fn compact_asset_columns_preserve_sources_before_details_when_content_is_narrow() {
    assert_eq!(
        resolve_compact_column_budget(640.0, SOURCES_WIDTH, DETAILS_WIDTH, PANEL_GAP, true,),
        CompactColumnBudget {
            collapse_sources: false,
            collapse_details: true,
            sources_width: COMPACT_SOURCES_WIDTH,
            details_width: COMPACT_DETAILS_WIDTH,
        }
    );
}

#[test]
fn compact_asset_columns_keep_a_narrow_inspector_when_three_regions_fit() {
    assert_eq!(
        resolve_compact_column_budget(900.0, SOURCES_WIDTH, DETAILS_WIDTH, PANEL_GAP, true,),
        CompactColumnBudget {
            collapse_sources: false,
            collapse_details: false,
            sources_width: COMPACT_SOURCES_WIDTH,
            details_width: COMPACT_DETAILS_WIDTH,
        }
    );
}

#[test]
fn regular_asset_columns_restore_sources_and_details_when_content_budget_remains() {
    assert_eq!(
        resolve_compact_column_budget(1260.0, SOURCES_WIDTH, DETAILS_WIDTH, PANEL_GAP, true,),
        CompactColumnBudget {
            collapse_sources: false,
            collapse_details: false,
            sources_width: COMPACT_SOURCES_WIDTH,
            details_width: COMPACT_DETAILS_WIDTH,
        }
    );
}

#[test]
fn short_asset_surface_keeps_details_collapsed_at_regular_width() {
    assert!(
        resolve_compact_column_budget(1260.0, SOURCES_WIDTH, DETAILS_WIDTH, PANEL_GAP, false,)
            .collapse_details
    );
}

#[test]
fn column_budget_uses_a_relative_content_reserve() {
    assert!(MINIMUM_CONTENT_WIDTH_FRACTION >= 0.5);
    assert!(MINIMUM_CONTENT_WIDTH_FRACTION < 1.0);
}

#[test]
fn ultra_width_collapses_sources_before_content_is_squeezed() {
    let budget = resolve_compact_column_budget(
        NARROW_BREAKPOINT_WIDTH - 1.0,
        SOURCES_WIDTH,
        DETAILS_WIDTH,
        PANEL_GAP,
        false,
    );
    assert!(budget.collapse_sources);
    assert!(budget.collapse_details);
}
