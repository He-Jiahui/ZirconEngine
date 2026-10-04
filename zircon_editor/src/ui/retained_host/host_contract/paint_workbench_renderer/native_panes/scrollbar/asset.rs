use crate::ui::workbench::asset_content_layout::{
    AssetContentRect, AssetContentScrollbarExtent, AssetContentScrollbarViewport,
};

use super::super::super::super::data::FrameRect;

pub(super) fn asset_tree_viewport_frame(body: &FrameRect) -> FrameRect {
    let viewport_y = crate::ui::retained_host::asset_pointer::asset_tree_viewport_y();
    FrameRect {
        x: body.x,
        y: body.y + viewport_y,
        width: body.width,
        height: (body.height - viewport_y).max(0.0),
    }
}

pub(super) fn asset_scrollbar_viewport(
    viewport: AssetContentScrollbarViewport,
    body: &FrameRect,
) -> FrameRect {
    match viewport {
        AssetContentScrollbarViewport::ActivityTree => asset_tree_viewport_frame(body),
        AssetContentScrollbarViewport::Local(viewport) => {
            translated_asset_content_rect(viewport, body)
        }
    }
}

pub(super) fn asset_scrollbar_content_extent(extent: AssetContentScrollbarExtent) -> f32 {
    match extent {
        AssetContentScrollbarExtent::Pixels(extent) => extent,
        AssetContentScrollbarExtent::TreeRows(row_count) => {
            crate::ui::retained_host::asset_pointer::asset_tree_content_height(row_count)
        }
        AssetContentScrollbarExtent::ReferenceRows(row_count) => {
            crate::ui::retained_host::asset_pointer::asset_reference_content_height(row_count)
        }
    }
}

fn translated_asset_content_rect(rect: AssetContentRect, body: &FrameRect) -> FrameRect {
    FrameRect {
        x: body.x + rect.x,
        y: body.y + rect.y,
        width: rect.width,
        height: rect.height,
    }
}

#[cfg(test)]
#[path = "tests/asset.rs"]
mod tests;
