use crate::core::math::UVec2;

use super::super::{
    GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPageShadowPatch, GlyphAtlasPageKey,
    GlyphAtlasSet, GlyphAtlasUploadCommand, GlyphAtlasUploadMode,
};
use super::staging::{
    GlyphAtlasBitmapPageUploadStaging, GlyphAtlasBitmapUploadSourceBytes,
    GlyphAtlasBitmapUploadStagingPlan, glyph_atlas_bitmap_upload_staging_plan,
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
    pub(crate) page_generation: u64,
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
    pub(crate) requeued_uploads: Vec<GlyphAtlasBitmapRequeuedUpload>,
    pub(crate) skipped_failure_count: usize,
    pub(crate) stale_page_generation_count: usize,
    pub(crate) face_invalidated_count: usize,
}

impl GlyphAtlasBitmapTextureUploadRequestPlan {
    pub(crate) fn has_requests(&self) -> bool {
        !self.requests.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapFaceValidity {
    Valid,
    Invalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapRequeueReason {
    MissingPage,
    PageGenerationMismatch,
    FaceInvalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRequeuedUpload {
    pub(crate) upload_command_index: usize,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) requested_page_generation: u64,
    pub(crate) current_page_generation: Option<u64>,
    pub(crate) reason: GlyphAtlasBitmapRequeueReason,
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

pub(crate) fn glyph_atlas_bitmap_page_shadow_commit(
    run: &GlyphAtlasBitmapRunPlan,
    prepared_upload: GlyphAtlasBitmapPreparedUploadPlan,
    upload_was_submitted: bool,
) -> GlyphAtlasBitmapPageShadowCommit {
    let zero_initialized_pages = prepared_upload
        .staging
        .pages
        .iter()
        .filter(|page| run.zero_initialize_shadow_pages.contains(&page.page_key))
        .map(|page| page.page_key)
        .collect();
    if !upload_was_submitted {
        return GlyphAtlasBitmapPageShadowCommit {
            failed_zero_initialized_pages: zero_initialized_pages,
            ..GlyphAtlasBitmapPageShadowCommit::default()
        };
    }

    GlyphAtlasBitmapPageShadowCommit {
        patches: prepared_upload
            .staging
            .pages
            .into_iter()
            .map(|page| GlyphAtlasBitmapPageShadowPatch {
                page_key: page.page_key,
                page_generation: page.page_generation,
                target_rect: page.target_rect,
                bytes_per_row: page.bytes_per_row,
                bytes: page.bytes,
            })
            .collect(),
        zero_initialized_pages,
        failed_zero_initialized_pages: Default::default(),
    }
}

pub(crate) fn glyph_atlas_bitmap_texture_upload_request_plan(
    staged_uploads: &GlyphAtlasBitmapStagedUploadPlan,
) -> GlyphAtlasBitmapTextureUploadRequestPlan {
    glyph_atlas_bitmap_texture_upload_request_plan_for_current_atlas(
        staged_uploads,
        None,
        GlyphAtlasBitmapFaceValidity::Valid,
    )
}

pub(crate) fn glyph_atlas_bitmap_texture_upload_request_plan_with_atlas(
    staged_uploads: &GlyphAtlasBitmapStagedUploadPlan,
    atlas: &GlyphAtlasSet,
) -> GlyphAtlasBitmapTextureUploadRequestPlan {
    glyph_atlas_bitmap_texture_upload_request_plan_for_current_atlas(
        staged_uploads,
        Some(atlas),
        GlyphAtlasBitmapFaceValidity::Valid,
    )
}

pub(crate) fn glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity(
    staged_uploads: &GlyphAtlasBitmapStagedUploadPlan,
    atlas: &GlyphAtlasSet,
    face_validity: GlyphAtlasBitmapFaceValidity,
) -> GlyphAtlasBitmapTextureUploadRequestPlan {
    glyph_atlas_bitmap_texture_upload_request_plan_for_current_atlas(
        staged_uploads,
        Some(atlas),
        face_validity,
    )
}

fn glyph_atlas_bitmap_texture_upload_request_plan_for_current_atlas(
    staged_uploads: &GlyphAtlasBitmapStagedUploadPlan,
    atlas: Option<&GlyphAtlasSet>,
    face_validity: GlyphAtlasBitmapFaceValidity,
) -> GlyphAtlasBitmapTextureUploadRequestPlan {
    let mut plan = GlyphAtlasBitmapTextureUploadRequestPlan {
        skipped_failure_count: staged_uploads.failures.len(),
        ..GlyphAtlasBitmapTextureUploadRequestPlan::default()
    };

    for (upload_command_index, upload) in staged_uploads.uploads.iter().copied().enumerate() {
        if let Some(requeue) =
            staged_upload_requeue(upload_command_index, upload, atlas, face_validity)
        {
            match requeue.reason {
                GlyphAtlasBitmapRequeueReason::MissingPage
                | GlyphAtlasBitmapRequeueReason::PageGenerationMismatch => {
                    plan.stale_page_generation_count += 1;
                }
                GlyphAtlasBitmapRequeueReason::FaceInvalidated => {
                    plan.face_invalidated_count += 1;
                }
            }
            plan.requeued_uploads.push(requeue);
            continue;
        }

        plan.requests
            .push(glyph_atlas_bitmap_texture_upload_request(upload));
    }

    plan
}

pub(crate) fn glyph_atlas_bitmap_staged_upload_plan(
    staging: &GlyphAtlasBitmapUploadStagingPlan,
    upload_commands: &[GlyphAtlasUploadCommand],
) -> GlyphAtlasBitmapStagedUploadPlan {
    let mut uploads = Vec::new();
    let mut failures = Vec::new();
    let mut claimed_staging_pages = vec![false; staging.pages.len()];

    for (upload_command_index, command) in upload_commands.iter().copied().enumerate() {
        let Some((staging_page_index, page)) =
            staging.pages.iter().enumerate().find(|(index, page)| {
                !claimed_staging_pages[*index]
                    && page.page_key == command.page_key
                    && page.target_rect == command.rect
            })
        else {
            failures.push(staged_upload_failure(
                upload_command_index,
                command.page_key,
                GlyphAtlasBitmapStagedUploadFailureReason::MissingStagingPage,
            ));
            continue;
        };
        claimed_staging_pages[staging_page_index] = true;
        let command = compact_staged_upload_command(command, page);

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
        page_generation: command.page_generation,
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

fn staged_upload_requeue(
    upload_command_index: usize,
    upload: GlyphAtlasBitmapStagedUpload,
    atlas: Option<&GlyphAtlasSet>,
    face_validity: GlyphAtlasBitmapFaceValidity,
) -> Option<GlyphAtlasBitmapRequeuedUpload> {
    let current_page_generation =
        atlas.and_then(|atlas| current_atlas_page_generation(upload, atlas));

    if matches!(face_validity, GlyphAtlasBitmapFaceValidity::Invalidated) {
        return Some(requeued_upload(
            upload_command_index,
            upload,
            current_page_generation,
            GlyphAtlasBitmapRequeueReason::FaceInvalidated,
        ));
    }

    let atlas = atlas?;
    match current_atlas_page_generation(upload, atlas) {
        Some(current_generation) if current_generation == upload.command.page_generation => None,
        Some(current_generation) => Some(requeued_upload(
            upload_command_index,
            upload,
            Some(current_generation),
            GlyphAtlasBitmapRequeueReason::PageGenerationMismatch,
        )),
        None => Some(requeued_upload(
            upload_command_index,
            upload,
            None,
            GlyphAtlasBitmapRequeueReason::MissingPage,
        )),
    }
}

fn current_atlas_page_generation(
    upload: GlyphAtlasBitmapStagedUpload,
    atlas: &GlyphAtlasSet,
) -> Option<u64> {
    atlas
        .page(
            upload.command.page_key.format,
            upload.command.page_key.page_index,
        )
        .map(|page| page.generation)
}

fn requeued_upload(
    upload_command_index: usize,
    upload: GlyphAtlasBitmapStagedUpload,
    current_page_generation: Option<u64>,
    reason: GlyphAtlasBitmapRequeueReason,
) -> GlyphAtlasBitmapRequeuedUpload {
    GlyphAtlasBitmapRequeuedUpload {
        upload_command_index,
        page_key: upload.command.page_key,
        requested_page_generation: upload.command.page_generation,
        current_page_generation,
        reason,
    }
}

fn staged_upload_source_range_fits(
    command: GlyphAtlasUploadCommand,
    page: &GlyphAtlasBitmapPageUploadStaging,
) -> bool {
    if matches!(command.mode, GlyphAtlasUploadMode::None) || command.rect.height == 0 {
        return false;
    }
    if command.rect != page.target_rect
        || command.bytes_per_row != page.bytes_per_row
        || command.rows_per_image != page.target_rect.height
    {
        return false;
    }

    let height = command.rect.height as u64;
    let upload_byte_len = command.upload_byte_len as u64;
    if upload_byte_len == 0 || upload_byte_len % height != 0 {
        return false;
    }
    let page_byte_len = page.bytes_per_row as u64 * command.rows_per_image as u64;
    if page_byte_len != page.bytes.len() as u64 {
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

fn compact_staged_upload_command(
    command: GlyphAtlasUploadCommand,
    page: &GlyphAtlasBitmapPageUploadStaging,
) -> GlyphAtlasUploadCommand {
    GlyphAtlasUploadCommand {
        source_offset: 0,
        bytes_per_row: page.bytes_per_row,
        rows_per_image: page.target_rect.height,
        ..command
    }
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
