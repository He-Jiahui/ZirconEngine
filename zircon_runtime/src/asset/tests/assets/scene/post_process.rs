use super::*;

#[test]
fn scene_asset_toml_roundtrip_preserves_post_process_components() {
    let post_process_settings = ScenePostProcessSettingsAsset {
        bloom: SceneBloomSettingsAsset {
            threshold: 0.3,
            intensity: 0.7,
            radius: 0.45,
        },
        color_grading: SceneColorGradingSettingsAsset {
            exposure: 0.8,
            contrast: 1.2,
            saturation: 0.7,
            gamma: 1.05,
            tint: [0.78, 0.86, 1.0],
        },
        effect_stack: ScenePostProcessEffectStackAsset {
            tonemap: SceneTonemapSettingsAsset {
                operator: SceneTonemapOperatorAsset::Aces,
                exposure_bias: -0.2,
                white_point: 1.3,
            },
            vignette: SceneVignetteSettingsAsset {
                intensity: 0.42,
                smoothness: 0.64,
                roundness: 0.9,
            },
            grain: SceneFilmGrainSettingsAsset {
                intensity: 0.05,
                response: 0.8,
            },
            dither: SceneDitherSettingsAsset {
                intensity: 0.02,
                scale: 1.5,
            },
            chromatic_aberration: SceneChromaticAberrationSettingsAsset {
                intensity: 0.03,
                sample_spread: 1.2,
            },
            fog: SceneFogSettingsAsset {
                density: 0.08,
                height_falloff: 0.12,
                color: [0.2, 0.24, 0.32],
            },
        },
    };
    let post_process_volume = ScenePostProcessVolumeAsset {
        active: true,
        is_global: true,
        priority: 2.0,
        weight: 0.75,
        blend_distance: 0.0,
        profile: ScenePostProcessVolumeProfileAsset {
            volumetric_fog: Some(SceneVolumetricFogSettingsAsset {
                density: 0.12,
                albedo: [0.8, 0.9, 1.0],
                phase_g: 0.65,
                height_falloff: 0.08,
                scattering_intensity: 1.5,
                depth_distribution_exp: 2.5,
                temporal: true,
            }),
            bloom: Some(SceneBloomSettingsAsset {
                threshold: 0.2,
                intensity: 0.9,
                radius: 0.6,
            }),
            color_grading: None,
            effect_stack: Some(ScenePostProcessEffectStackAsset {
                tonemap: SceneTonemapSettingsAsset {
                    operator: SceneTonemapOperatorAsset::Filmic,
                    exposure_bias: -0.1,
                    white_point: 1.15,
                },
                ..ScenePostProcessEffectStackAsset::default()
            }),
        },
    };
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 90,
                name: "MoodCamera".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    post_process_settings: Some(post_process_settings),
                    ..SceneCameraAsset::default()
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
            },
            SceneEntityAsset {
                entity: 91,
                name: "GlobalMoodVolume".to_string(),
                parent: None,
                transform: TransformAsset::default(),
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Static,
                camera: None,
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: Some(post_process_volume),
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
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
    assert!(document.contains("post_process_settings"));
    assert!(document.contains("post_process_volume"));
    assert!(document.contains("chromatic_aberration"));
    assert!(document.contains("volumetric_fog"));
    assert!(loaded.overview().entities[0].has_post_process_settings);
    assert!(loaded.overview().entities[1].has_post_process_volume);
}
