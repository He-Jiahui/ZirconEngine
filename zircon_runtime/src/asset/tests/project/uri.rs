use crate::core::resource::ResourceScheme;

use crate::asset::AssetUri;

#[test]
fn asset_uri_normalizes_res_and_lib_paths() {
    let res = AssetUri::parse("res://textures\\environment/sky.png").unwrap();
    let lib = AssetUri::parse("lib://imports\\model.cache").unwrap();

    assert_eq!(res.scheme(), ResourceScheme::Res);
    assert_eq!(res.path(), "textures/environment/sky.png");
    assert_eq!(res.to_string(), "res://textures/environment/sky.png");
    assert_eq!(lib.scheme(), ResourceScheme::Library);
    assert_eq!(lib.path(), "imports/model.cache");
    assert_eq!(lib.to_string(), "lib://imports/model.cache");
}

#[test]
fn asset_uri_rejects_escape_attempts() {
    assert!(AssetUri::parse("res://../outside.txt").is_err());
    assert!(AssetUri::parse("lib://../../outside.bin").is_err());
    assert!(AssetUri::parse("res://folder/../../outside.txt").is_err());
}
