use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRgbaColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl UiRgbaColor {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: red.clamp(0.0, 1.0),
            green: green.clamp(0.0, 1.0),
            blue: blue.clamp(0.0, 1.0),
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    pub fn from_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::new(
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
            f32::from(alpha) / 255.0,
        )
    }

    pub fn to_u8(self) -> [u8; 4] {
        [
            channel_to_u8(self.red),
            channel_to_u8(self.green),
            channel_to_u8(self.blue),
            channel_to_u8(self.alpha),
        ]
    }
}

impl Default for UiRgbaColor {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

fn channel_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiStyleColor {
    Role(String),
    Rgba(UiRgbaColor),
    Inherit,
    Transparent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleDimension {
    Auto,
    Fixed(f32),
    Fill,
    Style(String),
}

impl Default for StyleDimension {
    fn default() -> Self {
        Self::Auto
    }
}

pub type ButtonDimension = StyleDimension;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedElementStyle {
    pub background_color: Option<UiStyleColor>,
    pub foreground_color: Option<UiStyleColor>,
    pub border_color: Option<UiStyleColor>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub width: StyleDimension,
    pub height: StyleDimension,
    pub opacity: f32,
}

impl Default for UiResolvedElementStyle {
    fn default() -> Self {
        Self {
            background_color: None,
            foreground_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            width: StyleDimension::Auto,
            height: StyleDimension::Auto,
            opacity: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonVariant {
    /// Authored default. It resolves to the Material UI text button variant.
    Default,
    #[default]
    Text,
    Contained,
    Outlined,
}

impl ButtonVariant {
    pub const OPTIONS: [&'static str; 4] = ["default", "text", "contained", "outlined"];

    pub fn normalized(self) -> Self {
        match self {
            Self::Default => Self::Text,
            value => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonColor {
    Default,
    Inherit,
    Primary,
    Secondary,
    Success,
    Error,
    Info,
    Warning,
    Style(String),
    Custom(UiRgbaColor),
}

impl Default for ButtonColor {
    fn default() -> Self {
        Self::Primary
    }
}

impl ButtonColor {
    pub const OPTIONS: [&'static str; 8] = [
        "default",
        "inherit",
        "primary",
        "secondary",
        "success",
        "error",
        "info",
        "warning",
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
    Custom {
        width: ButtonDimension,
        height: ButtonDimension,
    },
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl ButtonSize {
    pub const OPTIONS: [&'static str; 3] = ["small", "medium", "large"];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonIconPlacement {
    #[default]
    None,
    Start,
    End,
    IconOnly,
}

impl ButtonIconPlacement {
    pub const OPTIONS: [&'static str; 4] = ["none", "start", "end", "icon_only"];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonInteractionState {
    #[default]
    Normal,
    Hover,
    Pressed,
    Focused,
    Disabled,
    Loading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonEventKind {
    Enter,
    Leave,
    Press,
    Release,
    Click,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedButtonStyle {
    pub variant: ButtonVariant,
    pub color: ButtonColor,
    pub size: ButtonSize,
    pub width: ButtonDimension,
    pub height: ButtonDimension,
    pub icon_placement: ButtonIconPlacement,
    pub interaction_state: ButtonInteractionState,
    pub loading: bool,
    pub disabled: bool,
    pub element: UiResolvedElementStyle,
}

impl Default for ResolvedButtonStyle {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Text,
            color: ButtonColor::Primary,
            size: ButtonSize::Medium,
            width: ButtonDimension::Auto,
            height: ButtonDimension::Auto,
            icon_placement: ButtonIconPlacement::None,
            interaction_state: ButtonInteractionState::Normal,
            loading: false,
            disabled: false,
            element: UiResolvedElementStyle::default(),
        }
    }
}
