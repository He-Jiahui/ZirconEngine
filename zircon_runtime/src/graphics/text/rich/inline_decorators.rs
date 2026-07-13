use crate::core::framework::render::{FontFamilyName, InlineObjectRef};
use crate::core::math::Vec2;

use super::{RichTextDecoration, RichTextDecorator};

const DEFAULT_ICON_FONT_FAMILY: &str = "Zircon Icons";
const MAX_INLINE_WIDGET_EXTENT: f32 = 16_384.0;

pub(super) struct IconTextDecorator;

impl RichTextDecorator for IconTextDecorator {
    fn tag(&self) -> &str {
        "icon"
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        let Some((glyph, font)) = value.and_then(parse_icon) else {
            return false;
        };
        decoration.inline = Some(InlineObjectRef::Icon { glyph, font });
        true
    }
}

pub(super) struct WidgetTextDecorator;

impl RichTextDecorator for WidgetTextDecorator {
    fn tag(&self) -> &str {
        "widget"
    }

    fn decorate(&self, value: Option<&str>, decoration: &mut RichTextDecoration) -> bool {
        let Some((id, size)) = value.and_then(parse_widget) else {
            return false;
        };
        decoration.inline = Some(InlineObjectRef::Widget { id, size });
        true
    }
}

fn parse_icon(value: &str) -> Option<(char, FontFamilyName)> {
    let (glyph, family) = value
        .split_once('|')
        .map(|(glyph, family)| (glyph.trim(), family.trim()))
        .unwrap_or((value.trim(), DEFAULT_ICON_FONT_FAMILY));
    let mut glyphs = glyph.chars();
    let glyph = glyphs.next()?;
    if glyphs.next().is_some() || family.is_empty() {
        return None;
    }
    Some((glyph, FontFamilyName::from(family)))
}

fn parse_widget(value: &str) -> Option<(u64, Vec2)> {
    let (id, extent) = value.split_once('|')?;
    let (width, height) = extent.split_once('x')?;
    let id = id.trim().parse().ok()?;
    let width = positive_extent(width)?;
    let height = positive_extent(height)?;
    Some((id, Vec2::new(width, height)))
}

fn positive_extent(value: &str) -> Option<f32> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|extent| extent.is_finite() && *extent > 0.0 && *extent <= MAX_INLINE_WIDGET_EXTENT)
}
