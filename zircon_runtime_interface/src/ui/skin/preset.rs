use serde::{Deserialize, Serialize};

pub const MATERIAL_DARK_SKIN_ID: &str = "material_dark";
pub const FYROX_PANEL_PRESET_ID: &str = "fyrox_panel";
pub const JETBRAINS_SHELL_PRESET_ID: &str = "jetbrains_shell";
pub const UNREAL_WINDOW_MODEL_PRESET_ID: &str = "unreal_window_model";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDesignPresetKind {
    Skin,
    Panel,
    Shell,
    WindowModel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDesignReference {
    MaterialUi,
    MaterialComponents,
    SlintMaterial,
    FyroxEditor,
    JetBrainsIde,
    UnrealEditor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiColorScheme {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiComponentVisualState {
    Normal,
    Hover,
    Pressed,
    Selected,
    Disabled,
    Focused,
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSemanticTokenFamily {
    Palette,
    Text,
    Action,
    Surface,
    Divider,
    Focus,
    Elevation,
    Radius,
    Spacing,
    Typography,
    IconSize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSemanticToken {
    pub name: String,
    pub family: UiSemanticTokenFamily,
    pub value: String,
}

impl UiSemanticToken {
    pub fn new(
        name: impl Into<String>,
        family: UiSemanticTokenFamily,
        value: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            family,
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDesignPresetDescriptor {
    pub id: String,
    pub display_name: String,
    pub kind: UiDesignPresetKind,
    pub default_color_scheme: Option<UiColorScheme>,
    pub references: Vec<UiDesignReference>,
    pub tokens: Vec<UiSemanticToken>,
    pub visual_states: Vec<UiComponentVisualState>,
    pub component_roles: Vec<String>,
}

impl UiDesignPresetDescriptor {
    pub fn material_dark() -> Self {
        Self {
            id: MATERIAL_DARK_SKIN_ID.to_string(),
            display_name: "Material Dark".to_string(),
            kind: UiDesignPresetKind::Skin,
            default_color_scheme: Some(UiColorScheme::Dark),
            references: vec![
                UiDesignReference::MaterialUi,
                UiDesignReference::MaterialComponents,
                UiDesignReference::SlintMaterial,
            ],
            tokens: material_dark_tokens(),
            visual_states: full_visual_state_set(),
            component_roles: vec![
                "primitive".to_string(),
                "layout".to_string(),
                "data_view".to_string(),
                "shell".to_string(),
            ],
        }
    }

    pub fn fyrox_panel() -> Self {
        Self {
            id: FYROX_PANEL_PRESET_ID.to_string(),
            display_name: "Fyrox Panel Model".to_string(),
            kind: UiDesignPresetKind::Panel,
            default_color_scheme: None,
            references: vec![UiDesignReference::FyroxEditor],
            tokens: Vec::new(),
            visual_states: Vec::new(),
            component_roles: vec![
                "scene_viewer".to_string(),
                "hierarchy".to_string(),
                "inspector".to_string(),
                "asset_browser".to_string(),
                "console".to_string(),
                "plugin_manager".to_string(),
            ],
        }
    }

    pub fn jetbrains_shell() -> Self {
        Self {
            id: JETBRAINS_SHELL_PRESET_ID.to_string(),
            display_name: "JetBrains Shell Model".to_string(),
            kind: UiDesignPresetKind::Shell,
            default_color_scheme: None,
            references: vec![UiDesignReference::JetBrainsIde],
            tokens: Vec::new(),
            visual_states: Vec::new(),
            component_roles: vec![
                "side_drawer".to_string(),
                "tool_window_tab".to_string(),
                "document_tab".to_string(),
                "floating_window".to_string(),
            ],
        }
    }

    pub fn unreal_window_model() -> Self {
        Self {
            id: UNREAL_WINDOW_MODEL_PRESET_ID.to_string(),
            display_name: "Unreal Window Model".to_string(),
            kind: UiDesignPresetKind::WindowModel,
            default_color_scheme: None,
            references: vec![UiDesignReference::UnrealEditor],
            tokens: Vec::new(),
            visual_states: Vec::new(),
            component_roles: vec![
                "workbench_window".to_string(),
                "asset_editor_window".to_string(),
                "prefab_editor_window".to_string(),
                "material_editor_window".to_string(),
                "animation_editor_window".to_string(),
            ],
        }
    }

    pub fn token(&self, name: &str) -> Option<&UiSemanticToken> {
        self.tokens.iter().find(|token| token.name == name)
    }

    pub fn has_visual_state(&self, state: UiComponentVisualState) -> bool {
        self.visual_states.contains(&state)
    }

    pub fn has_reference(&self, reference: UiDesignReference) -> bool {
        self.references.contains(&reference)
    }
}

fn full_visual_state_set() -> Vec<UiComponentVisualState> {
    vec![
        UiComponentVisualState::Normal,
        UiComponentVisualState::Hover,
        UiComponentVisualState::Pressed,
        UiComponentVisualState::Selected,
        UiComponentVisualState::Disabled,
        UiComponentVisualState::Focused,
        UiComponentVisualState::Error,
        UiComponentVisualState::Warning,
    ]
}

fn material_dark_tokens() -> Vec<UiSemanticToken> {
    use UiSemanticTokenFamily::{
        Action, Divider, Elevation, Focus, IconSize, Palette, Radius, Spacing, Surface, Text,
        Typography,
    };

    vec![
        UiSemanticToken::new("palette.mode", Palette, "dark"),
        UiSemanticToken::new("palette.primary.main", Palette, "#90caf9"),
        UiSemanticToken::new("palette.secondary.main", Palette, "#ce93d8"),
        UiSemanticToken::new("palette.error.main", Palette, "#f44336"),
        UiSemanticToken::new("palette.warning.main", Palette, "#ffa726"),
        UiSemanticToken::new("palette.info.main", Palette, "#29b6f6"),
        UiSemanticToken::new("palette.success.main", Palette, "#66bb6a"),
        UiSemanticToken::new("text.primary", Text, "#ffffff"),
        UiSemanticToken::new("text.secondary", Text, "rgba(255,255,255,0.70)"),
        UiSemanticToken::new("text.disabled", Text, "rgba(255,255,255,0.50)"),
        UiSemanticToken::new("action.active", Action, "#ffffff"),
        UiSemanticToken::new("action.hover", Action, "rgba(255,255,255,0.08)"),
        UiSemanticToken::new("action.pressed", Action, "rgba(255,255,255,0.12)"),
        UiSemanticToken::new("action.selected", Action, "rgba(255,255,255,0.16)"),
        UiSemanticToken::new("action.disabled", Action, "rgba(255,255,255,0.30)"),
        UiSemanticToken::new("surface.background", Surface, "#121212"),
        UiSemanticToken::new("surface.paper", Surface, "#1e1e1e"),
        UiSemanticToken::new("surface.panel", Surface, "#232323"),
        UiSemanticToken::new("surface.window", Surface, "#181818"),
        UiSemanticToken::new("divider.default", Divider, "rgba(255,255,255,0.12)"),
        UiSemanticToken::new("focus.ring", Focus, "#90caf9"),
        UiSemanticToken::new("elevation.0", Elevation, "0"),
        UiSemanticToken::new("elevation.1", Elevation, "1"),
        UiSemanticToken::new("elevation.2", Elevation, "2"),
        UiSemanticToken::new("radius.xs", Radius, "2"),
        UiSemanticToken::new("radius.sm", Radius, "4"),
        UiSemanticToken::new("radius.md", Radius, "6"),
        UiSemanticToken::new("spacing.1", Spacing, "4"),
        UiSemanticToken::new("spacing.2", Spacing, "8"),
        UiSemanticToken::new("spacing.3", Spacing, "12"),
        UiSemanticToken::new(
            "typography.font_family",
            Typography,
            "Inter, Roboto, Segoe UI",
        ),
        UiSemanticToken::new("typography.body.size", Typography, "13"),
        UiSemanticToken::new("typography.caption.size", Typography, "11"),
        UiSemanticToken::new("icon.size.sm", IconSize, "16"),
        UiSemanticToken::new("icon.size.md", IconSize, "20"),
    ]
}
