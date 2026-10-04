use super::ViewerMaterialFixture;

#[test]
fn fixture_identities_are_stable() {
    assert_eq!(
        ViewerMaterialFixture::MetalMirror.project_asset_identity_prefix(),
        "viewer-project-v4"
    );
    assert_eq!(
        ViewerMaterialFixture::MetalMirror.cli_value(),
        "metal-mirror"
    );
    assert_eq!(
        ViewerMaterialFixture::DielectricIor.cli_value(),
        "dielectric-ior"
    );
    assert_eq!(
        ViewerMaterialFixture::DielectricIor.project_root_component(),
        Some("dielectric-ior")
    );
    assert_eq!(
        ViewerMaterialFixture::DielectricIor.project_asset_identity_prefix(),
        "viewer-project-v4/dielectric-ior"
    );
}

#[test]
fn dielectric_fixture_has_a_non_default_f0_input_and_separate_project_identity() {
    assert_eq!(ViewerMaterialFixture::MetalMirror.dielectric_ior(), None);
    assert_eq!(
        ViewerMaterialFixture::DielectricIor.dielectric_ior(),
        Some(2.0)
    );
    assert!(ViewerMaterialFixture::DielectricIor.requires_generic_forward_pipeline());
    assert!(!ViewerMaterialFixture::MetalMirror.requires_generic_forward_pipeline());
}

#[test]
fn fixture_cli_values_are_closed() {
    assert_eq!(
        ViewerMaterialFixture::from_cli_value("metal-mirror"),
        Ok(ViewerMaterialFixture::MetalMirror)
    );
    assert_eq!(
        ViewerMaterialFixture::from_cli_value("dielectric-ior"),
        Ok(ViewerMaterialFixture::DielectricIor)
    );
    assert!(ViewerMaterialFixture::from_cli_value("glass").is_err());
}
