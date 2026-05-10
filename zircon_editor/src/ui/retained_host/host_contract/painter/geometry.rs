use super::super::data::{FrameRect, TemplateNodeFrameData};

pub(super) fn frame_or(frame: &FrameRect, fallback: FrameRect) -> FrameRect {
    if is_visible_frame(frame) {
        frame.clone()
    } else {
        fallback
    }
}

pub(super) fn is_visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.5
        && frame.height > 0.5
}

pub(super) fn frame_from_template(frame: &TemplateNodeFrameData) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn translated(frame: &FrameRect, origin_x: f32, origin_y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn intersect(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    if !is_visible_frame(left) || !is_visible_frame(right) {
        return None;
    }

    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    let width = x1 - x0;
    let height = y1 - y0;
    (width > 0.5 && height > 0.5).then_some(FrameRect {
        x: x0,
        y: y0,
        width,
        height,
    })
}

pub(super) fn inset(frame: &FrameRect, amount: f32) -> FrameRect {
    let amount = amount.max(0.0);
    FrameRect {
        x: frame.x + amount,
        y: frame.y + amount,
        width: (frame.width - amount * 2.0).max(0.0),
        height: (frame.height - amount * 2.0).max(0.0),
    }
}

pub(super) struct PixelRect {
    pub(super) x0: u32,
    pub(super) y0: u32,
    pub(super) x1: u32,
    pub(super) y1: u32,
}

impl PixelRect {
    pub(super) fn from_frame(
        frame: &FrameRect,
        clip: Option<&FrameRect>,
        width: u32,
        height: u32,
    ) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let frame = match clip {
            Some(clip) => intersect(frame, clip)?,
            None if is_visible_frame(frame) => frame.clone(),
            None => return None,
        };

        let x0 = frame.x.floor().max(0.0).min(width as f32) as u32;
        let y0 = frame.y.floor().max(0.0).min(height as f32) as u32;
        let x1 = (frame.x + frame.width).ceil().max(0.0).min(width as f32) as u32;
        let y1 = (frame.y + frame.height).ceil().max(0.0).min(height as f32) as u32;
        (x0 < x1 && y0 < y1).then_some(Self { x0, y0, x1, y1 })
    }
}
