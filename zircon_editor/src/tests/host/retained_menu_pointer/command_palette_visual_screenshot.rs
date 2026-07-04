use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};
use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;

const COMMAND_PALETTE_COMPONENT_SCREENSHOT: &str = "editor-components-command-palette-900x360.png";

#[test]
#[ignore = "writes local command palette component screenshot artifact for visual review"]
fn capture_command_palette_component_visual_artifact() {
    let width = 900;
    let height = 360;
    let bytes = paint_template_nodes_for_test_with_background(
        width,
        height,
        EditorPaletteTokens::WORKBENCH_SURFACE[0],
        model_rc(command_palette_component_nodes()),
    );
    let output_path = visual_layout_output_path(COMMAND_PALETTE_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("command palette component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn command_palette_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![TemplatePaneNodeData {
        control_id: "WorkbenchCommandPaletteVisual".into(),
        node_id: "WorkbenchCommandPaletteVisual.node".into(),
        role: "CommandPalette".into(),
        component_role: "command-palette".into(),
        popup_open: true,
        search_query: "build".into(),
        structured_options: model_rc(vec![
            option("build.project", "Build Project", true, false, true, false),
            option(
                "build.assets",
                "Build Asset Cache",
                false,
                false,
                false,
                false,
            ),
            option("project.open", "Open Project", false, true, false, true),
        ]),
        frame: frame(170.0, 54.0, 560.0, 220.0),
        ..TemplatePaneNodeData::default()
    }]
}

fn option(
    id: &str,
    label: &str,
    selected: bool,
    disabled: bool,
    focused: bool,
    special: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: label.into(),
        selected,
        disabled,
        focused,
        special,
        matched: true,
        ..TemplatePaneOptionData::default()
    }
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

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
}

fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    output_dir.join(filename)
}
