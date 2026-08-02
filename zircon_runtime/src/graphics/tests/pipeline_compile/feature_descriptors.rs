use super::*;

#[test]
fn feature_pass_descriptors_drive_executor_ids_and_resource_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    let depth_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "depth-prepass")
        .expect("default forward pipeline should include depth prepass");
    assert_eq!(
        depth_pass.executor_id.as_deref(),
        Some("mesh.depth-prepass")
    );
    assert!(
        depth_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "depth prepass should declare the depth target it writes"
    );
    assert!(
        depth_pass
            .resources
            .iter()
            .all(|resource| { resource.name != PostProcessGraphResourceNames::GBUFFER_NORMAL }),
        "pure-depth prepass must not declare the deferred normal target"
    );

    let preview_sky_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "preview-sky")
        .expect("default forward pipeline should include preview sky pass");
    assert_eq!(
        preview_sky_pass.executor_id.as_deref(),
        Some("sky.preview-scene-color")
    );
    assert!(
        preview_sky_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_COLOR
                && resource.access == RenderGraphResourceAccessKind::Write
        }),
        "preview sky should initialize scene color through the render graph"
    );
    assert!(
        preview_sky_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "preview sky should depth-test against opaque geometry through the render graph"
    );

    let velocity_object_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "velocity-object")
        .expect("default forward pipeline should include object velocity pass");
    assert_eq!(
        velocity_object_pass.executor_id.as_deref(),
        Some("temporal.velocity-object")
    );
    assert!(
        velocity_object_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "object velocity pass should read scene depth for depth-tested dynamic object writes"
    );
    assert!(
        velocity_object_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                && resource.kind == RenderGraphResourceKind::TransientTexture
                && resource.access == RenderGraphResourceAccessKind::Write
                && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
        }),
        "object velocity pass should initialize the graph-owned velocity target"
    );

    let velocity_camera_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "velocity-camera")
        .expect("default forward pipeline should include camera velocity pass");
    assert_eq!(
        velocity_camera_pass.executor_id.as_deref(),
        Some("temporal.velocity-camera")
    );
    assert!(
        velocity_camera_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "camera velocity pass should read scene depth for per-pixel reconstruction"
    );
    assert!(
        velocity_camera_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                && resource.kind == RenderGraphResourceKind::TransientTexture
                && resource.access == RenderGraphResourceAccessKind::Write
                && resource.attachment_ops == Some(RenderGraphAttachmentOps::load_store())
        }),
        "camera velocity pass should load the object velocity target before filling static pixels"
    );

    let motion_vector_tile_max_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-vector-tile-max")
        .expect("default forward pipeline should include motion-vector tile reconstruction pass");
    assert_eq!(
        motion_vector_tile_max_pass.executor_id.as_deref(),
        Some("post.motion-vector-tile-max")
    );
    assert!(
        motion_vector_tile_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Read
            }),
        "motion-vector tile reconstruction should read the raw scene motion-vector target"
    );
    assert!(
        motion_vector_tile_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
            }),
        "motion-vector tile reconstruction should write the graph-owned tile-max target"
    );

    let motion_vector_tile_max_coarse_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-vector-tile-max-coarse")
        .expect(
            "default forward pipeline should include coarse motion-vector tile reconstruction pass",
        );
    assert_eq!(
        motion_vector_tile_max_coarse_pass.executor_id.as_deref(),
        Some("post.motion-vector-tile-max-coarse")
    );
    assert!(
        motion_vector_tile_max_coarse_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Read
            }),
        "coarse motion-vector tile reconstruction should read the first tile-max target"
    );
    assert!(
        motion_vector_tile_max_coarse_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
            }),
        "coarse motion-vector tile reconstruction should write the graph-owned coarse tile-max target"
    );

    let motion_vector_neighbor_max_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "motion-vector-neighbor-max")
        .expect("default forward pipeline should include motion-vector reconstruction pass");
    assert_eq!(
        motion_vector_neighbor_max_pass.executor_id.as_deref(),
        Some("post.motion-vector-neighbor-max")
    );
    assert!(
        motion_vector_neighbor_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Read
            }),
        "motion-vector reconstruction should read the coarse tile-max motion-vector target"
    );
    assert!(
        motion_vector_neighbor_max_pass
            .resources
            .iter()
            .any(|resource| {
                resource.name == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
                    && resource.kind == RenderGraphResourceKind::TransientTexture
                    && resource.access == RenderGraphResourceAccessKind::Write
                    && resource.attachment_ops == Some(RenderGraphAttachmentOps::clear_store())
            }),
        "motion-vector reconstruction should write the graph-owned neighbor-max target"
    );

    let opaque_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque-mesh")
        .expect("default forward pipeline should include opaque mesh pass");
    assert_eq!(opaque_pass.executor_id.as_deref(), Some("mesh.opaque"));
    let overlay_pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "overlay-gizmo")
        .expect("default forward pipeline should include overlay pass");
    assert!(
        overlay_pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::SCENE_DEPTH
                && resource.access == RenderGraphResourceAccessKind::Read
        }),
        "overlay executor should declare its depth read instead of borrowing the target privately"
    );

    let lifetimes = compiled.graph().resource_lifetimes();
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-depth" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-color" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::SCENE_VELOCITY
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY
            && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors() {
    let mut extract = extract_with_camera(ViewportCameraSnapshot {
        hdr: true,
        msaa_samples: 4,
        ..ViewportCameraSnapshot::default()
    });
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("test extract should carry a selected camera descriptor")
        .target = RenderCameraTarget::Headless {
        size: UVec2::new(1280, 720),
    };
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();

    let scene_color = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-color")
        .expect("scene-color should be a graph resource");
    assert!(matches!(
        &scene_color.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 1280
                && desc.height == 720
                && desc.format == TextureFormat::Rg11b10Ufloat
                && desc.sample_count == 4
    ));

    let scene_depth = compiled
        .graph()
        .resource_lifetimes()
        .iter()
        .find(|lifetime| lifetime.name == "scene-depth")
        .expect("scene-depth should be a graph resource");
    assert!(matches!(
        &scene_depth.desc,
        RenderGraphResourceDesc::Texture(desc)
            if desc.width == 1280
                && desc.height == 720
                && desc.format == TextureFormat::Depth32Float
                && desc.sample_count == 4
    ));

    for (resource_name, expected_width, expected_height, expected_format) in [
        (
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            1280,
            720,
            TextureFormat::Rg16Float,
        ),
        (
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            1280,
            720,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::HZB_FURTHEST,
            1024,
            512,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            1280,
            720,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
            640,
            360,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
            320,
            180,
            TextureFormat::Rgba16Float,
        ),
        (
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            1280,
            720,
            TextureFormat::Rgba8Unorm,
        ),
    ] {
        let lifetime = graph_resource_lifetime(&compiled, resource_name);
        assert!(matches!(
            &lifetime.desc,
            RenderGraphResourceDesc::Texture(desc)
                if desc.width == expected_width
                    && desc.height == expected_height
                    && desc.format == expected_format
                    && desc.sample_count == 1
        ), "post-process resource {resource_name} descriptor drifted: expected {expected_width}x{expected_height} {expected_format:?} with one sample, got {:?}", lifetime.desc);
    }
}
