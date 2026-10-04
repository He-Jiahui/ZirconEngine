use crate::core::resource::ResourceLocator;

use super::display_name_for_level;

#[test]
fn display_name_prefers_label_when_present() {
    let uri = ResourceLocator::parse("res://scenes/main.scene.toml#CameraPreview").unwrap();
    assert_eq!(
        display_name_for_level(&uri).as_deref(),
        Some("CameraPreview")
    );
}

#[test]
fn display_name_falls_back_to_last_path_segment() {
    let uri = ResourceLocator::parse("res://scenes/main.scene.toml").unwrap();
    assert_eq!(
        display_name_for_level(&uri).as_deref(),
        Some("main.scene.toml")
    );
}
