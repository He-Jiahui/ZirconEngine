use super::*;

#[test]
fn missing_virtual_geometry_settings_stay_disabled() {
    assert_eq!(
        VirtualGeometryCookRequest::from_import_settings(&toml::Table::new()).unwrap(),
        VirtualGeometryCookRequest::Disabled
    );
}

#[test]
fn enabled_request_preserves_explicit_cook_configuration() {
    let settings = toml::from_str(
        r#"
                [virtual_geometry]
                enabled = true
                cluster_triangle_count = 8
                page_cluster_count = 4
            "#,
    )
    .unwrap();
    let request = VirtualGeometryCookRequest::from_import_settings(&settings).unwrap();

    assert!(request.is_enabled());
    assert_eq!(
        request.cook_config_for(Some("Mesh0"), "res://models/mesh.gltf"),
        Some(VirtualGeometryCookConfig {
            cluster_triangle_count: 8,
            page_cluster_count: 4,
            mesh_name: Some("Mesh0".to_string()),
            source_hint: Some("res://models/mesh.gltf".to_string()),
        })
    );
}

#[test]
fn malformed_virtual_geometry_settings_are_rejected() {
    let settings = toml::from_str(
        r#"
                [virtual_geometry]
                enabled = true
                cluster_triangle_count = 0
            "#,
    )
    .unwrap();

    assert!(VirtualGeometryCookRequest::from_import_settings(&settings)
        .unwrap_err()
        .contains("cluster_triangle_count"));
}
