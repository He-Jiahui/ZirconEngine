use super::*;

#[test]
fn scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields() {
    let camera_target = AssetReference::new(
        AssetUuid::from_stable_label("camera-target"),
        AssetUri::parse("res://textures/camera-target.png").unwrap(),
    );
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 9,
            name: "RenderCamera".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 0x0000_0002,
            mobility: SceneMobilityAsset::Dynamic,
            camera: Some(SceneCameraAsset {
                core_pipeline: CorePipelineKind::Core2d,
                projection_mode: ProjectionMode::Orthographic,
                fov_y_radians: 0.75,
                ortho_size: 12.0,
                z_near: 0.05,
                z_far: 500.0,
                target: SceneCameraTargetAsset::Texture {
                    texture: camera_target.clone(),
                },
                viewport: Some(SceneViewportRectAsset {
                    physical_position: [32, 48],
                    physical_size: [640, 360],
                    depth_min: 0.1,
                    depth_max: 0.9,
                }),
                order: 3,
                active: false,
                hdr: true,
                exposure_ev100: 11.0,
                clear_color: RenderCameraClearColor::None,
                msaa_samples: 4,
                post_process_settings: None,
            }),
            mesh: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: Vec::new(),
        }],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert_eq!(loaded.direct_references(), vec![camera_target]);
    assert!(document.contains("core_pipeline"));
    assert!(document.contains("projection_mode"));
    assert!(document.contains("camera-target.png"));
}

#[test]
fn scene_camera_asset_defaults_bevy_camera_fields_when_omitted() {
    let document = r#"
[[entities]]
entity = 9
name = "LegacyCamera"
parent = 0
active = true
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
camera = { fov_y_radians = 1.0, z_near = 0.25, z_far = 900.0 }
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let camera = loaded.entities[0].camera.as_ref().unwrap();

    assert_eq!(camera.core_pipeline, CorePipelineKind::Core3d);
    assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
    assert_eq!(camera.ortho_size, 5.0);
    assert!(matches!(
        &camera.target,
        SceneCameraTargetAsset::PrimarySurface
    ));
    assert_eq!(camera.viewport, None);
    assert_eq!(camera.order, 0);
    assert!(camera.active);
    assert!(!camera.hdr);
    assert_eq!(camera.clear_color, RenderCameraClearColor::Default);
    assert_eq!(camera.msaa_samples, 1);
    assert_eq!(camera.post_process_settings, None);
}
