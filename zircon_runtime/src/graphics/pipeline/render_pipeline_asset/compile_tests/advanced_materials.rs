use super::*;
use crate::core::framework::render::AdvancedPbrMaterialFrameUsage;

#[test]
fn render_advanced_material_scene_copy_is_absent_without_specular_transmission() {
    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline.compile(&test_extract()).unwrap();

        assert!(
            !compiled
                .graph()
                .passes()
                .iter()
                .any(|pass| pass.name == "transmission.scene_copy")
        );
        assert_pass_does_not_read(
            &compiled,
            "transparent-mesh",
            PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
        );
        assert!(
            !compiled
                .graph()
                .passes()
                .iter()
                .any(|pass| pass.name.starts_with("transmission-mesh"))
        );
    }
}

#[test]
fn render_advanced_material_scene_copy_runs_after_sky_before_transparency() {
    let mut extract = test_extract();
    extract.lighting.advanced_lighting.material_features = AdvancedPbrMaterialFrameUsage {
        specular_transmission: true,
        ..Default::default()
    };

    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline.compile(&extract).unwrap();
        let pass_names = compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>();
        let sky = pass_names
            .iter()
            .position(|pass| *pass == "preview-sky")
            .unwrap();
        let scene_copy = pass_names
            .iter()
            .position(|pass| *pass == "transmission.scene_copy")
            .unwrap();
        let transparent = pass_names
            .iter()
            .position(|pass| *pass == "transparent-mesh")
            .unwrap();

        assert!(sky < scene_copy && scene_copy < transparent);
        assert_pass_reads(
            &compiled,
            "transmission.scene_copy",
            PostProcessGraphResourceNames::SCENE_COLOR,
        );
        assert_pass_writes(
            &compiled,
            "transmission.scene_copy",
            PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
        );
        assert_pass_reads(
            &compiled,
            "transmission-mesh.0",
            PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
        );
        assert_pass_does_not_read(
            &compiled,
            "transparent-mesh",
            PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
        );

        let copy_desc = texture_lifetime(
            &compiled,
            PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
        );
        assert_eq!(copy_desc.format, crate::rhi::TextureFormat::Rgba16Float);
        assert_eq!(copy_desc.sample_count, 1);
    }
}

#[test]
fn render_advanced_material_transmission_steps_alternate_copy_and_nonoverlapping_draws() {
    let mut extract = test_extract();
    extract.lighting.advanced_lighting.material_features = AdvancedPbrMaterialFrameUsage {
        specular_transmission: true,
        ..Default::default()
    };
    extract.lighting.advanced_lighting.screen_space_transmission =
        crate::core::framework::render::ScreenSpaceTransmissionSettings::new(3);

    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline.compile(&extract).unwrap();
        let pass_names = compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>();
        let expected = [
            "transmission.scene_copy",
            "transmission-mesh.0",
            "transmission.scene_copy.1",
            "transmission-mesh.1",
            "transmission.scene_copy.2",
            "transmission-mesh.2",
        ];
        let indices = expected
            .iter()
            .map(|name| pass_names.iter().position(|pass| pass == name).unwrap())
            .collect::<Vec<_>>();

        assert!(indices.windows(2).all(|window| window[0] < window[1]));
        for step in 0..3 {
            let draw_name = format!("transmission-mesh.{step}");
            assert_pass_reads(
                &compiled,
                &draw_name,
                PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
            );
            assert_pass_writes(
                &compiled,
                &draw_name,
                PostProcessGraphResourceNames::SCENE_COLOR,
            );
        }
    }
}

#[test]
fn render_advanced_material_zero_copy_steps_keep_one_environment_only_draw() {
    let mut extract = test_extract();
    extract.lighting.advanced_lighting.material_features = AdvancedPbrMaterialFrameUsage {
        specular_transmission: true,
        ..Default::default()
    };
    extract.lighting.advanced_lighting.screen_space_transmission =
        crate::core::framework::render::ScreenSpaceTransmissionSettings::new(0);

    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    assert!(
        !compiled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name.starts_with("transmission.scene_copy"))
    );
    assert!(
        compiled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name == "transmission-mesh.0")
    );
    assert_pass_does_not_read(
        &compiled,
        "transmission-mesh.0",
        PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
    );
}

#[test]
fn render_advanced_opaque_forward_runs_after_sky_before_scene_copy() {
    let mut extract = test_extract();
    extract.lighting.advanced_lighting.material_features = AdvancedPbrMaterialFrameUsage {
        clearcoat: true,
        specular_transmission: true,
        late_forward_opaque: true,
        ..Default::default()
    };

    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        let compiled = pipeline.compile(&extract).unwrap();
        let pass_names = compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>();
        let sky = pass_names
            .iter()
            .position(|pass| *pass == "preview-sky")
            .unwrap();
        let advanced = pass_names
            .iter()
            .position(|pass| *pass == "advanced-pbr-opaque")
            .unwrap();
        let scene_copy = pass_names
            .iter()
            .position(|pass| *pass == "transmission.scene_copy")
            .unwrap();
        let transparent = pass_names
            .iter()
            .position(|pass| *pass == "transparent-mesh")
            .unwrap();

        assert!(sky < advanced && advanced < scene_copy && scene_copy < transparent);
        assert_pass_writes(
            &compiled,
            "advanced-pbr-opaque",
            PostProcessGraphResourceNames::SCENE_COLOR,
        );
    }
}
