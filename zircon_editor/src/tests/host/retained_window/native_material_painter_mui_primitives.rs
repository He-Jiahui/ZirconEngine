use std::rc::Rc;

use crate::ui::retained_host::primitives::{
    Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel,
};
use crate::ui::retained_host::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
    TemplateNodeFrameData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::style::{
    ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};

const BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const MID_BACKGROUND: [u8; 4] = [100, 100, 100, 255];
const MUI_BACKDROP_ON_MID_BACKGROUND: [u8; 4] = [49, 49, 49, 255];
const MATERIAL_PROGRESS_TRACK: [u8; 4] = [42, 52, 60, 255];
const MATERIAL_ACCENT: [u8; 4] = [53, 199, 208, 255];
const MATERIAL_DIVIDER: [u8; 4] = [75, 98, 109, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];
const MATERIAL_SKELETON_BG: [u8; 4] = [58, 66, 73, 255];
const MUI_SKELETON_WAVE_ON_BG: [u8; 4] = [85, 92, 98, 255];
const MATERIAL_WARNING_CONTAINER: [u8; 4] = [70, 49, 18, 255];
const MUI_TOOLTIP_BG: [u8; 4] = [97, 97, 97, 255];
const MUI_TOOLTIP_BG_FADE_HALF_ON_BLACK: [u8; 4] = [48, 48, 48, 255];
const MUI_SNACKBAR_BG: [u8; 4] = [50, 50, 50, 255];
const MUI_X_GRID_HEADER: [u8; 4] = [47, 70, 80, 255];
const MUI_X_GRID_SELECTED_ROW: [u8; 4] = [15, 101, 116, 255];
const MUI_X_GRID_ROW: [u8; 4] = [32, 40, 48, 255];
const MUI_X_CUSTOM_SURFACE: [u8; 4] = [24, 57, 91, 255];
const MUI_X_SURFACE_INSET: [u8; 4] = [18, 24, 30, 255];
const MUI_X_TREE_SURFACE: [u8; 4] = [29, 71, 47, 255];
const MUI_X_TREE_MARKER: [u8; 4] = [92, 190, 122, 255];
const MUI_X_PICKER_SECONDARY: [u8; 4] = [156, 39, 176, 255];
const MUI_X_CHART_PLOT_BG: [u8; 4] = [32, 40, 48, 255];
const MUI_X_CHART_PRIMARY: [u8; 4] = [53, 199, 208, 255];
const MUI_X_CHART_SUCCESS: [u8; 4] = [92, 190, 122, 255];
const MUI_X_CHAT_ERROR_SURFACE: [u8; 4] = [76, 36, 39, 255];
const MUI_X_CHAT_BUBBLE: [u8; 4] = [32, 40, 48, 255];
const MUI_X_CHAT_SELECTED_BUBBLE: [u8; 4] = [15, 101, 116, 255];
const MUI_AVATAR_SURFACE: [u8; 4] = [24, 57, 91, 255];
const MUI_AVATAR_IMAGE: [u8; 4] = [201, 42, 33, 255];
const MUI_BADGE_ERROR: [u8; 4] = [211, 47, 47, 255];
const MUI_CHIP_WARNING: [u8; 4] = [237, 108, 2, 255];
const MUI_CHIP_PRIMARY: [u8; 4] = [25, 118, 210, 255];
const MUI_CHIP_PRIMARY_DARK: [u8; 4] = [21, 101, 192, 255];
const MATERIAL_BORDER: [u8; 4] = [75, 98, 109, 255];
const MATERIAL_FOCUS_RING: [u8; 4] = [128, 234, 255, 255];
const MATERIAL_ERROR: [u8; 4] = [239, 112, 102, 255];
const MUI_FIELD_FILLED_BACKGROUND_ON_BLACK: [u8; 4] = [23, 23, 23, 255];

#[test]
fn native_template_painter_draws_mui_linear_progress_track_and_fill() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "LinearProgress".into(),
        node_id: "LinearProgress.node".into(),
        role: "Progress".into(),
        component_role: "progress".into(),
        component_variant: "determinate linear".into(),
        value_percent: 0.62,
        frame: frame(4.0, 8.0, 100.0, 8.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(120, 28, nodes);

    assert_eq!(pixel(&bytes, 120, 12, 12), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 120, 92, 12), MATERIAL_PROGRESS_TRACK);
    assert_eq!(pixel(&bytes, 120, 112, 12), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_circular_progress_ring() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "CircularProgress".into(),
        node_id: "CircularProgress.node".into(),
        role: "Progress".into(),
        component_role: "progress".into(),
        component_variant: "circular determinate".into(),
        value_percent: 0.5,
        frame: frame(4.0, 4.0, 32.0, 32.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(44, 44, nodes);

    assert_eq!(pixel(&bytes, 44, 20, 5), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 44, 20, 34), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 44, 5, 20), MATERIAL_PROGRESS_TRACK);
    assert_eq!(pixel(&bytes, 44, 20, 20), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_skeleton_variants() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "RoundedSkeleton".into(),
            node_id: "RoundedSkeleton.node".into(),
            role: "Skeleton".into(),
            component_role: "skeleton".into(),
            component_variant: "rounded".into(),
            frame: frame(4.0, 4.0, 80.0, 16.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "CircularSkeleton".into(),
            node_id: "CircularSkeleton.node".into(),
            role: "Skeleton".into(),
            component_role: "skeleton".into(),
            component_variant: "circular".into(),
            frame: frame(4.0, 28.0, 80.0, 16.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 52, nodes);

    assert_eq!(pixel(&bytes, 96, 44, 12), MATERIAL_SKELETON_BG);
    assert_eq!(pixel(&bytes, 96, 4, 36), BACKGROUND);
    assert_eq!(pixel(&bytes, 96, 44, 36), MATERIAL_SKELETON_BG);
}

#[test]
fn native_template_painter_draws_mui_skeleton_text_wave_and_hides_children() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "TextSkeleton".into(),
            node_id: "TextSkeleton.node".into(),
            role: "Skeleton".into(),
            component_role: "skeleton".into(),
            component_variant: "text wave withChildren".into(),
            frame: frame(4.0, 4.0, 100.0, 20.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "TextSkeletonChild".into(),
            node_id: "TextSkeletonChild.node".into(),
            role: "Label".into(),
            component_variant: "muiSkeletonChild".into(),
            text: "Loading".into(),
            button_style: resolved_background(MUI_AVATAR_IMAGE),
            frame: frame(12.0, 10.0, 40.0, 8.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(116, 32, nodes);

    assert_eq!(pixel(&bytes, 116, 10, 6), BACKGROUND);
    assert_eq!(pixel(&bytes, 116, 20, 14), MATERIAL_SKELETON_BG);
    assert_eq!(pixel(&bytes, 116, 44, 14), MUI_SKELETON_WAVE_ON_BG);
}

#[test]
fn native_template_painter_draws_mui_backdrop_scrim_and_invisible_variant() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Backdrop".into(),
            node_id: "Backdrop.node".into(),
            role: "Backdrop".into(),
            component_role: "backdrop".into(),
            popup_open: true,
            frame: frame(0.0, 0.0, 32.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "InvisibleBackdrop".into(),
            node_id: "InvisibleBackdrop.node".into(),
            role: "Backdrop".into(),
            component_role: "backdrop".into(),
            component_variant: "invisible".into(),
            popup_open: true,
            frame: frame(36.0, 0.0, 32.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test_with_background(72, 36, MID_BACKGROUND, nodes);

    assert_eq!(pixel(&bytes, 72, 16, 16), MUI_BACKDROP_ON_MID_BACKGROUND);
    assert_eq!(pixel(&bytes, 72, 52, 16), MID_BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_overlay_surface_tones() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Tooltip".into(),
            node_id: "Tooltip.node".into(),
            role: "Panel".into(),
            component_role: "tooltip".into(),
            surface_variant: "tooltip".into(),
            corner_radius: 4.0,
            frame: frame(4.0, 4.0, 60.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "Snackbar".into(),
            node_id: "Snackbar.node".into(),
            role: "Panel".into(),
            component_role: "snackbar".into(),
            surface_variant: "snackbar".into(),
            corner_radius: 4.0,
            elevation: 6.0,
            frame: frame(4.0, 28.0, 80.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 56, nodes);

    assert_eq!(pixel(&bytes, 96, 12, 12), MUI_TOOLTIP_BG);
    assert_eq!(pixel(&bytes, 96, 12, 36), MUI_SNACKBAR_BG);
}

#[test]
fn native_template_painter_draws_mui_alert_severity_surface() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "WarningAlert".into(),
        node_id: "WarningAlert.node".into(),
        role: "Alert".into(),
        component_role: "alert".into(),
        surface_variant: "alert".into(),
        validation_level: "warning".into(),
        corner_radius: 4.0,
        frame: frame(4.0, 4.0, 88.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(100, 36, nodes);

    assert_eq!(pixel(&bytes, 100, 44, 16), MATERIAL_WARNING_CONTAINER);
}

#[test]
fn native_template_painter_draws_mui_divider_middle_horizontal_line() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "HorizontalDivider".into(),
        node_id: "HorizontalDivider.node".into(),
        role: "Divider".into(),
        component_role: "divider".into(),
        component_variant: "middle horizontal".into(),
        frame: frame(4.0, 4.0, 120.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(132, 36, nodes);

    assert_eq!(pixel(&bytes, 132, 18, 16), BACKGROUND);
    assert_eq!(pixel(&bytes, 132, 22, 16), MATERIAL_DIVIDER);
    assert_eq!(pixel(&bytes, 132, 106, 16), MATERIAL_DIVIDER);
    assert_eq!(pixel(&bytes, 132, 110, 16), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_divider_vertical_with_children_gap() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "VerticalDivider".into(),
        node_id: "VerticalDivider.node".into(),
        role: "Divider".into(),
        component_role: "divider".into(),
        component_variant: "middle vertical flexItem withChildren".into(),
        text: " ".into(),
        frame: frame(10.0, 4.0, 24.0, 80.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(48, 92, nodes);

    assert_eq!(pixel(&bytes, 48, 22, 8), BACKGROUND);
    assert_eq!(pixel(&bytes, 48, 22, 14), MATERIAL_DIVIDER);
    assert_eq!(pixel(&bytes, 48, 22, 44), BACKGROUND);
    assert_eq!(pixel(&bytes, 48, 22, 74), MATERIAL_DIVIDER);
}

#[test]
fn native_template_painter_draws_mui_timeline_dot_connector_and_separator_geometry() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "TimelineSeparator".into(),
            node_id: "TimelineSeparator.node".into(),
            role: "TimelineSeparator".into(),
            component_role: "timeline-separator".into(),
            surface_variant: "elevated".into(),
            frame: frame(4.0, 4.0, 18.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "TimelineDot".into(),
            node_id: "TimelineDot.node".into(),
            role: "TimelineDot".into(),
            component_role: "timeline-dot".into(),
            component_variant: "outlined secondary".into(),
            text_tone: "secondary".into(),
            frame: frame(8.0, 4.0, 10.0, 10.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "TimelineConnector".into(),
            node_id: "TimelineConnector.node".into(),
            role: "TimelineConnector".into(),
            component_role: "timeline-connector".into(),
            button_style: resolved_background(MATERIAL_ACCENT),
            frame: frame(12.0, 16.0, 2.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(36, 40, nodes);

    assert_eq!(pixel(&bytes, 36, 5, 20), BACKGROUND);
    assert_eq!(pixel(&bytes, 36, 13, 4), MUI_SECONDARY_MAIN);
    assert_eq!(pixel(&bytes, 36, 13, 9), BACKGROUND);
    assert_eq!(pixel(&bytes, 36, 12, 24), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 36, 15, 24), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_text_field_variants_without_hiding_value_text() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "OutlinedTextField".into(),
            node_id: "OutlinedTextField.node".into(),
            role: "TextField".into(),
            component_role: "input-field".into(),
            component_variant: "outlined".into(),
            value_text: "Atlas".into(),
            frame: frame(4.0, 4.0, 96.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "FilledTextField".into(),
            node_id: "FilledTextField.node".into(),
            role: "TextField".into(),
            component_role: "input-field".into(),
            component_variant: "filled focused".into(),
            focused: true,
            value_text: "Focused".into(),
            frame: frame(4.0, 44.0, 96.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "StandardTextField".into(),
            node_id: "StandardTextField.node".into(),
            role: "TextField".into(),
            component_role: "input-field".into(),
            component_variant: "standard error".into(),
            validation_level: "error".into(),
            value_text: "Error".into(),
            frame: frame(4.0, 84.0, 96.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(112, 124, nodes);

    assert_eq!(pixel(&bytes, 112, 52, 4), MATERIAL_BORDER);
    assert_eq!(pixel(&bytes, 112, 52, 20), BACKGROUND);
    assert!(
        region_changed(&bytes, 112, 10, 12, 48, 12),
        "outlined text field should still draw its editable value text"
    );
    assert_eq!(
        pixel(&bytes, 112, 12, 52),
        MUI_FIELD_FILLED_BACKGROUND_ON_BLACK
    );
    assert_eq!(pixel(&bytes, 112, 52, 74), MATERIAL_FOCUS_RING);
    assert_ne!(pixel(&bytes, 112, 12, 100), BACKGROUND);
    assert_eq!(pixel(&bytes, 112, 52, 114), MATERIAL_ERROR);
}

#[test]
fn native_template_painter_draws_mui_svg_icon_from_name_without_preview() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "AddCircleIcon".into(),
        node_id: "AddCircleIcon.node".into(),
        role: "SvgIcon".into(),
        component_role: "svg-icon".into(),
        icon_name: "AddCircle".into(),
        button_style: resolved_foreground(MUI_SECONDARY_MAIN),
        frame: frame(4.0, 4.0, 40.0, 40.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(52, 52, nodes);

    assert_eq!(pixel(&bytes, 52, 5, 5), BACKGROUND);
    assert!(
        contains_pixel(&bytes, MUI_SECONDARY_MAIN),
        "SvgIcon should load the local MUI module and use the resolved foreground tint"
    );
}

#[test]
fn native_template_painter_draws_missing_mui_icon_fallback() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "MissingIcon".into(),
        node_id: "MissingIcon.node".into(),
        role: "Icon".into(),
        component_role: "icon".into(),
        icon_name: "missing_zircon_mui_icon".into(),
        button_style: resolved_foreground(MUI_CHIP_WARNING),
        frame: frame(4.0, 4.0, 32.0, 32.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(44, 44, nodes);

    assert!(
        contains_pixel(&bytes, MUI_CHIP_WARNING),
        "missing Icon nodes should produce a visible tinted fallback instead of a blank slot"
    );
}

#[test]
fn native_template_painter_draws_mui_avatar_rounded_fallback_shape() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "RoundedAvatar".into(),
        node_id: "RoundedAvatar.node".into(),
        role: "Avatar".into(),
        component_role: "avatar".into(),
        component_variant: "rounded colorDefault".into(),
        text: "ZR".into(),
        button_style: resolved_avatar_style(
            MUI_AVATAR_SURFACE,
            MUI_SECONDARY_MAIN,
            Some(MUI_SECONDARY_MAIN),
            1.0,
            4.0,
        ),
        corner_radius: 4.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 24.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(40, 36, nodes);

    assert_eq!(pixel(&bytes, 40, 4, 4), BACKGROUND);
    assert_eq!(pixel(&bytes, 40, 8, 4), MUI_SECONDARY_MAIN);
    assert_eq!(pixel(&bytes, 40, 27, 16), MUI_SECONDARY_MAIN);
    assert_eq!(pixel(&bytes, 40, 26, 16), MUI_AVATAR_SURFACE);
}

#[test]
fn native_template_painter_clips_mui_avatar_image_to_circular_shape() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ImageAvatar".into(),
        node_id: "ImageAvatar.node".into(),
        role: "Avatar".into(),
        component_role: "avatar".into(),
        component_variant: "circular".into(),
        has_preview_image: true,
        preview_image: solid_preview_image(MUI_AVATAR_IMAGE),
        button_style: resolved_avatar_style(MUI_AVATAR_SURFACE, MUI_SECONDARY_MAIN, None, 0.0, 0.0),
        frame: frame(4.0, 4.0, 24.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(40, 36, nodes);

    assert_eq!(pixel(&bytes, 40, 4, 4), BACKGROUND);
    assert_eq!(pixel(&bytes, 40, 5, 5), BACKGROUND);
    assert_eq!(pixel(&bytes, 40, 16, 4), MUI_AVATAR_IMAGE);
    assert_eq!(pixel(&bytes, 40, 16, 16), MUI_AVATAR_IMAGE);
}

#[test]
fn native_template_painter_draws_mui_badge_standard_bottom_left_overlay() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ErrorBadge".into(),
        node_id: "ErrorBadge.node".into(),
        role: "Badge".into(),
        component_role: "badge".into(),
        component_variant:
            "standard error circular bottom left overlapCircular anchorOriginBottomLeftCircular"
                .into(),
        text: "Alerts".into(),
        value_text: "12".into(),
        button_style: resolved_avatar_style(
            MUI_X_SURFACE_INSET,
            MATERIAL_ACCENT,
            Some(MUI_SECONDARY_MAIN),
            1.0,
            10.0,
        ),
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(16.0, 4.0, 64.0, 28.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(96, 48, nodes);

    assert_eq!(pixel(&bytes, 96, 20, 12), MUI_X_SURFACE_INSET);
    assert_eq!(pixel(&bytes, 96, 10, 28), BACKGROUND);
    assert_eq!(pixel(&bytes, 96, 24, 28), MUI_BADGE_ERROR);
}

#[test]
fn native_template_painter_hides_mui_badge_invisible_dot_and_consumes_badge_slot() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "HiddenBadge".into(),
            node_id: "HiddenBadge.node".into(),
            role: "Badge".into(),
            component_role: "badge".into(),
            component_variant: "dot invisible error circular bottom left".into(),
            frame: frame(16.0, 4.0, 64.0, 28.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "HiddenBadgeSlot".into(),
            node_id: "HiddenBadge.slot".into(),
            role: "Label".into(),
            component_role: "label".into(),
            component_variant: "muiBadgeSlot dot invisible error circular bottom left".into(),
            text: "x".into(),
            button_style: resolved_background(MUI_BADGE_ERROR),
            frame: frame(20.0, 20.0, 18.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 48, nodes);

    assert_eq!(pixel(&bytes, 96, 28, 28), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_chip_outlined_delete_icon_geometry() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "WarningChip".into(),
        node_id: "WarningChip.node".into(),
        role: "Chip".into(),
        component_role: "chip".into(),
        component_variant: "outlined small warning clickable deletable hasDeleteIcon".into(),
        text: "Warn".into(),
        button_style: resolved_foreground_border_style(MUI_CHIP_WARNING, MUI_CHIP_WARNING, 1.0),
        frame: frame(4.0, 4.0, 80.0, 28.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(96, 40, nodes);

    assert_eq!(pixel(&bytes, 96, 20, 6), MUI_CHIP_WARNING);
    assert!(color_near(pixel(&bytes, 96, 20, 18), MUI_CHIP_WARNING, 3));
    assert!(region_contains_color_near(
        &bytes,
        96,
        67,
        13,
        11,
        11,
        MUI_CHIP_WARNING,
        3
    ));
}

#[test]
fn native_template_painter_draws_mui_chip_avatar_and_consumes_chip_slot() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "PrimaryChip".into(),
            node_id: "PrimaryChip.node".into(),
            role: "Chip".into(),
            component_role: "chip".into(),
            component_variant: "filled medium primary hasAvatar".into(),
            text: "Build".into(),
            frame: frame(4.0, 4.0, 104.0, 36.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "PrimaryChipAvatar".into(),
            node_id: "PrimaryChip.avatar".into(),
            role: "Avatar".into(),
            component_role: "avatar".into(),
            component_variant: "chipSlotAvatar".into(),
            button_style: resolved_background(MUI_AVATAR_IMAGE),
            frame: frame(8.0, 8.0, 24.0, 24.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(120, 48, nodes);

    assert_eq!(pixel(&bytes, 120, 50, 20), MUI_CHIP_PRIMARY);
    assert_eq!(pixel(&bytes, 120, 21, 22), MUI_CHIP_PRIMARY_DARK);
}

#[test]
fn native_template_painter_sorts_template_nodes_by_mui_z_index() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Tooltip".into(),
            node_id: "Tooltip.node".into(),
            role: "Panel".into(),
            component_role: "tooltip".into(),
            surface_variant: "tooltip".into(),
            z_index: 1500,
            frame: frame(4.0, 4.0, 32.0, 20.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "NormalPanel".into(),
            node_id: "NormalPanel.node".into(),
            role: "Panel".into(),
            component_role: "panel".into(),
            surface_variant: "primary".into(),
            z_index: 0,
            frame: frame(4.0, 4.0, 32.0, 20.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(48, 32, nodes);

    assert_eq!(pixel(&bytes, 48, 12, 12), MUI_TOOLTIP_BG);
}

#[test]
fn native_template_painter_applies_mui_transition_opacity() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "FadeTooltip".into(),
        node_id: "FadeTooltip.node".into(),
        role: "Panel".into(),
        component_role: "tooltip".into(),
        surface_variant: "tooltip".into(),
        transition_kind: "fade".into(),
        transition_progress: 0.5,
        frame: frame(4.0, 4.0, 32.0, 20.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(48, 32, nodes);

    assert_eq!(pixel(&bytes, 48, 12, 12), MUI_TOOLTIP_BG_FADE_HALF_ON_BLACK);
}

#[test]
fn native_template_painter_draws_mui_x_data_grid_chrome() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "DataGrid".into(),
        node_id: "DataGrid.node".into(),
        role: "DataGrid".into(),
        component_role: "mui-x-data-grid".into(),
        selected: true,
        checked: true,
        focused: true,
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 96.0, 38.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 52, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 10), MUI_X_GRID_HEADER);
    assert_eq!(pixel(&bytes, 112, 20, 22), MUI_X_GRID_SELECTED_ROW);
    assert_eq!(pixel(&bytes, 112, 20, 27), MUI_X_GRID_ROW);
}

#[test]
fn native_template_painter_draws_mui_x_tree_view_items() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "MaterialTreeView".into(),
        node_id: "MaterialTreeView.node".into(),
        role: "MaterialTreeView".into(),
        component_role: "mui-x-tree-view".into(),
        selected: true,
        checked: true,
        expanded: true,
        popup_open: true,
        focused: true,
        corner_radius: 10.0,
        frame: frame(4.0, 4.0, 96.0, 38.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 52, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 40), MUI_X_TREE_SURFACE);
    assert_eq!(pixel(&bytes, 112, 20, 12), MUI_X_GRID_SELECTED_ROW);
    assert_eq!(pixel(&bytes, 112, 26, 24), MUI_X_GRID_HEADER);
    assert_eq!(pixel(&bytes, 112, 12, 11), MUI_X_TREE_MARKER);
}

#[test]
fn native_template_painter_draws_mui_x_root_custom_surface_color() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "CustomDataGrid".into(),
        node_id: "CustomDataGrid.node".into(),
        role: "DataGrid".into(),
        component_role: "panel".into(),
        button_style: resolved_background(MUI_X_CUSTOM_SURFACE),
        frame: frame(4.0, 4.0, 96.0, 38.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 52, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 10), MUI_X_GRID_HEADER);
    assert_eq!(pixel(&bytes, 112, 50, 38), MUI_X_CUSTOM_SURFACE);
}

#[test]
fn native_template_painter_draws_mui_x_date_time_picker_field_and_popup() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "DateTimePickers".into(),
        node_id: "DateTimePickers.node".into(),
        role: "DateTimePickers".into(),
        component_role: "mui-x-date-time-pickers".into(),
        component_variant: "desktop".into(),
        selected: true,
        popup_open: true,
        focused: true,
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 96.0, 50.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 64, nodes);

    assert_eq!(pixel(&bytes, 112, 48, 12), MUI_X_SURFACE_INSET);
    assert_eq!(pixel(&bytes, 112, 91, 12), MUI_X_PICKER_SECONDARY);
    assert_eq!(pixel(&bytes, 112, 20, 44), MUI_X_GRID_ROW);
    assert_eq!(pixel(&bytes, 112, 50, 42), MUI_X_PICKER_SECONDARY);
}

#[test]
fn native_template_painter_draws_mui_x_chart_plot_and_series() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "Charts".into(),
        node_id: "Charts.node".into(),
        role: "Charts".into(),
        component_role: "mui-x-charts".into(),
        component_variant: "loading".into(),
        focused: true,
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 96.0, 48.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 60, nodes);

    assert_eq!(pixel(&bytes, 112, 90, 20), MUI_X_CHART_PLOT_BG);
    assert_eq!(pixel(&bytes, 112, 30, 36), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 112, 50, 36), MUI_X_CHART_SUCCESS);
}

#[test]
fn native_template_painter_draws_mui_x_chart_subtype_feedback() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "LineChart".into(),
            node_id: "LineChart.node".into(),
            role: "LineChart".into(),
            component_role: "mui-x-line-chart".into(),
            focused: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(4.0, 4.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "PieChart".into(),
            node_id: "PieChart.node".into(),
            role: "PieChart".into(),
            component_role: "mui-x-pie-chart".into(),
            selected: true,
            checked: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(104.0, 4.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "Gauge".into(),
            node_id: "Gauge.node".into(),
            role: "Gauge".into(),
            component_role: "mui-x-gauge".into(),
            value_percent: 0.68,
            focused: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(4.0, 60.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "SparkLineChart".into(),
            node_id: "SparkLineChart.node".into(),
            role: "SparkLineChart".into(),
            component_role: "mui-x-sparkline".into(),
            hovered: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(104.0, 60.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(212, 116, nodes);

    assert_eq!(pixel(&bytes, 212, 72, 20), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 212, 160, 28), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 212, 144, 28), MUI_X_CHART_SUCCESS);
    assert_eq!(pixel(&bytes, 212, 152, 28), MUI_X_CHART_PLOT_BG);
    assert_eq!(pixel(&bytes, 212, 52, 72), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 212, 164, 78), MUI_X_CHART_PRIMARY);
}

#[test]
fn native_template_painter_draws_mui_x_agent_chat_and_composer() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "AgentChat".into(),
            node_id: "AgentChat.node".into(),
            role: "AgentChat".into(),
            component_role: "mui-x-agent-chat".into(),
            component_variant: "streaming".into(),
            validation_level: "error".into(),
            focused: true,
            frame: frame(4.0, 4.0, 96.0, 44.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "ChatComposer".into(),
            node_id: "ChatComposer.node".into(),
            role: "ChatComposer".into(),
            component_role: "mui-x-chat-composer".into(),
            focused: true,
            frame: frame(4.0, 52.0, 96.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(112, 76, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 6), MUI_X_CHAT_ERROR_SURFACE);
    assert_eq!(pixel(&bytes, 112, 12, 12), MUI_X_CHAT_BUBBLE);
    assert_eq!(pixel(&bytes, 112, 82, 26), MUI_X_CHAT_SELECTED_BUBBLE);
    assert_eq!(pixel(&bytes, 112, 20, 43), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 112, 88, 61), MUI_X_CHART_PRIMARY);
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn contains_pixel(bytes: &[u8], color: [u8; 4]) -> bool {
    bytes.chunks_exact(4).any(|pixel| pixel == color.as_slice())
}

fn color_near(actual: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    actual
        .into_iter()
        .zip(expected)
        .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance)
}

fn region_contains_color_near(
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

fn region_changed(
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

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
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

fn resolved_foreground(color: [u8; 4]) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        element: UiResolvedElementStyle {
            foreground_color: Some(style_color(color)),
            ..UiResolvedElementStyle::default()
        },
        ..ResolvedButtonStyle::default()
    }
}

fn resolved_avatar_style(
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

fn resolved_foreground_border_style(
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

fn style_color(color: [u8; 4]) -> UiStyleColor {
    UiStyleColor::Rgba(UiRgbaColor::from_u8(color[0], color[1], color[2], color[3]))
}

fn solid_preview_image(color: [u8; 4]) -> Image {
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
