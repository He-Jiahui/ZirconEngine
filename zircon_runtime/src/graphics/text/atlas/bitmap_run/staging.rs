use std::collections::BTreeMap;

use super::super::GlyphAtlasPageKey;
use super::types::{GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapUploadCopy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapUploadSourceBytes<'a> {
    pub(crate) source_index: usize,
    pub(crate) bytes: &'a [u8],
}

impl<'a> GlyphAtlasBitmapUploadSourceBytes<'a> {
    pub(crate) fn new(source_index: usize, bytes: &'a [u8]) -> Self {
        Self {
            source_index,
            bytes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageUploadStaging {
    pub(crate) page_key: GlyphAtlasPageKey,
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
    let mut pages_by_key = BTreeMap::<GlyphAtlasPageKey, GlyphAtlasBitmapPageUploadStaging>::new();
    let mut failures = Vec::new();

    for copy in &run.upload_copies {
        let Some(page) = run
            .atlas
            .page(copy.page_key.format, copy.page_key.page_index)
        else {
            failures.push(staging_failure(
                copy,
                GlyphAtlasBitmapUploadStagingFailureReason::MissingPage,
            ));
            continue;
        };
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

        let bytes_per_pixel = page.storage_format.bytes_per_pixel();
        let page_byte_len = (page.size.x as usize)
            .saturating_mul(page.size.y as usize)
            .saturating_mul(bytes_per_pixel as usize);
        let page_staging = pages_by_key.entry(copy.page_key).or_insert_with(|| {
            GlyphAtlasBitmapPageUploadStaging {
                page_key: copy.page_key,
                bytes_per_row: page.size.x.saturating_mul(bytes_per_pixel),
                bytes: vec![0; page_byte_len],
            }
        });
        copy_upload_source_bytes(page_staging, copy, bytes, &mut failures);
    }

    GlyphAtlasBitmapUploadStagingPlan {
        pages: pages_by_key.into_values().collect(),
        failures,
    }
}

fn copy_upload_source_bytes(
    page_staging: &mut GlyphAtlasBitmapPageUploadStaging,
    copy: &GlyphAtlasBitmapUploadCopy,
    source_bytes: &[u8],
    failures: &mut Vec<GlyphAtlasBitmapUploadStagingFailure>,
) {
    let Some(atlas_byte_offset) = usize::try_from(copy.atlas_byte_offset).ok() else {
        failures.push(staging_failure(
            copy,
            GlyphAtlasBitmapUploadStagingFailureReason::DestinationRangeOutOfBounds,
        ));
        return;
    };
    let source_bytes_per_row = copy.source_bytes_per_row as usize;
    let atlas_bytes_per_row = copy.atlas_bytes_per_row as usize;
    let mut row_ranges = Vec::with_capacity(copy.content_size.y as usize);

    for row in 0..copy.content_size.y as usize {
        let source_start = row.saturating_mul(source_bytes_per_row);
        let source_end = source_start.saturating_add(source_bytes_per_row);
        if source_end > source_bytes.len() {
            failures.push(staging_failure(
                copy,
                GlyphAtlasBitmapUploadStagingFailureReason::SourceRangeOutOfBounds,
            ));
            return;
        }

        let destination_start =
            atlas_byte_offset.saturating_add(row.saturating_mul(atlas_bytes_per_row));
        let destination_end = destination_start.saturating_add(source_bytes_per_row);
        if destination_end > page_staging.bytes.len() {
            failures.push(staging_failure(
                copy,
                GlyphAtlasBitmapUploadStagingFailureReason::DestinationRangeOutOfBounds,
            ));
            return;
        }

        row_ranges.push((source_start, source_end, destination_start, destination_end));
    }

    for (source_start, source_end, destination_start, destination_end) in row_ranges {
        page_staging.bytes[destination_start..destination_end]
            .copy_from_slice(&source_bytes[source_start..source_end]);
    }
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
