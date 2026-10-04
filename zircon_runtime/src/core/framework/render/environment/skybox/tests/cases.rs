use super::*;
use crate::core::framework::render::{
    build_source_cubemap_from_equirect, source_cubemap_environment_with_bake_artifact,
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactPayload,
    SourceCubemapIrradianceCube,
};
use crate::core::math::Vec4;

#[test]
fn procedural_default_matches_existing_preview_gradient() {
    let skybox = SkyboxSettings::procedural_default();

    assert_eq!(skybox.mode, SkyboxMode::ProceduralGradient);
    assert_eq!(
        skybox.procedural.horizon_color,
        Vec4::new(0.16, 0.19, 0.24, 1.0)
    );
    assert_eq!(
        skybox.procedural.zenith_color,
        Vec4::new(0.36, 0.46, 0.63, 1.0)
    );
    assert_eq!(
        skybox.procedural.ground_color,
        Vec4::new(0.09, 0.11, 0.14, 1.0)
    );
}

#[test]
fn disabled_skybox_has_no_ibl_bake_key() {
    assert!(SkyboxSettings::none().ibl_bake_key().is_none());
}

#[test]
fn ibl_bake_key_ignores_intensity_and_rotation() {
    let mut first = ProceduralSkyParams::default_gradient();
    let mut second = first;
    second.intensity = 3.5;
    second.rotation_radians = 1.25;

    assert_eq!(first.ibl_bake_key(), second.ibl_bake_key());

    first.horizon_color.x += 0.01;
    assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
}

#[test]
fn ibl_bake_key_tracks_effective_directional_sun_parameters() {
    let mut base = ProceduralSkyParams::default_gradient();
    base.sun_direction = Vec4::new(0.0, 2.0, 1.0, 0.0);
    base.sun_color = Vec4::new(1.0, 0.8, 0.6, 1.0);
    base.sun_intensity = 4.0;
    base.sun_angular_radius_radians = 0.04;
    let base_key = base.ibl_bake_key();

    let variants = [
        ProceduralSkyParams {
            sun_direction: Vec4::new(1.0, 2.0, 1.0, 0.0),
            ..base
        },
        ProceduralSkyParams {
            sun_color: Vec4::new(0.9, 0.8, 0.6, 1.0),
            ..base
        },
        ProceduralSkyParams {
            sun_intensity: 5.0,
            ..base
        },
        ProceduralSkyParams {
            sun_angular_radius_radians: 0.06,
            ..base
        },
    ];

    assert_ne!(base_key.source_hash, [0; 4]);
    for variant in variants {
        assert_ne!(base_key, variant.ibl_bake_key());
    }
}

#[test]
fn ibl_bake_key_uses_normalized_sun_direction_and_ignores_disabled_sun() {
    let mut enabled = ProceduralSkyParams::default_gradient();
    enabled.sun_direction = Vec4::new(0.0, 2.0, 1.0, 0.0);
    enabled.sun_intensity = 4.0;
    let mut scaled = enabled;
    scaled.sun_direction *= 3.0;

    assert_eq!(enabled.ibl_bake_key(), scaled.ibl_bake_key());

    let disabled = ProceduralSkyParams::default_gradient();
    let changed_but_disabled = ProceduralSkyParams {
        sun_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
        sun_color: Vec4::new(0.25, 0.5, 0.75, 1.0),
        sun_angular_radius_radians: 0.2,
        ..disabled
    };
    assert_eq!(disabled.ibl_bake_key(), changed_but_disabled.ibl_bake_key());
    assert_eq!(disabled.ibl_bake_key().source_hash, [0; 4]);

    let invalid_direction = ProceduralSkyParams {
        sun_direction: Vec4::ZERO,
        sun_intensity: 4.0,
        ..disabled
    };
    assert_eq!(invalid_direction.ibl_bake_key().source_hash, [0; 4]);
}

#[test]
fn resolved_sun_keeps_a_strict_cosine_interval_after_radius_clamping() {
    let mut sky = ProceduralSkyParams::default_gradient();
    sky.sun_intensity = 1.0;
    sky.sun_angular_radius_radians = 0.0;

    let sun = sky.resolved_sun();

    assert!(sun.intensity_and_cosines.y < sun.intensity_and_cosines.z);
}

#[test]
fn ibl_bake_key_tracks_source_revision() {
    let first = ProceduralSkyParams::default_gradient();
    let mut second = first;
    second.source_revision += 1;

    assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
}

#[test]
fn source_cubemap_bake_key_tracks_source_hash() {
    let first = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        2,
        [1, 2, 3, 4],
    );
    let second = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        2,
        [1, 2, 3, 5],
    );

    assert_ne!(first.ibl_bake_key(), second.ibl_bake_key());
    let skybox = SkyboxSettings::source_cubemap(first.clone());
    assert_eq!(skybox.ibl_bake_key(), Some(first.ibl_bake_key()));
    assert_eq!(skybox.source_cubemap_environment(), Some(&first));
}

#[test]
fn source_cubemap_environment_can_carry_optional_iem_without_changing_bake_key() {
    let mip_chain = build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]);
    let bake_key = SourceCubemapEnvironment::new(mip_chain.clone(), 3, [1, 2, 3, 4]).ibl_bake_key();
    let environment =
        SourceCubemapEnvironment::new(mip_chain, 3, [1, 2, 3, 4]).with_irradiance_cube(
            SourceCubemapIrradianceCube::new(1, vec![[0.25, 0.5, 0.75]; 6]),
        );

    assert_eq!(environment.ibl_bake_key(), bake_key);
    assert_eq!(
        environment
            .irradiance_cube()
            .map(SourceCubemapIrradianceCube::face_size),
        Some(1)
    );
}

#[test]
fn source_cubemap_upload_key_tracks_optional_iem_content() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    );
    let without_iem = environment.texture_upload_key();
    let first_iem = environment
        .clone()
        .with_irradiance_cube(SourceCubemapIrradianceCube::new(
            1,
            vec![[0.25, 0.5, 0.75]; 6],
        ))
        .texture_upload_key();
    let changed_iem = environment
        .with_irradiance_cube(SourceCubemapIrradianceCube::new(
            1,
            vec![[0.5, 0.25, 0.75]; 6],
        ))
        .texture_upload_key();

    assert_ne!(without_iem, first_iem);
    assert_ne!(first_iem, changed_iem);
}

#[test]
fn source_cubemap_prepared_upload_artifact_requires_current_upload_key() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    )
    .with_prepared_upload_artifact();
    assert!(environment.prepared_upload_artifact().is_some());

    let changed_irradiance = environment.with_irradiance_cube(SourceCubemapIrradianceCube::new(
        1,
        vec![[0.25, 0.5, 0.75]; 6],
    ));
    assert!(changed_irradiance.prepared_upload_artifact().is_none());
    assert!(
        changed_irradiance.upload_artifact.is_none(),
        "changing irradiance must release the obsolete upload bytes before rebuilding"
    );
    assert!(
        changed_irradiance
            .with_prepared_upload_artifact()
            .prepared_upload_artifact()
            .is_some(),
        "preparing after an upload-key change must replace the stale artifact"
    );
}

#[test]
fn source_cubemap_skybox_prepares_upload_artifact_before_render_extract() {
    let skybox = SkyboxSettings::source_cubemap(SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    ));

    assert!(
        skybox
            .source_cubemap_environment()
            .and_then(SourceCubemapEnvironment::prepared_upload_artifact)
            .is_some(),
        "skybox construction must not defer cubemap byte encoding to render submission"
    );
}

#[test]
fn source_cubemap_reuses_prepared_upload_artifact_for_unchanged_irradiance() {
    let irradiance = SourceCubemapIrradianceCube::new(1, vec![[0.25, 0.5, 0.75]; 6]);
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    )
    .with_irradiance_cube(irradiance.clone())
    .with_prepared_upload_artifact();

    let unchanged_irradiance = environment.with_irradiance_cube(irradiance);

    assert!(
        unchanged_irradiance.upload_artifact.is_some(),
        "unchanged irradiance content must retain its prepared upload artifact"
    );
}

#[test]
fn source_cubemap_environment_equality_ignores_prepared_upload_cache() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    );
    let prepared = environment.clone().with_prepared_upload_artifact();

    assert_eq!(environment, prepared);
}

#[test]
fn source_cubemap_bake_replacement_discards_prepared_upload_cache() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    )
    .with_prepared_upload_artifact();
    let replacement = build_source_cubemap_from_equirect(1, |_, _| [0.75, 0.5, 0.25, 1.0]);
    let request = environment.ibl_bake_artifact_request(IblBakeArtifactContents::PMREM_SH9);
    let payload = IblBakeArtifactPayload::from_source_cubemap(
        IblBakeArtifactDescriptor::current_for_request(&request),
        &replacement,
        None,
    )
    .expect("replacement cubemap should encode as a current CPU artifact");

    let environment = source_cubemap_environment_with_bake_artifact(&environment, &payload)
        .expect("current CPU artifact should hydrate the source environment");

    assert!(environment.prepared_upload_artifact().is_some());
    assert_eq!(
        environment.accepted_bake_artifact_descriptor(),
        Some(payload.descriptor())
    );
}

#[test]
fn source_cubemap_artifact_provenance_does_not_change_gpu_upload_identity() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    );
    let upload_key = environment.texture_upload_key();
    let with_provenance = environment.with_bake_artifact_hash([9, 8, 7, 6]);

    assert_eq!(with_provenance.bake_artifact_hash, [9, 8, 7, 6]);
    assert_eq!(with_provenance.texture_upload_key(), upload_key);
    assert!(with_provenance
        .accepted_bake_artifact_descriptor()
        .is_none());
}

#[test]
fn source_cubemap_manual_irradiance_change_clears_accepted_artifact_descriptor() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(1, |_, _| [0.25, 0.5, 0.75, 1.0]),
        3,
        [1, 2, 3, 4],
    );
    let descriptor = IblBakeArtifactDescriptor::current_for_request(
        &environment.ibl_bake_artifact_request(IblBakeArtifactContents::PMREM_SH9),
    );
    let changed = environment
        .with_accepted_bake_artifact_descriptor(descriptor)
        .with_irradiance_cube(SourceCubemapIrradianceCube::new(
            1,
            vec![[0.25, 0.5, 0.75]; 6],
        ));

    assert!(changed.accepted_bake_artifact_descriptor().is_none());
}

#[test]
fn source_cubemap_builds_ibl_bake_request_from_source_mip_chain_shape() {
    let environment = SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(4, |_, _| [0.25, 0.5, 0.75, 1.0]),
        7,
        [9, 8, 7, 6],
    );

    let request = environment.ibl_bake_artifact_request(IblBakeArtifactContents::SH9);

    assert_eq!(request.bake_key(), environment.ibl_bake_key());
    assert_eq!(request.source_face_size(), 4);
    assert_eq!(request.source_mip_count(), 3);
    assert_eq!(request.required_contents(), IblBakeArtifactContents::SH9);
}
