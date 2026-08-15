use crate::asset::load::texture::{decode_image_file, generate_checker_texture, TextureLoadError};
use crate::asset::tests::project::unique_temp_project_root;

#[test]
fn builtin_checker_texture_has_rgba_payload() {
    let payload = generate_checker_texture();

    assert_eq!(
        payload.rgba.len(),
        payload.width as usize * payload.height as usize * 4
    );
}

#[test]
fn missing_image_file_reports_typed_texture_load_error() {
    let root = unique_temp_project_root("texture_load_missing_image");
    let missing = root.join("missing.png");
    let path = missing.to_string_lossy().to_string();

    let error = decode_image_file(&path).expect_err("missing image should fail");

    match error {
        TextureLoadError::OpenImage {
            path: actual_path,
            source,
        } => {
            assert_eq!(actual_path, path);
            assert!(matches!(source, image::ImageError::IoError(_)));
        }
    }

    let _ = std::fs::remove_dir_all(root);
}
