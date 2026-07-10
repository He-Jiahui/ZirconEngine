use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};
use zircon_runtime::asset::{AssetImportContext, AssetUri, ImportedAsset};
use zircon_runtime::core::framework::render::RenderImageDimension;

use crate::plugin_registration;

#[test]
fn six_file_cubemap_manifest_imports_in_wgpu_face_order() {
    let root = test_root("six-files");
    let sources = (0_u8..6)
        .map(|face| {
            let name = format!("face_{face}.png");
            save_image(&root.join(&name), solid_image(2, 2, [face, 0, 0, 255]));
            format!("\"{name}\"")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let manifest = format!("layout = \"six_files\"\nsources = [{sources}]\n");

    let texture = import_manifest(&root, "six.zcube", &manifest);

    assert_cube_descriptor(&texture, 2);
    for face in 0_usize..6 {
        assert_eq!(texture.rgba[face * 2 * 2 * 4], face as u8);
    }
}

#[test]
fn horizontal_cross_cubemap_manifest_uses_cmft_tile_layout() {
    let root = test_root("horizontal-cross");
    let mut cross = RgbaImage::new(8, 6);
    let offsets = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)];
    for (face, (column, row)) in offsets.into_iter().enumerate() {
        fill_tile(&mut cross, column, row, [face as u8, 0, 0, 255]);
    }
    save_image(&root.join("cross.png"), cross);
    let manifest = "layout = \"horizontal_cross\"\nsources = [\"cross.png\"]\n";

    let texture = import_manifest(&root, "horizontal.zcube", manifest);

    assert_cube_descriptor(&texture, 2);
    for face in 0_usize..6 {
        assert_eq!(texture.rgba[face * 2 * 2 * 4], face as u8);
    }
}

#[test]
fn vertical_cross_cubemap_manifest_rotates_negative_z_like_cmft() {
    let root = test_root("vertical-cross");
    let mut cross = RgbaImage::new(6, 8);
    let offsets = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1)];
    for (face, (column, row)) in offsets.into_iter().enumerate() {
        fill_tile(&mut cross, column, row, [face as u8, 0, 0, 255]);
    }
    let negative_z = [[1_u8, 2_u8], [3_u8, 4_u8]];
    for (y, row) in negative_z.into_iter().enumerate() {
        for (x, value) in row.into_iter().enumerate() {
            cross.put_pixel(2 + x as u32, 6 + y as u32, Rgba([value, 0, 0, 255]));
        }
    }
    save_image(&root.join("cross.png"), cross);
    let manifest = "layout = \"vertical_cross\"\nsources = [\"cross.png\"]\n";

    let texture = import_manifest(&root, "vertical.zcube", manifest);

    assert_cube_descriptor(&texture, 2);
    let negative_z_offset = 5 * 2 * 2 * 4;
    let values = (0..4)
        .map(|pixel| texture.rgba[negative_z_offset + pixel * 4])
        .collect::<Vec<_>>();
    assert_eq!(values, vec![4, 3, 2, 1]);
}

#[test]
fn equirectangular_cubemap_manifest_resamples_all_six_faces() {
    let root = test_root("equirectangular");
    let image = RgbaImage::from_fn(8, 4, |x, y| Rgba([(x * 24) as u8, (y * 64) as u8, 0, 255]));
    save_image(&root.join("environment.png"), image);
    let manifest = "layout = \"equirectangular\"\nsources = [\"environment.png\"]\n";

    let texture = import_manifest(&root, "environment.zcube", manifest);

    assert_cube_descriptor(&texture, 2);
    assert_eq!(texture.rgba.len(), 2 * 2 * 6 * 4);
    assert_ne!(&texture.rgba[0..16], &texture.rgba[16..32]);
}

#[test]
fn texture_array_manifest_imports_files_and_stacked_slice() {
    let root = test_root("array");
    save_image(&root.join("red.png"), solid_image(2, 2, [1, 0, 0, 255]));
    save_image(&root.join("blue.png"), solid_image(2, 2, [2, 0, 0, 255]));
    let files_manifest = "sources = [\"red.png\", \"blue.png\"]\n";

    let files = import_manifest(&root, "files.zarray", files_manifest);

    assert_eq!(files.render_image_descriptor().array_layer_count, 2);
    assert_eq!(files.rgba[0], 1);
    assert_eq!(files.rgba[16], 2);

    let stacked = RgbaImage::from_fn(2, 4, |_x, y| Rgba([if y < 2 { 3 } else { 4 }, 0, 0, 255]));
    save_image(&root.join("stacked.png"), stacked);
    let slice_manifest = "source = \"stacked.png\"\nrow_count = 2\n";

    let sliced = import_manifest(&root, "slice.zarray", slice_manifest);

    assert_eq!(sliced.render_image_descriptor().array_layer_count, 2);
    assert_eq!(sliced.rgba[0], 3);
    assert_eq!(sliced.rgba[16], 4);
}

fn import_manifest(root: &Path, name: &str, manifest: &str) -> zircon_runtime::asset::TextureAsset {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(Path::new(name))
        .expect("manifest importer registration");
    let source_path = root.join(name);
    let context = AssetImportContext::new(
        source_path,
        AssetUri::parse(&format!("res://textures/{name}")).expect("valid manifest uri"),
        manifest.as_bytes().to_vec(),
        Default::default(),
    );
    let outcome = importer.import(&context).expect("manifest import");
    match &outcome.root_entry().expect("root texture entry").asset {
        ImportedAsset::Texture(texture) => texture.clone(),
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

fn assert_cube_descriptor(texture: &zircon_runtime::asset::TextureAsset, face_size: u32) {
    let descriptor = texture.render_image_descriptor();
    assert_eq!(texture.width, face_size);
    assert_eq!(texture.height, face_size);
    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.array_layer_count, 6);
}

fn solid_image(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba(color))
}

fn fill_tile(image: &mut RgbaImage, column: u32, row: u32, color: [u8; 4]) {
    for y in 0..2 {
        for x in 0..2 {
            image.put_pixel(column * 2 + x, row * 2 + y, Rgba(color));
        }
    }
}

fn save_image(path: &Path, image: RgbaImage) {
    image.save(path).expect("save test image");
}

fn test_root(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-texture-importer-{label}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("create test directory");
    root
}
