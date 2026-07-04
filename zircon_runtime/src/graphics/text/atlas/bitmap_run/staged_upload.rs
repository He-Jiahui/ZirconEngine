use std::collections::BTreeMap;

use crate::core::math::UVec2;

use super::super::{GlyphAtlasPageKey, GlyphAtlasUploadCommand, GlyphAtlasUploadMode};
use super::staging::{
    glyph_atlas_bitmap_upload_staging_plan, GlyphAtlasBitmapPageUploadStaging,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasBitmapUploadStagingPlan,
};
use super::types::GlyphAtlasBitmapRunPlan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapStagedUpload {
    pub(crate) staging_page_index: usize,
    pub(crate) command: GlyphAtlasUploadCommand,
    pub(crate) staging_page_byte_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapStagedUploadFailureReason {
    MissingStagingPage,
    SourceRangeOutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapStagedUploadFailure {
    pub(crate) upload_command_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) reason: GlyphAtlasBitmapStagedUploadFailureReason,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapStagedUploadPlan {
    pub(crate) uploads: Vec<GlyphAtlasBitmapStagedUpload>,
    pub(crate) failures: Vec<GlyphAtlasBitmapStagedUploadFailure>,
}

impl GlyphAtlasBitmapStagedUploadPlan {
    pub(crate) fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPreparedUploadPlan {
    pub(crate) staging: GlyphAtlasBitmapUploadStagingPlan,
    pub(crate) staged_uploads: GlyphAtlasBitmapStagedUploadPlan,
}

impl GlyphAtlasBitmapPreparedUploadPlan {
    pub(crate) fn has_failures(&self) -> bool {
        self.staging.has_failures() || self.staged_uploads.has_failures()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapTextureUploadRequest {
    pub(crate) staging_page_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) origin_xy: UVec2,
    pub(crate) origin_layer: u32,
    pub(crate) extent: UVec2,
    pub(crate) source_offset: u64,
    pub(crate) bytes_per_row: u32,
    pub(crate) rows_per_image: u32,
    pub(crate) upload_byte_len: usize,
    pub(crate) staging_page_byte_len: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapTextureUploadRequestPlan {
    pub(crate) requests: Vec<GlyphAtlasBitmapTextureUploadRequest>,
    pub(crate) skipped_failure_count: usize,
}

impl GlyphAtlasBitmapTextureUploadRequestPlan {
    pub(crate) fn has_requests(&self) -> bool {
        !self.requests.is_empty()
    }
}

pub(crate) fn glyph_atlas_bitmap_prepared_upload_plan<'a, I>(
    run: &GlyphAtlasBitmapRunPlan,
    source_bytes: I,
) -> GlyphAtlasBitmapPreparedUploadPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
{
    let staging = glyph_atlas_bitmap_upload_staging_plan(run, source_bytes);
    let staged_uploads = if staging.has_failures() {
        GlyphAtlasBitmapStagedUploadPlan::default()
    } else {
        glyph_atlas_bitmap_staged_upload_plan(&staging, &run.upload_commands)
    };

    GlyphAtlasBitmapPreparedUploadPlan {
        staging,
        staged_uploads,
    }
}

pub(crate) fn glyph_atlas_bitmap_texture_upload_request_plan(
    staged_uploads: &GlyphAtlasBitmapStagedUploadPlan,
) -> GlyphAtlasBitmapTextureUploadRequestPlan {
    let requests = staged_uploads
        .uploads
        .iter()
        .copied()
        .map(glyph_atlas_bitmap_texture_upload_request)
        .collect();

    GlyphAtlasBitmapTextureUploadRequestPlan {
        requests,
        skipped_failure_count: staged_uploads.failures.len(),
    }
}

pub(crate) fn glyph_atlas_bitmap_staged_upload_plan(
    staging: &GlyphAtlasBitmapUploadStagingPlan,
    upload_commands: &[GlyphAtlasUploadCommand],
) -> GlyphAtlasBitmapStagedUploadPlan {
    let pages_by_key = staging
        .pages
        .iter()
        .enumerate()
        .map(|(index, page)| (page.page_key, (index, page)))
        .collect::<BTreeMap<_, _>>();
    let mut uploads = Vec::new();
    let mut failures = Vec::new();

    for (upload_command_index, command) in upload_commands.iter().copied().enumerate() {
        let Some((staging_page_index, page)) = pages_by_key.get(&command.page_key).copied() else {
            failures.push(staged_upload_failure(
                upload_command_index,
                command.page_key,
                GlyphAtlasBitmapStagedUploadFailureReason::MissingStagingPage,
            ));
            continue;
        };

        if !staged_upload_source_range_fits(command, page) {
            failures.push(staged_upload_failure(
                upload_command_index,
                command.page_key,
                GlyphAtlasBitmapStagedUploadFailureReason::SourceRangeOutOfBounds,
            ));
            continue;
        }

        uploads.push(GlyphAtlasBitmapStagedUpload {
            staging_page_index,
            command,
            staging_page_byte_len: page.bytes.len(),
        });
    }

    GlyphAtlasBitmapStagedUploadPlan { uploads, failures }
}

fn glyph_atlas_bitmap_texture_upload_request(
    upload: GlyphAtlasBitmapStagedUpload,
) -> GlyphAtlasBitmapTextureUploadRequest {
    let command = upload.command;
    GlyphAtlasBitmapTextureUploadRequest {
        staging_page_index: upload.staging_page_index,
        page_key: command.page_key,
        origin_xy: UVec2::new(command.rect.x, command.rect.y),
        origin_layer: command.page_key.page_index,
        extent: UVec2::new(command.rect.width, command.rect.height),
        source_offset: command.source_offset,
        bytes_per_row: command.bytes_per_row,
        rows_per_image: command.rows_per_image,
        upload_byte_len: command.upload_byte_len,
        staging_page_byte_len: upload.staging_page_byte_len,
    }
}

fn staged_upload_source_range_fits(
    command: GlyphAtlasUploadCommand,
    page: &GlyphAtlasBitmapPageUploadStaging,
) -> bool {
    if matches!(command.mode, GlyphAtlasUploadMode::None) || command.rect.height == 0 {
        return false;
    }
    if command.bytes_per_row != page.bytes_per_row || command.rows_per_image == 0 {
        return false;
    }

    let height = command.rect.height as u64;
    let upload_byte_len = command.upload_byte_len as u64;
    if upload_byte_len == 0 || upload_byte_len % height != 0 {
        return false;
    }
    let page_byte_len = page.bytes_per_row as u64 * command.rows_per_image as u64;
    if page_byte_len > page.bytes.len() as u64 {
        return false;
    }

    let row_payload_len = upload_byte_len / height;
    if row_payload_len > command.bytes_per_row as u64 {
        return false;
    }

    let last_row_offset = command.source_offset.saturating_add(
        command.rect.height.saturating_sub(1) as u64 * command.bytes_per_row as u64,
    );
    last_row_offset.saturating_add(row_payload_len) <= page.bytes.len() as u64
}

fn staged_upload_failure(
    upload_command_index: usize,
    page_key: GlyphAtlasPageKey,
    reason: GlyphAtlasBitmapStagedUploadFailureReason,
) -> GlyphAtlasBitmapStagedUploadFailure {
    GlyphAtlasBitmapStagedUploadFailure {
        upload_command_index,
        page_key,
        reason,
    }
}
