use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::cascade_registry::insert_float_token;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorDensityTokens {
    pub gap_xsmall: f32,
    pub gap_tight: f32,
    pub gap_small: f32,
    pub gap_regular: f32,
    pub gap_medium: f32,
    pub gap_large: f32,
    pub drawer_padding: f32,
    pub panel_padding: f32,
    pub toolbar_action_width: f32,
    pub toolbar_wide_action_width: f32,
    pub ui_asset_action_min_width: f32,
    pub ui_asset_action_preferred_width: f32,
    pub ui_asset_action_max_width: f32,
    pub ui_asset_side_min_width: f32,
    pub ui_asset_side_preferred_width: f32,
    pub ui_asset_center_min_width: f32,
    pub ui_asset_center_preferred_width: f32,
    pub ui_asset_header_kind_min_width: f32,
    pub ui_asset_header_kind_preferred_width: f32,
    pub ui_asset_header_kind_max_width: f32,
    pub ui_asset_tool_min_width: f32,
    pub ui_asset_tool_preferred_width: f32,
    pub ui_asset_tool_max_width: f32,
    pub command_palette_min_width: f32,
    pub command_palette_preferred_width: f32,
    pub command_palette_max_width: f32,
    pub command_palette_min_height: f32,
    pub command_palette_preferred_height: f32,
    pub command_palette_max_height: f32,
    pub dialog_min_width: f32,
    pub dialog_preferred_width: f32,
    pub dialog_max_width: f32,
    pub dialog_min_height: f32,
    pub dialog_preferred_height: f32,
    pub dialog_max_height: f32,
    pub confirm_dialog_preferred_width: f32,
    pub confirm_dialog_max_width: f32,
    pub confirm_dialog_min_height: f32,
    pub confirm_dialog_preferred_height: f32,
    pub confirm_dialog_max_height: f32,
    pub notification_panel_min_width: f32,
    pub notification_panel_preferred_width: f32,
    pub notification_panel_max_width: f32,
    pub notification_panel_min_height: f32,
    pub notification_panel_preferred_height: f32,
    pub notification_panel_max_height: f32,
    pub caption_min_height: f32,
    pub caption_preferred_height: f32,
    pub caption_max_height: f32,
    pub label_min_height: f32,
    pub label_preferred_height: f32,
    pub label_max_height: f32,
    pub chip_min_width: f32,
    pub chip_preferred_width: f32,
    pub chip_max_width: f32,
    pub axis_value_field_min_width: f32,
    pub axis_value_field_preferred_width: f32,
    pub axis_value_field_max_width: f32,
    pub row_height: f32,
    pub left_drawer_width: f32,
    pub right_drawer_width: f32,
    pub bottom_output_height: f32,
    pub breakpoint_ultra_width: f32,
    pub breakpoint_narrow_width: f32,
    pub breakpoint_wide_width: f32,
    pub compact_side_width: f32,
    pub ultra_compact_side_width: f32,
    pub compact_left_drawer_max_width: f32,
    pub compact_right_drawer_max_width: f32,
    pub compact_side_min_width: f32,
    pub minimum_document_width_fraction: f32,
    pub ultra_compact_left_drawer_max_width: f32,
    pub ultra_compact_right_drawer_max_width: f32,
    pub compact_bottom_available_height: f32,
    pub compact_bottom_max_height: f32,
    pub compact_bottom_max_available_fraction: f32,
    pub compact_bottom_min_height: f32,
    pub ultra_compact_bottom_available_height: f32,
    pub ultra_compact_bottom_max_height: f32,
    pub ultra_compact_bottom_max_available_fraction: f32,
    pub ultra_compact_bottom_min_height: f32,
    pub minimum_window_width: f32,
    pub minimum_window_height: f32,
    pub ultra_minimum_window_width: f32,
    pub ultra_minimum_window_height: f32,
}

impl Default for EditorDensityTokens {
    fn default() -> Self {
        Self::workbench_dense()
    }
}

impl EditorDensityTokens {
    pub const WORKBENCH_ROW_HEIGHT: f32 = 28.0;

    pub fn workbench_dense() -> Self {
        Self {
            gap_xsmall: 2.0,
            gap_tight: 3.0,
            gap_small: 4.0,
            gap_regular: 6.0,
            gap_medium: 8.0,
            gap_large: 12.0,
            drawer_padding: 12.0,
            panel_padding: 16.0,
            toolbar_action_width: 76.0,
            toolbar_wide_action_width: 96.0,
            ui_asset_action_min_width: 40.0,
            ui_asset_action_preferred_width: 52.0,
            ui_asset_action_max_width: 64.0,
            ui_asset_side_min_width: 128.0,
            ui_asset_side_preferred_width: 220.0,
            ui_asset_center_min_width: 256.0,
            ui_asset_center_preferred_width: 420.0,
            ui_asset_header_kind_min_width: 64.0,
            ui_asset_header_kind_preferred_width: 70.0,
            ui_asset_header_kind_max_width: 84.0,
            ui_asset_tool_min_width: 56.0,
            ui_asset_tool_preferred_width: 64.0,
            ui_asset_tool_max_width: 80.0,
            command_palette_min_width: 520.0,
            command_palette_preferred_width: 560.0,
            command_palette_max_width: 640.0,
            command_palette_min_height: 180.0,
            command_palette_preferred_height: 220.0,
            command_palette_max_height: 280.0,
            dialog_min_width: 420.0,
            dialog_preferred_width: 480.0,
            dialog_max_width: 560.0,
            dialog_min_height: 180.0,
            dialog_preferred_height: 220.0,
            dialog_max_height: 320.0,
            confirm_dialog_preferred_width: 460.0,
            confirm_dialog_max_width: 540.0,
            confirm_dialog_min_height: 174.0,
            confirm_dialog_preferred_height: 210.0,
            confirm_dialog_max_height: 300.0,
            notification_panel_min_width: 280.0,
            notification_panel_preferred_width: 320.0,
            notification_panel_max_width: 380.0,
            notification_panel_min_height: 160.0,
            notification_panel_preferred_height: 220.0,
            notification_panel_max_height: 320.0,
            caption_min_height: 18.0,
            caption_preferred_height: 20.0,
            caption_max_height: 22.0,
            label_min_height: 20.0,
            label_preferred_height: 22.0,
            label_max_height: 28.0,
            chip_min_width: 40.0,
            chip_preferred_width: 80.0,
            chip_max_width: 160.0,
            axis_value_field_min_width: 54.0,
            axis_value_field_preferred_width: 62.0,
            axis_value_field_max_width: 72.0,
            row_height: Self::WORKBENCH_ROW_HEIGHT,
            left_drawer_width: 332.0,
            right_drawer_width: 404.0,
            bottom_output_height: 228.0,
            breakpoint_ultra_width: 480.0,
            breakpoint_narrow_width: 640.0,
            breakpoint_wide_width: 1260.0,
            compact_side_width: 1100.0,
            ultra_compact_side_width: 760.0,
            compact_left_drawer_max_width: 340.0,
            compact_right_drawer_max_width: 220.0,
            compact_side_min_width: 196.0,
            minimum_document_width_fraction: 0.5,
            ultra_compact_left_drawer_max_width: 220.0,
            ultra_compact_right_drawer_max_width: 160.0,
            compact_bottom_available_height: 900.0,
            compact_bottom_max_height: 148.0,
            compact_bottom_max_available_fraction: 0.23,
            compact_bottom_min_height: 120.0,
            ultra_compact_bottom_available_height: 420.0,
            ultra_compact_bottom_max_height: 96.0,
            ultra_compact_bottom_max_available_fraction: 0.20,
            ultra_compact_bottom_min_height: 80.0,
            minimum_window_width: 640.0,
            minimum_window_height: 420.0,
            ultra_minimum_window_width: 420.0,
            ultra_minimum_window_height: 360.0,
        }
    }
}

impl EditorDensityTokens {
    pub(super) fn insert_density_cascade_tokens(&self, values: &mut BTreeMap<String, Value>) {
        for (name, value) in [
            ("editor.density.gap.xsmall", self.gap_xsmall),
            ("editor.density.gap.tight", self.gap_tight),
            ("editor.density.gap.small", self.gap_small),
            ("editor.density.gap.regular", self.gap_regular),
            ("editor.density.gap.medium", self.gap_medium),
            ("editor.density.gap.large", self.gap_large),
            ("editor.density.drawer_padding", self.drawer_padding),
            ("editor.density.panel_padding", self.panel_padding),
            (
                "editor.density.toolbar_action_width",
                self.toolbar_action_width,
            ),
            (
                "editor.density.toolbar_wide_action_width",
                self.toolbar_wide_action_width,
            ),
            (
                "editor.density.ui_asset_action.min_width",
                self.ui_asset_action_min_width,
            ),
            (
                "editor.density.ui_asset_action.preferred_width",
                self.ui_asset_action_preferred_width,
            ),
            (
                "editor.density.ui_asset_action.max_width",
                self.ui_asset_action_max_width,
            ),
            (
                "editor.density.ui_asset.side.min_width",
                self.ui_asset_side_min_width,
            ),
            (
                "editor.density.ui_asset.side.preferred_width",
                self.ui_asset_side_preferred_width,
            ),
            (
                "editor.density.ui_asset.center.min_width",
                self.ui_asset_center_min_width,
            ),
            (
                "editor.density.ui_asset.center.preferred_width",
                self.ui_asset_center_preferred_width,
            ),
            (
                "editor.density.ui_asset.header_kind.min_width",
                self.ui_asset_header_kind_min_width,
            ),
            (
                "editor.density.ui_asset.header_kind.preferred_width",
                self.ui_asset_header_kind_preferred_width,
            ),
            (
                "editor.density.ui_asset.header_kind.max_width",
                self.ui_asset_header_kind_max_width,
            ),
            (
                "editor.density.ui_asset.tool.min_width",
                self.ui_asset_tool_min_width,
            ),
            (
                "editor.density.ui_asset.tool.preferred_width",
                self.ui_asset_tool_preferred_width,
            ),
            (
                "editor.density.ui_asset.tool.max_width",
                self.ui_asset_tool_max_width,
            ),
            (
                "editor.density.command_palette.min_width",
                self.command_palette_min_width,
            ),
            (
                "editor.density.command_palette.preferred_width",
                self.command_palette_preferred_width,
            ),
            (
                "editor.density.command_palette.max_width",
                self.command_palette_max_width,
            ),
            (
                "editor.density.command_palette.min_height",
                self.command_palette_min_height,
            ),
            (
                "editor.density.command_palette.preferred_height",
                self.command_palette_preferred_height,
            ),
            (
                "editor.density.command_palette.max_height",
                self.command_palette_max_height,
            ),
            ("editor.density.dialog.min_width", self.dialog_min_width),
            (
                "editor.density.dialog.preferred_width",
                self.dialog_preferred_width,
            ),
            ("editor.density.dialog.max_width", self.dialog_max_width),
            ("editor.density.dialog.min_height", self.dialog_min_height),
            (
                "editor.density.dialog.preferred_height",
                self.dialog_preferred_height,
            ),
            ("editor.density.dialog.max_height", self.dialog_max_height),
            (
                "editor.density.confirm_dialog.preferred_width",
                self.confirm_dialog_preferred_width,
            ),
            (
                "editor.density.confirm_dialog.max_width",
                self.confirm_dialog_max_width,
            ),
            (
                "editor.density.confirm_dialog.min_height",
                self.confirm_dialog_min_height,
            ),
            (
                "editor.density.confirm_dialog.preferred_height",
                self.confirm_dialog_preferred_height,
            ),
            (
                "editor.density.confirm_dialog.max_height",
                self.confirm_dialog_max_height,
            ),
            (
                "editor.density.notification_panel.min_width",
                self.notification_panel_min_width,
            ),
            (
                "editor.density.notification_panel.preferred_width",
                self.notification_panel_preferred_width,
            ),
            (
                "editor.density.notification_panel.max_width",
                self.notification_panel_max_width,
            ),
            (
                "editor.density.notification_panel.min_height",
                self.notification_panel_min_height,
            ),
            (
                "editor.density.notification_panel.preferred_height",
                self.notification_panel_preferred_height,
            ),
            (
                "editor.density.notification_panel.max_height",
                self.notification_panel_max_height,
            ),
            ("editor.density.caption.min_height", self.caption_min_height),
            (
                "editor.density.caption.preferred_height",
                self.caption_preferred_height,
            ),
            ("editor.density.caption.max_height", self.caption_max_height),
            ("editor.density.label.min_height", self.label_min_height),
            (
                "editor.density.label.preferred_height",
                self.label_preferred_height,
            ),
            ("editor.density.label.max_height", self.label_max_height),
            ("editor.density.chip.min_width", self.chip_min_width),
            (
                "editor.density.chip.preferred_width",
                self.chip_preferred_width,
            ),
            ("editor.density.chip.max_width", self.chip_max_width),
            (
                "editor.density.axis_value_field.min_width",
                self.axis_value_field_min_width,
            ),
            (
                "editor.density.axis_value_field.preferred_width",
                self.axis_value_field_preferred_width,
            ),
            (
                "editor.density.axis_value_field.max_width",
                self.axis_value_field_max_width,
            ),
            ("editor.density.row_height", self.row_height),
            ("editor.density.left_drawer_width", self.left_drawer_width),
            ("editor.density.right_drawer_width", self.right_drawer_width),
            (
                "editor.density.bottom_output_height",
                self.bottom_output_height,
            ),
            (
                "editor.density.breakpoint_ultra_width",
                self.breakpoint_ultra_width,
            ),
            (
                "editor.density.breakpoint_narrow_width",
                self.breakpoint_narrow_width,
            ),
            (
                "editor.density.breakpoint_wide_width",
                self.breakpoint_wide_width,
            ),
            ("editor.density.compact_side_width", self.compact_side_width),
            (
                "editor.density.ultra_compact_side_width",
                self.ultra_compact_side_width,
            ),
            (
                "editor.density.compact_left_drawer_max_width",
                self.compact_left_drawer_max_width,
            ),
            (
                "editor.density.compact_right_drawer_max_width",
                self.compact_right_drawer_max_width,
            ),
            (
                "editor.density.compact_side_min_width",
                self.compact_side_min_width,
            ),
            (
                "editor.density.minimum_document_width_fraction",
                self.minimum_document_width_fraction,
            ),
            (
                "editor.density.ultra_compact_left_drawer_max_width",
                self.ultra_compact_left_drawer_max_width,
            ),
            (
                "editor.density.ultra_compact_right_drawer_max_width",
                self.ultra_compact_right_drawer_max_width,
            ),
            (
                "editor.density.compact_bottom_available_height",
                self.compact_bottom_available_height,
            ),
            (
                "editor.density.compact_bottom_max_height",
                self.compact_bottom_max_height,
            ),
            (
                "editor.density.compact_bottom_max_available_fraction",
                self.compact_bottom_max_available_fraction,
            ),
            (
                "editor.density.compact_bottom_min_height",
                self.compact_bottom_min_height,
            ),
            (
                "editor.density.ultra_compact_bottom_available_height",
                self.ultra_compact_bottom_available_height,
            ),
            (
                "editor.density.ultra_compact_bottom_max_height",
                self.ultra_compact_bottom_max_height,
            ),
            (
                "editor.density.ultra_compact_bottom_max_available_fraction",
                self.ultra_compact_bottom_max_available_fraction,
            ),
            (
                "editor.density.ultra_compact_bottom_min_height",
                self.ultra_compact_bottom_min_height,
            ),
            (
                "editor.density.minimum_window_width",
                self.minimum_window_width,
            ),
            (
                "editor.density.minimum_window_height",
                self.minimum_window_height,
            ),
            (
                "editor.density.ultra_minimum_window_width",
                self.ultra_minimum_window_width,
            ),
            (
                "editor.density.ultra_minimum_window_height",
                self.ultra_minimum_window_height,
            ),
        ] {
            insert_float_token(values, name, value);
        }
    }
}
