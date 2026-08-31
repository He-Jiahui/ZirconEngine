use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::BADGE_CIRCULAR_OFFSET_RATIO;

pub(super) fn badge_anchor_point(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let variant = badge_anchor_variant(&node.component_variant);
    let offset_x = if variant.circular {
        rect.width * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let offset_y = if variant.circular {
        rect.height * BADGE_CIRCULAR_OFFSET_RATIO
    } else {
        0.0
    };
    let x = if variant.left {
        rect.x + offset_x
    } else {
        rect.x + rect.width - offset_x
    };
    let y = if variant.bottom {
        rect.y + rect.height - offset_y
    } else {
        rect.y + offset_y
    };
    (x, y)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BadgeAnchorVariant {
    circular: bool,
    left: bool,
    bottom: bool,
}

fn badge_anchor_variant(component_variant: &str) -> BadgeAnchorVariant {
    let mut variant = BadgeAnchorVariant::default();
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        variant.circular |=
            part.eq_ignore_ascii_case("circular") || part.eq_ignore_ascii_case("overlapCircular");
        variant.left |= part.eq_ignore_ascii_case("left")
            || part.eq_ignore_ascii_case("anchorOriginTopLeft")
            || part.eq_ignore_ascii_case("anchorOriginBottomLeft");
        variant.bottom |= part.eq_ignore_ascii_case("bottom")
            || part.eq_ignore_ascii_case("anchorOriginBottomLeft")
            || part.eq_ignore_ascii_case("anchorOriginBottomRight");
    }
    variant
}

#[cfg(test)]
#[path = "anchor/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
