use super::super::super::super::data::FrameRect;
use super::style::WorkbenchScrollbarMetrics;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WorkbenchScrollbarGeometry {
    pub track: FrameRect,
    pub thumb: FrameRect,
}

pub(super) fn vertical_scrollbar_geometry(
    viewport: &FrameRect,
    scroll_offset: f32,
    content_extent: f32,
    metrics: WorkbenchScrollbarMetrics,
) -> Option<WorkbenchScrollbarGeometry> {
    let viewport_extent = viewport.height.max(0.0);
    let content_extent = content_extent.max(0.0);
    if viewport.width <= 0.0
        || viewport_extent <= 0.0
        || content_extent <= viewport_extent
        || !viewport.width.is_finite()
        || !viewport.height.is_finite()
        || !content_extent.is_finite()
    {
        return None;
    }

    let inset = metrics
        .track_inset
        .max(0.0)
        .min((viewport.width * 0.25).max(0.0))
        .min((viewport_extent * 0.25).max(0.0));
    let thickness = metrics
        .thickness
        .max(0.0)
        .min((viewport.width - inset * 2.0).max(0.0));
    let track_height = (viewport_extent - inset * 2.0).max(0.0);
    if thickness <= 0.0 || track_height <= 0.0 {
        return None;
    }

    let track = FrameRect {
        x: viewport.x + viewport.width - inset - thickness,
        y: viewport.y + inset,
        width: thickness,
        height: track_height,
    };
    let proportional_thumb = track_height * (viewport_extent / content_extent);
    let min_thumb = metrics.min_thumb_length.max(0.0).min(track_height);
    let thumb_height = proportional_thumb.max(min_thumb).min(track_height);
    let max_scroll = (content_extent - viewport_extent).max(0.0);
    let travel = (track_height - thumb_height).max(0.0);
    let thumb_y = if max_scroll > 0.0 {
        track.y + (scroll_offset.max(0.0).min(max_scroll) / max_scroll) * travel
    } else {
        track.y
    };

    Some(WorkbenchScrollbarGeometry {
        track,
        thumb: FrameRect {
            x: viewport.x + viewport.width - inset - thickness,
            y: thumb_y,
            width: thickness,
            height: thumb_height,
        },
    })
}
