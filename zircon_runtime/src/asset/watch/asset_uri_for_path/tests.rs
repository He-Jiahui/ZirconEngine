use std::path::Path;

use crate::asset::AssetUri;
use crate::core::resource::ResourceLocatorError;

use super::asset_uri_for_path;

#[test]
fn nested_path_projects_to_resource_uri() {
    let assets_root = Path::new("sandbox/assets");
    let path = Path::new("sandbox/assets/materials/grid.zmaterial");

    assert_eq!(
        asset_uri_for_path(assets_root, path).unwrap(),
        AssetUri::parse("res://materials/grid.zmaterial").unwrap()
    );
}

#[test]
fn path_outside_root_is_rejected() {
    let assets_root = Path::new("sandbox/assets");
    let path = Path::new("sandbox/outside/grid.zmaterial");

    assert!(matches!(
        asset_uri_for_path(assets_root, path),
        Err(ResourceLocatorError::EscapeAttempt(_))
    ));
}
