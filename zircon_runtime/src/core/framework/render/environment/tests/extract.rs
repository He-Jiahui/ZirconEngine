use super::*;
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

#[test]
fn default_environment_is_disabled() {
    let environment = EnvironmentExtract::default();

    assert!(!environment.skybox_enabled());
    assert!(environment.ibl_bake_key().is_none());
}

#[test]
fn preview_skybox_flag_maps_to_procedural_environment() {
    let environment = EnvironmentExtract::from_preview_skybox_enabled(true);

    assert!(environment.skybox_enabled());
    assert!(environment.ibl_bake_key().is_some());
}

#[test]
fn source_cubemap_extract_supplies_ibl_bake_request_shape() {
    let environment = EnvironmentExtract::source_cubemap(SourceCubemapEnvironment::new(
        crate::core::framework::render::build_source_cubemap_from_equirect(4, |_, _| {
            [0.25, 0.5, 0.75, 1.0]
        }),
        2,
        [1, 2, 3, 4],
    ));

    let request = environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::IEM)
        .expect("source cubemap environment should produce an IBL bake request");

    assert!(
        environment
            .skybox
            .source_cubemap_environment()
            .and_then(SourceCubemapEnvironment::prepared_upload_artifact)
            .is_some(),
        "environment extraction must prepare uploads before render submission"
    );

    assert_eq!(request.source_face_size(), 4);
    assert_eq!(request.source_mip_count(), 3);
    assert_eq!(request.pmrem_face_size(), SOURCE_CUBEMAP_PMREM_FACE_SIZE);
    assert_eq!(request.pmrem_mip_count(), SOURCE_CUBEMAP_PMREM_MIP_COUNT);
    assert_eq!(request.required_contents(), IblBakeArtifactContents::IEM);
}

#[test]
fn baked_environment_requires_one_light_set_generation() {
    let lightmaps = LightmapConsumeContract::new(
        3,
        ResourceId::from_stable_label("res://lighting/test.lightmap-array"),
        super::super::LightmapAtlasDescriptor {
            page_size: 4,
            page_count: 1,
            format: super::super::LightmapAtlasFormat::Rgba16Float,
        },
        vec![(
            11,
            super::super::LightmapInstanceSlot {
                atlas_page: 0,
                uv_rect: Vec4::new(1.0, 1.0, 0.0, 0.0),
            },
        )],
    );
    let mismatched_grid = LightProbeGridData {
        light_set_generation: 4,
        bounds_min: Vec3::ZERO,
        cell_size: Vec3::ONE,
        dims: [1, 1, 1],
        sh: vec![super::super::ShL2Rgb::default()],
    };

    assert_eq!(
        EnvironmentExtract::disabled()
            .try_with_baked_lighting(lightmaps.clone(), Some(mismatched_grid)),
        Err(LightmapContractValidationError::GenerationMismatch)
    );

    let environment = EnvironmentExtract::disabled()
        .try_with_baked_lighting(lightmaps, None)
        .expect("matching baked contract should be accepted");
    assert_eq!(
        environment
            .baked_lighting()
            .expect("lightmap contract should be stored")
            .light_set_generation,
        3
    );
    assert!(environment.light_probe_grid().is_none());
}
