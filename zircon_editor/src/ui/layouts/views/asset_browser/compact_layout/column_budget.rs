const MINIMUM_CONTENT_WIDTH_FRACTION: f32 = 0.52;
pub(super) const NARROW_BREAKPOINT_WIDTH: f32 = 640.0;
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
    let collapse_sources =
        sources_width > f32::EPSILON && (viewport_width < NARROW_BREAKPOINT_WIDTH || !sources_fit);
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
    let (side_width, visible_side_count) = match side_widths {
        [] => (0.0_f32, 0_usize),
        [width] if *width > f32::EPSILON => (*width, 1),
        [first, second] => {
            let first_visible = *first > f32::EPSILON;
            let second_visible = *second > f32::EPSILON;
            (
                if first_visible { *first } else { 0.0 }
                    + if second_visible { *second } else { 0.0 },
                usize::from(first_visible) + usize::from(second_visible),
            )
        }
        _ => side_widths
            .iter()
            .copied()
            .filter(|width| *width > f32::EPSILON)
            .fold((0.0_f32, 0_usize), |(width, count), side_width| {
                (width + side_width, count + 1)
            }),
    };
    let remaining_content_width =
        (viewport_width - side_width - gap * visible_side_count as f32).max(0.0);
    remaining_content_width >= viewport_width * MINIMUM_CONTENT_WIDTH_FRACTION
}

#[cfg(test)]
#[path = "column_budget/tests/fast_width_tests.rs"]
mod fast_width_tests;

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/column_budget.rs"]
mod tests;
