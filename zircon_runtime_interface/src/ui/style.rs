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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiThemeTokenRef(pub String);

impl UiThemeTokenRef {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiThemeDocument {
    pub id: String,
    pub palette: UiThemePalette,
    pub typography: Vec<UiThemeTypographyVariant>,
    pub shape: UiThemeShape,
    pub spacing: Vec<f32>,
    pub control_sizes: UiThemeControlSizes,
    pub elevation: Vec<UiThemeElevation>,
}

impl Default for UiThemeDocument {
    fn default() -> Self {
        Self::dark()
    }
}

impl UiThemeDocument {
    pub fn dark() -> Self {
        Self {
            id: "zircon.dark".to_string(),
            palette: UiThemePalette::dark(),
            typography: vec![
                UiThemeTypographyVariant {
                    variant: "body".to_string(),
                    family: "Inter".to_string(),
                    size: 13.0,
                    weight: 400,
                    line_height: 1.45,
                },
                UiThemeTypographyVariant {
                    variant: "caption".to_string(),
                    family: "Inter".to_string(),
                    size: 11.0,
                    weight: 400,
                    line_height: 1.35,
                },
                UiThemeTypographyVariant {
                    variant: "title".to_string(),
                    family: "Inter".to_string(),
                    size: 15.0,
                    weight: 600,
                    line_height: 1.35,
                },
            ],
            shape: UiThemeShape::default(),
            spacing: vec![0.0, 4.0, 8.0, 12.0, 16.0, 24.0, 32.0],
            control_sizes: UiThemeControlSizes::default(),
            elevation: vec![
                UiThemeElevation::level(0, 0.0, 0.0, 0.0, 0.0),
                UiThemeElevation::level(1, 0.0, 2.0, 8.0, 0.0),
                UiThemeElevation::level(2, 0.0, 4.0, 16.0, 0.0),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiThemePalette {
    #[serde(default = "default_theme_surface")]
    pub surface: [UiRgbaColor; 4],
    #[serde(default = "default_theme_text_primary")]
    pub text_primary: UiRgbaColor,
    #[serde(default = "default_theme_text_secondary")]
    pub text_secondary: UiRgbaColor,
    #[serde(default = "default_theme_text_disabled")]
    pub text_disabled: UiRgbaColor,
    #[serde(default = "default_theme_accent")]
    pub accent: UiRgbaColor,
    #[serde(default = "default_theme_success")]
    pub success: UiRgbaColor,
    #[serde(default = "default_theme_info")]
    pub info: UiRgbaColor,
    #[serde(default = "default_theme_warning")]
    pub warning: UiRgbaColor,
    #[serde(default = "default_theme_error")]
    pub error: UiRgbaColor,
    #[serde(default = "default_theme_separator")]
    pub separator: UiRgbaColor,
}

impl Default for UiThemePalette {
    fn default() -> Self {
        Self::dark()
    }
}

impl UiThemePalette {
    pub fn dark() -> Self {
        Self {
            surface: [
                default_theme_surface()[0],
                default_theme_surface()[1],
                default_theme_surface()[2],
                default_theme_surface()[3],
            ],
            text_primary: default_theme_text_primary(),
            text_secondary: default_theme_text_secondary(),
            text_disabled: default_theme_text_disabled(),
            accent: default_theme_accent(),
            success: default_theme_success(),
            info: default_theme_info(),
            warning: default_theme_warning(),
            error: default_theme_error(),
            separator: default_theme_separator(),
        }
    }
}

fn default_theme_surface() -> [UiRgbaColor; 4] {
    [
        UiRgbaColor::from_u8(17, 20, 22, 255),
        UiRgbaColor::from_u8(23, 26, 29, 255),
        UiRgbaColor::from_u8(27, 31, 35, 255),
        UiRgbaColor::from_u8(37, 43, 49, 255),
    ]
}

fn default_theme_text_primary() -> UiRgbaColor {
    UiRgbaColor::from_u8(232, 236, 238, 255)
}

fn default_theme_text_secondary() -> UiRgbaColor {
    UiRgbaColor::from_u8(164, 174, 180, 255)
}

fn default_theme_text_disabled() -> UiRgbaColor {
    UiRgbaColor::from_u8(101, 111, 118, 255)
}

fn default_theme_accent() -> UiRgbaColor {
    UiRgbaColor::from_u8(60, 199, 214, 255)
}

fn default_theme_success() -> UiRgbaColor {
    UiRgbaColor::from_u8(85, 190, 120, 255)
}

fn default_theme_info() -> UiRgbaColor {
    UiRgbaColor::from_u8(95, 170, 230, 255)
}

fn default_theme_warning() -> UiRgbaColor {
    UiRgbaColor::from_u8(220, 172, 80, 255)
}

fn default_theme_error() -> UiRgbaColor {
    UiRgbaColor::from_u8(235, 96, 92, 255)
}

fn default_theme_separator() -> UiRgbaColor {
    UiRgbaColor::from_u8(57, 65, 71, 255)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiThemeTypographyVariant {
    #[serde(default = "default_typography_variant_name")]
    pub variant: String,
    #[serde(default = "default_typography_family")]
    pub family: String,
    #[serde(default = "default_typography_size")]
    pub size: f32,
    #[serde(default = "default_typography_weight")]
    pub weight: u16,
    #[serde(default = "default_typography_line_height")]
    pub line_height: f32,
}

impl Default for UiThemeTypographyVariant {
    fn default() -> Self {
        Self {
            variant: default_typography_variant_name(),
            family: default_typography_family(),
            size: default_typography_size(),
            weight: default_typography_weight(),
            line_height: default_typography_line_height(),
        }
    }
}

fn default_typography_variant_name() -> String {
    "body".to_string()
}

fn default_typography_family() -> String {
    "Inter".to_string()
}

fn default_typography_size() -> f32 {
    13.0
}

fn default_typography_weight() -> u16 {
    400
}

fn default_typography_line_height() -> f32 {
    1.45
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiThemeShape {
    #[serde(default = "default_radius_small")]
    pub radius_small: f32,
    #[serde(default = "default_radius_medium")]
    pub radius_medium: f32,
    #[serde(default = "default_radius_large")]
    pub radius_large: f32,
    #[serde(default = "default_radius_panel")]
    pub radius_panel: f32,
}

impl Default for UiThemeShape {
    fn default() -> Self {
        Self {
            radius_small: default_radius_small(),
            radius_medium: default_radius_medium(),
            radius_large: default_radius_large(),
            radius_panel: default_radius_panel(),
        }
    }
}

fn default_radius_small() -> f32 {
    4.0
}

fn default_radius_medium() -> f32 {
    5.0
}

fn default_radius_large() -> f32 {
    8.0
}

fn default_radius_panel() -> f32 {
    12.0
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiThemeControlSizes {
    #[serde(default = "default_control_height")]
    pub default_height: f32,
    #[serde(default = "compact_control_height")]
    pub compact_height: f32,
    #[serde(default = "dense_control_height")]
    pub dense_height: f32,
}

impl Default for UiThemeControlSizes {
    fn default() -> Self {
        Self {
            default_height: default_control_height(),
            compact_height: compact_control_height(),
            dense_height: dense_control_height(),
        }
    }
}

fn default_control_height() -> f32 {
    40.0
}

fn compact_control_height() -> f32 {
    32.0
}

fn dense_control_height() -> f32 {
    28.0
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiThemeElevation {
    pub level: u8,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
}

impl UiThemeElevation {
    pub fn level(level: u8, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        Self {
            level,
            offset_x,
            offset_y,
            blur,
            spread,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPainterFamily {
    #[default]
    Generic,
    Button,
    IconButton,
    Toggle,
    Checkbox,
    Radio,
    Slider,
    Dropdown,
    PopupRow,
    Alert,
    Tooltip,
    TextField,
    ListRow,
    TreeRow,
    TableRow,
    Tab,
    Toast,
    Chrome,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiPainterResolvedState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
    Checked,
    Selected,
    Open,
    Dragging,
    DropHovered,
    Loading,
}

/// Cross-host pseudo-state consumed by painter style selectors.
///
/// Slate keeps event replies separate from widget state; Zircon mirrors that split by storing the
/// durable state here and letting each painter family choose its resolved visual style from it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPainterState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub checked: bool,
    pub selected: bool,
    pub open: bool,
    pub dragging: bool,
    pub drop_hovered: bool,
    pub loading: bool,
}

/// Shared visual-state priority selector for runtime and native painters.
///
/// Widget behavior produces semantic state; this selector is the single contract that turns that
/// state into a concrete visual state for a painter family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiPainterStyleSelector;

impl UiPainterStyleSelector {
    pub const fn resolved_state_for_family(
        state: UiPainterState,
        family: UiPainterFamily,
    ) -> UiPainterResolvedState {
        match family {
            UiPainterFamily::Button => Self::button_resolved_state(state),
            UiPainterFamily::Checkbox | UiPainterFamily::Radio | UiPainterFamily::Toggle => {
                Self::selection_control_resolved_state(state)
            }
            UiPainterFamily::Slider => Self::slider_resolved_state(state),
            UiPainterFamily::Generic
            | UiPainterFamily::IconButton
            | UiPainterFamily::Dropdown
            | UiPainterFamily::PopupRow
            | UiPainterFamily::Alert
            | UiPainterFamily::Tooltip
            | UiPainterFamily::TextField
            | UiPainterFamily::ListRow
            | UiPainterFamily::TreeRow
            | UiPainterFamily::TableRow
            | UiPainterFamily::Tab
            | UiPainterFamily::Toast
            | UiPainterFamily::Chrome => Self::interactive_resolved_state(state),
        }
    }

    pub const fn interactive_resolved_state(state: UiPainterState) -> UiPainterResolvedState {
        if state.disabled {
            UiPainterResolvedState::Disabled
        } else if state.loading {
            UiPainterResolvedState::Loading
        } else if state.pressed {
            UiPainterResolvedState::Pressed
        } else if state.focused {
            UiPainterResolvedState::Focused
        } else if state.open {
            UiPainterResolvedState::Open
        } else if state.dragging {
            UiPainterResolvedState::Dragging
        } else if state.drop_hovered {
            UiPainterResolvedState::DropHovered
        } else if state.hovered {
            UiPainterResolvedState::Hovered
        } else if state.selected {
            UiPainterResolvedState::Selected
        } else if state.checked {
            UiPainterResolvedState::Checked
        } else {
            UiPainterResolvedState::Normal
        }
    }

    pub const fn selection_control_resolved_state(state: UiPainterState) -> UiPainterResolvedState {
        if state.disabled {
            UiPainterResolvedState::Disabled
        } else if state.loading {
            UiPainterResolvedState::Loading
        } else if state.pressed {
            UiPainterResolvedState::Pressed
        } else if state.focused {
            UiPainterResolvedState::Focused
        } else if state.dragging {
            UiPainterResolvedState::Dragging
        } else if state.drop_hovered {
            UiPainterResolvedState::DropHovered
        } else if state.hovered {
            UiPainterResolvedState::Hovered
        } else if state.selected {
            UiPainterResolvedState::Selected
        } else if state.checked {
            UiPainterResolvedState::Checked
        } else {
            UiPainterResolvedState::Normal
        }
    }

    pub const fn slider_resolved_state(state: UiPainterState) -> UiPainterResolvedState {
        if state.disabled {
            UiPainterResolvedState::Disabled
        } else if state.loading {
            UiPainterResolvedState::Loading
        } else if state.pressed {
            UiPainterResolvedState::Pressed
        } else if state.focused {
            UiPainterResolvedState::Focused
        } else if state.dragging {
            UiPainterResolvedState::Dragging
        } else if state.drop_hovered {
            UiPainterResolvedState::DropHovered
        } else if state.hovered {
            UiPainterResolvedState::Hovered
        } else {
            UiPainterResolvedState::Normal
        }
    }

    pub const fn button_resolved_state(state: UiPainterState) -> UiPainterResolvedState {
        if state.disabled {
            UiPainterResolvedState::Disabled
        } else if state.loading {
            UiPainterResolvedState::Loading
        } else if state.pressed {
            UiPainterResolvedState::Pressed
        } else if state.is_focus_visible() {
            UiPainterResolvedState::Focused
        } else if state.is_pointer_hot() || state.open {
            UiPainterResolvedState::Hovered
        } else {
            UiPainterResolvedState::Normal
        }
    }

    pub const fn button_interaction_state(state: UiPainterState) -> ButtonInteractionState {
        match Self::button_resolved_state(state) {
            UiPainterResolvedState::Disabled => ButtonInteractionState::Disabled,
            UiPainterResolvedState::Loading => ButtonInteractionState::Loading,
            UiPainterResolvedState::Pressed => ButtonInteractionState::Pressed,
            UiPainterResolvedState::Focused => ButtonInteractionState::Focused,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => ButtonInteractionState::Hover,
            UiPainterResolvedState::Normal
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected => ButtonInteractionState::Normal,
        }
    }
}

impl UiPainterState {
    pub const fn normal() -> Self {
        Self {
            hovered: false,
            pressed: false,
            focused: false,
            disabled: false,
            checked: false,
            selected: false,
            open: false,
            dragging: false,
            drop_hovered: false,
            loading: false,
        }
    }

    pub const fn is_active(self) -> bool {
        self.pressed || self.checked || self.selected || self.open || self.dragging
    }

    pub const fn is_focus_visible(self) -> bool {
        self.focused || self.checked || self.selected
    }

    pub const fn is_pointer_hot(self) -> bool {
        self.hovered || self.drop_hovered || self.dragging
    }

    pub const fn resolved_state_for_family(
        self,
        family: UiPainterFamily,
    ) -> UiPainterResolvedState {
        UiPainterStyleSelector::resolved_state_for_family(self, family)
    }

    pub const fn interactive_resolved_state(self) -> UiPainterResolvedState {
        UiPainterStyleSelector::interactive_resolved_state(self)
    }

    pub const fn selection_control_resolved_state(self) -> UiPainterResolvedState {
        UiPainterStyleSelector::selection_control_resolved_state(self)
    }

    pub const fn slider_resolved_state(self) -> UiPainterResolvedState {
        UiPainterStyleSelector::slider_resolved_state(self)
    }

    pub const fn button_resolved_state(self) -> UiPainterResolvedState {
        UiPainterStyleSelector::button_resolved_state(self)
    }

    pub const fn button_interaction_state(self) -> ButtonInteractionState {
        UiPainterStyleSelector::button_interaction_state(self)
    }
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
