use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::style::{
    UiPainterFamily, UiPainterResolvedState, UiPainterState, UiPainterStyleSelector, UiRgbaColor,
    UiThemeControlSizes, UiThemeDocument, UiThemeElevation, UiThemePalette, UiThemeShape,
    UiThemeTypographyVariant,
};

mod cascade_registry;
mod state_roles;

use cascade_registry::{
    insert_color_token, insert_float_token, insert_integer_token, insert_string_token,
};

pub use state_roles::{EditorStateColorRole, EditorStateRoleTokens};

pub const EDITOR_WORKBENCH_TOKENS_ID: &str = "zircon.editor.workbench";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorDesignTokens {
    pub id: String,
    pub palette: EditorPaletteTokens,
    pub typography: EditorTypographyTokens,
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
            typography: EditorTypographyTokens::workbench_default(),
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

    /// Returns the canonical editor tokens plus their cascade custom-property aliases.
    ///
    /// Canonical entries are the only stored values. Each `--editor-*` entry resolves
    /// through its matching `$editor.*` token so inline and cascade consumers cannot
    /// drift into separate token-to-value maps.
    pub fn cascade_token_values(&self) -> BTreeMap<String, Value> {
        cascade_registry::cascade_token_values(self)
    }

    pub fn density_value_for_token_name(&self, token_name: &str) -> Option<f32> {
        cascade_registry::numeric_token_value(&self.cascade_token_values(), token_name)
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
            typography: self.typography.to_theme_variants(),
            shape: UiThemeShape {
                radius_small: self.controls.small_radius,
                radius_medium: self.controls.control_radius,
                radius_large: self.controls.large_radius,
                radius_panel: self.controls.panel_radius,
            },
            spacing: vec![
                0.0,
                self.density.gap_xsmall,
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
            | UiPainterResolvedState::Open => self.palette.surface_selected,
            UiPainterResolvedState::Loading => self.palette.accent,
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
            | UiPainterResolvedState::Open => self.palette.text_primary,
            UiPainterResolvedState::Loading => self.palette.surface[0],
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
pub struct EditorTypographyTokens {
    pub ui_family: String,
    pub ui_strong_family: String,
    pub code_family: String,
    pub utility_tab_text_role: EditorUtilityTabTextRole,
    pub font_smoothing: EditorFontSmoothing,
    /// Body font size in 96-DPI logical pixels, after converting authored points.
    pub body_size: f32,
    /// Caption font size in 96-DPI logical pixels, after converting authored points.
    pub caption_size: f32,
    /// Viewport overlay font size in 96-DPI logical pixels, after converting authored points.
    pub overlay_size: f32,
    /// Title font size in 96-DPI logical pixels, after converting authored points.
    pub title_size: f32,
    pub body_weight: u16,
    pub medium_weight: u16,
    pub strong_weight: u16,
    pub emphasis_weight: u16,
    pub code_weight: u16,
    pub line_height: f32,
}

impl Default for EditorTypographyTokens {
    fn default() -> Self {
        Self::workbench_default()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorFontSmoothing {
    #[default]
    Grayscale,
    Subpixel,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorUtilityTabTextRole {
    #[default]
    Ui,
    Code,
}

impl EditorTypographyTokens {
    pub const DEFAULT_UI_FAMILY: &'static str = "system-ui";
    pub const DEFAULT_CODE_FAMILY: &'static str = "monospace";
    pub const WORKBENCH_BODY_SIZE: f32 = Self::points_to_logical_pixels(10.0);
    pub const WORKBENCH_CAPTION_SIZE: f32 = Self::points_to_logical_pixels(8.0);
    pub const WORKBENCH_OVERLAY_SIZE: f32 = Self::points_to_logical_pixels(9.0);
    pub const WORKBENCH_TITLE_SIZE: f32 = Self::points_to_logical_pixels(14.0);
    pub const WORKBENCH_LINE_HEIGHT_RATIO: f32 = 1.2;

    /// Matches Slate's point-to-unit conversion at its 96-DPI render baseline.
    pub const fn points_to_logical_pixels(point_size: f32) -> f32 {
        point_size * (96.0 / 72.0)
    }

    pub fn workbench_default() -> Self {
        Self {
            ui_family: Self::DEFAULT_UI_FAMILY.to_string(),
            ui_strong_family: Self::DEFAULT_UI_FAMILY.to_string(),
            code_family: Self::DEFAULT_CODE_FAMILY.to_string(),
            utility_tab_text_role: EditorUtilityTabTextRole::Ui,
            font_smoothing: EditorFontSmoothing::Grayscale,
            body_size: Self::WORKBENCH_BODY_SIZE,
            caption_size: Self::WORKBENCH_CAPTION_SIZE,
            overlay_size: Self::WORKBENCH_OVERLAY_SIZE,
            title_size: Self::WORKBENCH_TITLE_SIZE,
            body_weight: 400,
            medium_weight: 500,
            strong_weight: 600,
            emphasis_weight: 700,
            code_weight: 400,
            line_height: Self::WORKBENCH_LINE_HEIGHT_RATIO,
        }
    }

    pub fn to_theme_variants(&self) -> Vec<UiThemeTypographyVariant> {
        vec![
            UiThemeTypographyVariant {
                variant: "body".to_string(),
                family: self.ui_family.clone(),
                size: self.body_size,
                weight: self.body_weight,
                line_height: self.line_height,
            },
            UiThemeTypographyVariant {
                variant: "caption".to_string(),
                family: self.ui_family.clone(),
                size: self.caption_size,
                weight: self.body_weight,
                line_height: self.line_height,
            },
            UiThemeTypographyVariant {
                variant: "overlay".to_string(),
                family: self.ui_strong_family.clone(),
                size: self.overlay_size,
                weight: self.emphasis_weight,
                line_height: self.line_height,
            },
            UiThemeTypographyVariant {
                variant: "title".to_string(),
                family: self.ui_strong_family.clone(),
                size: self.title_size,
                weight: self.strong_weight,
                line_height: self.line_height,
            },
            UiThemeTypographyVariant {
                variant: "code".to_string(),
                family: self.code_family.clone(),
                size: self.body_size,
                weight: self.code_weight,
                line_height: self.line_height,
            },
        ]
    }

    fn insert_cascade_tokens(&self, values: &mut BTreeMap<String, Value>) {
        insert_string_token(values, "editor.typography.ui.family", &self.ui_family);
        insert_string_token(
            values,
            "editor.typography.ui.strong.family",
            &self.ui_strong_family,
        );
        insert_string_token(values, "editor.typography.code.family", &self.code_family);
        insert_string_token(
            values,
            "editor.typography.utility_tab.text_role",
            self.utility_tab_text_role.token_value(),
        );
        insert_string_token(
            values,
            "editor.typography.font_smoothing",
            self.font_smoothing.token_value(),
        );
        insert_float_token(values, "editor.typography.body.size", self.body_size);
        insert_float_token(values, "editor.typography.caption.size", self.caption_size);
        insert_float_token(values, "editor.typography.overlay.size", self.overlay_size);
        insert_float_token(values, "editor.typography.title.size", self.title_size);
        insert_integer_token(values, "editor.typography.body.weight", self.body_weight);
        insert_integer_token(
            values,
            "editor.typography.medium.weight",
            self.medium_weight,
        );
        insert_integer_token(
            values,
            "editor.typography.strong.weight",
            self.strong_weight,
        );
        insert_integer_token(
            values,
            "editor.typography.emphasis.weight",
            self.emphasis_weight,
        );
        insert_integer_token(values, "editor.typography.code.weight", self.code_weight);
        insert_float_token(values, "editor.typography.line_height", self.line_height);
    }
}

impl EditorFontSmoothing {
    fn token_value(self) -> &'static str {
        match self {
            Self::Grayscale => "grayscale",
            Self::Subpixel => "subpixel",
        }
    }
}

impl EditorUtilityTabTextRole {
    fn token_value(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Code => "code",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorPaletteTokens {
    pub surface: [UiRgbaColor; 4],
    #[serde(default = "default_palette_surface_recessed")]
    pub surface_recessed: UiRgbaColor,
    #[serde(default = "default_palette_surface_hover")]
    pub surface_hover: UiRgbaColor,
    #[serde(default = "default_palette_surface_selected")]
    pub surface_selected: UiRgbaColor,
    #[serde(default = "default_palette_surface_disabled")]
    pub surface_disabled: UiRgbaColor,
    pub accent: UiRgbaColor,
    #[serde(default = "default_palette_accent_soft")]
    pub accent_soft: UiRgbaColor,
    pub border: UiRgbaColor,
    #[serde(default = "default_palette_border_disabled")]
    pub border_disabled: UiRgbaColor,
    #[serde(default = "default_palette_separator_strong")]
    pub separator_strong: UiRgbaColor,
    #[serde(default = "default_palette_separator_soft")]
    pub separator_soft: UiRgbaColor,
    pub text_primary: UiRgbaColor,
    pub text_secondary: UiRgbaColor,
    pub text_disabled: UiRgbaColor,
    pub success: UiRgbaColor,
    #[serde(default = "default_palette_success_container")]
    pub success_container: UiRgbaColor,
    pub info: UiRgbaColor,
    #[serde(default = "default_palette_info_container")]
    pub info_container: UiRgbaColor,
    pub warning: UiRgbaColor,
    #[serde(default = "default_palette_warning_container")]
    pub warning_container: UiRgbaColor,
    pub error: UiRgbaColor,
    #[serde(default = "default_palette_error_container")]
    pub error_container: UiRgbaColor,
    #[serde(default = "default_palette_popup")]
    pub popup: UiRgbaColor,
    #[serde(default = "default_palette_track")]
    pub track: UiRgbaColor,
    #[serde(default = "default_palette_focus_ring")]
    pub focus_ring: UiRgbaColor,
    #[serde(default = "default_palette_shadow")]
    pub shadow: UiRgbaColor,
}

impl Default for EditorPaletteTokens {
    fn default() -> Self {
        Self::workbench_dark()
    }
}

impl EditorPaletteTokens {
    pub const WORKBENCH_SURFACE: [[u8; 4]; 4] = [
        [17, 20, 22, 255],
        [23, 26, 29, 255],
        [27, 31, 35, 255],
        [37, 43, 49, 255],
    ];
    pub const WORKBENCH_SURFACE_RECESSED: [u8; 4] = [15, 19, 22, 255];
    pub const WORKBENCH_SURFACE_HOVER: [u8; 4] = [42, 48, 54, 255];
    pub const WORKBENCH_SURFACE_SELECTED: [u8; 4] = [23, 57, 66, 255];
    pub const WORKBENCH_SURFACE_DISABLED: [u8; 4] = [34, 39, 43, 255];
    pub const WORKBENCH_ACCENT: [u8; 4] = [60, 199, 214, 255];
    pub const WORKBENCH_ACCENT_SOFT: [u8; 4] = [23, 67, 77, 255];
    pub const WORKBENCH_BORDER: [u8; 4] = [50, 58, 65, 255];
    pub const WORKBENCH_BORDER_DISABLED: [u8; 4] = [44, 50, 55, 255];
    pub const WORKBENCH_SEPARATOR_STRONG: [u8; 4] = [65, 75, 84, 255];
    pub const WORKBENCH_SEPARATOR_SOFT: [u8; 4] = [38, 45, 51, 255];
    pub const WORKBENCH_TEXT_PRIMARY: [u8; 4] = [232, 236, 238, 255];
    pub const WORKBENCH_TEXT_SECONDARY: [u8; 4] = [164, 174, 180, 255];
    pub const WORKBENCH_TEXT_DISABLED: [u8; 4] = [101, 111, 118, 255];
    pub const WORKBENCH_SUCCESS: [u8; 4] = [85, 190, 120, 255];
    pub const WORKBENCH_SUCCESS_CONTAINER: [u8; 4] = [29, 71, 47, 255];
    pub const WORKBENCH_INFO: [u8; 4] = [95, 170, 230, 255];
    pub const WORKBENCH_INFO_CONTAINER: [u8; 4] = [24, 57, 91, 255];
    pub const WORKBENCH_WARNING: [u8; 4] = [220, 172, 80, 255];
    pub const WORKBENCH_WARNING_CONTAINER: [u8; 4] = [70, 49, 18, 255];
    pub const WORKBENCH_ERROR: [u8; 4] = [235, 96, 92, 255];
    pub const WORKBENCH_ERROR_CONTAINER: [u8; 4] = [76, 36, 39, 255];
    pub const WORKBENCH_POPUP: [u8; 4] = [20, 22, 24, 255];
    pub const WORKBENCH_TRACK: [u8; 4] = [44, 51, 57, 255];
    pub const WORKBENCH_FOCUS_RING: [u8; 4] = [56, 189, 208, 255];
    pub const WORKBENCH_SHADOW: [u8; 4] = [0, 0, 0, 115];

    pub fn workbench_dark() -> Self {
        Self {
            surface: Self::WORKBENCH_SURFACE.map(Self::rgba),
            surface_recessed: Self::rgba(Self::WORKBENCH_SURFACE_RECESSED),
            surface_hover: Self::rgba(Self::WORKBENCH_SURFACE_HOVER),
            surface_selected: Self::rgba(Self::WORKBENCH_SURFACE_SELECTED),
            surface_disabled: Self::rgba(Self::WORKBENCH_SURFACE_DISABLED),
            accent: Self::rgba(Self::WORKBENCH_ACCENT),
            accent_soft: Self::rgba(Self::WORKBENCH_ACCENT_SOFT),
            border: Self::rgba(Self::WORKBENCH_BORDER),
            border_disabled: Self::rgba(Self::WORKBENCH_BORDER_DISABLED),
            separator_strong: Self::rgba(Self::WORKBENCH_SEPARATOR_STRONG),
            separator_soft: Self::rgba(Self::WORKBENCH_SEPARATOR_SOFT),
            text_primary: Self::rgba(Self::WORKBENCH_TEXT_PRIMARY),
            text_secondary: Self::rgba(Self::WORKBENCH_TEXT_SECONDARY),
            text_disabled: Self::rgba(Self::WORKBENCH_TEXT_DISABLED),
            success: Self::rgba(Self::WORKBENCH_SUCCESS),
            success_container: Self::rgba(Self::WORKBENCH_SUCCESS_CONTAINER),
            info: Self::rgba(Self::WORKBENCH_INFO),
            info_container: Self::rgba(Self::WORKBENCH_INFO_CONTAINER),
            warning: Self::rgba(Self::WORKBENCH_WARNING),
            warning_container: Self::rgba(Self::WORKBENCH_WARNING_CONTAINER),
            error: Self::rgba(Self::WORKBENCH_ERROR),
            error_container: Self::rgba(Self::WORKBENCH_ERROR_CONTAINER),
            popup: Self::rgba(Self::WORKBENCH_POPUP),
            track: Self::rgba(Self::WORKBENCH_TRACK),
            focus_ring: Self::rgba(Self::WORKBENCH_FOCUS_RING),
            shadow: Self::rgba(Self::WORKBENCH_SHADOW),
        }
    }

    fn rgba(bytes: [u8; 4]) -> UiRgbaColor {
        UiRgbaColor::from_u8(bytes[0], bytes[1], bytes[2], bytes[3])
    }

    fn insert_cascade_tokens(&self, values: &mut BTreeMap<String, Value>) {
        for (index, color) in self.surface.iter().copied().enumerate() {
            insert_color_token(values, &format!("editor.surface.{index}"), color);
        }
        insert_color_token(values, "editor.surface.recessed", self.surface_recessed);
        insert_color_token(values, "editor.surface.hover", self.surface_hover);
        insert_color_token(values, "editor.surface.selected", self.surface_selected);
        insert_color_token(values, "editor.surface.disabled", self.surface_disabled);
        insert_color_token(values, "editor.accent", self.accent);
        insert_color_token(values, "editor.accent.soft", self.accent_soft);
        insert_color_token(values, "editor.border", self.border);
        insert_color_token(values, "editor.border.disabled", self.border_disabled);
        insert_color_token(values, "editor.separator.strong", self.separator_strong);
        insert_color_token(values, "editor.separator.soft", self.separator_soft);
        insert_color_token(values, "editor.text.primary", self.text_primary);
        insert_color_token(values, "editor.text.secondary", self.text_secondary);
        insert_color_token(values, "editor.text.disabled", self.text_disabled);
        insert_color_token(values, "editor.semantic.success", self.success);
        insert_color_token(
            values,
            "editor.semantic.success.container",
            self.success_container,
        );
        insert_color_token(values, "editor.semantic.info", self.info);
        insert_color_token(
            values,
            "editor.semantic.info.container",
            self.info_container,
        );
        insert_color_token(values, "editor.semantic.warning", self.warning);
        insert_color_token(
            values,
            "editor.semantic.warning.container",
            self.warning_container,
        );
        insert_color_token(values, "editor.semantic.error", self.error);
        insert_color_token(
            values,
            "editor.semantic.error.container",
            self.error_container,
        );
        insert_color_token(values, "editor.popup", self.popup);
        insert_color_token(values, "editor.track", self.track);
        insert_color_token(values, "editor.focus.ring", self.focus_ring);
        insert_color_token(values, "editor.shadow", self.shadow);
    }
}

fn default_palette_surface_recessed() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SURFACE_RECESSED)
}

fn default_palette_surface_hover() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SURFACE_HOVER)
}

fn default_palette_surface_selected() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SURFACE_SELECTED)
}

fn default_palette_surface_disabled() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SURFACE_DISABLED)
}

fn default_palette_accent_soft() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_ACCENT_SOFT)
}

fn default_palette_border_disabled() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_BORDER_DISABLED)
}

fn default_palette_separator_strong() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SEPARATOR_STRONG)
}

fn default_palette_separator_soft() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SEPARATOR_SOFT)
}

fn default_palette_success_container() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SUCCESS_CONTAINER)
}

fn default_palette_info_container() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_INFO_CONTAINER)
}

fn default_palette_warning_container() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_WARNING_CONTAINER)
}

fn default_palette_error_container() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_ERROR_CONTAINER)
}

fn default_palette_popup() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_POPUP)
}

fn default_palette_track() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_TRACK)
}

fn default_palette_focus_ring() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_FOCUS_RING)
}

fn default_palette_shadow() -> UiRgbaColor {
    EditorPaletteTokens::rgba(EditorPaletteTokens::WORKBENCH_SHADOW)
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorControlTokens {
    pub large_height: f32,
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
            large_height: 48.0,
            default_height: 32.0,
            compact_height: 30.0,
            dense_height: 28.0,
            small_radius: 4.0,
            control_radius: 5.0,
            large_radius: 8.0,
            panel_radius: 8.0,
            border_width: 1.0,
        }
    }

    fn insert_cascade_tokens(&self, values: &mut BTreeMap<String, Value>) {
        insert_float_token(values, "editor.control.height.large", self.large_height);
        insert_float_token(values, "editor.control.height.default", self.default_height);
        insert_float_token(values, "editor.control.height.compact", self.compact_height);
        insert_float_token(values, "editor.control.height.dense", self.dense_height);
        insert_float_token(values, "editor.control.radius.small", self.small_radius);
        insert_float_token(values, "editor.control.radius.control", self.control_radius);
        insert_float_token(values, "editor.control.radius.large", self.large_radius);
        insert_float_token(values, "editor.control.radius.panel", self.panel_radius);
        insert_float_token(values, "editor.control.border_width", self.border_width);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorDensityTokens {
    pub gap_xsmall: f32,
    pub gap_small: f32,
    pub gap_medium: f32,
    pub gap_large: f32,
    pub drawer_padding: f32,
    pub panel_padding: f32,
    pub activity_rail_width: f32,
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
            gap_small: 4.0,
            gap_medium: 8.0,
            gap_large: 12.0,
            drawer_padding: 12.0,
            panel_padding: 16.0,
            activity_rail_width: 72.0,
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

    fn insert_cascade_tokens(&self, values: &mut BTreeMap<String, Value>) {
        insert_float_token(values, "editor.density.gap.xsmall", self.gap_xsmall);
        insert_float_token(values, "editor.density.gap.small", self.gap_small);
        insert_float_token(values, "editor.density.gap.medium", self.gap_medium);
        insert_float_token(values, "editor.density.gap.large", self.gap_large);
        insert_float_token(values, "editor.density.drawer_padding", self.drawer_padding);
        insert_float_token(values, "editor.density.panel_padding", self.panel_padding);
        insert_float_token(
            values,
            "editor.density.activity_rail_width",
            self.activity_rail_width,
        );
        insert_float_token(values, "editor.density.row_height", self.row_height);
        insert_float_token(
            values,
            "editor.density.left_drawer_width",
            self.left_drawer_width,
        );
        insert_float_token(
            values,
            "editor.density.right_drawer_width",
            self.right_drawer_width,
        );
        insert_float_token(
            values,
            "editor.density.bottom_output_height",
            self.bottom_output_height,
        );
        insert_float_token(
            values,
            "editor.density.breakpoint_ultra_width",
            self.breakpoint_ultra_width,
        );
        insert_float_token(
            values,
            "editor.density.breakpoint_narrow_width",
            self.breakpoint_narrow_width,
        );
        insert_float_token(
            values,
            "editor.density.breakpoint_wide_width",
            self.breakpoint_wide_width,
        );
        insert_float_token(
            values,
            "editor.density.compact_side_width",
            self.compact_side_width,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_side_width",
            self.ultra_compact_side_width,
        );
        insert_float_token(
            values,
            "editor.density.compact_left_drawer_max_width",
            self.compact_left_drawer_max_width,
        );
        insert_float_token(
            values,
            "editor.density.compact_right_drawer_max_width",
            self.compact_right_drawer_max_width,
        );
        insert_float_token(
            values,
            "editor.density.compact_side_min_width",
            self.compact_side_min_width,
        );
        insert_float_token(
            values,
            "editor.density.minimum_document_width_fraction",
            self.minimum_document_width_fraction,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_left_drawer_max_width",
            self.ultra_compact_left_drawer_max_width,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_right_drawer_max_width",
            self.ultra_compact_right_drawer_max_width,
        );
        insert_float_token(
            values,
            "editor.density.compact_bottom_available_height",
            self.compact_bottom_available_height,
        );
        insert_float_token(
            values,
            "editor.density.compact_bottom_max_height",
            self.compact_bottom_max_height,
        );
        insert_float_token(
            values,
            "editor.density.compact_bottom_max_available_fraction",
            self.compact_bottom_max_available_fraction,
        );
        insert_float_token(
            values,
            "editor.density.compact_bottom_min_height",
            self.compact_bottom_min_height,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_bottom_available_height",
            self.ultra_compact_bottom_available_height,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_bottom_max_height",
            self.ultra_compact_bottom_max_height,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_bottom_max_available_fraction",
            self.ultra_compact_bottom_max_available_fraction,
        );
        insert_float_token(
            values,
            "editor.density.ultra_compact_bottom_min_height",
            self.ultra_compact_bottom_min_height,
        );
        insert_float_token(
            values,
            "editor.density.minimum_window_width",
            self.minimum_window_width,
        );
        insert_float_token(
            values,
            "editor.density.minimum_window_height",
            self.minimum_window_height,
        );
        insert_float_token(
            values,
            "editor.density.ultra_minimum_window_width",
            self.ultra_minimum_window_width,
        );
        insert_float_token(
            values,
            "editor.density.ultra_minimum_window_height",
            self.ultra_minimum_window_height,
        );
    }
}
