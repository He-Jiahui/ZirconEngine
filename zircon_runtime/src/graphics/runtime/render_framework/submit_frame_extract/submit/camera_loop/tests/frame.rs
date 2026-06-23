use super::*;

#[test]
fn camera_loop_frame_submissions_project_selected_children_and_terminal_ui() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/frame-texture",
    ));
    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    )
    .with_viewport_rect(Some(RenderViewportRect::new(
        UVec2::new(0, 0),
        UVec2::new(32, 64),
    )))
    .with_stack([2]);
    let overlay =
        descriptor(0, 2, CameraRenderType::Overlay, base.target.clone()).with_viewport_rect(Some(
            RenderViewportRect::new(UVec2::new(8, 0), UVec2::new(24, 64)),
        ));
    let primary = descriptor(
        4,
        3,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_viewport_rect(Some(RenderViewportRect::new(
        UVec2::new(32, 0),
        UVec2::new(32, 64),
    )));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(2),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![base, overlay, primary]);
    let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
        .with_ui(Some(UiRenderExtract::default()))
        .with_prepared_runtime_sidebands(RenderPreparedRuntimeSidebands::new(
            RenderPluginRendererOutputs {
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 7,
                    spawned_total: 7,
                    indirect_draw_args: [6, 7, 0, 0],
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
            vec![9],
            vec![13],
        ));
    frame.scene.preview.clear_color = Vec4::new(0.25, 0.5, 0.75, 1.0);

    let submissions = camera_loop_frame_submissions(frame).expect("frame submissions");

    assert_eq!(submissions.len(), 3);
    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.frame.camera().entity)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3)]
    );
    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.receives_terminal_ui)
            .collect::<Vec<_>>(),
        vec![false, false, true]
    );
    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.frame.ui.is_some())
            .collect::<Vec<_>>(),
        vec![false, false, true]
    );
    assert!(
        !ViewportCameraStackOutputPolicy::from(submissions[0].output_policy)
            .owns_final_target_output()
    );
    assert!(
        ViewportCameraStackOutputPolicy::from(submissions[1].output_policy)
            .owns_final_target_output()
    );
    assert!(
        !ViewportCameraStackOutputPolicy::from(submissions[1].output_policy)
            .owns_viewport_submission()
    );
    assert!(
        ViewportCameraStackOutputPolicy::from(submissions[2].output_policy)
            .owns_viewport_submission()
    );
    assert_eq!(
        submissions[0].frame.render_region().physical_size(),
        UVec2::new(32, 64)
    );
    assert_eq!(
        submissions[2].frame.render_region().physical_position(),
        UVec2::new(32, 0)
    );
    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.frame.scene.preview.clear_color)
            .collect::<Vec<_>>(),
        vec![Vec4::new(0.25, 0.5, 0.75, 1.0); 3]
    );
    assert!(submissions[0].frame.prepared_runtime_sidebands.is_empty());
    assert!(submissions[1].frame.prepared_runtime_sidebands.is_empty());
    assert_eq!(
        submissions[2]
            .frame
            .prepared_runtime_sidebands
            .particle_readback_outputs()
            .alive_count,
        7
    );
    assert_eq!(
        submissions[2]
            .frame
            .prepared_runtime_sidebands
            .hybrid_gi_evictable_probe_ids(),
        &[9]
    );
    assert_eq!(
        submissions[2]
            .frame
            .prepared_runtime_sidebands
            .virtual_geometry_evictable_page_ids(),
        &[13]
    );
}

#[test]
fn submit_camera_loop_frame_streams_selected_children_and_restores_source_fields() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/stream-texture",
    ));
    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    )
    .with_stack([2]);
    let overlay = descriptor(0, 2, CameraRenderType::Overlay, base.target.clone());
    let primary = descriptor(
        4,
        3,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(3),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![base, overlay, primary]);
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
        .with_ui(Some(UiRenderExtract::default()));
    let submissions = camera_loop_submissions(&frame.extract).expect("frame submissions");
    let mut seen_cameras = Vec::new();
    let mut terminal_ui = Vec::new();
    let mut output_owners = Vec::new();

    stream_camera_loop_frame_submissions(frame, submissions, |frame, output_policy| {
        seen_cameras.push(frame.camera().entity);
        terminal_ui.push(frame.ui.is_some());
        output_owners
            .push(ViewportCameraStackOutputPolicy::from(output_policy).owns_viewport_submission());
        assert!(
            frame.extract.geometry.virtual_geometry.is_none(),
            "streaming submit should restore source advanced extract state before each child"
        );
        frame.extract_mut().geometry.virtual_geometry =
            Some(RenderVirtualGeometryExtract::default());
        frame.viewport_size = UVec2::new(7, 7);
        frame.extract_mut().view.target_size = Some(UVec2::new(7, 7));
        Ok(())
    })
    .expect("streamed frame submissions");

    assert_eq!(seen_cameras, vec![Some(1), Some(2), Some(3)]);
    assert_eq!(terminal_ui, vec![false, false, true]);
    assert_eq!(output_owners, vec![false, false, true]);
}

fn camera_loop_frame_submissions(
    frame: ViewportRenderFrame,
) -> Result<Vec<CameraLoopFrameSubmission>, RenderFrameworkError> {
    let submissions = camera_loop_submissions(&frame.extract)?;
    let terminal_submission_index = submissions.len().saturating_sub(1);
    let mut source_frame = Some(frame);
    let mut frame_submissions = Vec::with_capacity(submissions.len());

    for (index, submission) in submissions.into_iter().enumerate() {
        let receives_terminal_ui = submission.receives_terminal_ui;
        let mut projected_frame = if index == terminal_submission_index {
            let Some(frame) = source_frame.take() else {
                return Err(camera_loop_source_frame_consumed_error());
            };
            project_owned_frame_to_selected_camera(frame, submission.camera)
        } else {
            let Some(frame) = source_frame.as_ref() else {
                return Err(camera_loop_source_frame_consumed_error());
            };
            project_borrowed_frame_to_selected_camera(frame, submission.camera)
        };
        if !receives_terminal_ui {
            projected_frame = projected_frame.with_ui(None);
        }
        frame_submissions.push(CameraLoopFrameSubmission {
            frame: projected_frame,
            receives_terminal_ui,
            output_policy: submission.output_policy,
        });
    }

    Ok(frame_submissions)
}

fn project_borrowed_frame_to_selected_camera(
    frame: &ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) -> ViewportRenderFrame {
    let extract = frame
        .extract
        .as_ref()
        .clone()
        .with_selected_camera_descriptor(camera);
    let mut projected = ViewportRenderFrame::from_extract(extract, frame.viewport_size)
        .with_shader_quality(frame.shader_quality())
        .with_output_target(frame.output_target())
        .with_ui(frame.ui.clone())
        .with_previous_motion_vector_camera(frame.previous_motion_vector_camera().cloned())
        .with_virtual_geometry_debug_snapshot(frame.virtual_geometry_debug_snapshot.clone())
        .with_camera_stack_output_policy(frame.camera_stack_output_policy());
    if let Some(frame_visibility) = frame.frame_visibility.clone() {
        projected = projected.with_frame_visibility(frame_visibility);
    }
    projected.scene = frame.scene.clone();
    projected
}

fn project_owned_frame_to_selected_camera(
    frame: ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) -> ViewportRenderFrame {
    let ViewportRenderFrame {
        scene,
        extract,
        viewport_size,
        shader_quality,
        ui,
        output_target,
        previous_motion_vector_camera,
        frame_visibility,
        virtual_geometry_debug_snapshot,
        prepared_runtime_sidebands,
        camera_stack_output_policy,
        ..
    } = frame;
    let extract = std::sync::Arc::try_unwrap(extract)
        .unwrap_or_else(|extract| (*extract).clone())
        .with_selected_camera_descriptor(camera);
    let mut projected = ViewportRenderFrame::from_extract(extract, viewport_size)
        .with_shader_quality(shader_quality)
        .with_output_target(output_target)
        .with_ui(ui)
        .with_previous_motion_vector_camera(previous_motion_vector_camera)
        .with_virtual_geometry_debug_snapshot(virtual_geometry_debug_snapshot)
        .with_prepared_runtime_sidebands(prepared_runtime_sidebands)
        .with_camera_stack_output_policy(camera_stack_output_policy);
    if let Some(frame_visibility) = frame_visibility {
        projected = projected.with_frame_visibility(frame_visibility);
    }
    projected.scene = scene;
    projected
}

fn camera_loop_source_frame_consumed_error() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: "camera-loop source frame consumed before terminal camera".to_string(),
    }
}

struct CameraLoopFrameSubmission {
    frame: ViewportRenderFrame,
    receives_terminal_ui: bool,
    output_policy: CameraLoopOutputPolicy,
}
