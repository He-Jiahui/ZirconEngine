use std::collections::HashMap;

use crate::asset::UiIconAsset;
use crate::ui::icon_atlas::{parse_ui_svg_icon_cached, UiSvgIconDocument, UiSvgIconParseError};

const DEFAULT_ATLAS_SLOT_PADDING_PX: u32 = 2;
const DEFAULT_ATLAS_MIN_SIDE_PX: u32 = 64;
const DEFAULT_ATLAS_MAX_SIDE_PX: u32 = 4096;

#[derive(Clone, Debug, PartialEq)]
pub struct UiIconRasterRequest {
    pub icon_id: String,
    pub asset: UiIconAsset,
    pub dpi_scale: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiIconAtlasPlan {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub slots: Vec<UiIconAtlasSlot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiIconAtlasSlot {
    pub icon_id: String,
    pub semantic_id: String,
    pub rect: UiIconAtlasRect,
    pub uv: UiIconAtlasUvRect,
    pub pixel_size: u32,
    pub svg: Option<UiSvgIconDocument>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiIconAtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiIconAtlasUvRect {
    pub min_u: f32,
    pub min_v: f32,
    pub max_u: f32,
    pub max_v: f32,
}

#[derive(Clone, Debug)]
pub struct UiIconAtlasBuilder {
    padding_px: u32,
    min_side_px: u32,
    max_side_px: u32,
}

impl Default for UiIconAtlasBuilder {
    fn default() -> Self {
        Self {
            padding_px: DEFAULT_ATLAS_SLOT_PADDING_PX,
            min_side_px: DEFAULT_ATLAS_MIN_SIDE_PX,
            max_side_px: DEFAULT_ATLAS_MAX_SIDE_PX,
        }
    }
}

impl UiIconAtlasBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_padding_px(mut self, padding_px: u32) -> Self {
        self.padding_px = padding_px;
        self
    }

    pub fn with_min_side_px(mut self, min_side_px: u32) -> Self {
        self.min_side_px = min_side_px.max(1);
        self
    }

    pub fn build_plan(
        &self,
        requests: impl IntoIterator<Item = UiIconRasterRequest>,
    ) -> Result<UiIconAtlasPlan, UiSvgIconParseError> {
        let requests = deduplicate_requests(requests);
        if requests.is_empty() {
            return Ok(UiIconAtlasPlan::default());
        }

        let mut pending = requests
            .into_iter()
            .map(|request| {
                let pixel_size = icon_pixel_size(&request.asset, request.dpi_scale);
                let svg = request
                    .asset
                    .source
                    .text
                    .as_deref()
                    .map(parse_ui_svg_icon_cached)
                    .transpose()?;
                Ok(PendingIconSlot {
                    request,
                    pixel_size,
                    svg,
                })
            })
            .collect::<Result<Vec<_>, UiSvgIconParseError>>()?;
        pending.sort_by(|a, b| {
            a.request.icon_id.cmp(&b.request.icon_id).then_with(|| {
                a.request
                    .asset
                    .semantic_id
                    .cmp(&b.request.asset.semantic_id)
            })
        });

        let cell_size = pending
            .iter()
            .map(|slot| {
                slot.pixel_size
                    .saturating_add(self.padding_px.saturating_mul(2))
            })
            .max()
            .unwrap_or(1)
            .max(1);
        let columns = square_grid_columns(pending.len());
        let atlas_width = (cell_size * columns as u32)
            .max(self.min_side_px)
            .min(self.max_side_px);
        let rows = pending.len().div_ceil(columns);
        let atlas_height = (cell_size * rows as u32)
            .max(self.min_side_px)
            .min(self.max_side_px);

        let slots = pending
            .into_iter()
            .enumerate()
            .map(|(index, pending)| {
                let column = index % columns;
                let row = index / columns;
                let x = column as u32 * cell_size + self.padding_px;
                let y = row as u32 * cell_size + self.padding_px;
                let rect = UiIconAtlasRect {
                    x,
                    y,
                    width: pending.pixel_size,
                    height: pending.pixel_size,
                };
                UiIconAtlasSlot {
                    icon_id: pending.request.icon_id,
                    semantic_id: pending.request.asset.semantic_id,
                    rect,
                    uv: uv_rect(rect, atlas_width, atlas_height),
                    pixel_size: pending.pixel_size,
                    svg: pending.svg,
                }
            })
            .collect();

        Ok(UiIconAtlasPlan {
            atlas_width,
            atlas_height,
            slots,
        })
    }
}

struct PendingIconSlot {
    request: UiIconRasterRequest,
    pixel_size: u32,
    svg: Option<UiSvgIconDocument>,
}

fn deduplicate_requests(
    requests: impl IntoIterator<Item = UiIconRasterRequest>,
) -> Vec<UiIconRasterRequest> {
    let mut by_icon = HashMap::new();
    for request in requests {
        by_icon.entry(request.icon_id.clone()).or_insert(request);
    }
    by_icon.into_values().collect()
}

fn icon_pixel_size(asset: &UiIconAsset, dpi_scale: f32) -> u32 {
    let scale = if dpi_scale.is_finite() && dpi_scale > 0.0 {
        // Icon bitmaps are resolution-dependent UI resources. Never let a producer request a
        // below-native raster; values above one remain available for deliberate supersampling.
        dpi_scale.max(1.0)
    } else {
        1.0
    };
    (asset.default_size.max(1.0) * scale).ceil().max(1.0) as u32
}

fn square_grid_columns(slot_count: usize) -> usize {
    (slot_count as f32).sqrt().ceil().max(1.0) as usize
}

fn uv_rect(rect: UiIconAtlasRect, atlas_width: u32, atlas_height: u32) -> UiIconAtlasUvRect {
    let atlas_width = atlas_width.max(1) as f32;
    let atlas_height = atlas_height.max(1) as f32;
    UiIconAtlasUvRect {
        min_u: rect.x as f32 / atlas_width,
        min_v: rect.y as f32 / atlas_height,
        max_u: rect.x.saturating_add(rect.width) as f32 / atlas_width,
        max_v: rect.y.saturating_add(rect.height) as f32 / atlas_height,
    }
}

#[cfg(test)]
#[path = "atlas/hash_dedup_tests.rs"]
mod hash_dedup_tests;
