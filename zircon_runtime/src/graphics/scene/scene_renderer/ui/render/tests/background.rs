use super::*;

#[test]
fn screen_space_ui_background_lookup_scans_blockers_once() {
    let source = include_str!("../background.rs");
    let indexed_blocker = ["latest_blocker", "_order"].concat();
    let nested_blocker_scan = ["!self.blockers.iter()", ".any"].concat();

    assert!(source.contains(&indexed_blocker));
    assert!(!source.contains(&nested_blocker_scan));
}

#[test]
fn screen_space_ui_plan_keeps_transparent_text_background_unknown() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(11),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#11223380".to_string()),
                        foreground_color: Some("#ddeeff".to_string()),
                        text_render_mode: UiTextRenderMode::Native,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Overlay".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].background_color, None);
}

#[test]
fn screen_space_ui_loaded_framebuffer_background_uses_empty_scene_clear_color() {
    let frame = empty_scene_frame(Vec4::new(0.02, 0.03, 0.04, 1.0));

    assert_eq!(
        framebuffer_background_color(
            &frame,
            RenderGraphAttachmentOps::load_store(),
            wgpu::Color::TRANSPARENT,
        ),
        Some([0.02, 0.03, 0.04, 1.0])
    );
}

#[test]
fn screen_space_ui_loaded_framebuffer_background_rejects_overlay_content() {
    let mut overlays = RenderOverlayExtract::default();
    overlays.grid = Some(GridOverlayExtract {
        visible: true,
        snap_enabled: false,
    });
    let frame = empty_scene_frame(Vec4::new(0.02, 0.03, 0.04, 1.0)).with_runtime_overlays(overlays);

    assert_eq!(
        framebuffer_background_color(
            &frame,
            RenderGraphAttachmentOps::load_store(),
            wgpu::Color::TRANSPARENT,
        ),
        None
    );
}

#[test]
fn screen_space_ui_loaded_framebuffer_background_rejects_gpu_particle_content() {
    let mut extract = empty_frame_extract(Vec4::new(0.02, 0.03, 0.04, 1.0));
    extract.particles.gpu_frame = Some(RenderParticleGpuFrameExtract {
        alive_count: 1,
        ..RenderParticleGpuFrameExtract::default()
    });
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(200, 100));

    assert_eq!(
        framebuffer_background_color(
            &frame,
            RenderGraphAttachmentOps::load_store(),
            wgpu::Color::TRANSPARENT,
        ),
        None
    );
}

fn empty_scene_frame(clear_color: Vec4) -> ViewportRenderFrame {
    ViewportRenderFrame::from_snapshot(empty_scene_snapshot(clear_color), UVec2::new(200, 100))
}

fn empty_frame_extract(clear_color: Vec4) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(0),
        empty_scene_snapshot(clear_color),
    )
}

fn empty_scene_snapshot(clear_color: Vec4) -> RenderSceneSnapshot {
    let environment = EnvironmentExtract::default();
    let preview = PreviewEnvironmentExtract::from_environment(&environment, false, clear_color);
    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment,
        preview,
        virtual_geometry_debug: None,
    }
}
