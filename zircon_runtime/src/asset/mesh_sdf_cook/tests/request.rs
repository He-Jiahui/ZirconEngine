use super::*;

#[test]
fn mesh_sdf_cook_is_disabled_without_an_explicit_request() {
    assert_eq!(
        MeshSdfCookRequest::from_import_settings(&toml::Table::new()).unwrap(),
        MeshSdfCookRequest::Disabled
    );
}

#[test]
fn enabled_request_preserves_bounded_settings() {
    let settings = toml::from_str(
        r#"
                [mesh_sdf]
                enabled = true
                max_dimension = 24
                max_voxel_count = 8192
                max_payload_bytes = 32768
                surface_band_voxels = 3
                two_sided = true
            "#,
    )
    .unwrap();

    assert_eq!(
        MeshSdfCookRequest::from_import_settings(&settings)
            .unwrap()
            .settings(),
        Some(MeshSdfCookSettings {
            max_dimension: 24,
            max_voxel_count: 8192,
            max_payload_bytes: 32768,
            surface_band_voxels: 3,
            two_sided: true,
        })
    );
}

#[test]
fn malformed_or_unbounded_settings_are_rejected() {
    let settings = toml::from_str(
        r#"
                [mesh_sdf]
                enabled = true
                max_dimension = 1024
            "#,
    )
    .unwrap();

    assert!(MeshSdfCookRequest::from_import_settings(&settings)
        .unwrap_err()
        .contains("max_dimension"));
}
