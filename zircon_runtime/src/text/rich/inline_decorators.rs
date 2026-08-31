use crate::core::math::Vec2;
use crate::core::resource::ResourceId;
use crate::text::{InlineBaseline, InlineObjectRef, RichIconAssetId, RichInlineWidgetSlotId};

use super::resource_admission::controlled_resource_locator;
use super::{RichTextDecoration, RichTextDecorator};

const DEFAULT_INLINE_ICON_SIZE_PX: f32 = 16.0;
const MAX_INLINE_WIDGET_EXTENT: f32 = 16_384.0;

pub(super) struct IconTextDecorator;

impl RichTextDecorator for IconTextDecorator {
    fn tag(&self) -> &str {
        "icon"
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        let Some((asset, size, baseline, alternative_text)) = value.and_then(parse_icon) else {
            return false;
        };
        decoration.inline = Some(InlineObjectRef::Icon {
            asset,
            size,
            baseline,
            alternative_text,
        });
        true
    }
}

pub(super) struct WidgetTextDecorator;

impl RichTextDecorator for WidgetTextDecorator {
    fn tag(&self) -> &str {
        "widget"
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        let Some((slot, size)) = value.and_then(parse_widget) else {
            return false;
        };
        decoration.inline = Some(InlineObjectRef::Widget { slot, size });
        true
    }
}

fn parse_icon(value: &str) -> Option<(RichIconAssetId, Vec2, InlineBaseline, Option<String>)> {
    let mut fields = value.splitn(4, '|');
    let locator = controlled_resource_locator(fields.next()?)?;
    let size = match fields.next().filter(|extent| !extent.trim().is_empty()) {
        Some(extent) => parse_extent(extent)?,
        None => Vec2::new(DEFAULT_INLINE_ICON_SIZE_PX, DEFAULT_INLINE_ICON_SIZE_PX),
    };
    let baseline = match fields.next().filter(|baseline| !baseline.trim().is_empty()) {
        Some(baseline) => parse_baseline(baseline)?,
        None => InlineBaseline::Baseline,
    };
    let alternative_text = fields.next().map(str::to_owned);
    Some((
        RichIconAssetId::from_resource_id(ResourceId::from_locator(&locator)),
        size,
        baseline,
        alternative_text,
    ))
}

fn parse_widget(value: &str) -> Option<(RichInlineWidgetSlotId, Vec2)> {
    let (id, extent) = value.split_once('|')?;
    let slot = RichInlineWidgetSlotId::new(id.trim().parse().ok()?);
    Some((slot, parse_extent(extent)?))
}

fn parse_extent(value: &str) -> Option<Vec2> {
    let (width, height) = value.split_once('x')?;
    Some(Vec2::new(positive_extent(width)?, positive_extent(height)?))
}

fn parse_baseline(value: &str) -> Option<InlineBaseline> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(InlineBaseline::Baseline),
        "center" => Some(InlineBaseline::Center),
        "top" => Some(InlineBaseline::Top),
        "bottom" => Some(InlineBaseline::Bottom),
        _ => None,
    }
}

fn positive_extent(value: &str) -> Option<f32> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|extent| extent.is_finite() && *extent > 0.0 && *extent <= MAX_INLINE_WIDGET_EXTENT)
}
