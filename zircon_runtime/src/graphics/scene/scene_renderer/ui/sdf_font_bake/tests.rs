use super::*;
use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::{SdfAtlasPlan, SdfAtlasSlot};
use crate::graphics::text::atlas::{GlyphAtlasFormat, GlyphAtlasPageKey};
#[cfg(target_os = "windows")]
use crate::graphics::text::font::shared_font_database_snapshot;
use std::path::PathBuf;

#[test]
fn sdf_font_bake_produces_distinct_ascii_glyph_patterns() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A', 'I', 'O']);

    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);

    let a = slot_pixels(&atlas.pixels, plan.atlas_size.x, plan.slots[0].rect);
    let i = slot_pixels(&atlas.pixels, plan.atlas_size.x, plan.slots[1].rect);
    let o = slot_pixels(&atlas.pixels, plan.atlas_size.x, plan.slots[2].rect);
    assert_ne!(a, i);
    assert_ne!(a, o);
    assert_ne!(i, o);
    assert_eq!(atlas.report.slot_count, 3);
    assert_eq!(atlas.report.visible_glyph_count, 3);
    assert_eq!(atlas.report.empty_glyph_count, 0);
    assert_eq!(
        atlas.report.atlas_byte_len,
        (plan.atlas_size.x * plan.atlas_size.y) as usize
    );
    assert!(atlas.report.nonzero_pixel_count > 0);
    assert!(atlas.report.loaded_font_count >= 1);
}

#[test]
fn sdf_font_bake_does_not_match_the_old_rounded_rect_placeholder() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A']);

    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);

    let actual = slot_pixels(&atlas.pixels, plan.atlas_size.x, plan.slots[0].rect);
    let placeholder =
        old_rounded_rect_placeholder(plan.slots[0].rect.width, plan.slots[0].rect.height);
    assert_ne!(actual, placeholder);
    assert!(atlas.report.nonzero_pixel_count > 0);
}

#[test]
fn sdf_font_bake_writes_page_indexed_slots_into_matching_layers() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_page_glyphs(&[('A', 0), ('B', 1)]);

    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);

    let page_byte_len = (plan.atlas_size.x * plan.atlas_size.y) as usize;
    let a = slot_pixels_for_page(
        &atlas.pixels,
        plan.atlas_size.x,
        page_byte_len,
        0,
        plan.slots[0].rect,
    );
    let b = slot_pixels_for_page(
        &atlas.pixels,
        plan.atlas_size.x,
        page_byte_len,
        1,
        plan.slots[1].rect,
    );
    assert_eq!(atlas.pixels.len(), page_byte_len * 2);
    assert!(a.iter().any(|pixel| *pixel != 0));
    assert!(b.iter().any(|pixel| *pixel != 0));
    assert_ne!(a, b);
}

#[test]
fn sdf_font_bake_measures_whitespace_without_atlas_bitmap() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();

    let metrics = bake.measure_glyph(
        ' ',
        Some(DEFAULT_FONT_ASSET),
        Some("Studio Mono"),
        None,
        UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        18.0,
        &mut font_database,
        &asset_manager,
    );

    assert!(metrics.advance > 0.0);
    assert_eq!(metrics.bitmap_width, 0);
    assert_eq!(metrics.bitmap_height, 0);
}

#[test]
fn sdf_font_bake_handles_missing_glyph_with_stable_empty_fallback() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['\u{10ffff}']);

    let metrics = bake.measure_glyph(
        '\u{10ffff}',
        Some(DEFAULT_FONT_ASSET),
        Some("Studio Mono"),
        None,
        UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        18.0,
        &mut font_database,
        &asset_manager,
    );

    assert!(metrics.advance > 0.0);

    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);
    assert_eq!(atlas.glyphs.len(), 1);
    assert!(atlas.glyphs[0].metrics.advance > 0.0);
    assert_eq!(
        atlas.pixels.len(),
        (plan.atlas_size.x * plan.atlas_size.y) as usize
    );
    assert_eq!(atlas.report.slot_count, 1);
    assert_eq!(atlas.report.atlas_byte_len, atlas.pixels.len());
}

#[test]
fn sdf_font_query_for_key_preserves_font_weight() {
    let query = font_query_for_key(&SdfAtlasGlyphKey {
        glyph: 'A',
        glyph_id: None,
        font_id: None,
        font: Some(DEFAULT_FONT_ASSET.to_string()),
        font_family: Some("Studio Mono".to_string()),
        language: None,
        font_weight: 650,
        bake_params: SdfBakeParams::default(),
    });

    assert_eq!(query.weight, FontWeight::clamped(650));
}

#[test]
fn sdf_font_bake_falls_back_when_fontsdf_cannot_open_requested_face_index() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let manifest = write_face_index_manifest(1);
    let plan = atlas_plan_for_asset('A', manifest.path().to_string_lossy().as_ref());

    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);

    assert_eq!(atlas.report.slot_count, 1);
    assert_eq!(atlas.report.visible_glyph_count, 1);
    assert!(atlas.report.nonzero_pixel_count > 0);
}

#[cfg(target_os = "windows")]
#[test]
fn sdf_font_bake_rasterizes_materialized_system_cjk_face() {
    let mut bake = SdfFontBakeCache::new();
    let (_, mut font_database) = shared_font_database_snapshot();
    let asset_manager = ProjectAssetManager::default();
    let face = font_database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;
    assert!(bake.ensure_sdf_font(face, &font_database));

    let mut plan = atlas_plan_for_glyphs(&['本']);
    plan.slots[0].key.font_family = Some("Microsoft YaHei UI".to_string());
    plan.slots[0].key.language = Some("zh-Hans".to_string());
    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);

    assert_eq!(atlas.report.slot_count, 1);
    assert_eq!(atlas.report.visible_glyph_count, 1);
    assert_eq!(atlas.report.empty_glyph_count, 0);
    assert!(atlas.report.nonzero_pixel_count > 0);
    assert_eq!(atlas.report.loaded_font_count, 1);
}

#[cfg(target_os = "windows")]
#[test]
fn sdf_font_bake_prefers_shaped_glyph_id_on_authoritative_face() {
    let mut bake = SdfFontBakeCache::new();
    let (_, font_database) = shared_font_database_snapshot();
    let face = font_database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;
    assert!(bake.ensure_sdf_font(face, &font_database));
    let font = bake.fonts.get(&face).expect("materialized SDF font");
    let shaped_id = font.lookup_glyph_index('布');
    let scalar_id = font.lookup_glyph_index('。');
    assert_ne!(shaped_id, 0);
    assert_ne!(shaped_id, scalar_id);
    let key = SdfAtlasGlyphKey {
        glyph: '。',
        glyph_id: Some(shaped_id as u32),
        font_id: Some(face.0),
        font: Some(DEFAULT_FONT_ASSET.to_string()),
        font_family: Some("Microsoft YaHei UI".to_string()),
        language: Some("zh-hans".to_string()),
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        bake_params: SdfBakeParams::default(),
    };

    assert_eq!(glyph_index(font, &key), shaped_id);
}

#[test]
fn sdf_font_bake_report_handles_empty_atlas_plan() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = SdfAtlasPlan {
        atlas_size: UVec2::new(1, 1),
        atlas_set: Default::default(),
        slots: Vec::new(),
        runs: Vec::new(),
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    };

    let atlas = bake.build_atlas(&plan, &mut font_database, &asset_manager);

    assert_eq!(atlas.pixels, vec![0]);
    assert_eq!(
        atlas.report,
        SdfAtlasBakeReport {
            slot_count: 0,
            visible_glyph_count: 0,
            empty_glyph_count: 0,
            atlas_byte_len: 1,
            nonzero_pixel_count: 0,
            loaded_font_count: 0,
        }
    );
}

fn atlas_plan_for_glyphs(glyphs: &[char]) -> SdfAtlasPlan {
    let slots = glyphs
        .iter()
        .enumerate()
        .map(|(index, glyph)| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph: *glyph,
                glyph_id: None,
                font_id: None,
                font: Some(DEFAULT_FONT_ASSET.to_string()),
                font_family: Some("Studio Mono".to_string()),
                language: None,
                font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
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
        atlas_size: UVec2::new(256, 256),
        atlas_set: Default::default(),
        slots,
        runs: Vec::new(),
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    }
}

fn atlas_plan_for_page_glyphs(glyphs: &[(char, u32)]) -> SdfAtlasPlan {
    let slots = glyphs
        .iter()
        .map(|(glyph, page_index)| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph: *glyph,
                glyph_id: None,
                font_id: None,
                font: Some(DEFAULT_FONT_ASSET.to_string()),
                font_family: Some("Studio Mono".to_string()),
                language: None,
                font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
                bake_params: SdfBakeParams::default(),
            },
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, *page_index),
            rect: SdfAtlasRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64,
            },
        })
        .collect();
    SdfAtlasPlan {
        atlas_size: UVec2::new(256, 256),
        atlas_set: Default::default(),
        slots,
        runs: Vec::new(),
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    }
}

fn atlas_plan_for_asset(glyph: char, asset_ref: &str) -> SdfAtlasPlan {
    SdfAtlasPlan {
        atlas_size: UVec2::new(64, 64),
        atlas_set: Default::default(),
        slots: vec![SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph,
                glyph_id: None,
                font_id: None,
                font: Some(asset_ref.to_string()),
                font_family: Some("Fira Unsupported Face".to_string()),
                language: None,
                font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
                bake_params: SdfBakeParams::default(),
            },
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
            rect: SdfAtlasRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64,
            },
        }],
        runs: Vec::new(),
        rebuilt_pages: Vec::new(),
        allocation_failures: Vec::new(),
    }
}

struct TemporaryFontManifest {
    manifest: PathBuf,
    source: PathBuf,
}

impl TemporaryFontManifest {
    fn path(&self) -> &std::path::Path {
        &self.manifest
    }
}

impl Drop for TemporaryFontManifest {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.source);
        let _ = std::fs::remove_file(&self.manifest);
    }
}

fn write_face_index_manifest(face_index: u32) -> TemporaryFontManifest {
    let root = std::env::temp_dir();
    let stem = format!("zircon-runtime-text-sdf-face-index-{}", std::process::id());
    let manifest = root.join(format!("{stem}.font.toml"));
    let source_name = format!("{stem}.ttf");
    let source = root.join(&source_name);
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("fonts")
            .join("FiraSans-Regular.ttf"),
        &source,
    )
    .unwrap();
    std::fs::write(
        &manifest,
        format!(
            "source = \"{source_name}\"\nfamily = \"Fira Unsupported Face\"\nface_index = {face_index}\n"
        ),
    )
    .unwrap();
    TemporaryFontManifest { manifest, source }
}

fn slot_pixels(pixels: &[u8], atlas_width: u32, rect: SdfAtlasRect) -> Vec<u8> {
    let mut slot = Vec::with_capacity(rect.width as usize * rect.height as usize);
    for y in rect.y..rect.y + rect.height {
        let start = y as usize * atlas_width as usize + rect.x as usize;
        let end = start + rect.width as usize;
        slot.extend_from_slice(&pixels[start..end]);
    }
    slot
}

fn slot_pixels_for_page(
    pixels: &[u8],
    atlas_width: u32,
    page_byte_len: usize,
    page_index: u32,
    rect: SdfAtlasRect,
) -> Vec<u8> {
    let page_offset = page_byte_len * page_index as usize;
    let mut slot = Vec::with_capacity(rect.width as usize * rect.height as usize);
    for y in rect.y..rect.y + rect.height {
        let start = page_offset + y as usize * atlas_width as usize + rect.x as usize;
        let end = start + rect.width as usize;
        slot.extend_from_slice(&pixels[start..end]);
    }
    slot
}

fn old_rounded_rect_placeholder(width: u32, height: u32) -> Vec<u8> {
    const PADDING: f32 = 4.0;
    const SPREAD: f32 = 6.0;
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let half_width = (center_x - PADDING).max(1.0);
    let half_height = (center_y - PADDING).max(1.0);
    let mut pixels = Vec::with_capacity(width as usize * height as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 + 0.5 - center_x).abs() - half_width;
            let dy = (y as f32 + 0.5 - center_y).abs() - half_height;
            let outside_x = dx.max(0.0);
            let outside_y = dy.max(0.0);
            let outside_distance = (outside_x * outside_x + outside_y * outside_y).sqrt();
            let inside_distance = dx.max(dy).min(0.0);
            let signed_inside_distance = -(outside_distance + inside_distance);
            pixels.push(
                ((0.5 + signed_inside_distance / SPREAD).clamp(0.0, 1.0) * 255.0).round() as u8,
            );
        }
    }

    pixels
}
