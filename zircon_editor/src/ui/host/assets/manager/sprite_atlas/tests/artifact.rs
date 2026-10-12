use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;

use crate::ui::host::editor_asset_manager::manager::sprite_atlas::{
    pack_sprite_atlas_sources, SpriteAtlasSourceImage,
};

use super::*;

#[test]
fn sprite_atlas_artifact_writer_writes_png_and_runtime_valid_manifest() {
    let root = unique_temp_project_root("sprite_atlas_artifact_writer");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "SpriteAtlasArtifactProject",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let config = SpriteAtlasBuildConfig {
        output_stem: "icons".to_string(),
        initial_size: (4, 4),
        max_size: (4, 4),
        padding: (0, 0),
    };
    let source = SpriteAtlasSourceImage {
        name: "search".to_string(),
        source_uri: Some(AssetUri::parse("res://ui/search.png").unwrap()),
        width: 2,
        height: 2,
        rgba: vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255],
    };
    let packed = pack_sprite_atlas_sources(&config, &[source]).unwrap();

    let report = write_sprite_atlas_artifacts(&project, &config, &packed).unwrap();

    assert_eq!(
        report.atlas_texture.to_string(),
        "lib://editor-sprite-atlases/icons.png"
    );
    assert_eq!(
        report.manifest.to_string(),
        "lib://editor-sprite-atlases/icons.toml"
    );
    assert!(report.image_path.exists());
    let manifest = fs::read_to_string(&report.manifest_path).unwrap();
    let decoded: SpriteAtlasAsset = toml::from_str(&manifest).unwrap();
    zircon_runtime::asset::validate_sprite_atlas_asset(&decoded).unwrap();
    assert_eq!(decoded.atlas_texture, report.atlas_texture);
    let image = image::open(&report.image_path).unwrap().into_rgba8();
    assert_eq!(image.dimensions(), (4, 4));
    let rect = decoded.entries[0].pixel_rect;
    assert_eq!(image.get_pixel(rect.x, rect.y).0, [1, 2, 3, 255]);
    assert_eq!(image.get_pixel(rect.x + 1, rect.y + 1).0, [10, 11, 12, 255]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sprite_atlas_artifact_writer_rejects_uri_metacharacter_output_stem() {
    let root = unique_temp_project_root("sprite_atlas_artifact_writer_bad_stem");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "SpriteAtlasArtifactProject",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let config = SpriteAtlasBuildConfig {
        output_stem: "icons#dark".to_string(),
        initial_size: (4, 4),
        max_size: (4, 4),
        padding: (0, 0),
    };
    let source = SpriteAtlasSourceImage {
        name: "search".to_string(),
        source_uri: Some(AssetUri::parse("res://ui/search.png").unwrap()),
        width: 2,
        height: 2,
        rgba: vec![255; 16],
    };
    let good_config = SpriteAtlasBuildConfig {
        output_stem: "icons".to_string(),
        ..config.clone()
    };
    let packed = pack_sprite_atlas_sources(&good_config, &[source]).unwrap();

    assert_eq!(
        write_sprite_atlas_artifacts(&project, &config, &packed),
        Err(SpriteAtlasBuildError::InvalidConfig(
            "output_stem must be a single safe file stem".to_string()
        ))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sprite_atlas_artifact_writer_does_not_clone_the_full_rgba_payload() {
    let source = include_str!("../artifact.rs");
    let full_clone = ["packed", ".rgba", ".clone()"].concat();

    assert!(!source.contains(&full_clone));
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_editor_{label}_{nanos}"))
}
