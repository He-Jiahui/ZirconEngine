use std::path::{Path, PathBuf};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};
use crate::ui::retained_host::{
    paint_template_nodes_for_test_with_background, TemplateNodeFrameData, TemplatePaneNodeData,
};

const MATERIAL_FEEDBACK_COMPONENT_SCREENSHOT: &str =
    "editor-components-material-feedback-primitives-900x360.png";
const MATERIAL_FEEDBACK_ATLAS_WIDTH: u32 = 900;
const MATERIAL_FEEDBACK_ATLAS_HEIGHT: u32 = 360;
const MATERIAL_FEEDBACK_ATLAS_BACKGROUND: [u8; 4] = [17, 20, 22, 255];

#[test]
fn material_feedback_component_visual_paints_foundation_primitives() {
    let bytes = material_feedback_component_bytes();

    let shell_surface = pixel_at(&bytes, 8, 8);
    let identity_panel = pixel_at(&bytes, 30, 96);
    let loading_panel = pixel_at(&bytes, 318, 96);
    let overlay_panel = pixel_at(&bytes, 622, 96);

    assert_ne!(
        identity_panel, shell_surface,
        "identity primitive panel should paint a Slate-style container layer"
    );
    assert_ne!(
        loading_panel, shell_surface,
        "loading primitive panel should paint a separate container layer"
    );
    assert_ne!(
        overlay_panel, shell_surface,
        "overlay primitive panel should paint a separate container layer"
    );

    let paper_surface = pixel_at(&bytes, 48, 132);
    assert_ne!(
        paper_surface, identity_panel,
        "Paper primitive should paint an elevated material surface inside the panel"
    );
    assert!(
        distinct_pixel_count(&bytes, 78, 132, 48, 44, &[identity_panel, paper_surface]) > 0,
        "image avatar should paint clipped preview pixels inside the Paper surface"
    );
    let initials_avatar_surface = pixel_at(&bytes, 154, 144);
    assert_ne!(
        initials_avatar_surface, identity_panel,
        "initials avatar should paint a rounded retained background"
    );
    assert!(
        distinct_pixel_count(
            &bytes,
            152,
            139,
            44,
            34,
            &[identity_panel, initials_avatar_surface],
        ) > 0,
        "initials avatar should paint centered runtime text"
    );

    let badge_root_surface = pixel_at(&bytes, 154, 214);
    let badge_overlay_surface = pixel_at(&bytes, 226, 204);
    assert_ne!(
        badge_root_surface, identity_panel,
        "Badge root should paint its retained label frame"
    );
    assert_ne!(
        badge_overlay_surface, badge_root_surface,
        "Badge overlay should paint independently from the root frame"
    );
    assert!(
        distinct_pixel_count(&bytes, 226, 198, 28, 20, &[badge_overlay_surface]) > 0,
        "Badge overlay should paint measured overlay text through the shared text path"
    );

    let divider_line = pixel_at(&bytes, 344, 143);
    assert_ne!(
        divider_line, loading_panel,
        "horizontal divider should paint a retained separator line"
    );
    assert!(
        distinct_pixel_count(&bytes, 430, 132, 68, 24, &[loading_panel, divider_line]) > 0,
        "labeled divider should reserve a text gap and paint the label"
    );
    let vertical_divider_line = pixel_at(&bytes, 526, 196);
    assert_ne!(
        vertical_divider_line, loading_panel,
        "vertical divider should paint from a narrow relative frame"
    );

    let skeleton_base = pixel_at(&bytes, 348, 184);
    let skeleton_wave = pixel_at(&bytes, 391, 184);
    assert_ne!(
        skeleton_base, loading_panel,
        "Skeleton primitive should paint a loading surface"
    );
    assert_ne!(
        skeleton_wave, skeleton_base,
        "wave skeleton should paint a moving highlight segment"
    );
    let linear_fill = pixel_at(&bytes, 348, 250);
    let linear_track = pixel_at(&bytes, 456, 250);
    assert_ne!(
        linear_fill, linear_track,
        "determinate linear progress should separate fill and track"
    );
    let indeterminate_segment = pixel_at(&bytes, 388, 280);
    assert_ne!(
        indeterminate_segment, linear_track,
        "indeterminate linear progress should paint relative percent segments"
    );

    let backdrop_surface = pixel_at(&bytes, 640, 130);
    let raised_paper = pixel_at(&bytes, 672, 150);
    assert_ne!(
        backdrop_surface, overlay_panel,
        "Backdrop primitive should paint an open scrim over its panel area"
    );
    assert_ne!(
        raised_paper, backdrop_surface,
        "Paper primitive should layer above the backdrop scrim"
    );
    assert!(
        distinct_pixel_count(&bytes, 745, 139, 46, 46, &[raised_paper, backdrop_surface]) > 0,
        "circular progress should paint rasterized track and fill pixels"
    );
}

#[test]
#[ignore = "writes local material/feedback primitive screenshot artifact for visual review"]
fn capture_material_feedback_component_visual_artifact() {
    let bytes = material_feedback_component_bytes();
    let output_path = visual_layout_output_path(MATERIAL_FEEDBACK_COMPONENT_SCREENSHOT);

    image::save_buffer_with_format(
        &output_path,
        &bytes,
        MATERIAL_FEEDBACK_ATLAS_WIDTH,
        MATERIAL_FEEDBACK_ATLAS_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("material/feedback primitive component screenshot should be written as PNG");

    assert!(
        output_path.exists(),
        "expected visual screenshot at {}",
        output_path.display()
    );
}

fn material_feedback_component_bytes() -> Vec<u8> {
    paint_template_nodes_for_test_with_background(
        MATERIAL_FEEDBACK_ATLAS_WIDTH,
        MATERIAL_FEEDBACK_ATLAS_HEIGHT,
        MATERIAL_FEEDBACK_ATLAS_BACKGROUND,
        model_rc(material_feedback_component_nodes()),
    )
}

fn material_feedback_component_nodes() -> Vec<TemplatePaneNodeData> {
    vec![
        surface("MaterialFeedbackRoot", "shell", 0.0, 0.0, 900.0, 360.0),
        label(
            "MaterialFeedbackTitle",
            "Material Feedback Primitives",
            22.0,
            20.0,
            320.0,
            22.0,
            13.0,
            "",
        ),
        label(
            "MaterialFeedbackSubtitle",
            "Paper, avatar, badge, dividers, skeletons, progress and backdrop",
            22.0,
            42.0,
            720.0,
            18.0,
            10.0,
            "muted",
        ),
        surface("IdentityPrimitivePanel", "panel", 18.0, 78.0, 262.0, 230.0),
        surface("LoadingPrimitivePanel", "panel", 310.0, 78.0, 270.0, 230.0),
        surface("OverlayPrimitivePanel", "panel", 612.0, 78.0, 270.0, 230.0),
        label(
            "IdentityPrimitiveTitle",
            "Identity",
            36.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        paper("IdentityPaper", "elevation", 38.0, 120.0, 214.0, 76.0, 2.0),
        avatar_image("AvatarPreview", 56.0, 132.0, 44.0, 44.0),
        avatar_text("AvatarInitials", "SL", 140.0, 132.0, 44.0, 44.0, "primary"),
        avatar_text("AvatarMuted", "QA", 198.0, 132.0, 36.0, 36.0, "secondary"),
        badge(
            "InboxBadge",
            "Inbox",
            "12",
            44.0,
            214.0,
            82.0,
            34.0,
            "primary",
        ),
        badge(
            "WarningBadge",
            "Bake",
            "3",
            154.0,
            214.0,
            72.0,
            34.0,
            "warning",
        ),
        badge_dot("SyncDotBadge", "Live", 44.0, 262.0, 70.0, 26.0),
        label(
            "LoadingPrimitiveTitle",
            "Loading + Rules",
            328.0,
            96.0,
            140.0,
            18.0,
            11.0,
            "",
        ),
        divider(
            "HorizontalDivider",
            "Alpha Stage",
            "horizontal",
            336.0,
            134.0,
            206.0,
            18.0,
        ),
        divider("VerticalDivider", "", "vertical", 520.0, 164.0, 12.0, 66.0),
        skeleton("WaveTextSkeleton", "text wave", 336.0, 176.0, 150.0, 16.0),
        skeleton("RoundedSkeleton", "rounded wave", 336.0, 204.0, 132.0, 20.0),
        skeleton("CircularSkeleton", "circular", 488.0, 194.0, 38.0, 38.0),
        progress("LinearProgress", "", 336.0, 248.0, 190.0, 6.0, 0.58),
        progress(
            "IndeterminateProgress",
            "indeterminate",
            336.0,
            278.0,
            190.0,
            6.0,
            0.0,
        ),
        label(
            "OverlayPrimitiveTitle",
            "Overlay",
            630.0,
            96.0,
            120.0,
            18.0,
            11.0,
            "",
        ),
        backdrop("OpenBackdrop", 632.0, 124.0, 226.0, 128.0),
        paper("BackdropPaper", "outlined", 662.0, 140.0, 160.0, 78.0, 4.0),
        circular_progress("CircularProgress", 744.0, 138.0, 48.0, 48.0, 0.72),
        progress("OverlayLinearProgress", "", 684.0, 234.0, 116.0, 6.0, 0.42),
        label(
            "BackdropCaption",
            "Backdrop keeps overlay rhythm without fixed pixel layout",
            632.0,
            268.0,
            218.0,
            16.0,
            10.0,
            "muted",
        ),
    ]
}

fn paper(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    elevation: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Paper".into(),
        component_role: "paper".into(),
        component_variant: variant.into(),
        elevation,
        border_width: if variant.contains("outlined") {
            1.0
        } else {
            0.0
        },
        corner_radius: 6.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn avatar_text(
    control_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Avatar".into(),
        component_role: "avatar".into(),
        text: text.into(),
        text_tone: tone.into(),
        border_width: 1.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn avatar_image(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Avatar".into(),
        component_role: "avatar".into(),
        has_preview_image: true,
        preview_image: avatar_preview_image(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn badge(
    control_id: &str,
    label_text: &str,
    value_text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    tone: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Badge".into(),
        component_role: "badge".into(),
        component_variant: tone.into(),
        text: label_text.into(),
        value_text: value_text.into(),
        border_width: 1.0,
        font_size: 12.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn badge_dot(
    control_id: &str,
    label_text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_variant: "dot success".into(),
        value_text: String::new().into(),
        ..badge(control_id, label_text, "", x, y, width, height, "success")
    }
}

fn divider(
    control_id: &str,
    text: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Divider".into(),
        component_role: "divider".into(),
        component_variant: variant.into(),
        text: text.into(),
        text_tone: "secondary".into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn skeleton(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Skeleton".into(),
        component_role: "skeleton".into(),
        component_variant: variant.into(),
        corner_radius: 5.0,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn progress(
    control_id: &str,
    variant: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    value_percent: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Progress".into(),
        component_role: "progress-bar".into(),
        component_variant: variant.into(),
        value_percent,
        corner_radius: height * 0.5,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn circular_progress(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    value_percent: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        role: "CircularProgress".into(),
        component_role: "circular-progress".into(),
        component_variant: "circular".into(),
        ..progress(control_id, "circular", x, y, width, height, value_percent)
    }
}

fn backdrop(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Backdrop".into(),
        component_role: "backdrop".into(),
        surface_variant: "backdrop".into(),
        popup_open: true,
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
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
        node_id: node_id(control_id),
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
        node_id: node_id(control_id),
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        font_size,
        text_tone: tone.into(),
        frame: frame(x, y, width, height),
        ..TemplatePaneNodeData::default()
    }
}

fn avatar_preview_image() -> Image {
    image_from_fn(5, 5, |x, y| match (x + y * 2) % 5 {
        0 => [50, 138, 166, 255],
        1 => [28, 38, 44, 255],
        2 => [95, 177, 183, 255],
        3 => [219, 125, 64, 255],
        _ => [183, 78, 71, 255],
    })
}

fn image_from_fn<F>(width: u32, height: u32, mut color_at: F) -> Image
where
    F: FnMut(u32, u32) -> [u8; 4],
{
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&color_at(x, y));
        }
    }
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixels, width, height,
    ))
}

fn node_id(control_id: &str) -> String {
    format!("{control_id}.node")
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
    let index = ((y as usize * MATERIAL_FEEDBACK_ATLAS_WIDTH as usize) + x as usize) * 4;
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
            let index = ((py as usize * MATERIAL_FEEDBACK_ATLAS_WIDTH as usize) + px as usize) * 4;
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
