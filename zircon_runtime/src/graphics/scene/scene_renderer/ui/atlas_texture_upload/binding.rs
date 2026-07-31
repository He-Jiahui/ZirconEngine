use crate::text::atlas::{GlyphAtlasBitmapPageUploadStaging, GlyphAtlasBitmapTextureUploadRequest};

use super::write::{GlyphAtlasTextureUploadWrite, write_glyph_atlas_texture_upload_bytes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GlyphAtlasBitmapTextureUploadBinding<'a> {
    pub(super) request_index: usize,
    pub(super) write: GlyphAtlasTextureUploadWrite,
    pub(super) bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GlyphAtlasBitmapTextureUploadBindingFailureReason {
    MissingStagingPage,
    StagingPageKeyMismatch,
    StagingPageGenerationMismatch,
    StagingPageTargetRectMismatch,
    StagingPageRowStrideMismatch,
    StagingPageByteLengthMismatch,
    RequestRangeOutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GlyphAtlasBitmapTextureUploadBindingFailure {
    pub(super) request_index: usize,
    pub(super) reason: GlyphAtlasBitmapTextureUploadBindingFailureReason,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct GlyphAtlasBitmapTextureUploadBindingPlan<'a> {
    pub(super) bindings: Vec<GlyphAtlasBitmapTextureUploadBinding<'a>>,
    pub(super) failures: Vec<GlyphAtlasBitmapTextureUploadBindingFailure>,
}

impl GlyphAtlasBitmapTextureUploadBindingPlan<'_> {
    pub(super) fn has_bindings(&self) -> bool {
        !self.bindings.is_empty()
    }

    pub(super) fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
}

pub(super) fn glyph_atlas_bitmap_texture_upload_binding_plan<'a>(
    staging_pages: &'a [GlyphAtlasBitmapPageUploadStaging],
    requests: &[GlyphAtlasBitmapTextureUploadRequest],
) -> GlyphAtlasBitmapTextureUploadBindingPlan<'a> {
    let mut bindings = Vec::new();
    let mut failures = Vec::new();

    for (request_index, request) in requests.iter().copied().enumerate() {
        let Some(staging_page) = staging_pages.get(request.staging_page_index) else {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::MissingStagingPage,
            ));
            continue;
        };
        if staging_page.page_key != request.page_key {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageKeyMismatch,
            ));
            continue;
        }
        if staging_page.page_generation != request.page_generation {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageGenerationMismatch,
            ));
            continue;
        }
        if staging_page.target_rect.x != request.origin_xy.x
            || staging_page.target_rect.y != request.origin_xy.y
            || staging_page.target_rect.width != request.extent.x
            || staging_page.target_rect.height != request.extent.y
        {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageTargetRectMismatch,
            ));
            continue;
        }
        if staging_page.bytes_per_row != request.bytes_per_row {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageRowStrideMismatch,
            ));
            continue;
        }
        if staging_page.bytes.len() != request.staging_page_byte_len {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageByteLengthMismatch,
            ));
            continue;
        }
        if !bitmap_upload_request_range_fits(request, staging_page.bytes.len()) {
            failures.push(bitmap_upload_binding_failure(
                request_index,
                GlyphAtlasBitmapTextureUploadBindingFailureReason::RequestRangeOutOfBounds,
            ));
            continue;
        }

        bindings.push(GlyphAtlasBitmapTextureUploadBinding {
            request_index,
            write: glyph_atlas_texture_upload_write_for_bitmap_request(request),
            bytes: staging_page.bytes.as_slice(),
        });
    }

    GlyphAtlasBitmapTextureUploadBindingPlan { bindings, failures }
}

pub(super) fn write_glyph_atlas_bitmap_texture_upload_bindings(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    bindings: &[GlyphAtlasBitmapTextureUploadBinding<'_>],
) {
    for binding in bindings {
        write_glyph_atlas_texture_upload_bytes(queue, texture, binding.bytes, binding.write);
    }
}

fn glyph_atlas_texture_upload_write_for_bitmap_request(
    request: GlyphAtlasBitmapTextureUploadRequest,
) -> GlyphAtlasTextureUploadWrite {
    GlyphAtlasTextureUploadWrite {
        origin_x: request.origin_xy.x,
        origin_y: request.origin_xy.y,
        origin_layer: request.origin_layer,
        source_offset: request.source_offset,
        bytes_per_row: request.bytes_per_row,
        rows_per_image: request.rows_per_image,
        extent_width: request.extent.x,
        extent_height: request.extent.y,
        extent_layers: 1,
    }
}

fn bitmap_upload_binding_failure(
    request_index: usize,
    reason: GlyphAtlasBitmapTextureUploadBindingFailureReason,
) -> GlyphAtlasBitmapTextureUploadBindingFailure {
    GlyphAtlasBitmapTextureUploadBindingFailure {
        request_index,
        reason,
    }
}

fn bitmap_upload_request_range_fits(
    request: GlyphAtlasBitmapTextureUploadRequest,
    staging_page_byte_len: usize,
) -> bool {
    if request.extent.x == 0
        || request.extent.y == 0
        || request.upload_byte_len == 0
        || request.rows_per_image < request.extent.y
    {
        return false;
    }

    let row_count = request.extent.y as usize;
    if request.upload_byte_len % row_count != 0 {
        return false;
    }
    let row_copy_byte_len = request.upload_byte_len / row_count;
    let last_row_index = u64::from(request.extent.y.saturating_sub(1));
    let Some(last_row_offset) = last_row_index
        .checked_mul(u64::from(request.bytes_per_row))
        .and_then(|offset| request.source_offset.checked_add(offset))
    else {
        return false;
    };
    let Some(source_end) = last_row_offset.checked_add(row_copy_byte_len as u64) else {
        return false;
    };
    source_end <= staging_page_byte_len as u64
}
