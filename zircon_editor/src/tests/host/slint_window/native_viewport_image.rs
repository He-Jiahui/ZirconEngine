use crate::ui::slint_host::{
    FrameRect, HostDocumentDockSurfaceData, HostWindowLayoutData, PaneData, PaneSurfaceHostContext,
    SceneViewportChromeData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostWindow,
};
use slint::{Image, ModelRc, PhysicalSize, Rgba8Pixel, SharedPixelBuffer, VecModel};

#[test]
fn native_host_painter_composites_latest_viewport_image_into_scene_body() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(240, 180));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(240.0, 180.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(240.0, 180.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 200.0, 116.0),
        header_frame: host_frame(0.0, 0.0, 200.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 200.0, 90.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let before = ui
        .window()
        .take_snapshot()
        .expect("pre-image scene snapshot should render");
    ui.global::<PaneSurfaceHostContext>()
        .set_viewport_image(solid_viewport_image([201, 42, 33, 255]));
    let after = ui
        .window()
        .take_snapshot()
        .expect("post-image scene snapshot should render");

    assert_eq!(
        pixel(after.width(), after.as_bytes(), 120, 112),
        [201, 42, 33, 255],
        "native Scene pane body should draw the latest renderer viewport image"
    );
    assert_ne!(
        pixel(before.width(), before.as_bytes(), 120, 112),
        [201, 42, 33, 255],
        "baseline Scene pane body should not already contain the renderer image color"
    );
}

#[test]
fn native_host_painter_draws_template_svg_image_pixels() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(180, 120));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(180.0, 120.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(180.0, 120.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(20.0, 40.0, 140.0, 56.0),
        header_frame: host_frame(0.0, 0.0, 140.0, 0.0),
        content_frame: host_frame(0.0, 0.0, 140.0, 56.0),
        pane: pane_with_nodes(vec![template_image_node(
            "SvgImageDemo",
            solid_viewport_image([201, 42, 33, 255]),
            18.0,
            10.0,
            32.0,
            32.0,
        )]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("template image snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 55, 66),
        [201, 42, 33, 255],
        "template Image/SvgIcon nodes should draw loaded preview pixels instead of placeholders"
    );
}

fn solid_viewport_image(color: [u8; 4]) -> Image {
    let pixels = [color, color, color, color].concat();
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixels, 2, 2,
    ))
}

fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 40.0, width, height - 64.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        document_region_frame: host_frame(20.0, 40.0, width - 40.0, height - 64.0),
        viewport_content_frame: host_frame(20.0, 93.0, width - 40.0, height - 117.0),
        ..HostWindowLayoutData::default()
    }
}

fn scene_pane() -> PaneData {
    PaneData {
        kind: "Scene".into(),
        title: "Scene".into(),
        show_toolbar: true,
        viewport: SceneViewportChromeData {
            tool: "Move".into(),
            transform_space: "Global".into(),
            display_mode: "Lit".into(),
            grid_mode: "Grid".into(),
            ..SceneViewportChromeData::default()
        },
        ..PaneData::default()
    }
}

fn pane_with_nodes(nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "Project".into(),
        title: "Project".into(),
        show_toolbar: false,
        viewport: SceneViewportChromeData::default(),
        ..PaneData::default()
    };
    pane.project_overview.nodes = ModelRc::from(std::rc::Rc::new(VecModel::from(nodes)));
    pane
}

fn template_image_node(
    control_id: &str,
    preview_image: Image,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: "Image".into(),
        media_source: "ui/editor/showcase_checker.svg".into(),
        has_preview_image: true,
        preview_image,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
