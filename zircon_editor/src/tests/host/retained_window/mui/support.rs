pub(super) use std::rc::Rc;

pub(super) use crate::ui::retained_host::primitives::{
    Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel,
};
pub(super) use crate::ui::retained_host::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
    template_node_command_summary_for_test, TemplateNodeFrameData, TemplatePaneNodeData,
};
pub(super) use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;
pub(super) use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

pub(super) const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
pub(super) const MID_BACKGROUND: [u8; 4] = [100, 100, 100, 255];
pub(super) const MATERIAL_PROGRESS_TRACK: [u8; 4] = EditorPaletteTokens::WORKBENCH_TRACK;
pub(super) const MATERIAL_ACCENT: [u8; 4] = EditorPaletteTokens::WORKBENCH_ACCENT;
pub(super) const MATERIAL_DIVIDER: [u8; 4] = EditorPaletteTokens::WORKBENCH_BORDER;
pub(super) const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];
pub(super) const MATERIAL_SKELETON_BG: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE_HOVER;
pub(super) const MUI_SKELETON_WAVE_ON_BG: [u8; 4] = EditorPaletteTokens::WORKBENCH_SEPARATOR_SOFT;
pub(super) const MATERIAL_WARNING_CONTAINER: [u8; 4] =
    EditorPaletteTokens::WORKBENCH_WARNING_CONTAINER;
pub(super) const MUI_TOOLTIP_BG: [u8; 4] = EditorPaletteTokens::WORKBENCH_POPUP;
pub(super) const MUI_SNACKBAR_BG: [u8; 4] = EditorPaletteTokens::WORKBENCH_POPUP;
pub(super) const MUI_X_GRID_HEADER: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE_HOVER;
pub(super) const MUI_X_GRID_SELECTED_ROW: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE_SELECTED;
pub(super) const MUI_X_GRID_ROW: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE[2];
pub(super) const MUI_X_CUSTOM_SURFACE: [u8; 4] = [24, 57, 91, 255];
pub(super) const MUI_X_SURFACE_INSET: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE_RECESSED;
pub(super) const MUI_X_TREE_SURFACE: [u8; 4] = EditorPaletteTokens::WORKBENCH_SUCCESS_CONTAINER;
pub(super) const MUI_X_TREE_MARKER: [u8; 4] = EditorPaletteTokens::WORKBENCH_SUCCESS;
pub(super) const MUI_X_PICKER_SECONDARY: [u8; 4] = EditorPaletteTokens::WORKBENCH_ACCENT_SOFT;
pub(super) const MUI_X_CHART_PLOT_BG: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE[2];
pub(super) const MUI_X_CHART_PRIMARY: [u8; 4] = EditorPaletteTokens::WORKBENCH_ACCENT;
pub(super) const MUI_X_CHART_SUCCESS: [u8; 4] = EditorPaletteTokens::WORKBENCH_SUCCESS;
pub(super) const MUI_X_CHAT_ERROR_SURFACE: [u8; 4] = EditorPaletteTokens::WORKBENCH_ERROR_CONTAINER;
pub(super) const MUI_X_CHAT_BUBBLE: [u8; 4] = EditorPaletteTokens::WORKBENCH_SURFACE[2];
pub(super) const MUI_X_CHAT_SELECTED_BUBBLE: [u8; 4] =
    EditorPaletteTokens::WORKBENCH_SURFACE_SELECTED;
pub(super) const MUI_AVATAR_SURFACE: [u8; 4] = [24, 57, 91, 255];
pub(super) const MUI_AVATAR_IMAGE: [u8; 4] = [201, 42, 33, 255];
pub(super) const MUI_BADGE_ERROR: [u8; 4] = EditorPaletteTokens::WORKBENCH_ERROR;
pub(super) const MUI_CHIP_WARNING: [u8; 4] = [237, 108, 2, 255];
pub(super) const MUI_CHIP_PRIMARY: [u8; 4] = EditorPaletteTokens::WORKBENCH_ACCENT;
pub(super) const MUI_CHIP_PRIMARY_DARK: [u8; 4] = EditorPaletteTokens::WORKBENCH_ACCENT;
pub(super) const MATERIAL_BORDER: [u8; 4] = EditorPaletteTokens::WORKBENCH_BORDER;
pub(super) const MATERIAL_FOCUS_RING: [u8; 4] = EditorPaletteTokens::WORKBENCH_FOCUS_RING;
pub(super) const MATERIAL_ERROR: [u8; 4] = EditorPaletteTokens::WORKBENCH_ERROR;
pub(super) const MUI_FIELD_FILLED_BACKGROUND_ON_BLACK: [u8; 4] =
    EditorPaletteTokens::WORKBENCH_SURFACE_RECESSED;

/// Expected output of the shared linear-light compositor for the default scrim.
pub(super) fn mui_backdrop_on_mid_background() -> [u8; 4] {
    blend_linear_over(MID_BACKGROUND, EditorPaletteTokens::WORKBENCH_SHADOW, 1.0)
}

/// Expected output when the shared popup token is painted at half opacity over black.
pub(super) fn mui_tooltip_bg_fade_half_on_black() -> [u8; 4] {
    blend_linear_over(BACKGROUND, MUI_TOOLTIP_BG, 0.5)
}

fn blend_linear_over(destination: [u8; 4], source: [u8; 4], coverage: f32) -> [u8; 4] {
    let source_alpha = (source[3] as f32 / 255.0) * coverage.clamp(0.0, 1.0);
    let destination_alpha = destination[3] as f32 / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if output_alpha <= f32::EPSILON {
        return [0, 0, 0, 0];
    }
    let inverse = 1.0 - source_alpha;
    let mut output = [0; 4];
    for channel in 0..3 {
        let premultiplied = srgb_byte_to_linear(source[channel]) * source_alpha
            + srgb_byte_to_linear(destination[channel]) * destination_alpha * inverse;
        output[channel] = linear_to_srgb_byte(premultiplied / output_alpha);
    }
    output[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    output
}

fn srgb_byte_to_linear(value: u8) -> f32 {
    let value = value as f32 / 255.0;
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb_byte(value: f32) -> u8 {
    let index = (value.clamp(0.0, 1.0) * 4_096.0).round() as usize;
    let linear = index as f32 / 4_096.0;
    let encoded = if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

pub(super) fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

pub(super) fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

pub(super) fn contains_pixel(bytes: &[u8], color: [u8; 4]) -> bool {
    bytes.chunks_exact(4).any(|pixel| pixel == color.as_slice())
}

pub(super) fn color_near(actual: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    actual
        .into_iter()
        .zip(expected)
        .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance)
}

pub(super) fn region_contains_color_near(
    bytes: &[u8],
    width: u32,
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
    color: [u8; 4],
    tolerance: u8,
) -> bool {
    let y1 = y.saturating_add(region_height);
    let x1 = x.saturating_add(region_width);
    (y..y1).any(|row| {
        (x..x1).any(|column| color_near(pixel(bytes, width, column, row), color, tolerance))
    })
}

pub(super) fn region_changed(
    bytes: &[u8],
    width: u32,
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> bool {
    let y1 = y.saturating_add(region_height);
    let x1 = x.saturating_add(region_width);
    (y..y1).any(|row| (x..x1).any(|column| pixel(bytes, width, column, row) != BACKGROUND))
}

pub(super) fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

pub(super) fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                color[0], color[1], color[2], color[3],
            ))),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

pub(super) fn resolved_foreground(color: [u8; 4]) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            foreground_color: Some(style_color(color)),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

pub(super) fn resolved_avatar_style(
    background: [u8; 4],
    foreground: [u8; 4],
    border: Option<[u8; 4]>,
    border_width: f32,
    corner_radius: f32,
) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            background_color: Some(style_color(background)),
            foreground_color: Some(style_color(foreground)),
            border_color: border.map(style_color),
            border_width,
            corner_radius,
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

pub(super) fn resolved_foreground_border_style(
    foreground: [u8; 4],
    border: [u8; 4],
    border_width: f32,
) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            foreground_color: Some(style_color(foreground)),
            border_color: Some(style_color(border)),
            border_width,
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

pub(super) fn style_color(color: [u8; 4]) -> UiStyleColor {
    UiStyleColor::Rgba(UiRgbaColor::from_u8(color[0], color[1], color[2], color[3]))
}

pub(super) fn solid_preview_image(color: [u8; 4]) -> Image {
    let width = 24;
    let height = 24;
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for _ in 0..width * height {
        pixels.extend_from_slice(&color);
    }
    Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixels, width, height,
    ))
}
