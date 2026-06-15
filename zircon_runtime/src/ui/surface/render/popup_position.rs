use toml::Value;
use zircon_runtime_interface::ui::{layout::UiFrame, tree::UiTemplateNodeMetadata};

#[derive(Clone, Copy, Debug)]
pub(super) enum PopupPlacement {
    TopStart,
    Top,
    TopEnd,
    BottomStart,
    Bottom,
    BottomEnd,
    LeftStart,
    Left,
    LeftEnd,
    RightStart,
    Right,
    RightEnd,
}

impl PopupPlacement {
    fn from_metadata(metadata: &UiTemplateNodeMetadata, default: Self) -> Self {
        string_attribute(metadata, "placement")
            .and_then(|placement| Self::parse(placement, default))
            .unwrap_or(default)
    }

    fn parse(raw: &str, default: Self) -> Option<Self> {
        let normalized = raw
            .trim()
            .replace('_', "-")
            .replace(' ', "-")
            .to_ascii_lowercase();
        let mut parts = normalized.split('-').filter(|part| !part.is_empty());
        let side = parts.next()?;
        let align = parts.next();
        let default_align = default.align();
        match (side, align.unwrap_or(default_align.as_str())) {
            ("top", "start") => Some(Self::TopStart),
            ("top", "center") | ("top", "middle") => Some(Self::Top),
            ("top", "end") => Some(Self::TopEnd),
            ("bottom", "start") => Some(Self::BottomStart),
            ("bottom", "center") | ("bottom", "middle") => Some(Self::Bottom),
            ("bottom", "end") => Some(Self::BottomEnd),
            ("left", "start") => Some(Self::LeftStart),
            ("left", "center") | ("left", "middle") => Some(Self::Left),
            ("left", "end") => Some(Self::LeftEnd),
            ("right", "start") => Some(Self::RightStart),
            ("right", "center") | ("right", "middle") => Some(Self::Right),
            ("right", "end") => Some(Self::RightEnd),
            _ => None,
        }
    }

    fn side(self) -> PopupSide {
        match self {
            Self::TopStart | Self::Top | Self::TopEnd => PopupSide::Top,
            Self::BottomStart | Self::Bottom | Self::BottomEnd => PopupSide::Bottom,
            Self::LeftStart | Self::Left | Self::LeftEnd => PopupSide::Left,
            Self::RightStart | Self::Right | Self::RightEnd => PopupSide::Right,
        }
    }

    fn align(self) -> PopupAlign {
        match self {
            Self::TopStart | Self::BottomStart | Self::LeftStart | Self::RightStart => {
                PopupAlign::Start
            }
            Self::Top | Self::Bottom | Self::Left | Self::Right => PopupAlign::Center,
            Self::TopEnd | Self::BottomEnd | Self::LeftEnd | Self::RightEnd => PopupAlign::End,
        }
    }

    fn flipped(self) -> Self {
        match self {
            Self::TopStart => Self::BottomStart,
            Self::Top => Self::Bottom,
            Self::TopEnd => Self::BottomEnd,
            Self::BottomStart => Self::TopStart,
            Self::Bottom => Self::Top,
            Self::BottomEnd => Self::TopEnd,
            Self::LeftStart => Self::RightStart,
            Self::Left => Self::Right,
            Self::LeftEnd => Self::RightEnd,
            Self::RightStart => Self::LeftStart,
            Self::Right => Self::Left,
            Self::RightEnd => Self::LeftEnd,
        }
    }

    fn anchor_origin(self) -> (HorizontalOrigin, VerticalOrigin) {
        match self.side() {
            PopupSide::Top => (horizontal_origin(self.align()), VerticalOrigin::Top),
            PopupSide::Bottom => (horizontal_origin(self.align()), VerticalOrigin::Bottom),
            PopupSide::Left => (HorizontalOrigin::Left, vertical_origin(self.align())),
            PopupSide::Right => (HorizontalOrigin::Right, vertical_origin(self.align())),
        }
    }

    fn transform_origin(self) -> (HorizontalOrigin, VerticalOrigin) {
        match self.side() {
            PopupSide::Top => (horizontal_origin(self.align()), VerticalOrigin::Bottom),
            PopupSide::Bottom => (horizontal_origin(self.align()), VerticalOrigin::Top),
            PopupSide::Left => (HorizontalOrigin::Right, vertical_origin(self.align())),
            PopupSide::Right => (HorizontalOrigin::Left, vertical_origin(self.align())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PopupSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum PopupAlign {
    Start,
    Center,
    End,
}

impl PopupAlign {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum HorizontalOrigin {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum VerticalOrigin {
    Top,
    Center,
    Bottom,
}

pub(super) fn has_popup_position_metadata(metadata: &UiTemplateNodeMetadata) -> bool {
    [
        "placement",
        "popup_anchor_x",
        "popup_anchor_y",
        "popup_anchor_width",
        "popup_anchor_height",
        "anchor_origin_vertical",
        "anchor_origin_horizontal",
        "transform_origin_vertical",
        "transform_origin_horizontal",
        "popup_offset_x",
        "popup_offset_y",
        "offset_x",
        "offset_y",
    ]
    .iter()
    .any(|key| metadata.attributes.contains_key(*key))
}

pub(super) fn popup_layout_bounds(
    owner_frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> Option<UiFrame> {
    clip_frame
        .filter(valid_bounds)
        .filter(|bounds| *bounds != owner_frame)
}

pub(super) fn popup_anchor_frame(
    metadata: &UiTemplateNodeMetadata,
    fallback_frame: UiFrame,
) -> UiFrame {
    let Some(x) = number_attribute(metadata, "popup_anchor_x") else {
        return fallback_frame;
    };
    let Some(y) = number_attribute(metadata, "popup_anchor_y") else {
        return fallback_frame;
    };
    let width = number_attribute(metadata, "popup_anchor_width")
        .unwrap_or(0.0)
        .max(0.0);
    let height = number_attribute(metadata, "popup_anchor_height")
        .unwrap_or(0.0)
        .max(0.0);
    if x.is_finite() && y.is_finite() && width.is_finite() && height.is_finite() {
        UiFrame::new(x, y, width, height)
    } else {
        fallback_frame
    }
}

pub(super) fn anchored_popup_frame(
    metadata: &UiTemplateNodeMetadata,
    anchor_frame: UiFrame,
    popup_width: f32,
    popup_height: f32,
    bounds: Option<UiFrame>,
    default_placement: PopupPlacement,
    placement_gap: f32,
) -> Option<UiFrame> {
    if popup_width <= 0.0
        || popup_height <= 0.0
        || !popup_width.is_finite()
        || !popup_height.is_finite()
    {
        return None;
    }

    let placement = PopupPlacement::from_metadata(metadata, default_placement);
    let width = bounds
        .map(|bounds| popup_width.min(bounds.width.max(1.0)))
        .unwrap_or(popup_width)
        .max(1.0);
    let height = popup_height.max(1.0);
    let mut frame = placement_candidate(
        metadata,
        anchor_frame,
        width,
        height,
        placement,
        placement_gap,
    );
    if let Some(bounds) = bounds.filter(valid_bounds) {
        let flipped_placement = placement.flipped();
        if overflows_primary_axis(frame, placement.side(), bounds) {
            let flipped = placement_candidate(
                metadata,
                anchor_frame,
                width,
                height,
                flipped_placement,
                placement_gap,
            );
            if fits_primary_axis(flipped, flipped_placement.side(), bounds) {
                frame = flipped;
            }
        }
        frame = clamp_to_bounds(frame, bounds);
    }
    Some(frame)
}

fn placement_candidate(
    metadata: &UiTemplateNodeMetadata,
    anchor_frame: UiFrame,
    popup_width: f32,
    popup_height: f32,
    placement: PopupPlacement,
    placement_gap: f32,
) -> UiFrame {
    let (default_anchor_horizontal, default_anchor_vertical) = placement.anchor_origin();
    let (default_transform_horizontal, default_transform_vertical) = placement.transform_origin();
    let anchor_horizontal = horizontal_origin_attribute(metadata, "anchor_origin_horizontal")
        .unwrap_or(default_anchor_horizontal);
    let anchor_vertical = vertical_origin_attribute(metadata, "anchor_origin_vertical")
        .unwrap_or(default_anchor_vertical);
    let transform_horizontal = horizontal_origin_attribute(metadata, "transform_origin_horizontal")
        .unwrap_or(default_transform_horizontal);
    let transform_vertical = vertical_origin_attribute(metadata, "transform_origin_vertical")
        .unwrap_or(default_transform_vertical);

    let anchor_x = origin_x(anchor_frame, anchor_horizontal);
    let anchor_y = origin_y(anchor_frame, anchor_vertical);
    let transform_x = transform_factor_x(transform_horizontal) * popup_width;
    let transform_y = transform_factor_y(transform_vertical) * popup_height;
    let offset_x = number_attribute(metadata, "popup_offset_x")
        .or_else(|| number_attribute(metadata, "offset_x"))
        .unwrap_or(0.0);
    let offset_y = number_attribute(metadata, "popup_offset_y")
        .or_else(|| number_attribute(metadata, "offset_y"))
        .unwrap_or(0.0);

    let mut x = anchor_x - transform_x + offset_x;
    let mut y = anchor_y - transform_y + offset_y;
    match placement.side() {
        PopupSide::Top => y -= placement_gap,
        PopupSide::Bottom => y += placement_gap,
        PopupSide::Left => x -= placement_gap,
        PopupSide::Right => x += placement_gap,
    }
    UiFrame::new(x, y, popup_width, popup_height)
}

fn overflows_primary_axis(frame: UiFrame, side: PopupSide, bounds: UiFrame) -> bool {
    match side {
        PopupSide::Top => frame.y < bounds.y,
        PopupSide::Bottom => frame.bottom() > bounds.bottom(),
        PopupSide::Left => frame.x < bounds.x,
        PopupSide::Right => frame.right() > bounds.right(),
    }
}

fn fits_primary_axis(frame: UiFrame, side: PopupSide, bounds: UiFrame) -> bool {
    match side {
        PopupSide::Top => frame.y >= bounds.y,
        PopupSide::Bottom => frame.bottom() <= bounds.bottom(),
        PopupSide::Left => frame.x >= bounds.x,
        PopupSide::Right => frame.right() <= bounds.right(),
    }
}

fn clamp_to_bounds(frame: UiFrame, bounds: UiFrame) -> UiFrame {
    let width = frame.width.min(bounds.width.max(1.0)).max(1.0);
    let max_x = (bounds.x + bounds.width - width).max(bounds.x);
    let max_y = (bounds.y + bounds.height - frame.height).max(bounds.y);
    UiFrame::new(
        frame.x.clamp(bounds.x, max_x),
        frame.y.clamp(bounds.y, max_y),
        width,
        frame.height,
    )
}

fn horizontal_origin(align: PopupAlign) -> HorizontalOrigin {
    match align {
        PopupAlign::Start => HorizontalOrigin::Left,
        PopupAlign::Center => HorizontalOrigin::Center,
        PopupAlign::End => HorizontalOrigin::Right,
    }
}

fn vertical_origin(align: PopupAlign) -> VerticalOrigin {
    match align {
        PopupAlign::Start => VerticalOrigin::Top,
        PopupAlign::Center => VerticalOrigin::Center,
        PopupAlign::End => VerticalOrigin::Bottom,
    }
}

fn origin_x(frame: UiFrame, origin: HorizontalOrigin) -> f32 {
    match origin {
        HorizontalOrigin::Left => frame.x,
        HorizontalOrigin::Center => frame.x + frame.width * 0.5,
        HorizontalOrigin::Right => frame.right(),
    }
}

fn origin_y(frame: UiFrame, origin: VerticalOrigin) -> f32 {
    match origin {
        VerticalOrigin::Top => frame.y,
        VerticalOrigin::Center => frame.y + frame.height * 0.5,
        VerticalOrigin::Bottom => frame.bottom(),
    }
}

fn transform_factor_x(origin: HorizontalOrigin) -> f32 {
    match origin {
        HorizontalOrigin::Left => 0.0,
        HorizontalOrigin::Center => 0.5,
        HorizontalOrigin::Right => 1.0,
    }
}

fn transform_factor_y(origin: VerticalOrigin) -> f32 {
    match origin {
        VerticalOrigin::Top => 0.0,
        VerticalOrigin::Center => 0.5,
        VerticalOrigin::Bottom => 1.0,
    }
}

fn horizontal_origin_attribute(
    metadata: &UiTemplateNodeMetadata,
    key: &str,
) -> Option<HorizontalOrigin> {
    match string_attribute(metadata, key)?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "left" | "start" => Some(HorizontalOrigin::Left),
        "center" | "middle" => Some(HorizontalOrigin::Center),
        "right" | "end" => Some(HorizontalOrigin::Right),
        _ => None,
    }
}

fn vertical_origin_attribute(
    metadata: &UiTemplateNodeMetadata,
    key: &str,
) -> Option<VerticalOrigin> {
    match string_attribute(metadata, key)?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "top" | "start" => Some(VerticalOrigin::Top),
        "center" | "middle" => Some(VerticalOrigin::Center),
        "bottom" | "end" => Some(VerticalOrigin::Bottom),
        _ => None,
    }
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    let value = metadata.attributes.get(key)?;
    match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    }
    .filter(|value| value.is_finite())
}

fn valid_bounds(frame: &UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
