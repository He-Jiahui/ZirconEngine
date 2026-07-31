use super::*;
use crate::core::math::UVec2;
use crate::text::sdf::{SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot, SdfBakeParams, SdfMode};

#[test]
fn sdf_upload_report_uses_full_texture_when_atlas_resizes() {
    let report = sdf_atlas_upload_report(
        &atlas_plan(2),
        SdfAtlasCacheReport {
            previous_slot_count: 0,
            current_slot_count: 2,
            retained_slot_count: 0,
            stable_slot_count: 0,
            relocated_slot_count: 0,
            added_slot_count: 2,
            evicted_slot_count: 0,
            atlas_resized: true,
            dirty_rect: Some(sdf_rect(0, 0, 128, 64)),
            dirty_pages: dirty_cache_pages(&[sdf_rect(0, 0, 128, 64)]),
        },
        true,
        512 * 512,
        true,
    );

    assert_eq!(report.mode, SdfAtlasUploadMode::FullTexture);
    assert!(report.full_texture);
    assert_eq!(report.byte_len, 512 * 512);
    assert_eq!(report.dirty_slot_count, 2);
    assert_eq!(report.dirty_rect, Some(sdf_rect(0, 0, 512, 512)));
    assert_eq!(report.dirty_byte_len, 512 * 512);
    assert_eq!(
        report.dirty_pages,
        dirty_upload_pages(&[sdf_rect(0, 0, 512, 512)], 512 * 512)
    );
}

#[test]
fn sdf_upload_report_skips_stable_partial_upload_when_no_dirty_rect() {
    let stable = sdf_atlas_upload_report(
        &atlas_plan(2),
        SdfAtlasCacheReport {
            previous_slot_count: 2,
            current_slot_count: 2,
            retained_slot_count: 2,
            stable_slot_count: 2,
            relocated_slot_count: 0,
            added_slot_count: 0,
            evicted_slot_count: 0,
            atlas_resized: false,
            dirty_rect: None,
            dirty_pages: Vec::new(),
        },
        false,
        512 * 512,
        false,
    );

    assert_eq!(stable.mode, SdfAtlasUploadMode::None);
    assert_eq!(stable.byte_len, 0);
    assert!(!stable.full_texture);
    assert_eq!(stable.dirty_slot_count, 0);
    assert_eq!(stable.dirty_rect, None);
    assert_eq!(stable.dirty_byte_len, 0);
    assert!(stable.dirty_pages.is_empty());
}

#[test]
fn sdf_upload_report_tracks_partial_dirty_slots() {
    let relocated = sdf_atlas_upload_report(
        &atlas_plan(2),
        SdfAtlasCacheReport {
            previous_slot_count: 3,
            current_slot_count: 3,
            retained_slot_count: 2,
            stable_slot_count: 1,
            relocated_slot_count: 1,
            added_slot_count: 1,
            evicted_slot_count: 1,
            atlas_resized: false,
            dirty_rect: Some(sdf_rect(0, 0, 128, 64)),
            dirty_pages: dirty_cache_pages(&[sdf_rect(0, 0, 128, 64)]),
        },
        false,
        512 * 512,
        false,
    );

    assert_eq!(relocated.mode, SdfAtlasUploadMode::DirtySlots);
    assert!(!relocated.full_texture);
    assert_eq!(relocated.byte_len, 2 * 64 * 64);
    assert_eq!(relocated.dirty_slot_count, 2);
    assert_eq!(relocated.dirty_rect, Some(sdf_rect(0, 0, 128, 64)));
    assert_eq!(relocated.dirty_byte_len, 2 * 64 * 64);
    assert_eq!(
        relocated.dirty_pages,
        dirty_upload_pages(&[sdf_rect(0, 0, 128, 64)], 128 * 64)
    );
}

#[test]
fn sdf_upload_report_uses_merged_dirty_rect_area() {
    let report = sdf_atlas_upload_report(
        &atlas_plan(4),
        SdfAtlasCacheReport {
            previous_slot_count: 4,
            current_slot_count: 4,
            retained_slot_count: 4,
            stable_slot_count: 2,
            relocated_slot_count: 2,
            added_slot_count: 0,
            evicted_slot_count: 0,
            atlas_resized: false,
            dirty_rect: Some(sdf_rect(0, 0, 256, 64)),
            dirty_pages: dirty_cache_pages(&[sdf_rect(0, 0, 256, 64)]),
        },
        false,
        512 * 512,
        false,
    );

    assert_eq!(report.mode, SdfAtlasUploadMode::DirtySlots);
    assert_eq!(report.byte_len, 256 * 64);
    assert_eq!(report.dirty_slot_count, 2);
    assert_eq!(report.dirty_rect, Some(sdf_rect(0, 0, 256, 64)));
    assert_eq!(report.dirty_byte_len, 256 * 64);
    assert_eq!(
        report.dirty_pages,
        dirty_upload_pages(&[sdf_rect(0, 0, 256, 64)], 256 * 64)
    );
}

#[test]
fn sdf_upload_commands_use_full_atlas_rect_for_resize() {
    let report = sdf_atlas_upload_report(
        &atlas_plan(2),
        SdfAtlasCacheReport {
            previous_slot_count: 0,
            current_slot_count: 2,
            retained_slot_count: 0,
            stable_slot_count: 0,
            relocated_slot_count: 0,
            added_slot_count: 2,
            evicted_slot_count: 0,
            atlas_resized: true,
            dirty_rect: Some(sdf_rect(0, 0, 128, 64)),
            dirty_pages: dirty_cache_pages(&[sdf_rect(0, 0, 128, 64)]),
        },
        true,
        512 * 512,
        true,
    );

    let commands = sdf_atlas_upload_commands(&atlas_plan(2), &report, 512 * 512);
    assert_eq!(commands.len(), 1);
    let command = commands[0];

    assert_eq!(command.mode, GlyphAtlasUploadMode::FullPage);
    assert_eq!(
        command.page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(command.rect, glyph_rect(0, 0, 512, 512));
    assert_eq!(command.source_offset, 0);
    assert_eq!(command.bytes_per_row, 512);
    assert_eq!(command.rows_per_image, 512);
    assert_eq!(command.upload_byte_len, 512 * 512);
}

#[test]
fn sdf_upload_commands_use_dirty_rect_origin_and_full_atlas_stride() {
    let report = sdf_atlas_upload_report(
        &atlas_plan(4),
        SdfAtlasCacheReport {
            previous_slot_count: 4,
            current_slot_count: 4,
            retained_slot_count: 4,
            stable_slot_count: 2,
            relocated_slot_count: 2,
            added_slot_count: 0,
            evicted_slot_count: 0,
            atlas_resized: false,
            dirty_rect: Some(sdf_rect(64, 128, 128, 64)),
            dirty_pages: dirty_cache_pages(&[sdf_rect(64, 128, 128, 64)]),
        },
        false,
        512 * 512,
        false,
    );

    let commands = sdf_atlas_upload_commands(&atlas_plan(4), &report, 512 * 512);
    assert_eq!(commands.len(), 1);
    let command = commands[0];

    assert_eq!(command.mode, GlyphAtlasUploadMode::PartialRect);
    assert_eq!(
        command.page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(command.rect, glyph_rect(64, 128, 128, 64));
    assert_eq!(command.source_offset, 128 * 512 + 64);
    assert_eq!(command.bytes_per_row, 512);
    assert_eq!(command.rows_per_image, 512);
    assert_eq!(command.upload_byte_len, 128 * 64);
}

#[test]
fn sdf_upload_command_is_absent_for_stable_partial_frames() {
    let report = sdf_atlas_upload_report(
        &atlas_plan(1),
        SdfAtlasCacheReport {
            previous_slot_count: 1,
            current_slot_count: 1,
            retained_slot_count: 1,
            stable_slot_count: 1,
            relocated_slot_count: 0,
            added_slot_count: 0,
            evicted_slot_count: 0,
            atlas_resized: false,
            dirty_rect: None,
            dirty_pages: Vec::new(),
        },
        false,
        512 * 512,
        false,
    );

    assert!(sdf_atlas_upload_commands(&atlas_plan(1), &report, 512 * 512).is_empty());
}

#[test]
fn sdf_upload_report_emits_dirty_commands_for_pages_beyond_page_zero() {
    let dirty_rect = sdf_rect(64, 128, 128, 64);
    let report = sdf_atlas_upload_report(
        &atlas_plan_with_page_slots(vec![slot_on_page('B', 1, dirty_rect)]),
        SdfAtlasCacheReport {
            previous_slot_count: 1,
            current_slot_count: 1,
            retained_slot_count: 1,
            stable_slot_count: 0,
            relocated_slot_count: 1,
            added_slot_count: 0,
            evicted_slot_count: 0,
            atlas_resized: false,
            dirty_rect: None,
            dirty_pages: vec![dirty_cache_page(1, dirty_rect)],
        },
        false,
        512 * 512,
        false,
    );

    assert_eq!(report.mode, SdfAtlasUploadMode::DirtySlots);
    assert_eq!(report.byte_len, 128 * 64);
    assert_eq!(report.dirty_rect, None);
    assert_eq!(
        report.dirty_pages,
        vec![SdfAtlasUploadPageReport {
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1),
            dirty_rect,
            byte_len: 128 * 64,
        }]
    );

    let commands = sdf_atlas_upload_commands(
        &atlas_plan_with_page_slots(vec![slot_on_page('B', 1, dirty_rect)]),
        &report,
        512 * 512,
    );

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].mode, GlyphAtlasUploadMode::PartialRect);
    assert_eq!(
        commands[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1)
    );
    assert_eq!(commands[0].rect, glyph_rect(64, 128, 128, 64));
    assert_eq!(commands[0].source_offset, (128 * 512 + 64) as u64);
    assert_eq!(commands[0].bytes_per_row, 512);
    assert_eq!(commands[0].rows_per_image, 512);
    assert_eq!(commands[0].upload_byte_len, 128 * 64);
}

#[test]
fn sdf_upload_report_emits_full_page_commands_for_all_resized_layers() {
    let report = sdf_atlas_upload_report(
        &atlas_plan_with_page_slots(vec![
            slot_on_page('A', 0, sdf_rect(0, 0, 64, 64)),
            slot_on_page('B', 1, sdf_rect(0, 0, 64, 64)),
        ]),
        SdfAtlasCacheReport {
            previous_slot_count: 0,
            current_slot_count: 2,
            retained_slot_count: 0,
            stable_slot_count: 0,
            relocated_slot_count: 0,
            added_slot_count: 2,
            evicted_slot_count: 0,
            atlas_resized: true,
            dirty_rect: Some(sdf_rect(0, 0, 64, 64)),
            dirty_pages: vec![
                dirty_cache_page(0, sdf_rect(0, 0, 64, 64)),
                dirty_cache_page(1, sdf_rect(0, 0, 64, 64)),
            ],
        },
        true,
        2 * 512 * 512,
        true,
    );

    assert_eq!(report.mode, SdfAtlasUploadMode::FullTexture);
    assert_eq!(report.byte_len, 2 * 512 * 512);
    assert_eq!(
        report.dirty_pages,
        dirty_upload_pages_for_indices(&[0, 1], sdf_rect(0, 0, 512, 512), 512 * 512)
    );

    let commands = sdf_atlas_upload_commands(
        &atlas_plan_with_page_slots(vec![
            slot_on_page('A', 0, sdf_rect(0, 0, 64, 64)),
            slot_on_page('B', 1, sdf_rect(0, 0, 64, 64)),
        ]),
        &report,
        2 * 512 * 512,
    );

    assert_eq!(commands.len(), 2);
    assert_eq!(
        commands[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(commands[0].source_offset, 0);
    assert_eq!(
        commands[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1)
    );
    assert_eq!(commands[1].source_offset, 512 * 512);
}

#[test]
fn sdf_upload_commands_preserve_mixed_page_stride_and_source_offsets() {
    let page_area = 512 * 512;
    let source_byte_len = page_area + page_area * 4;
    let plan = atlas_plan_with_page_slots(vec![
        slot_on_format(
            'A',
            SdfMode::Sdf,
            GlyphAtlasFormat::Sdf,
            sdf_rect(0, 0, 64, 64),
        ),
        slot_on_format(
            'M',
            SdfMode::Msdf,
            GlyphAtlasFormat::Msdf,
            sdf_rect(0, 0, 64, 64),
        ),
    ]);
    let report = sdf_atlas_upload_report(
        &plan,
        SdfAtlasCacheReport {
            current_slot_count: 2,
            added_slot_count: 2,
            atlas_resized: true,
            ..Default::default()
        },
        true,
        source_byte_len,
        true,
    );

    assert_eq!(report.mode, SdfAtlasUploadMode::FullTexture);
    assert_eq!(report.byte_len, source_byte_len);
    assert_eq!(report.dirty_pages.len(), 2);
    assert_eq!(report.dirty_pages[0].byte_len, page_area);
    assert_eq!(report.dirty_pages[1].byte_len, page_area * 4);

    let commands = sdf_atlas_upload_commands(&plan, &report, source_byte_len);
    assert_eq!(commands.len(), 2);
    assert_eq!(
        commands[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(commands[0].source_offset, 0);
    assert_eq!(commands[0].bytes_per_row, 512);
    assert_eq!(commands[0].upload_byte_len, page_area);
    assert_eq!(
        commands[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Msdf, 0)
    );
    assert_eq!(commands[1].source_offset, page_area as u64);
    assert_eq!(commands[1].bytes_per_row, 512 * 4);
    assert_eq!(commands[1].upload_byte_len, page_area * 4);
}

#[test]
fn sdf_upload_partial_msdf_rect_uses_rgba_row_stride() {
    let page_area = 512 * 512;
    let source_byte_len = page_area + page_area * 4;
    let dirty_rect = sdf_rect(64, 128, 128, 64);
    let plan = atlas_plan_with_page_slots(vec![
        slot_on_format(
            'A',
            SdfMode::Sdf,
            GlyphAtlasFormat::Sdf,
            sdf_rect(0, 0, 64, 64),
        ),
        slot_on_format('M', SdfMode::Msdf, GlyphAtlasFormat::Msdf, dirty_rect),
    ]);
    let report = sdf_atlas_upload_report(
        &plan,
        SdfAtlasCacheReport {
            previous_slot_count: 2,
            current_slot_count: 2,
            retained_slot_count: 2,
            stable_slot_count: 1,
            relocated_slot_count: 1,
            dirty_pages: vec![SdfAtlasDirtyPageReport {
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Msdf, 0),
                dirty_rect,
            }],
            ..Default::default()
        },
        false,
        source_byte_len,
        false,
    );

    assert_eq!(report.mode, SdfAtlasUploadMode::DirtySlots);
    assert_eq!(report.byte_len, 128 * 64 * 4);
    let commands = sdf_atlas_upload_commands(&plan, &report, source_byte_len);
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].mode, GlyphAtlasUploadMode::PartialRect);
    assert_eq!(
        commands[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Msdf, 0)
    );
    assert_eq!(commands[0].rect, glyph_rect(64, 128, 128, 64));
    assert_eq!(
        commands[0].source_offset,
        (page_area + (128 * 512 + 64) * 4) as u64
    );
    assert_eq!(commands[0].bytes_per_row, 512 * 4);
    assert_eq!(commands[0].rows_per_image, 512);
    assert_eq!(commands[0].upload_byte_len, 128 * 64 * 4);
}

#[test]
fn sdf_upload_commands_build_page_offsets_once_and_borrow_report() {
    let upload_source = include_str!("../sdf_upload.rs");
    let resources_source = include_str!("../sdf_render/atlas_resources.rs");

    assert!(upload_source.contains("fn sdf_atlas_source_pages("));
    assert!(upload_source.contains("binary_search_by_key("));
    assert!(!upload_source.contains("fn offset_upload_command_for_source_page("));
    assert!(!resources_source.contains("upload.clone()"));
}

fn atlas_plan(slot_count: usize) -> SdfAtlasPlan {
    let slots = (0..slot_count)
        .map(|index| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph: char::from_u32('A' as u32 + index as u32).unwrap_or('A'),
                glyph_id: None,
                font_id: None,
                font_instance_id: None,
                font: Some("res://fonts/default.font.toml".to_string()),
                font_family: Some("Zircon Sans".to_string()),
                language: None,
                font_weight: 400,
                bake_params: SdfBakeParams::default(),
            },
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
            rect: SdfAtlasRect {
                x: index as u32 * 64,
                y: 0,
                width: 64,
                height: 64,
            },
        })
        .collect();
    SdfAtlasPlan {
        atlas_size: UVec2::splat(512),
        atlas_set: crate::text::atlas::GlyphAtlasSet::from_page(GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
            UVec2::splat(512),
        )),
        slots,
        runs: Vec::new(),
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    }
}

fn atlas_plan_with_page_slots(slots: Vec<SdfAtlasSlot>) -> SdfAtlasPlan {
    let page_keys = slots
        .iter()
        .map(|slot| slot.page_key)
        .collect::<std::collections::BTreeSet<_>>();
    let mut atlas_set = crate::text::atlas::GlyphAtlasSet::default();
    for page_key in page_keys {
        atlas_set = atlas_set.with_page(GlyphAtlasPageSpec::new(page_key, UVec2::splat(512)));
    }
    SdfAtlasPlan {
        atlas_size: UVec2::splat(512),
        atlas_set,
        slots,
        runs: Vec::new(),
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    }
}

fn slot_on_page(glyph: char, page_index: u32, rect: SdfAtlasRect) -> SdfAtlasSlot {
    SdfAtlasSlot {
        key: SdfAtlasGlyphKey {
            glyph,
            glyph_id: None,
            font_id: None,
            font_instance_id: None,
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Zircon Sans".to_string()),
            language: None,
            font_weight: 400,
            bake_params: SdfBakeParams::default(),
        },
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index),
        rect,
    }
}

fn slot_on_format(
    glyph: char,
    mode: SdfMode,
    format: GlyphAtlasFormat,
    rect: SdfAtlasRect,
) -> SdfAtlasSlot {
    let mut bake_params = SdfBakeParams::default();
    bake_params.mode = mode;
    SdfAtlasSlot {
        key: SdfAtlasGlyphKey {
            glyph,
            glyph_id: None,
            font_id: None,
            font_instance_id: None,
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Zircon Sans".to_string()),
            language: None,
            font_weight: 400,
            bake_params,
        },
        page_key: GlyphAtlasPageKey::new(format, 0),
        rect,
    }
}

fn sdf_rect(x: u32, y: u32, width: u32, height: u32) -> SdfAtlasRect {
    SdfAtlasRect {
        x,
        y,
        width,
        height,
    }
}

fn glyph_rect(x: u32, y: u32, width: u32, height: u32) -> crate::text::atlas::GlyphAtlasRect {
    crate::text::atlas::GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}

fn dirty_cache_pages(rects: &[SdfAtlasRect]) -> Vec<SdfAtlasDirtyPageReport> {
    rects
        .iter()
        .copied()
        .map(|dirty_rect| dirty_cache_page(0, dirty_rect))
        .collect()
}

fn dirty_cache_page(page_index: u32, dirty_rect: SdfAtlasRect) -> SdfAtlasDirtyPageReport {
    SdfAtlasDirtyPageReport {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index),
        dirty_rect,
    }
}

fn dirty_upload_pages(rects: &[SdfAtlasRect], byte_len: usize) -> Vec<SdfAtlasUploadPageReport> {
    rects
        .iter()
        .copied()
        .map(|dirty_rect| dirty_upload_page(0, dirty_rect, byte_len))
        .collect()
}

fn dirty_upload_pages_for_indices(
    page_indices: &[u32],
    dirty_rect: SdfAtlasRect,
    byte_len: usize,
) -> Vec<SdfAtlasUploadPageReport> {
    page_indices
        .iter()
        .copied()
        .map(|page_index| dirty_upload_page(page_index, dirty_rect, byte_len))
        .collect()
}

fn dirty_upload_page(
    page_index: u32,
    dirty_rect: SdfAtlasRect,
    byte_len: usize,
) -> SdfAtlasUploadPageReport {
    SdfAtlasUploadPageReport {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index),
        dirty_rect,
        byte_len,
    }
}
