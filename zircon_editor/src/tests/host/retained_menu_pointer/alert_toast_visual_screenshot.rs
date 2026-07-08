use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::Color;
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const ALERT_TOAST_COMPONENT_SCREENSHOT: &str = "editor-components-alert-toasts-900x360.png";
const ALERT_TOAST_ATLAS_WIDTH: u32 = 900;
const ALERT_TOAST_ATLAS_HEIGHT: u32 = 360;
const ALERT_TOAST_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];
const DECLARED_TOAST_MARK: [u8; 4] = [32, 159, 169, 255];
const DECLARED_TOAST_ACTION: [u8; 4] = [35, 143, 152, 255];

#[test]
fn alert_toast_component_visual_paints_tones_actions_focus_and_disabled() {
    let bytes = alert_toast_component_bytes();

    let alerts_panel = pixel_at(&bytes, 30, 92);
    let info_surface = pixel_at(&bytes, 48, 124);
    let success_surface = pixel_at(&bytes, 48, 166);
    let warning_surface = pixel_at(&bytes, 48, 208);
    let error_surface = pixel_at(&bytes, 48, 250);

    assert_ne!(
        info_surface, ALERT_TOAST_ATLAS_BACKGROUND,
        "info alert should paint a retained tone surface"
    );
    assert_ne!(
        success_surface, info_surface,
        "success alert should not reuse the info tone surface"
    );
    assert_ne!(
        warning_surface, info_surface,
        "warning alert should not reuse the info tone surface"
    );
    assert_ne!(
        error_surface, info_surface,
        "error alert should not reuse the info tone surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            78,
            116,
            166,
            22,
            &[ALERT_TOAST_ATLAS_BACKGROUND, alerts_panel, info_surface],
        ) > 0,
        "info alert should paint runtime text through the shared text path"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            48,
            158,
            28,
            28,
            &[ALERT_TOAST_ATLAS_BACKGROUND, alerts_panel, success_surface],
        ) > 0,
        "success alert should paint the tone mark glyph"
    );

    let toast_panel = pixel_at(&bytes, 310, 92);
    let toast_surface = pixel_at(&bytes, 430, 144);
    assert!(
        distinct_pixel_count(
            &bytes,
            350,
            136,
            126,
            22,
            &[ALERT_TOAST_ATLAS_BACKGROUND, toast_panel, toast_surface],
        ) > 0,
        "ordinary toast should paint completion text"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            508,
            136,
            38,
            22,
            &[ALERT_TOAST_ATLAS_BACKGROUND, toast_panel, toast_surface],
        ) > 0,
        "wide toast should paint its action label"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            548,
            136,
            18,
            22,
            &[ALERT_TOAST_ATLAS_BACKGROUND, toast_panel, toast_surface],
        ) > 0,
        "wide toast should paint its close mark"
    );
    assert!(
        exact_pixel_count(&bytes, 326, 198, 20, 20, DECLARED_TOAST_MARK) > 0,
        "declared toast should paint its declared status mark color"
    );
    assert!(
        exact_pixel_count(&bytes, 508, 196, 44, 22, DECLARED_TOAST_ACTION) > 0,
        "declared toast should paint its declared action color"
    );

    let state_panel = pixel_at(&bytes, 598, 92);
    let pressed_border = pixel_at(&bytes, 620, 116);
    let focused_border = pixel_at(&bytes, 620, 158);
    assert_ne!(
        pressed_border, focused_border,
        "pressed warning alert should use active border while focused warning keeps tone border"
    );

    let hovered_toast_surface = pixel_at(&bytes, 730, 218);
    assert_ne!(
        hovered_toast_surface, toast_surface,
        "hovered toast should paint hover surface instead of ordinary surface"
    );
    let disabled_toast_surface = pixel_at(&bytes, 730, 260);
    assert_ne!(
        disabled_toast_surface, hovered_toast_surface,
        "disabled toast should paint unavailable surface instead of hover surface"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            638,
            252,
            126,
            22,
            &[
                ALERT_TOAST_ATLAS_BACKGROUND,
                state_panel,
                disabled_toast_surface
            ],
        ) > 0,
        "disabled toast should still paint muted text and status mark"
    );
}

#[test]
#[ignore = "writes local alert/toast component screenshot artifact for visual review"]
fn capture_alert_toast_component_visual_artifact() {
    let bytes = alert_toast_component_bytes();
    let output_path = visual_layout_output_path(ALERT_TOAST_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        ALERT_TOAST_ATLAS_WIDTH,
        ALERT_TOAST_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("alert/toast component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn alert_toast_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        ALERT_TOAST_ATLAS_WIDTH,
        ALERT_TOAST_ATLAS_HEIGHT,
        ALERT_TOAST_ATLAS_BACKGROUND,
        model_rc(alert_toast_component_nodes()),
    )
}

fn alert_toast_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("AlertToastRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "AlertToastTitle",
            "Alerts and Toasts",
            22.0,
            20.0,
            260.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "AlertToastSubtitle",
            "Inline tones, notification toast action, close mark, focused, pressed and disabled states",
            22.0,
            42.0,
            740.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("AlertTonePanel", "panel", 18.0, 78.0, 260.0, 220.0),
        surface("ToastPanel", "panel", 300.0, 78.0, 278.0, 220.0),
        surface("AlertToastStatePanel", "inset", 600.0, 78.0, 282.0, 220.0),
        label("AlertToneTitle", "Inline Alert", 36.0, 96.0, 120.0, 18.0, 11.0, ""),
        label("ToastTitle", "Toast", 318.0, 96.0, 120.0, 18.0, 11.0, ""),
        label("AlertToastStateTitle", "States", 618.0, 96.0, 120.0, 18.0, 11.0, ""),
        alert(
            "WorkbenchInfoAlert",
            "Info: Asset indexed",
            "info",
            36.0,
            116.0,
            224.0,
            32.0,
            AlertToastState::Normal,
        ),
        alert(
            "WorkbenchSuccessAlert",
            "Success: Scene saved",
            "success",
            36.0,
            158.0,
            224.0,
            32.0,
            AlertToastState::Normal,
        ),
        alert(
            "WorkbenchWarningAlert",
            "Warning: Rebuild lighting",
            "warning",
            36.0,
            200.0,
            224.0,
            32.0,
            AlertToastState::Normal,
        ),
        alert(
            "WorkbenchErrorAlert",
            "Error: Missing mesh",
            "error",
            36.0,
            242.0,
            224.0,
            32.0,
            AlertToastState::Normal,
        ),
        toast(
            "Operation completed",
            318.0,
            126.0,
            240.0,
            36.0,
            AlertToastState::Normal,
            ToastColors::Default,
        ),
        toast(
            "Completed successfully",
            318.0,
            188.0,
            240.0,
            36.0,
            AlertToastState::Open,
            ToastColors::Declared,
        ),
        alert(
            "WorkbenchWarningAlert",
            "Pressed: confirm",
            "warning",
            618.0,
            116.0,
            230.0,
            32.0,
            AlertToastState::Pressed,
        ),
        alert(
            "WorkbenchWarningAlert",
            "Focused: keyboard",
            "warning",
            618.0,
            158.0,
            230.0,
            32.0,
            AlertToastState::Focused,
        ),
        toast(
            "Operation completed",
            618.0,
            200.0,
            240.0,
            36.0,
            AlertToastState::Hovered,
            ToastColors::Default,
        ),
        toast(
            "Operation completed",
            618.0,
            242.0,
            240.0,
            36.0,
            AlertToastState::Disabled,
            ToastColors::Default,
        ),
    ]
}

#[derive(Clone, Copy)]
enum AlertToastState {
    Normal,
    Hovered,
    Focused,
    Pressed,
    Open,
    Disabled,
}

#[derive(Clone, Copy)]
enum ToastColors {
    Default,
    Declared,
}

fn alert(
    control_id: &str,
    text: &str,
    tone: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: AlertToastState,
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Alert".into(),
        component_role: "alert".into(),
        text: text.into(),
        validation_level: tone.into(),
        icon_name: tone.into(),
        hovered: matches!(state, AlertToastState::Hovered),
        focused: matches!(state, AlertToastState::Focused),
        pressed: matches!(state, AlertToastState::Pressed),
        disabled: matches!(state, AlertToastState::Disabled),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    };
    node.popup_open = matches!(state, AlertToastState::Open);
    node
}

fn toast(
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    state: AlertToastState,
    colors: ToastColors,
) -> TemplatePaneNodeData {
    let mut node = alert(
        "WorkbenchToastRoot",
        text,
        "success",
        x,
        y,
        width,
        height,
        state,
    );
    node.value_number = 12.0;
    if matches!(colors, ToastColors::Declared) {
        node.label_color = Color::from_rgb_u8(
            DECLARED_TOAST_MARK[0],
            DECLARED_TOAST_MARK[1],
            DECLARED_TOAST_MARK[2],
        );
        node.value_color = Color::from_rgb_u8(
            DECLARED_TOAST_ACTION[0],
            DECLARED_TOAST_ACTION[1],
            DECLARED_TOAST_ACTION[2],
        );
    }
    node
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
    let index = ((y as usize * ALERT_TOAST_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * ALERT_TOAST_ATLAS_WIDTH as usize) + px as usize) * 4;
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
            let index = ((py as usize * ALERT_TOAST_ATLAS_WIDTH as usize) + px as usize) * 4;
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
