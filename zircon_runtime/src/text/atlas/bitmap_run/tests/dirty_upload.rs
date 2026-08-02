use super::super::super::{
    glyph_atlas_bitmap_staged_upload_plan,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas,
    glyph_atlas_bitmap_upload_staging_plan, glyph_atlas_upload_command, GlyphAtlasBitmapRunPlan,
    GlyphAtlasBitmapUploadCopy, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasDirtyPage,
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet, GlyphAtlasUploadMode,
};
use super::{atlas_rect, source};
use crate::core::math::UVec2;

const TEXT_ATLAS_UPLOAD_BUDGET_BYTES_PER_FRAME: usize = 2 * 1024 * 1024;

#[test]
fn render_text_atlas_bitmap_run_promotes_full_page_dirty_to_full_upload() {
    let plan = super::glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(16, 16),
            8.0,
            256,
        )],
        UVec2::new(16, 16),
        23,
        1,
        0,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.dirty_pages.len(), 1);
    assert_eq!(plan.upload_commands.len(), 1);
    assert_eq!(plan.upload_commands[0].mode, GlyphAtlasUploadMode::FullPage);
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 16, 16));
    assert_eq!(plan.upload_commands[0].bytes_per_row, 16);
    assert_eq!(plan.upload_commands[0].rows_per_image, 16);
    assert_eq!(plan.upload_commands[0].upload_byte_len, 256);
}

#[test]
fn render_perf_text_async_upload_merges_per_page() {
    let plan = super::glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 4), 4.0, 16),
            source(GlyphAtlasFormat::AlphaMask, UVec2::new(4, 4), 12.0, 16),
        ],
        UVec2::new(16, 16),
        24,
        1,
        0,
    );
    let first = [1_u8; 16];
    let second = [2_u8; 16];
    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [
            GlyphAtlasBitmapUploadSourceBytes::new(0, &first),
            GlyphAtlasBitmapUploadSourceBytes::new(1, &second),
        ],
    );
    let staged = glyph_atlas_bitmap_staged_upload_plan(&staging, &plan.upload_commands);
    let requests = glyph_atlas_bitmap_texture_upload_request_plan_with_atlas(&staged, &plan.atlas);

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.dirty_pages.len(), 1);
    assert_eq!(plan.upload_commands.len(), 1);
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 8, 4));
    assert_eq!(staging.pages.len(), 1);
    assert!(staging.failures.is_empty());
    assert_eq!(staged.uploads.len(), 1);
    assert!(staged.failures.is_empty());
    assert_eq!(requests.requests.len(), 1);
    assert!(requests.requeued_uploads.is_empty());
    assert_eq!(requests.requests[0].extent, UVec2::new(8, 4));
}

#[test]
fn render_perf_text_typical_256_glyph_frame_stays_within_upload_budget() {
    let sources = (0..256)
        .map(|index| {
            let format = if index < 192 {
                GlyphAtlasFormat::AlphaMask
            } else {
                GlyphAtlasFormat::Color
            };
            let content_size = UVec2::new(64, 64);
            let source_byte_len = content_size.x as usize
                * content_size.y as usize
                * format.storage_format().bytes_per_pixel() as usize;
            source(format, content_size, index as f32 * 64.0, source_byte_len)
        })
        .collect::<Vec<_>>();
    let plan =
        super::glyph_atlas_bitmap_run_plan_with_padding(sources, UVec2::new(512, 512), 25, 4, 0);
    let upload_byte_len = plan
        .upload_commands
        .iter()
        .map(|command| command.upload_byte_len)
        .sum::<usize>();

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.glyphs.len(), 256);
    assert_eq!(plan.dirty_pages.len(), 4);
    assert_eq!(plan.upload_commands.len(), 4);
    assert!(plan
        .upload_commands
        .iter()
        .all(|command| command.mode == GlyphAtlasUploadMode::FullPage));
    assert_eq!(upload_byte_len, 1_835_008);
    assert!(upload_byte_len <= TEXT_ATLAS_UPLOAD_BUDGET_BYTES_PER_FRAME);
}

#[test]
fn render_text_atlas_bitmap_staging_packs_small_r8_region_without_page_zero_fill() {
    let plan = super::glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 16),
            4.0,
            8 * 16,
        )],
        UVec2::new(512, 512),
        26,
        1,
        0,
    );
    let source_bytes = [7_u8; 8 * 16];
    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &source_bytes)],
    );
    let staged = glyph_atlas_bitmap_staged_upload_plan(&staging, &plan.upload_commands);

    assert!(!staging.has_failures());
    assert_eq!(staging.pages.len(), 1);
    assert_eq!(staging.pages[0].bytes_per_row, 8);
    assert_eq!(staging.pages[0].bytes.len(), 8 * 16);
    assert_eq!(staged.uploads[0].command.source_offset, 0);
    assert_eq!(staged.uploads[0].command.rows_per_image, 16);
}

#[test]
fn render_text_atlas_bitmap_staging_packs_small_rgba_region_without_page_zero_fill() {
    let plan = super::glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::Color,
            UVec2::new(8, 16),
            4.0,
            8 * 16 * 4,
        )],
        UVec2::new(512, 512),
        27,
        1,
        0,
    );
    let source_bytes = [9_u8; 8 * 16 * 4];
    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &plan,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &source_bytes)],
    );
    let staged = glyph_atlas_bitmap_staged_upload_plan(&staging, &plan.upload_commands);

    assert!(!staging.has_failures());
    assert_eq!(staging.pages.len(), 1);
    assert_eq!(staging.pages[0].bytes_per_row, 8 * 4);
    assert_eq!(staging.pages[0].bytes.len(), 8 * 16 * 4);
    assert_eq!(staged.uploads[0].command.source_offset, 0);
    assert_eq!(staged.uploads[0].command.rows_per_image, 16);
}

#[test]
fn render_text_atlas_bitmap_uploads_distant_regions_without_bounding_union() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let atlas = GlyphAtlasSet::from_page(GlyphAtlasPageSpec::new(page_key, UVec2::new(512, 512)));
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);
    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 8, 16));
    dirty_page.mark_dirty(page_key, atlas_rect(504, 496, 8, 16));

    let commands = super::super::upload::bitmap_upload_commands(&atlas, &[dirty_page]);

    assert_eq!(commands.len(), 2);
    assert!(commands
        .iter()
        .all(|command| command.mode == GlyphAtlasUploadMode::PartialRect));
    assert_eq!(
        commands
            .iter()
            .map(|command| command.upload_byte_len)
            .sum::<usize>(),
        2 * 8 * 16
    );
}

#[test]
fn render_text_atlas_bitmap_staging_projects_off_origin_region_to_local_layout() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let page = GlyphAtlasPageSpec::new(page_key, UVec2::new(512, 512));
    let target_rect = atlas_rect(504, 496, 8, 16);
    let command = glyph_atlas_upload_command(
        &page,
        GlyphAtlasUploadMode::PartialRect,
        Some(target_rect),
        page.byte_len(),
    )
    .expect("test upload command");
    let run = GlyphAtlasBitmapRunPlan {
        atlas: GlyphAtlasSet::from_page(page),
        upload_copies: vec![GlyphAtlasBitmapUploadCopy {
            source_index: 0,
            page_key,
            atlas_rect: target_rect,
            content_size: UVec2::new(8, 16),
            source_bytes_per_row: 8,
            source_byte_len: 8 * 16,
        }],
        upload_commands: vec![command],
        ..GlyphAtlasBitmapRunPlan::default()
    };
    let source_bytes = [13_u8; 8 * 16];
    let staging = glyph_atlas_bitmap_upload_staging_plan(
        &run,
        [GlyphAtlasBitmapUploadSourceBytes::new(0, &source_bytes)],
    );
    let staged = glyph_atlas_bitmap_staged_upload_plan(&staging, &run.upload_commands);

    assert!(!staging.has_failures());
    assert_eq!(staging.pages[0].target_rect, target_rect);
    assert_eq!(staging.pages[0].bytes_per_row, 8);
    assert_eq!(staging.pages[0].bytes, source_bytes);
    assert_eq!(staged.uploads[0].command.source_offset, 0);
    assert_eq!(staged.uploads[0].command.bytes_per_row, 8);
    assert_eq!(staged.uploads[0].command.rows_per_image, 16);
    assert_eq!(staged.uploads[0].command.rect, target_rect);
}
