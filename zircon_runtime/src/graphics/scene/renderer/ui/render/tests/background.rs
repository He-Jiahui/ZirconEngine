use super::*;

#[test]
fn screen_space_ui_background_lookup_uses_one_reverse_effect_scan() {
    let source = include_str!("../background.rs");

    assert!(source.contains("for effect in self.effects.iter().rev()"));
    assert!(!source.contains("candidates: Vec<"));
    assert!(!source.contains("blockers: Vec<"));
}

#[test]
fn screen_space_ui_background_lookup_stops_at_newest_relevant_effect() {
    let viewport = UiFrame::new(0.0, 0.0, 200.0, 100.0);
    let query = UiFrame::new(10.0, 10.0, 40.0, 20.0);
    let mut tracker = ScreenSpaceUiBackgroundTracker::with_framebuffer_background(
        viewport,
        Some([0.1, 0.2, 0.3, 1.0]),
    );
    tracker.observe_command(
        &background_test_command(UiRenderCommandKind::Quad, query, Some("#224466"), None),
        viewport,
    );
    tracker.observe_command(
        &background_test_command(UiRenderCommandKind::Text, query, None, Some("blocked")),
        viewport,
    );

    let (blocked, blocker_visits) = tracker.color_for_frame_with_visit_count(query, None, viewport);

    assert_eq!(blocked, None);
    assert_eq!(blocker_visits, 1);

    tracker.observe_command(
        &background_test_command(UiRenderCommandKind::Quad, query, Some("#6688aa"), None),
        viewport,
    );
    let (covered, candidate_visits) =
        tracker.color_for_frame_with_visit_count(query, None, viewport);

    assert_eq!(
        covered,
        Some([
            0x66 as f32 / 255.0,
            0x88 as f32 / 255.0,
            0xaa as f32 / 255.0,
            1.0
        ])
    );
    assert_eq!(candidate_visits, 1);
}

fn background_test_command(
    kind: UiRenderCommandKind,
    frame: UiFrame,
    background_color: Option<&str>,
    text: Option<&str>,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind,
        frame,
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            background_color: background_color.map(str::to_string),
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: text.map(str::to_string),
        image: None,
        opacity: 1.0,
    }
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
