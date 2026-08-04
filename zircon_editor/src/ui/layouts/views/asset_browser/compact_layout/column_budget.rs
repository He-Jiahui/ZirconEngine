const MINIMUM_CONTENT_WIDTH_FRACTION: f32 = 0.52;
const COMPACT_SOURCES_WIDTH: f32 = 152.0;
const COMPACT_DETAILS_WIDTH: f32 = 204.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CompactColumnBudget {
    pub(super) collapse_sources: bool,
    pub(super) collapse_details: bool,
    pub(super) sources_width: f32,
    pub(super) details_width: f32,
}

pub(super) fn resolve_compact_column_budget(
    viewport_width: f32,
    sources_width: f32,
    details_width: f32,
    panel_gap: f32,
    details_allowed_by_height: bool,
) -> CompactColumnBudget {
    let viewport_width = finite_non_negative(viewport_width);
    let sources_width = finite_non_negative(sources_width);
    let details_width = finite_non_negative(details_width);
    let panel_gap = finite_non_negative(panel_gap);
    let sources_width = compact_side_width(sources_width, COMPACT_SOURCES_WIDTH);
    let details_width = compact_side_width(details_width, COMPACT_DETAILS_WIDTH);
    let sources_fit = side_panels_preserve_content(viewport_width, &[sources_width], panel_gap);
    let collapse_sources = sources_width > f32::EPSILON && !sources_fit;
    let details_fit =
        side_panels_preserve_content(viewport_width, &[sources_width, details_width], panel_gap);

    CompactColumnBudget {
        collapse_sources,
        collapse_details: details_width > f32::EPSILON
            && (!details_allowed_by_height || collapse_sources || !details_fit),
        sources_width,
        details_width,
    }
}

fn compact_side_width(preferred_width: f32, compact_width: f32) -> f32 {
    finite_non_negative(preferred_width).min(compact_width)
}

fn side_panels_preserve_content(viewport_width: f32, side_widths: &[f32], gap: f32) -> bool {
    if viewport_width <= f32::EPSILON {
        return false;
    }
    let (side_width, visible_side_count) = side_widths
        .iter()
        .copied()
        .filter(|width| *width > f32::EPSILON)
        .fold((0.0_f32, 0_usize), |(width, count), side_width| {
            (width + side_width, count + 1)
        });
    let remaining_content_width =
        (viewport_width - side_width - gap * visible_side_count as f32).max(0.0);
    remaining_content_width >= viewport_width * MINIMUM_CONTENT_WIDTH_FRACTION
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
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
}
