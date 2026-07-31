use std::{path::PathBuf, sync::Arc};

use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{
    AssetUri, FontAsset, FontAssetFamilyMember, FontAssetRenderStrategy, FontAssetVariationCoord,
};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

pub(super) const VARIABLE_FONT_ASSET_URI: &str = "res://fonts/bahnschrift-variable.font.toml";
pub(super) const VARIABLE_FONT_NARROW_FAMILY: &str = "Zircon Bahnschrift Narrow";
pub(super) const VARIABLE_FONT_WIDE_FAMILY: &str = "Zircon Bahnschrift Wide";

pub(super) fn product_fixture_asset_manager() -> (Arc<ProjectAssetManager>, PathBuf) {
    assert_checked_in_default_composite_package();
    let root = std::env::temp_dir().join(format!(
        "zircon-runtime-text-product-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock after unix epoch")
            .as_nanos()
    ));
    let paths = ProjectPaths::from_root(&root).expect("text product fixture project paths");
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .expect("text product fixture layout");
    let texture_uri =
        AssetUri::parse("res://ui/rich-inline-checker.png").expect("rich inline texture locator");
    ProjectManifest::new("RuntimeTextProductProof", texture_uri.clone(), 1)
        .save(paths.manifest_path())
        .expect("text product fixture manifest");
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    write_checker_texture(&asset_root);
    copy_checked_in_default_composite_font_package(&asset_root);
    #[cfg(target_os = "windows")]
    write_variable_font_asset(&asset_root);

    let manager = Arc::new(ProjectAssetManager::default());
    manager
        .open_project(root.to_string_lossy().as_ref())
        .expect("open text product fixture project");
    let texture_id = manager
        .resolve_asset_id(&texture_uri)
        .expect("imported rich inline texture id");
    manager
        .load_texture_asset(texture_id)
        .expect("load imported rich inline texture");
    let default_font_uri = AssetUri::parse("res://fonts/default.font.toml")
        .expect("default font product asset locator");
    let default_font_id = manager
        .resolve_asset_id(&default_font_uri)
        .expect("imported default CompositeFont asset id");
    let default_font = manager
        .load_font_asset(default_font_id)
        .expect("load imported default CompositeFont asset");
    assert_eq!(default_font.face_index, 0);
    assert!(default_font.composite_font.is_some());
    #[cfg(target_os = "windows")]
    {
        let font_uri =
            AssetUri::parse(VARIABLE_FONT_ASSET_URI).expect("variable font product asset locator");
        let font_id = manager
            .resolve_asset_id(&font_uri)
            .expect("imported variable font asset id");
        let asset = manager
            .load_font_asset(font_id)
            .expect("load imported variable font asset");
        assert_eq!(asset.family_members.len(), 2);
    }
    (manager, root)
}

fn assert_checked_in_default_composite_package() {
    let font_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let manifest = std::fs::read_to_string(font_dir.join("default.font.toml"))
        .expect("checked-in default CompositeFont manifest");
    let asset = FontAsset::from_toml_str(&manifest).expect("parse default CompositeFont manifest");
    let composite = asset
        .composite_font
        .as_ref()
        .expect("default font must activate a CompositeFont");
    assert_eq!(
        composite.sub_fonts.first().map(|font| font.family.as_str()),
        Some("Zircon Noto Sans CJK SC Proof")
    );
    let bytes = std::fs::read(font_dir.join(&asset.source))
        .expect("checked-in default CompositeFont source");
    let face = ttf_parser::Face::parse(&bytes, 1).expect("checked-in CJK face 1");
    assert!("中文排版引擎文本与布局竖排标点验证"
        .chars()
        .all(|character| face.glyph_index(character).is_some()));
}

fn copy_checked_in_default_composite_font_package(asset_root: &std::path::Path) {
    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let destination_dir = asset_root.join("fonts");
    std::fs::create_dir_all(&destination_dir).expect("default CompositeFont asset directory");
    for file_name in ["default.font.toml", "ZirconDefaultComposite-subset.ttc"] {
        std::fs::copy(source_dir.join(file_name), destination_dir.join(file_name))
            .expect("copy checked-in default CompositeFont package into product fixture");
    }
}

fn write_checker_texture(asset_root: &std::path::Path) {
    let texture_path = asset_root.join("ui").join("rich-inline-checker.png");
    std::fs::create_dir_all(texture_path.parent().expect("texture parent"))
        .expect("rich inline texture directory");
    let image = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_fn(8, 8, |x, y| {
        match (x >= 4, y >= 4) {
            (false, false) => image::Rgba([255, 28, 28, 255]),
            (true, false) => image::Rgba([28, 255, 28, 255]),
            (false, true) => image::Rgba([28, 28, 255, 255]),
            (true, true) => image::Rgba([255, 220, 28, 255]),
        }
    });
    image
        .save(&texture_path)
        .expect("write rich inline checker texture");
}

#[cfg(target_os = "windows")]
fn write_variable_font_asset(asset_root: &std::path::Path) {
    const SOURCE: &str = r"C:\Windows\Fonts\bahnschrift.ttf";
    let source_bytes = std::fs::read(SOURCE).expect("Windows Bahnschrift variable-font fixture");
    let face = ttf_parser::Face::parse(&source_bytes, 0).expect("parse Bahnschrift variable font");
    let width_axis = face
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width variation axis");
    assert!(width_axis.min_value < width_axis.max_value);

    let font_dir = asset_root.join("fonts");
    std::fs::create_dir_all(&font_dir).expect("variable font asset directory");
    std::fs::write(font_dir.join("bahnschrift-variable.ttf"), source_bytes)
        .expect("copy Bahnschrift variable font into fixture project");
    let asset = FontAsset {
        source: "bahnschrift-variable.ttf".to_string(),
        family: Some(VARIABLE_FONT_NARROW_FAMILY.to_string()),
        render_mode: Some(UiTextRenderMode::Sdf),
        face_index: 0,
        family_members: vec![
            variable_width_member(VARIABLE_FONT_NARROW_FAMILY, width_axis.min_value),
            variable_width_member(VARIABLE_FONT_WIDE_FAMILY, width_axis.max_value),
        ],
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        composite_font: None,
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
    };
    std::fs::write(
        font_dir.join("bahnschrift-variable.font.toml"),
        asset
            .to_toml_string()
            .expect("serialize variable font product asset"),
    )
    .expect("write variable font product asset");
}

#[cfg(target_os = "windows")]
fn variable_width_member(family: &str, value: f32) -> FontAssetFamilyMember {
    FontAssetFamilyMember {
        family: family.to_string(),
        face_index: 0,
        weight: Some(400),
        width_class: None,
        style: None,
        variations: vec![FontAssetVariationCoord {
            tag: "wdth".to_string(),
            value,
        }],
    }
}
