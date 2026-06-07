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
            | UiPainterFamily::Toast => Self::interactive_resolved_state(state),
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

    pub const fn selection_control_resolved_state(
        state: UiPainterState,
    ) -> UiPainterResolvedState {
        if state.disabled {
            UiPainterResolvedState::Disabled
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
