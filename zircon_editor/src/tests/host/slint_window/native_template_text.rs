use std::rc::Rc;

use crate::ui::slint_host::{
    FrameRect, HostDocumentDockSurfaceData, HostWindowLayoutData, PaneData,
    SceneViewportChromeData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostWindow,
};
use slint::{ModelRc, PhysicalSize, VecModel};

#[test]
fn rust_owned_template_text_keeps_short_labels_legible() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(180, 96));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(180.0, 96.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(180.0, 96.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 28.0, 180.0, 44.0),
        header_frame: host_frame(0.0, 0.0, 180.0, 0.0),
        content_frame: host_frame(0.0, 0.0, 180.0, 44.0),
        pane: pane_with_nodes(vec![short_label_node(
            "ShortAction",
            "Apply",
            8.0,
            6.0,
            64.0,
            14.0,
        )]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("short native template text should render");

    let lit_rows = lit_row_count(snapshot.width(), snapshot.as_bytes(), 8, 34, 64, 14);
    assert!(
        lit_rows >= 8,
        "short template labels should keep enough vertical glyph rows to remain legible, got {lit_rows}"
    );
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
        center_band_frame: host_frame(0.0, 28.0, width, height - 52.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        document_region_frame: host_frame(0.0, 28.0, width, height - 52.0),
        viewport_content_frame: host_frame(0.0, 28.0, width, height - 52.0),
        ..HostWindowLayoutData::default()
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
    pane.project_overview.nodes = model_rc(nodes);
    pane
}

fn short_label_node(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: "Button".into(),
        text: text.into(),
        button_variant: "primary".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn lit_row_count(
    width: u32,
    bytes: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((bytes.len() / 4 / width as usize) as u32);
    (y..y1)
        .filter(|row| {
            (x..x1).any(|column| is_template_text_pixel(pixel(width, bytes, column, *row)))
        })
        .count()
}

fn is_template_text_pixel(pixel: [u8; 4]) -> bool {
    pixel[0] >= 90 && pixel[1] >= 120 && pixel[2] >= 170
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

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}
