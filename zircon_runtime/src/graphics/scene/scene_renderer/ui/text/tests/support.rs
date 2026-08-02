use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::*;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{AssetUri, FontAsset};
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;

pub(super) fn text_batch(text: &str, mode: UiTextRenderMode) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        route_identity: ScreenSpaceUiTextRouteIdentity::new(
            "runtime.text.test",
            UiNodeId::new(1),
            None,
        ),
        command_generation: 1,
        text: text.to_string(),
        frame: UiFrame::new(0.0, 0.0, 128.0, 24.0),
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: false,
        glyph_artifact_line: None,
        layout_error: None,
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: match mode {
            UiTextRenderMode::Msdf => crate::text::sdf::SdfMode::Msdf,
            UiTextRenderMode::Mtsdf => crate::text::sdf::SdfMode::Mtsdf,
            UiTextRenderMode::Auto | UiTextRenderMode::Native | UiTextRenderMode::Sdf => {
                crate::text::sdf::SdfMode::Sdf
            }
        },
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}

pub(super) struct TextFontProject {
    pub(super) root: PathBuf,
    font_root: PathBuf,
}

impl TextFontProject {
    pub(super) const FONT_REF: &'static str = "res://fonts/late.font.toml";
    pub(super) const SHARED_FIRST_REF: &'static str = "res://fonts/shared-first.font.toml";
    pub(super) const SHARED_SECOND_REF: &'static str = "res://fonts/shared-second.font.toml";

    pub(super) fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let paths = ProjectPaths::from_root(&root).expect("project paths should build");
        paths
            .ensure_layout(&[RelPath::project_assets()])
            .expect("project layout should exist");
        ProjectManifest::new(
            "UI Text Font Cache",
            AssetUri::parse("res://empty.scene.toml").expect("startup uri should parse"),
            1,
        )
        .save(paths.manifest_path())
        .expect("project manifest should save");
        let font_root = paths.asset_root(&RelPath::project_assets()).join("fonts");
        fs::create_dir_all(&font_root).expect("font directory should exist");
        Self { root, font_root }
    }

    pub(super) fn write_font_asset(&self) {
        self.write_named_font_asset("late", Some("Late UI Font"), None);
    }

    pub(super) fn write_default_font_asset(
        &self,
        family: Option<&str>,
        composite_family: Option<&str>,
    ) {
        self.write_named_font_asset("default", family, composite_family);
    }

    pub(super) fn write_shared_font_source(&self) {
        fs::copy(default_font_path(), self.font_root.join("shared.ttf"))
            .expect("shared font fixture should copy");
    }

    pub(super) fn write_shared_font_manifest(&self, stem: &str) {
        let manifest = "source = \"shared.ttf\"\nfamily = \"Shared UI Font\"\n";
        fs::write(self.font_root.join(format!("{stem}.font.toml")), manifest)
            .expect("shared font manifest should write");
    }

    pub(super) fn remove_shared_font_manifest(&self, stem: &str) {
        fs::remove_file(self.font_root.join(format!("{stem}.font.toml")))
            .expect("shared font manifest should be removed");
    }

    fn write_named_font_asset(
        &self,
        stem: &str,
        family: Option<&str>,
        composite_family: Option<&str>,
    ) {
        let font_file = format!("{stem}.ttf");
        fs::copy(default_font_path(), self.font_root.join(&font_file))
            .expect("font fixture should copy");
        let mut manifest = format!("source = {font_file:?}\nrender_mode = \"sdf\"\n");
        if let Some(family) = family {
            manifest.push_str(&format!("family = {family:?}\n"));
        }
        if let Some(composite_family) = composite_family {
            manifest.push_str(&format!(
                "\n[composite_font]\ndefault_family = {composite_family:?}\n"
            ));
        }
        fs::write(self.font_root.join(format!("{stem}.font.toml")), manifest)
            .expect("font manifest should write");
    }

    pub(super) fn remove_font_asset(&self) {
        self.remove_named_font_asset("late");
    }

    pub(super) fn remove_named_font_asset(&self, stem: &str) {
        fs::remove_file(self.font_root.join(format!("{stem}.font.toml")))
            .expect("remove font manifest");
        fs::remove_file(self.font_root.join(format!("{stem}.ttf"))).expect("remove font source");
    }
}

impl Drop for TextFontProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(super) fn default_font_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join("FiraSans-Regular.ttf")
}

pub(super) struct RuntimeFontAssetGuard {
    pub(super) asset_ref: String,
    manifest_path: PathBuf,
    source_path: PathBuf,
}

impl RuntimeFontAssetGuard {
    pub(super) fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let font_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("fonts");
        Self {
            asset_ref: format!("res://fonts/{unique}.font.toml"),
            manifest_path: font_root.join(format!("{unique}.font.toml")),
            source_path: font_root.join(format!("{unique}.ttf")),
        }
    }

    pub(super) fn write(&self) -> FontAsset {
        fs::copy(default_font_path(), &self.source_path).expect("font fixture should copy");
        let source_name = self
            .source_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("font source name should be utf-8");
        let manifest = format!("source = {source_name:?}\nfamily = \"Recovered UI Font\"\n");
        fs::write(&self.manifest_path, &manifest).expect("font manifest should write");
        FontAsset::from_toml_str(&manifest).expect("font manifest should parse")
    }
}

impl Drop for RuntimeFontAssetGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.manifest_path);
        let _ = fs::remove_file(&self.source_path);
    }
}
