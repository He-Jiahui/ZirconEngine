use std::collections::BTreeMap;

use super::super::{GlyphAtlasPageKey, GlyphAtlasRect};
use super::types::{GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapUploadCopy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapUploadSourceBytes<'a> {
    pub(crate) source_index: usize,
    pub(crate) bytes: &'a [u8],
    pub(crate) face_epoch: u64,
}

impl<'a> GlyphAtlasBitmapUploadSourceBytes<'a> {
    pub(crate) fn new(source_index: usize, bytes: &'a [u8]) -> Self {
        Self {
            source_index,
            bytes,
            face_epoch: 0,
        }
    }

    pub(crate) fn with_face_epoch(source_index: usize, bytes: &'a [u8], face_epoch: u64) -> Self {
        Self {
            source_index,
            bytes,
            face_epoch,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageUploadStaging {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) page_generation: u64,
    pub(crate) target_rect: GlyphAtlasRect,
    pub(crate) bytes_per_row: u32,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapUploadStagingFailureReason {
    MissingPage,
    MissingSourceBytes,
    SourceLengthMismatch { expected: usize, actual: usize },
    SourceRangeOutOfBounds,
    DestinationRangeOutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapUploadStagingFailure {
    pub(crate) source_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) reason: GlyphAtlasBitmapUploadStagingFailureReason,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapUploadStagingPlan {
    pub(crate) pages: Vec<GlyphAtlasBitmapPageUploadStaging>,
    pub(crate) failures: Vec<GlyphAtlasBitmapUploadStagingFailure>,
}

impl GlyphAtlasBitmapUploadStagingPlan {
    pub(crate) fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
}

pub(crate) fn glyph_atlas_bitmap_upload_staging_plan<'a, I>(
    run: &GlyphAtlasBitmapRunPlan,
    source_bytes: I,
) -> GlyphAtlasBitmapUploadStagingPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
{
    let source_bytes_by_index = source_bytes
        .into_iter()
        .map(|source| (source.source_index, source.bytes))
        .collect::<BTreeMap<_, _>>();
    let mut pages = Vec::new();
    let mut failures = Vec::new();

    for command in &run.upload_commands {
        let copies = run
            .upload_copies
            .iter()
            .filter(|copy| {
                copy.page_key == command.page_key
                    && target_rect_contains(command.rect, copy.atlas_rect)
            })
            .collect::<Vec<_>>();
        if copies.is_empty() {
            continue;
        }
        let Some(page) = run
            .atlas
            .page(command.page_key.format, command.page_key.page_index)
        else {
            for copy in copies {
                failures.push(staging_failure(
                    copy,
                    GlyphAtlasBitmapUploadStagingFailureReason::MissingPage,
                ));
            }
            continue;
        };
        let bytes_per_pixel = page.storage_format.bytes_per_pixel();
        let mut page_staging = GlyphAtlasBitmapPageUploadStaging {
            page_key: command.page_key,
            page_generation: page.generation,
            target_rect: command.rect,
            bytes_per_row: command.rect.width.saturating_mul(bytes_per_pixel),
            bytes: vec![
                0;
                (command.rect.width as usize)
                    .saturating_mul(command.rect.height as usize)
                    .saturating_mul(bytes_per_pixel as usize)
            ],
        };
        if let Some(shadow_bytes) = run.atlas.bitmap_page_shadow_bytes(page) {
            seed_staging_from_page_shadow(&mut page_staging, page, shadow_bytes);
        }
        for copy in copies {
            let Some(bytes) = source_bytes_by_index.get(&copy.source_index) else {
                failures.push(staging_failure(
                    copy,
                    GlyphAtlasBitmapUploadStagingFailureReason::MissingSourceBytes,
                ));
                continue;
            };
            if bytes.len() != copy.source_byte_len {
                failures.push(staging_failure(
                    copy,
                    GlyphAtlasBitmapUploadStagingFailureReason::SourceLengthMismatch {
                        expected: copy.source_byte_len,
                        actual: bytes.len(),
                    },
                ));
                continue;
            }

            copy_upload_source_bytes(&mut page_staging, command.rect, copy, bytes, &mut failures);
        }
        pages.push(page_staging);
    }

    GlyphAtlasBitmapUploadStagingPlan { pages, failures }
}

fn seed_staging_from_page_shadow(
    page_staging: &mut GlyphAtlasBitmapPageUploadStaging,
    page: &super::super::GlyphAtlasPageSpec,
    shadow_bytes: &[u8],
) {
    let bytes_per_pixel = page.storage_format.bytes_per_pixel() as usize;
    let page_bytes_per_row = page.size.x as usize * bytes_per_pixel;
    let target = page_staging.target_rect;
    let target_row_byte_len = target.width as usize * bytes_per_pixel;
    if shadow_bytes.len() != page.byte_len()
        || page_staging.bytes_per_row as usize != target_row_byte_len
    {
        return;
    }

    for row in 0..target.height as usize {
        let source_start = (target.y as usize + row)
            .saturating_mul(page_bytes_per_row)
            .saturating_add(target.x as usize * bytes_per_pixel);
        let source_end = source_start.saturating_add(target_row_byte_len);
        let destination_start = row.saturating_mul(target_row_byte_len);
        let destination_end = destination_start.saturating_add(target_row_byte_len);
        page_staging.bytes[destination_start..destination_end]
            .copy_from_slice(&shadow_bytes[source_start..source_end]);
    }
}

fn copy_upload_source_bytes(
    page_staging: &mut GlyphAtlasBitmapPageUploadStaging,
    target_rect: GlyphAtlasRect,
    copy: &GlyphAtlasBitmapUploadCopy,
    source_bytes: &[u8],
    failures: &mut Vec<GlyphAtlasBitmapUploadStagingFailure>,
) {
    if !target_rect_contains(target_rect, copy.atlas_rect) {
        failures.push(staging_failure(
            copy,
            GlyphAtlasBitmapUploadStagingFailureReason::DestinationRangeOutOfBounds,
        ));
        return;
    }
    let source_bytes_per_row = copy.source_bytes_per_row as usize;
    let row_count = copy.content_size.y as usize;
    let Some(bytes_per_pixel) = usize::try_from(copy.content_size.x)
        .ok()
        .and_then(|width| source_bytes_per_row.checked_div(width))
        .filter(|bytes_per_pixel| *bytes_per_pixel > 0)
    else {
        failures.push(staging_failure(
            copy,
            GlyphAtlasBitmapUploadStagingFailureReason::DestinationRangeOutOfBounds,
        ));
        return;
    };
    let local_x = copy.atlas_rect.x.saturating_sub(target_rect.x) as usize;
    let local_y = copy.atlas_rect.y.saturating_sub(target_rect.y) as usize;
    let local_byte_offset = local_y
        .saturating_mul(page_staging.bytes_per_row as usize)
        .saturating_add(local_x.saturating_mul(bytes_per_pixel));

    if let Some(last_row) = row_count.checked_sub(1) {
        let source_end = last_row
            .saturating_mul(source_bytes_per_row)
            .saturating_add(source_bytes_per_row);
        if source_end > source_bytes.len() {
            failures.push(staging_failure(
                copy,
                GlyphAtlasBitmapUploadStagingFailureReason::SourceRangeOutOfBounds,
            ));
            return;
        }

        let destination_end = local_byte_offset
            .saturating_add(last_row.saturating_mul(page_staging.bytes_per_row as usize))
            .saturating_add(source_bytes_per_row);
        if destination_end > page_staging.bytes.len() {
            failures.push(staging_failure(
                copy,
                GlyphAtlasBitmapUploadStagingFailureReason::DestinationRangeOutOfBounds,
            ));
            return;
        }
    }

    for row in 0..row_count {
        let source_start = row * source_bytes_per_row;
        let source_end = source_start + source_bytes_per_row;
        let destination_start = local_byte_offset + row * page_staging.bytes_per_row as usize;
        let destination_end = destination_start + source_bytes_per_row;
        page_staging.bytes[destination_start..destination_end]
            .copy_from_slice(&source_bytes[source_start..source_end]);
    }
}

fn target_rect_contains(target: GlyphAtlasRect, rect: GlyphAtlasRect) -> bool {
    rect.x >= target.x
        && rect.y >= target.y
        && rect.x.saturating_add(rect.width) <= target.x.saturating_add(target.width)
        && rect.y.saturating_add(rect.height) <= target.y.saturating_add(target.height)
}

fn staging_failure(
    copy: &GlyphAtlasBitmapUploadCopy,
    reason: GlyphAtlasBitmapUploadStagingFailureReason,
) -> GlyphAtlasBitmapUploadStagingFailure {
    GlyphAtlasBitmapUploadStagingFailure {
        source_index: copy.source_index,
        page_key: copy.page_key,
        reason,
    }
}
