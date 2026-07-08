use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneActionData,
    TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;

const DIALOG_COMPONENT_SCREENSHOT: &str = "editor-components-dialogs-900x360.png";
const DIALOG_ATLAS_WIDTH: u32 = 900;
const DIALOG_ATLAS_HEIGHT: u32 = 360;
const DIALOG_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];
const DIALOG_ERROR: [u8; 4] = EditorPaletteTokens::WORKBENCH_ERROR;
const DIALOG_WARNING: [u8; 4] = EditorPaletteTokens::WORKBENCH_WARNING;
const DIALOG_DISABLED_TEXT: [u8; 4] = EditorPaletteTokens::WORKBENCH_TEXT_DISABLED;

#[test]
fn dialog_component_visual_paints_surface_content_actions_severity_and_disabled() {
    let bytes = dialog_component_bytes();

    let standard_panel = pixel_at(&bytes, 30, 92);
    let standard_surface = pixel_at(&bytes, 54, 132);
    assert_ne!(
        standard_surface, DIALOG_ATLAS_BACKGROUND,
        "ordinary dialog should paint a retained popup surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            58,
            136,
            166,
            22,
            &[DIALOG_ATLAS_BACKGROUND, standard_panel, standard_surface],
        ) > 0,
        "ordinary dialog should paint title text through the shared text path"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            58,
            164,
            176,
            22,
            &[DIALOG_ATLAS_BACKGROUND, standard_panel, standard_surface],
        ) > 0,
        "ordinary dialog should paint body copy through the shared text path"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            188,
            224,
            48,
            22,
            &[DIALOG_ATLAS_BACKGROUND, standard_panel, standard_surface],
        ) > 0,
        "ordinary dialog should paint the trailing action label"
    );

    let confirm_panel = pixel_at(&bytes, 310, 92);
    let error_surface = pixel_at(&bytes, 338, 132);
    assert!(
        exact_pixel_count(&bytes, 318, 116, 8, 126, DIALOG_ERROR) > 0,
        "destructive confirm dialog should paint an error severity strip"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            340,
            136,
            170,
            22,
            &[DIALOG_ATLAS_BACKGROUND, confirm_panel, error_surface],
        ) > 0,
        "destructive confirm dialog should paint title text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            462,
            224,
            58,
            22,
            &[DIALOG_ATLAS_BACKGROUND, confirm_panel, error_surface],
        ) > 0,
        "destructive confirm dialog should paint its confirm action"
    );

    let state_panel = pixel_at(&bytes, 610, 92);
    let warning_surface = pixel_at(&bytes, 638, 126);
    assert!(
        exact_pixel_count(&bytes, 618, 116, 8, 88, DIALOG_WARNING) > 0,
        "warning confirm dialog should paint a warning severity strip"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            640,
            136,
            156,
            18,
            &[DIALOG_ATLAS_BACKGROUND, state_panel, warning_surface],
        ) > 0,
        "warning confirm dialog should paint compact title text"
    );

    let disabled_surface = pixel_at(&bytes, 638, 222);
    assert_ne!(
        disabled_surface, warning_surface,
        "disabled dialog should use unavailable surface instead of warning surface"
    );
    assert!(
        exact_pixel_count(&bytes, 640, 232, 168, 22, DIALOG_DISABLED_TEXT) > 0,
        "disabled dialog should paint muted title or body text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            766,
            260,
            66,
            24,
            &[DIALOG_ATLAS_BACKGROUND, state_panel, disabled_surface],
        ) > 0,
        "disabled dialog should paint disabled action text"
    );
}

#[test]
#[ignore = "writes local dialog component screenshot artifact for visual review"]
fn capture_dialog_component_visual_artifact() {
    let bytes = dialog_component_bytes();
    let output_path = visual_layout_output_path(DIALOG_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        DIALOG_ATLAS_WIDTH,
        DIALOG_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("dialog component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn dialog_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        DIALOG_ATLAS_WIDTH,
        DIALOG_ATLAS_HEIGHT,
        DIALOG_ATLAS_BACKGROUND,
        model_rc(dialog_component_nodes()),
    )
}

fn dialog_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("DialogRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label("DialogTitle", "Dialogs", 22.0, 20.0, 220.0, 22.0, 13.0, ""),
        label(
            "DialogSubtitle",
            "Popup surface, title, body, severity strip, action labels and disabled state",
            22.0,
            42.0,
            720.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("DialogStandardPanel", "panel", 18.0, 78.0, 260.0, 220.0),
        surface("DialogConfirmPanel", "panel", 300.0, 78.0, 278.0, 220.0),
        surface("DialogStatePanel", "inset", 600.0, 78.0, 282.0, 220.0),
        label(
            "DialogStandardTitle",
            "Standard",
            36.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "DialogConfirmTitle",
            "Confirm",
            318.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        label(
            "DialogStateTitle",
            "States",
            618.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        dialog(DialogFixture {
            control_id: "WorkbenchDialog",
            title: "Scene Settings",
            body: "Review project defaults.",
            role: "Dialog",
            component_role: "dialog",
            variant: "",
            validation_level: "",
            actions: &["Apply"],
            x: 36.0,
            y: 116.0,
            width: 224.0,
            height: 128.0,
            disabled: false,
            pressed: false,
        }),
        dialog(DialogFixture {
            control_id: "WorkbenchConfirmError",
            title: "Delete prefab?",
            body: "This removes the scene link.",
            role: "ConfirmDialog",
            component_role: "confirm-dialog",
            variant: "error destructive",
            validation_level: "error",
            actions: &["Cancel", "Delete"],
            x: 318.0,
            y: 116.0,
            width: 240.0,
            height: 128.0,
            disabled: false,
            pressed: false,
        }),
        dialog(DialogFixture {
            control_id: "WorkbenchConfirmWarning",
            title: "Rebuild lighting?",
            body: "",
            role: "ConfirmDialog",
            component_role: "confirm-dialog",
            variant: "warning",
            validation_level: "warning",
            actions: &["Cancel", "Rebuild"],
            x: 618.0,
            y: 116.0,
            width: 230.0,
            height: 88.0,
            disabled: false,
            pressed: true,
        }),
        dialog(DialogFixture {
            control_id: "WorkbenchDialogDisabled",
            title: "Offline Sync",
            body: "",
            role: "Dialog",
            component_role: "dialog",
            variant: "disabled",
            validation_level: "",
            actions: &["Retry"],
            x: 618.0,
            y: 212.0,
            width: 230.0,
            height: 88.0,
            disabled: true,
            pressed: false,
        }),
    ]
}

struct DialogFixture<'a> {
    control_id: &'a str,
    title: &'a str,
    body: &'a str,
    role: &'a str,
    component_role: &'a str,
    variant: &'a str,
    validation_level: &'a str,
    actions: &'a [&'a str],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    disabled: bool,
    pressed: bool,
}

fn dialog(fixture: DialogFixture<'_>) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: fixture.control_id.into(),
        role: fixture.role.into(),
        component_role: fixture.component_role.into(),
        component_variant: fixture.variant.into(),
        validation_level: fixture.validation_level.into(),
        text: fixture.title.into(),
        value_text: fixture.body.into(),
        popup_open: true,
        disabled: fixture.disabled,
        pressed: fixture.pressed,
        actions: model_rc(
            fixture
                .actions
                .iter()
                .map(|label| action(label, &format!("{}.{}", fixture.control_id, label)))
                .collect(),
        ),
        frame: frame(fixture.x, fixture.y, fixture.width, fixture.height),
        ..TemplatePaneNodeData::default()
    }
}

fn action(label: &str, action_id: &str) -> TemplatePaneActionData {
    TemplatePaneActionData {
        label: label.into(),
        action_id: action_id.into(),
    }
}

fn surface(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: variant.into(),
        border_width: 1.0,
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn label(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font_size: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * DIALOG_ATLAS_WIDTH as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn distinct_pixel_count(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    excluded_colors: &[[u8; 4]],
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * DIALOG_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
            if !excluded_colors.contains(&color) {
                changed += 1;
            }
        }
    }
    changed
}

fn exact_pixel_count(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    expected: [u8; 4],
) -> usize {
    let mut matches = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * DIALOG_ATLAS_WIDTH as usize) + px as usize) * 4;
            let color = [
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ];
            if color == expected {
                matches += 1;
            }
        }
    }
    matches
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
