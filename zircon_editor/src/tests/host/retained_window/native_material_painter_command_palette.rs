use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, TemplateNodeFrameData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};
use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const POPUP_SURFACE: [u8; 4] = EditorPaletteTokens::WORKBENCH_POPUP;
const POPUP_BORDER: [u8; 4] = EditorPaletteTokens::WORKBENCH_BORDER;
const SEARCH_SURFACE: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE_RECESSED;
const FOCUS_RING: [u8; 4] = EditorPaletteTokens::WORKBENCH_FOCUS_RING;
const SELECTED_ROW: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE[3];

#[test]
fn native_template_painter_draws_command_palette_panel_search_and_rows() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "CommandPaletteDemo".into(),
        node_id: "CommandPaletteDemo.node".into(),
        role: "CommandPalette".into(),
        component_role: "command-palette".into(),
        popup_open: true,
        search_query: "build".into(),
        structured_options: model_rc(vec![
            option("build_project", "Build Project", true, false, true),
            option("build_assets", "Build Assets", false, true, false),
        ]),
        frame: frame(8.0, 8.0, 180.0, 112.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(208, 136, nodes);

    assert_eq!(pixel(&bytes, 208, 18, 14), POPUP_SURFACE);
    assert_eq!(pixel(&bytes, 208, 96, 8), POPUP_BORDER);
    assert_eq!(pixel(&bytes, 208, 24, 24), SEARCH_SURFACE);
    assert_eq!(pixel(&bytes, 208, 96, 18), FOCUS_RING);
    assert_eq!(pixel(&bytes, 208, 160, 66), SELECTED_ROW);
    assert_eq!(pixel(&bytes, 208, 16, 66), POPUP_BORDER);
    assert_eq!(pixel(&bytes, 208, 160, 92), POPUP_SURFACE);
    assert_eq!(pixel(&bytes, 208, 204, 12), BACKGROUND);
}

#[test]
fn native_template_painter_consumes_closed_command_palette_without_surface_fallback() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ClosedCommandPalette".into(),
        node_id: "ClosedCommandPalette.node".into(),
        role: "CommandPalette".into(),
        component_role: "command-palette".into(),
        popup_open: false,
        search_query: "build".into(),
        frame: frame(8.0, 8.0, 180.0, 112.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(208, 136, nodes);

    assert_eq!(changed_pixel_count(&bytes, BACKGROUND), 0);
}

fn model_rc<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(items)))
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn option(
    id: &str,
    label: &str,
    selected: bool,
    disabled: bool,
    focused: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: label.into(),
        selected,
        disabled,
        focused,
        matched: true,
        ..TemplatePaneOptionData::default()
    }
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * width + x) * 4) as usize;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn changed_pixel_count(bytes: &[u8], background: [u8; 4]) -> usize {
    bytes
        .chunks_exact(4)
        .filter(|pixel| {
            pixel[0] != background[0]
                || pixel[1] != background[1]
                || pixel[2] != background[2]
                || pixel[3] != background[3]
        })
        .count()
}
