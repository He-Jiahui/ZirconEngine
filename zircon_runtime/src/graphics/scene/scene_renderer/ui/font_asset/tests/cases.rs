use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::asset::FontAssetRenderStrategy;

#[test]
fn render_strategy_default_mode_feeds_ui_font_default() {
    let manifest = font_manifest(
        None,
        FontAssetRenderStrategy {
            default_mode: Some(UiTextRenderMode::Sdf),
            allow_native: None,
            allow_sdf: None,
        },
    );

    assert_eq!(
        effective_ui_font_render_mode(&manifest),
        Some(UiTextRenderMode::Sdf)
    );
}

#[test]
fn schema_v1_render_mode_takes_priority_over_strategy_default_mode() {
    let manifest = font_manifest(
        Some(UiTextRenderMode::Native),
        FontAssetRenderStrategy {
            default_mode: Some(UiTextRenderMode::Sdf),
            allow_native: None,
            allow_sdf: None,
        },
    );

    assert_eq!(
        effective_ui_font_render_mode(&manifest),
        Some(UiTextRenderMode::Native)
    );
}

#[test]
fn render_strategy_constraints_clamp_disallowed_auto_default() {
    let manifest = font_manifest(
        None,
        FontAssetRenderStrategy {
            default_mode: Some(UiTextRenderMode::Auto),
            allow_native: Some(false),
            allow_sdf: Some(true),
        },
    );

    assert_eq!(
        effective_ui_font_render_mode(&manifest),
        Some(UiTextRenderMode::Sdf)
    );
}

// Keeps the Frameworks05 -> Render18 graphics-only gate on canonical owner wiring.
#[test]
fn font_asset_wiring_reaches_render_volumetric_graphics_only_gate() {
    let temp = TempDirGuard::new("zircon-font-asset-wiring");
    let manifest_path = temp.path.join("wired.font.toml");
    let local_font = temp.path.join("wired.ttf");
    fs::copy(default_font_path(), &local_font).expect("font fixture should copy");
    fs::write(
        &manifest_path,
        "source = \"wired.ttf\"\nfamily = \"Wired Family\"\nrender_mode = \"sdf\"\n",
    )
    .expect("font manifest should be written");

    let loaded = load_ui_font_manifest_with_asset_manager(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
        None,
    )
    .expect("graphics font adapter should load through the canonical text source owner");

    assert_eq!(loaded.source_path, local_font);
    assert_eq!(loaded.family.as_deref(), Some("Wired Family"));
    assert_eq!(loaded.render_mode, Some(UiTextRenderMode::Sdf));
}

fn font_manifest(
    render_mode: Option<UiTextRenderMode>,
    render_strategy: FontAssetRenderStrategy,
) -> FontAsset {
    FontAsset {
        source: "FiraMono-subset.ttf".to_string(),
        family: Some("Studio Mono".to_string()),
        render_mode,
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        composite_font: None,
        render_strategy,
        metadata: None,
    }
}

fn default_font_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join("FiraMono-subset.ttf")
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("temp dir should be created");
        Self { path }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
