//! Cook the same scene through the real Web bundle writer, varying only UI admission inputs.
use super::{export_root, publish_acceptance_fixture, uri, FixtureAsset, FixtureOptions};
use crate::asset::{
    FontAsset, FontAssetMetadata, FontAssetSourceFormat, FontBlobArtifact, ImportedAsset,
    TextureAsset,
};
use std::path::Path;

const DEFAULT_FONT: &str = "res://fonts/default.font.toml";
const CUSTOM_FONT: &str = "res://fonts/browser-custom.font.toml";

fn project_font(locator: &'static str, preload: bool, corrupt: bool) -> FixtureAsset<'static> {
    let mut asset =
        FontAsset::from_toml_str("source = \"FiraSans-Regular.ttf\"\nfamily = \"Fira Sans\"\n")
            .unwrap();
    let format = FontAssetSourceFormat::Sfnt;
    let mut blob = FontBlobArtifact::from_decoded_bytes(
        format,
        include_bytes!("../../../../assets/fonts/FiraSans-Regular.ttf").to_vec(),
    );
    if corrupt {
        // Deliberately break only the nested hash. WebBundleWriter still writes a valid Raw
        // container and catalog hash, so the product reaches font database admission.
        let mut encoded = serde_json::to_value(&blob).unwrap();
        encoded["content_hash"] = serde_json::to_value([0_u8; 32]).unwrap();
        blob = serde_json::from_value(encoded).unwrap();
        assert!(!blob.has_valid_content_hash());
    }
    asset.metadata = Some(FontAssetMetadata {
        source_format: format,
        face_count: 1,
        cooked_blob: Some(blob),
        ..FontAssetMetadata::default()
    });
    FixtureAsset {
        locator,
        asset: ImportedAsset::Font(asset),
        preload,
    }
}

fn publish_case(
    root: &Path,
    name: &str,
    font_ref: Option<&str>,
    extra_assets: &[FixtureAsset<'_>],
    without_ui_root: bool,
) {
    let output = root.join(name);
    // A fresh directory per case makes accidental replacement or reparse traversal fail.
    std::fs::create_dir(&output).unwrap();
    publish_acceptance_fixture(
        output,
        FixtureOptions {
            font_ref,
            extra_assets,
            without_ui_root,
        },
    );
}

#[test]
#[ignore = "exports browser font-policy fixtures; run with a fresh ZIRCON_WEB_FIXTURE_OUTPUT"]
fn export_webgpu_font_policy_fixtures() {
    let root = export_root();
    publish_case(&root, "no-ui", None, &[], true);
    publish_case(&root, "implicit", None, &[], false);
    publish_case(
        &root,
        "explicit-default-missing",
        Some(DEFAULT_FONT),
        &[],
        false,
    );
    let ready = [project_font(DEFAULT_FONT, true, false)];
    publish_case(&root, "registered-default", None, &ready, false);
    publish_case(
        &root,
        "explicit-custom-missing",
        Some(CUSTOM_FONT),
        &ready,
        false,
    );
    let custom = [project_font(CUSTOM_FONT, true, false)];
    publish_case(
        &root,
        "explicit-custom-ready",
        Some(CUSTOM_FONT),
        &custom,
        false,
    );
    let not_resident = [project_font(DEFAULT_FONT, false, false)];
    publish_case(&root, "registered-not-resident", None, &not_resident, false);
    let wrong_kind = [FixtureAsset {
        locator: DEFAULT_FONT,
        asset: ImportedAsset::Texture(TextureAsset::new_rgba8(
            uri(DEFAULT_FONT),
            1,
            1,
            vec![255, 0, 0, 255],
        )),
        preload: true,
    }];
    publish_case(&root, "registered-wrong-kind", None, &wrong_kind, false);
    let corrupt = [project_font(DEFAULT_FONT, true, true)];
    publish_case(&root, "registered-corrupt-blob", None, &corrupt, false);
}
