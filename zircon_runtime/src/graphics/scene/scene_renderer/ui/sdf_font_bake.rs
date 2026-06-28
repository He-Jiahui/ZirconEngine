use std::collections::HashMap;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::FontFaceId;
use crate::graphics::text::font::FontDatabase;

use super::font_asset::{load_ui_font_manifest_with_asset_manager, LoadedUiFontManifest};
use super::sdf_atlas::{SdfAtlasGlyphKey, SdfAtlasPlan, SdfAtlasRect};

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";
const FALLBACK_ADVANCE_RATIO: f32 = 0.6;

pub(super) struct SdfFontBakeCache {
    fonts: HashMap<FontFaceId, fontsdf::Font>,
}

#[derive(Clone, Debug)]
pub(super) struct SdfAtlasBake {
    pub(super) pixels: Vec<u8>,
    pub(super) glyphs: Vec<SdfBakedGlyph>,
    pub(super) report: SdfAtlasBakeReport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasBakeReport {
    pub(super) slot_count: usize,
    pub(super) visible_glyph_count: usize,
    pub(super) empty_glyph_count: usize,
    pub(super) atlas_byte_len: usize,
    pub(super) nonzero_pixel_count: usize,
    pub(super) loaded_font_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SdfBakedGlyph {
    pub(super) metrics: SdfGlyphMetrics,
    pub(super) visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct SdfGlyphMetrics {
    pub(super) bitmap_width: u32,
    pub(super) bitmap_height: u32,
    pub(super) bitmap_left: f32,
    pub(super) bitmap_bottom: f32,
    pub(super) advance: f32,
    pub(super) ascent: f32,
}

impl SdfFontBakeCache {
    pub(super) fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    pub(super) fn build_atlas(
        &mut self,
        plan: &SdfAtlasPlan,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfAtlasBake {
        let width = plan.atlas_size.x.max(1);
        let height = plan.atlas_size.y.max(1);
        let mut pixels = vec![0; width as usize * height as usize];
        let mut glyphs = Vec::with_capacity(plan.slots.len());

        for slot in &plan.slots {
            let baked = self.bake_glyph(&slot.key, font_database, asset_manager);
            write_glyph_bitmap(
                &mut pixels,
                width,
                height,
                slot.rect,
                baked.metrics.bitmap_width,
                baked.metrics.bitmap_height,
                &baked.bitmap,
            );
            glyphs.push(SdfBakedGlyph {
                metrics: baked.metrics,
                visible: baked.visible,
            });
        }

        let visible_glyph_count = glyphs.iter().filter(|glyph| glyph.visible).count();
        let report = SdfAtlasBakeReport {
            slot_count: glyphs.len(),
            visible_glyph_count,
            empty_glyph_count: glyphs.len().saturating_sub(visible_glyph_count),
            atlas_byte_len: pixels.len(),
            nonzero_pixel_count: pixels.iter().filter(|pixel| **pixel != 0).count(),
            loaded_font_count: self.fonts.len(),
        };

        SdfAtlasBake {
            pixels,
            glyphs,
            report,
        }
    }

    pub(super) fn measure_glyph(
        &mut self,
        glyph: char,
        font: Option<&str>,
        font_family: Option<&str>,
        font_size: f32,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfGlyphMetrics {
        let key = SdfAtlasGlyphKey {
            glyph,
            font: font.map(str::to_string),
            font_family: font_family.map(str::to_string),
            font_size_milli: font_size_milli(font_size),
        };
        self.measure_key(&key, font_database, asset_manager)
    }

    fn measure_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfGlyphMetrics {
        let px = key.font_size_milli as f32 / 1000.0;
        let Some(font) = self.font_for_key(key, font_database, asset_manager) else {
            return fallback_metrics(px);
        };
        let index = glyph_index(font, key.glyph);
        let metrics = font.metrics_indexed_sdf(index, px);
        glyph_metrics(font, px, metrics)
    }

    fn bake_glyph(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> RawBakedGlyph {
        let px = key.font_size_milli as f32 / 1000.0;
        let Some(font) = self.font_for_key(key, font_database, asset_manager) else {
            return RawBakedGlyph::empty(fallback_metrics(px));
        };
        let index = glyph_index(font, key.glyph);
        let (metrics, bitmap) = font.rasterize_indexed_sdf(index, px);
        let metrics = glyph_metrics(font, px, metrics);
        let visible = metrics.bitmap_width > 0
            && metrics.bitmap_height > 0
            && bitmap.iter().any(|value| *value != 0);

        if visible {
            RawBakedGlyph {
                metrics,
                bitmap,
                visible,
            }
        } else {
            RawBakedGlyph::empty(metrics)
        }
    }

    fn font_for_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<&fontsdf::Font> {
        self.font_for_asset(key.font.as_deref(), font_database, asset_manager)
    }

    fn font_for_asset(
        &mut self,
        font_asset: Option<&str>,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<&fontsdf::Font> {
        let requested_face = resolve_font_face(font_asset, font_database, asset_manager);
        let default_face =
            resolve_font_face(Some(DEFAULT_FONT_ASSET), font_database, asset_manager);
        let face = [requested_face, default_face]
            .into_iter()
            .flatten()
            .find(|face| self.ensure_sdf_font(*face, font_database))?;

        self.fonts.get(&face)
    }

    fn ensure_sdf_font(&mut self, face: FontFaceId, font_database: &FontDatabase) -> bool {
        if self.fonts.contains_key(&face) {
            return true;
        }

        if font_database.face_index(face).ok() != Some(0) {
            return false;
        }

        let Some(font) = font_database
            .face_bytes(face)
            .ok()
            .and_then(|bytes| fontsdf::Font::from_bytes(bytes.as_ref()).ok())
        else {
            return false;
        };
        self.fonts.insert(face, font);
        true
    }
}

struct RawBakedGlyph {
    metrics: SdfGlyphMetrics,
    bitmap: Vec<u8>,
    visible: bool,
}

impl RawBakedGlyph {
    fn empty(metrics: SdfGlyphMetrics) -> Self {
        Self {
            metrics,
            bitmap: Vec::new(),
            visible: false,
        }
    }
}

fn resolve_font_face(
    font_asset: Option<&str>,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
) -> Option<FontFaceId> {
    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    let manifest = load_ui_font_manifest_with_asset_manager(asset, Some(asset_manager))?;
    register_loaded_font_manifest(font_database, &manifest)
}

fn register_loaded_font_manifest(
    font_database: &mut FontDatabase,
    manifest: &LoadedUiFontManifest,
) -> Option<FontFaceId> {
    if let Some(asset) = &manifest.asset {
        return font_database
            .register_font_asset(asset, &manifest.source_path)
            .ok()
            .and_then(|faces| faces.first().copied());
    }

    font_database
        .register_font_file(
            &manifest.source_path,
            manifest.family.as_deref(),
            manifest.face_index,
        )
        .ok()
}

fn glyph_index(font: &fontsdf::Font, glyph: char) -> u16 {
    if font.chars().contains_key(&glyph) {
        font.lookup_glyph_index(glyph)
    } else {
        0
    }
}

fn glyph_metrics(font: &fontsdf::Font, px: f32, metrics: fontsdf::Metrics) -> SdfGlyphMetrics {
    let ascent = font
        .inner()
        .horizontal_line_metrics(px)
        .map(|metrics| metrics.ascent)
        .unwrap_or(px);
    SdfGlyphMetrics {
        bitmap_width: metrics.width as u32,
        bitmap_height: metrics.height as u32,
        bitmap_left: metrics.xmin as f32,
        bitmap_bottom: metrics.ymin as f32,
        advance: metrics.advance_width.max(px * FALLBACK_ADVANCE_RATIO),
        ascent,
    }
}

fn fallback_metrics(px: f32) -> SdfGlyphMetrics {
    SdfGlyphMetrics {
        advance: px.max(1.0) * FALLBACK_ADVANCE_RATIO,
        ascent: px.max(1.0),
        ..SdfGlyphMetrics::default()
    }
}

fn font_size_milli(font_size: f32) -> u32 {
    (font_size.max(1.0) * 1000.0).round() as u32
}

fn write_glyph_bitmap(
    pixels: &mut [u8],
    atlas_width: u32,
    atlas_height: u32,
    rect: SdfAtlasRect,
    glyph_width: u32,
    glyph_height: u32,
    glyph_bitmap: &[u8],
) {
    let copy_width = glyph_width.min(rect.width);
    let copy_height = glyph_height.min(rect.height);
    let right = rect.x.saturating_add(copy_width).min(atlas_width);
    let bottom = rect.y.saturating_add(copy_height).min(atlas_height);

    for y in rect.y..bottom {
        for x in rect.x..right {
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            let src = local_y as usize * glyph_width as usize + local_x as usize;
            if let Some(value) = glyph_bitmap.get(src) {
                pixels[y as usize * atlas_width as usize + x as usize] = *value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::ProjectAssetManager;
    use crate::core::math::UVec2;
    use crate::graphics::scene::scene_renderer::ui::sdf_atlas::{SdfAtlasPlan, SdfAtlasSlot};
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
    fn sdf_font_bake_measures_whitespace_without_atlas_bitmap() {
        let mut bake = SdfFontBakeCache::new();
        let mut font_database = FontDatabase::with_default_fallbacks();
        let asset_manager = ProjectAssetManager::default();

        let metrics = bake.measure_glyph(
            ' ',
            Some(DEFAULT_FONT_ASSET),
            Some("Fira Mono"),
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
            Some("Fira Mono"),
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

    #[test]
    fn sdf_font_bake_report_handles_empty_atlas_plan() {
        let mut bake = SdfFontBakeCache::new();
        let mut font_database = FontDatabase::with_default_fallbacks();
        let asset_manager = ProjectAssetManager::default();
        let plan = SdfAtlasPlan {
            atlas_size: UVec2::new(1, 1),
            slots: Vec::new(),
            runs: Vec::new(),
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
                    font: Some(DEFAULT_FONT_ASSET.to_string()),
                    font_family: Some("Fira Mono".to_string()),
                    font_size_milli: 24_000,
                },
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
            slots,
            runs: Vec::new(),
        }
    }

    fn atlas_plan_for_asset(glyph: char, asset_ref: &str) -> SdfAtlasPlan {
        SdfAtlasPlan {
            atlas_size: UVec2::new(64, 64),
            slots: vec![SdfAtlasSlot {
                key: SdfAtlasGlyphKey {
                    glyph,
                    font: Some(asset_ref.to_string()),
                    font_family: Some("Fira Unsupported Face".to_string()),
                    font_size_milli: 24_000,
                },
                rect: SdfAtlasRect {
                    x: 0,
                    y: 0,
                    width: 64,
                    height: 64,
                },
            }],
            runs: Vec::new(),
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
}
