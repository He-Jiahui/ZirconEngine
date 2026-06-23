use serde::{Deserialize, Serialize};

use super::style::{
    UiPainterFamily, UiPainterResolvedState, UiPainterState, UiPainterStyleSelector, UiRgbaColor,
    UiThemeControlSizes, UiThemeDocument, UiThemeElevation, UiThemePalette, UiThemeShape,
};

pub const EDITOR_WORKBENCH_TOKENS_ID: &str = "zircon.editor.workbench";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorDesignTokens {
    pub id: String,
    pub palette: EditorPaletteTokens,
    pub controls: EditorControlTokens,
    pub density: EditorDensityTokens,
    pub state_roles: EditorStateRoleTokens,
}

impl Default for EditorDesignTokens {
    fn default() -> Self {
        Self::workbench_dark()
    }
}

impl EditorDesignTokens {
    pub fn workbench_dark() -> Self {
        Self {
            id: EDITOR_WORKBENCH_TOKENS_ID.to_string(),
            palette: EditorPaletteTokens::workbench_dark(),
            controls: EditorControlTokens::workbench_dense(),
            density: EditorDensityTokens::workbench_dense(),
            state_roles: EditorStateRoleTokens::workbench_dark(),
        }
    }

    pub fn color_for_state(&self, state: UiPainterResolvedState) -> UiRgbaColor {
        self.state_roles
            .role_for_state(state)
            .resolve(&self.palette)
    }

    pub fn density_value_for_token_name(&self, token_name: &str) -> Option<f32> {
        match token_name {
            "editor.density.gap.small" => Some(self.density.gap_small),
            "editor.density.gap.medium" => Some(self.density.gap_medium),
            "editor.density.gap.large" => Some(self.density.gap_large),
            "editor.density.drawer_padding" => Some(self.density.drawer_padding),
            "editor.density.panel_padding" => Some(self.density.panel_padding),
            "editor.density.row_height" => Some(self.density.row_height),
            "--left-drawer-width" => Some(self.density.left_drawer_width),
            "--right-drawer-width" => Some(self.density.right_drawer_width),
            "--bottom-output-height" => Some(self.density.bottom_output_height),
            _ => None,
        }
    }

    pub fn resolve_painter_style(
        &self,
        state: UiPainterState,
        family: UiPainterFamily,
    ) -> EditorResolvedPainterStyle {
        let resolved_state = UiPainterStyleSelector::resolved_state_for_family(state, family);
        EditorResolvedPainterStyle {
            family,
            state: resolved_state,
            background_color: self.background_color_for_resolved_state(resolved_state),
            foreground_color: self.foreground_color_for_resolved_state(resolved_state),
            border_color: self.border_color_for_resolved_state(resolved_state),
            border_width: self.controls.border_width,
            corner_radius: self.corner_radius_for_family(family),
            control_height: self.control_height_for_family(family),
        }
    }

    pub fn to_theme_document(&self) -> UiThemeDocument {
        UiThemeDocument {
            id: self.id.clone(),
            palette: UiThemePalette {
                surface: self.palette.surface,
                text_primary: self.palette.text_primary,
                text_secondary: self.palette.text_secondary,
                text_disabled: self.palette.text_disabled,
                accent: self.palette.accent,
                success: self.palette.success,
                info: self.palette.info,
                warning: self.palette.warning,
                error: self.palette.error,
                separator: self.palette.border,
            },
            typography: UiThemeDocument::dark().typography,
            shape: UiThemeShape {
                radius_small: self.controls.small_radius,
                radius_medium: self.controls.control_radius,
                radius_large: self.controls.large_radius,
                radius_panel: self.controls.panel_radius,
            },
            spacing: vec![
                0.0,
                self.density.gap_small,
                self.density.gap_medium,
                self.density.gap_large,
                self.density.drawer_padding,
                self.density.panel_padding,
            ],
            control_sizes: UiThemeControlSizes {
                default_height: self.controls.default_height,
                compact_height: self.controls.compact_height,
                dense_height: self.controls.dense_height,
            },
            elevation: vec![UiThemeElevation::level(0, 0.0, 0.0, 0.0, 0.0)],
        }
    }

    fn background_color_for_resolved_state(&self, state: UiPainterResolvedState) -> UiRgbaColor {
        match state {
            UiPainterResolvedState::Disabled => self.palette.surface[1],
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Loading => self.palette.accent,
            UiPainterResolvedState::Pressed => self.palette.surface[3],
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => self.palette.surface[2],
            UiPainterResolvedState::Normal => self.palette.surface[1],
        }
    }

    fn foreground_color_for_resolved_state(&self, state: UiPainterResolvedState) -> UiRgbaColor {
        match state {
            UiPainterResolvedState::Disabled => self.palette.text_disabled,
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Loading => self.palette.surface[0],
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Normal => self.palette.text_primary,
        }
    }

    fn border_color_for_resolved_state(&self, state: UiPainterResolvedState) -> UiRgbaColor {
        match state {
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Loading => self.palette.accent,
            UiPainterResolvedState::Disabled
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Normal => self.palette.border,
        }
    }

    fn corner_radius_for_family(&self, family: UiPainterFamily) -> f32 {
        match family {
            UiPainterFamily::IconButton
            | UiPainterFamily::Checkbox
            | UiPainterFamily::Radio
            | UiPainterFamily::Toggle => self.controls.small_radius,
            UiPainterFamily::Chrome
            | UiPainterFamily::Alert
            | UiPainterFamily::Tooltip
            | UiPainterFamily::Toast => self.controls.panel_radius,
            UiPainterFamily::Generic
            | UiPainterFamily::Button
            | UiPainterFamily::Slider
            | UiPainterFamily::Dropdown
            | UiPainterFamily::PopupRow
            | UiPainterFamily::TextField
            | UiPainterFamily::ListRow
            | UiPainterFamily::TreeRow
            | UiPainterFamily::TableRow
            | UiPainterFamily::Tab => self.controls.control_radius,
        }
    }

    fn control_height_for_family(&self, family: UiPainterFamily) -> f32 {
        match family {
            UiPainterFamily::ListRow
            | UiPainterFamily::TreeRow
            | UiPainterFamily::TableRow
            | UiPainterFamily::PopupRow
            | UiPainterFamily::Tab => self.density.row_height,
            UiPainterFamily::IconButton
            | UiPainterFamily::Checkbox
            | UiPainterFamily::Radio
            | UiPainterFamily::Toggle
            | UiPainterFamily::Slider => self.controls.dense_height,
            UiPainterFamily::Generic
            | UiPainterFamily::Button
            | UiPainterFamily::Dropdown
            | UiPainterFamily::Alert
            | UiPainterFamily::Tooltip
            | UiPainterFamily::TextField
            | UiPainterFamily::Toast
            | UiPainterFamily::Chrome => self.controls.default_height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorResolvedPainterStyle {
    pub family: UiPainterFamily,
    pub state: UiPainterResolvedState,
    pub background_color: UiRgbaColor,
    pub foreground_color: UiRgbaColor,
    pub border_color: UiRgbaColor,
    pub border_width: f32,
    pub corner_radius: f32,
    pub control_height: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorPaletteTokens {
    pub surface: [UiRgbaColor; 4],
    pub accent: UiRgbaColor,
    pub border: UiRgbaColor,
    pub text_primary: UiRgbaColor,
    pub text_secondary: UiRgbaColor,
    pub text_disabled: UiRgbaColor,
    pub success: UiRgbaColor,
    pub info: UiRgbaColor,
    pub warning: UiRgbaColor,
    pub error: UiRgbaColor,
}

impl Default for EditorPaletteTokens {
    fn default() -> Self {
        Self::workbench_dark()
    }
}

impl EditorPaletteTokens {
    pub fn workbench_dark() -> Self {
        Self {
            surface: [
                UiRgbaColor::from_u8(17, 20, 22, 255),
                UiRgbaColor::from_u8(23, 26, 29, 255),
                UiRgbaColor::from_u8(27, 31, 35, 255),
                UiRgbaColor::from_u8(37, 43, 49, 255),
            ],
            accent: UiRgbaColor::from_u8(60, 199, 214, 255),
            border: UiRgbaColor::from_u8(57, 65, 71, 255),
            text_primary: UiRgbaColor::from_u8(232, 236, 238, 255),
            text_secondary: UiRgbaColor::from_u8(164, 174, 180, 255),
            text_disabled: UiRgbaColor::from_u8(101, 111, 118, 255),
            success: UiRgbaColor::from_u8(85, 190, 120, 255),
            info: UiRgbaColor::from_u8(95, 170, 230, 255),
            warning: UiRgbaColor::from_u8(220, 172, 80, 255),
            error: UiRgbaColor::from_u8(235, 96, 92, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorControlTokens {
    pub default_height: f32,
    pub compact_height: f32,
    pub dense_height: f32,
    pub small_radius: f32,
    pub control_radius: f32,
    pub large_radius: f32,
    pub panel_radius: f32,
    pub border_width: f32,
}

impl Default for EditorControlTokens {
    fn default() -> Self {
        Self::workbench_dense()
    }
}

impl EditorControlTokens {
    pub fn workbench_dense() -> Self {
        Self {
            default_height: 32.0,
            compact_height: 32.0,
            dense_height: 28.0,
            small_radius: 4.0,
            control_radius: 5.0,
            large_radius: 8.0,
            panel_radius: 8.0,
            border_width: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorDensityTokens {
    pub gap_small: f32,
    pub gap_medium: f32,
    pub gap_large: f32,
    pub drawer_padding: f32,
    pub panel_padding: f32,
    pub row_height: f32,
    pub left_drawer_width: f32,
    pub right_drawer_width: f32,
    pub bottom_output_height: f32,
}

impl Default for EditorDensityTokens {
    fn default() -> Self {
        Self::workbench_dense()
    }
}

impl EditorDensityTokens {
    pub fn workbench_dense() -> Self {
        Self {
            gap_small: 4.0,
            gap_medium: 8.0,
            gap_large: 12.0,
            drawer_padding: 12.0,
            panel_padding: 16.0,
            row_height: 28.0,
            left_drawer_width: 332.0,
            right_drawer_width: 404.0,
            bottom_output_height: 228.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorStateColorRole {
    Surface0,
    Surface1,
    Surface2,
    Surface3,
    Accent,
    Border,
    TextPrimary,
    TextSecondary,
    TextDisabled,
}

impl EditorStateColorRole {
    pub fn resolve(self, palette: &EditorPaletteTokens) -> UiRgbaColor {
        match self {
            Self::Surface0 => palette.surface[0],
            Self::Surface1 => palette.surface[1],
            Self::Surface2 => palette.surface[2],
            Self::Surface3 => palette.surface[3],
            Self::Accent => palette.accent,
            Self::Border => palette.border,
            Self::TextPrimary => palette.text_primary,
            Self::TextSecondary => palette.text_secondary,
            Self::TextDisabled => palette.text_disabled,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorStateRoleTokens {
    pub default: EditorStateColorRole,
    pub hovered: EditorStateColorRole,
    pub pressed: EditorStateColorRole,
    pub selected: EditorStateColorRole,
    pub focused: EditorStateColorRole,
    pub disabled: EditorStateColorRole,
    pub loading: EditorStateColorRole,
}

impl Default for EditorStateRoleTokens {
    fn default() -> Self {
        Self::workbench_dark()
    }
}

impl EditorStateRoleTokens {
    pub fn workbench_dark() -> Self {
        Self {
            default: EditorStateColorRole::Surface1,
            hovered: EditorStateColorRole::Surface2,
            pressed: EditorStateColorRole::Surface3,
            selected: EditorStateColorRole::Accent,
            focused: EditorStateColorRole::Accent,
            disabled: EditorStateColorRole::TextDisabled,
            loading: EditorStateColorRole::Accent,
        }
    }

    pub fn role_for_state(self, state: UiPainterResolvedState) -> EditorStateColorRole {
        match state {
            UiPainterResolvedState::Disabled => self.disabled,
            UiPainterResolvedState::Loading => self.loading,
            UiPainterResolvedState::Pressed => self.pressed,
            UiPainterResolvedState::Focused => self.focused,
            UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open => self.selected,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => self.hovered,
            UiPainterResolvedState::Normal => self.default,
        }
    }
}
