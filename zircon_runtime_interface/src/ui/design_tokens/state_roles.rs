use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::ui::style::{UiPainterResolvedState, UiRgbaColor};

use super::{cascade_registry::insert_string_token, EditorPaletteTokens};

/// A semantic state-color role resolved against the active editor palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorStateColorRole {
    Surface0,
    Surface1,
    Surface2,
    Surface3,
    SurfaceSelected,
    Accent,
    FocusRing,
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
            Self::SurfaceSelected => palette.surface_selected,
            Self::Accent => palette.accent,
            Self::FocusRing => palette.focus_ring,
            Self::Border => palette.border,
            Self::TextPrimary => palette.text_primary,
            Self::TextSecondary => palette.text_secondary,
            Self::TextDisabled => palette.text_disabled,
        }
    }

    fn cascade_token_reference(self) -> &'static str {
        match self {
            Self::Surface0 => "$editor.surface.0",
            Self::Surface1 => "$editor.surface.1",
            Self::Surface2 => "$editor.surface.2",
            Self::Surface3 => "$editor.surface.3",
            Self::SurfaceSelected => "$editor.surface.selected",
            Self::Accent => "$editor.accent",
            Self::FocusRing => "$editor.focus.ring",
            Self::Border => "$editor.border",
            Self::TextPrimary => "$editor.text.primary",
            Self::TextSecondary => "$editor.text.secondary",
            Self::TextDisabled => "$editor.text.disabled",
        }
    }
}

/// Maps painter states to palette roles without encoding palette values in the selector.
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
            selected: EditorStateColorRole::SurfaceSelected,
            // Keyboard focus is an affordance, not persistent selection. The
            // corresponding painter style keeps this fill and composes the
            // focus-ring token as an independent outline.
            focused: EditorStateColorRole::Surface1,
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

    pub(super) fn insert_cascade_tokens(&self, values: &mut BTreeMap<String, Value>) {
        insert_string_token(
            values,
            "editor.state.default",
            self.default.cascade_token_reference(),
        );
        insert_string_token(
            values,
            "editor.state.hovered",
            self.hovered.cascade_token_reference(),
        );
        insert_string_token(
            values,
            "editor.state.pressed",
            self.pressed.cascade_token_reference(),
        );
        insert_string_token(
            values,
            "editor.state.selected",
            self.selected.cascade_token_reference(),
        );
        insert_string_token(
            values,
            "editor.state.focused",
            self.focused.cascade_token_reference(),
        );
        insert_string_token(
            values,
            "editor.state.disabled",
            self.disabled.cascade_token_reference(),
        );
        insert_string_token(
            values,
            "editor.state.loading",
            self.loading.cascade_token_reference(),
        );
    }
}
