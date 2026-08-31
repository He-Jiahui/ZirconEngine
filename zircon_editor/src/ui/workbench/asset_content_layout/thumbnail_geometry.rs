use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

use super::identity::BrowserThumbnailNodeRole;
use super::paint_metadata::AssetContentRect;

const THUMBNAIL_VISUAL_MIN_HEIGHT: f32 = 72.0;
const THUMBNAIL_VISUAL_MAX_HEIGHT: f32 = 88.0;
const THUMBNAIL_CARD_INSET: f32 = 8.0;
const THUMBNAIL_INFO_BAND_SINGLE_LINE_HEIGHT: f32 = 42.0;
const THUMBNAIL_INFO_BAND_STACKED_HEIGHT: f32 = 54.0;
const THUMBNAIL_INFO_TEXT_INSET_X: f32 = 5.0;
const THUMBNAIL_SELECTION_MARKER_WIDTH: f32 = 0.0;
const THUMBNAIL_NAME_PRIMARY_OFFSET_Y: f32 = 5.0;
const THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const THUMBNAIL_NAME_CONTINUATION_OFFSET_Y: f32 =
    THUMBNAIL_NAME_PRIMARY_OFFSET_Y + THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT;
const THUMBNAIL_NAME_CONTINUATION_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const THUMBNAIL_META_ROW_SINGLE_OFFSET_Y: f32 = 25.0;
const THUMBNAIL_META_ROW_STACKED_OFFSET_Y: f32 = 36.0;
const THUMBNAIL_TYPE_BADGE_MIN_WIDTH: f32 = 42.0;
const THUMBNAIL_TYPE_BADGE_MAX_WIDTH: f32 = 48.0;
const THUMBNAIL_TYPE_BADGE_HEIGHT: f32 = 13.0;
const THUMBNAIL_TYPE_BADGE_TEXT_INSET_X: f32 = 5.0;
const THUMBNAIL_TYPE_BADGE_PADDING_X: f32 = 6.0;
const THUMBNAIL_TYPE_BADGE_MAX_WIDTH_RATIO: f32 = 0.55;
const THUMBNAIL_META_ROW_GAP: f32 = 5.0;

/// Item-specific thumbnail frames derived from a stable materialized card.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AssetThumbnailCardGeometry {
    card: AssetContentRect,
    visual: AssetContentRect,
    info_band: AssetContentRect,
    selection_marker: AssetContentRect,
    name: AssetContentRect,
    name_continuation: AssetContentRect,
    type_badge: AssetContentRect,
    type_label: AssetContentRect,
    meta: AssetContentRect,
}

impl AssetThumbnailCardGeometry {
    pub(crate) fn for_role(self, role: BrowserThumbnailNodeRole) -> AssetContentRect {
        match role {
            BrowserThumbnailNodeRole::Card => self.card,
            BrowserThumbnailNodeRole::InfoBand => self.info_band,
            BrowserThumbnailNodeRole::SelectionMarker => self.selection_marker,
            BrowserThumbnailNodeRole::TypeBadge => self.type_badge,
            BrowserThumbnailNodeRole::Visual => self.visual,
            BrowserThumbnailNodeRole::NameContinuation => self.name_continuation,
            BrowserThumbnailNodeRole::Name => self.name,
            BrowserThumbnailNodeRole::Type => self.type_label,
            BrowserThumbnailNodeRole::Meta => self.meta,
        }
    }
}

pub(crate) fn asset_thumbnail_card_geometry(
    card: AssetContentRect,
    has_name_continuation: bool,
    type_label_width: f32,
) -> AssetThumbnailCardGeometry {
    if !card.width.is_finite()
        || !card.height.is_finite()
        || card.width <= 0.0
        || card.height <= 0.0
    {
        return collapsed_thumbnail_geometry(card);
    }
    let inner_x = card.x + THUMBNAIL_CARD_INSET;
    let inner_width = (card.width - THUMBNAIL_CARD_INSET * 2.0).max(24.0);
    let continuation_height = has_name_continuation
        .then_some(THUMBNAIL_NAME_CONTINUATION_LINE_HEIGHT)
        .unwrap_or(0.0);
    let band_height = thumbnail_info_band_height(continuation_height)
        .min((card.height - THUMBNAIL_CARD_INSET * 2.0).max(0.0));
    let band_y = card.y + card.height - THUMBNAIL_CARD_INSET - band_height;
    let visual_y = card.y + THUMBNAIL_CARD_INSET;
    let visual_height =
        (band_y - visual_y).clamp(THUMBNAIL_VISUAL_MIN_HEIGHT, THUMBNAIL_VISUAL_MAX_HEIGHT);
    let text_x = inner_x + THUMBNAIL_INFO_TEXT_INSET_X;
    let text_width = (inner_width - THUMBNAIL_INFO_TEXT_INSET_X * 2.0).max(16.0);
    let meta_row_y = band_y + thumbnail_meta_row_offset_y(continuation_height);
    let type_badge_width = thumbnail_type_badge_width(type_label_width, text_width);
    let type_text_x = text_x + THUMBNAIL_TYPE_BADGE_TEXT_INSET_X;
    let type_text_width = (type_badge_width - THUMBNAIL_TYPE_BADGE_TEXT_INSET_X * 2.0).max(0.0);
    let meta_x = text_x + type_badge_width + THUMBNAIL_META_ROW_GAP;
    let meta_width = (text_x + text_width - meta_x).max(0.0);

    AssetThumbnailCardGeometry {
        card,
        visual: AssetContentRect {
            x: inner_x,
            y: visual_y,
            width: inner_width,
            height: visual_height,
        },
        info_band: AssetContentRect {
            x: inner_x,
            y: band_y,
            width: inner_width,
            height: band_height,
        },
        selection_marker: AssetContentRect {
            x: inner_x,
            y: band_y,
            width: THUMBNAIL_SELECTION_MARKER_WIDTH,
            height: band_height,
        },
        name: AssetContentRect {
            x: text_x,
            y: band_y + THUMBNAIL_NAME_PRIMARY_OFFSET_Y,
            width: text_width,
            height: THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT,
        },
        name_continuation: AssetContentRect {
            x: text_x,
            y: band_y + THUMBNAIL_NAME_CONTINUATION_OFFSET_Y,
            width: text_width,
            height: continuation_height,
        },
        type_badge: AssetContentRect {
            x: text_x,
            y: meta_row_y,
            width: type_badge_width,
            height: THUMBNAIL_TYPE_BADGE_HEIGHT,
        },
        type_label: AssetContentRect {
            x: type_text_x,
            y: meta_row_y,
            width: type_text_width,
            height: THUMBNAIL_TYPE_BADGE_HEIGHT,
        },
        meta: AssetContentRect {
            x: meta_x,
            y: meta_row_y,
            width: meta_width,
            height: THUMBNAIL_TYPE_BADGE_HEIGHT,
        },
    }
}

fn collapsed_thumbnail_geometry(card: AssetContentRect) -> AssetThumbnailCardGeometry {
    let collapsed = AssetContentRect {
        x: card.x,
        y: card.y,
        width: 0.0,
        height: 0.0,
    };
    AssetThumbnailCardGeometry {
        card: collapsed,
        visual: collapsed,
        info_band: collapsed,
        selection_marker: collapsed,
        name: collapsed,
        name_continuation: collapsed,
        type_badge: collapsed,
        type_label: collapsed,
        meta: collapsed,
    }
}

fn thumbnail_meta_row_offset_y(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        THUMBNAIL_META_ROW_STACKED_OFFSET_Y
    } else {
        THUMBNAIL_META_ROW_SINGLE_OFFSET_Y
    }
}

fn thumbnail_info_band_height(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        THUMBNAIL_INFO_BAND_STACKED_HEIGHT
    } else {
        THUMBNAIL_INFO_BAND_SINGLE_LINE_HEIGHT
    }
}

fn thumbnail_type_badge_width(type_label_width: f32, text_width: f32) -> f32 {
    let type_label_width = if type_label_width.is_finite() {
        type_label_width.max(0.0)
    } else {
        0.0
    };
    let content_width = type_label_width + THUMBNAIL_TYPE_BADGE_PADDING_X * 2.0;
    let badge_max_width = THUMBNAIL_TYPE_BADGE_MAX_WIDTH
        .min(text_width * THUMBNAIL_TYPE_BADGE_MAX_WIDTH_RATIO)
        .max(THUMBNAIL_TYPE_BADGE_MIN_WIDTH);
    content_width
        .clamp(THUMBNAIL_TYPE_BADGE_MIN_WIDTH, badge_max_width)
        .max(0.0)
}
