use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use toml::Value;
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonIconPlacement, ButtonInteractionState, ButtonSize, ButtonVariant,
    ResolvedButtonStyle, StyleDimension, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
};
use zircon_runtime_interface::ui::v2::UiV2ResolvedStyle;

pub trait StyleProperty {
    type Value: Clone;

    fn default_value() -> Self::Value;
    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        let sheet = UiV2ResolvedStyle {
            self_values: values.clone(),
            slot: BTreeMap::new(),
            style_tokens: BTreeMap::new(),
        };
        Self::extract(&sheet)
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum StyleField<P: StyleProperty> {
    FromCascade(PhantomData<P>),
    OverrideValue(P::Value),
}

impl<P: StyleProperty> StyleField<P> {
    pub fn from_cascade() -> Self {
        Self::FromCascade(PhantomData)
    }

    pub fn override_value(value: P::Value) -> Self {
        Self::OverrideValue(value)
    }

    pub fn resolve(&self, style_sheets: &[Weak<StyleSheetScope>]) -> P::Value {
        match self {
            Self::FromCascade(_) => resolve_property::<P>(None, style_sheets),
            Self::OverrideValue(value) => value.clone(),
        }
    }
}

impl<P: StyleProperty> Default for StyleField<P> {
    fn default() -> Self {
        Self::from_cascade()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleSheetScope {
    pub style: UiV2ResolvedStyle,
}

impl StyleSheetScope {
    pub fn new(style: UiV2ResolvedStyle) -> Self {
        Self { style }
    }
}

pub fn resolve_property<P: StyleProperty>(
    override_value: Option<P::Value>,
    style_sheets: &[Weak<StyleSheetScope>],
) -> P::Value {
    if let Some(value) = override_value {
        return value;
    }
    for weak in style_sheets.iter().rev() {
        let Some(scope) = weak.upgrade() else {
            continue;
        };
        if let Some(value) = P::extract(&scope.style) {
            return value;
        }
    }
    P::default_value()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementStyleFields {
    pub background_color: StyleField<BackgroundColorProperty>,
    pub foreground_color: StyleField<ForegroundColorProperty>,
    pub border_color: StyleField<BorderColorProperty>,
    pub border_width: StyleField<BorderWidthProperty>,
    pub corner_radius: StyleField<CornerRadiusProperty>,
    pub width: StyleField<WidthProperty>,
    pub height: StyleField<HeightProperty>,
    pub opacity: StyleField<OpacityProperty>,
}

impl ElementStyleFields {
    pub fn resolve(&self, style_sheets: &[Weak<StyleSheetScope>]) -> UiResolvedElementStyle {
        UiResolvedElementStyle {
            background_color: self.background_color.resolve(style_sheets),
            foreground_color: self.foreground_color.resolve(style_sheets),
            border_color: self.border_color.resolve(style_sheets),
            border_width: self.border_width.resolve(style_sheets),
            corner_radius: self.corner_radius.resolve(style_sheets),
            width: self.width.resolve(style_sheets),
            height: self.height.resolve(style_sheets),
            opacity: self.opacity.resolve(style_sheets).clamp(0.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ButtonStyleFields {
    pub variant: StyleField<ButtonVariantProperty>,
    pub color: StyleField<ButtonColorProperty>,
    pub size: StyleField<ButtonSizeProperty>,
    pub width: StyleField<WidthProperty>,
    pub height: StyleField<HeightProperty>,
    pub icon_placement: StyleField<ButtonIconPlacementProperty>,
    pub interaction_state: StyleField<ButtonInteractionStateProperty>,
    pub loading: StyleField<ButtonLoadingProperty>,
    pub disabled: StyleField<ButtonDisabledProperty>,
    pub element: ElementStyleFields,
}

impl ButtonStyleFields {
    pub fn resolve(&self, style_sheets: &[Weak<StyleSheetScope>]) -> ResolvedButtonStyle {
        let variant = self.variant.resolve(style_sheets).normalized();
        let size = self.size.resolve(style_sheets);
        ResolvedButtonStyle {
            variant,
            color: self.color.resolve(style_sheets),
            size,
            width: self.width.resolve(style_sheets),
            height: self.height.resolve(style_sheets),
            icon_placement: self.icon_placement.resolve(style_sheets),
            interaction_state: self.interaction_state.resolve(style_sheets),
            loading: self.loading.resolve(style_sheets),
            disabled: self.disabled.resolve(style_sheets),
            element: self.element.resolve(style_sheets),
        }
    }
}

pub type SharedStyleSheetScope = Arc<StyleSheetScope>;

pub fn resolve_button_style_from_values(values: &BTreeMap<String, Value>) -> ResolvedButtonStyle {
    ResolvedButtonStyle {
        variant: value_or_default::<ButtonVariantProperty>(values).normalized(),
        color: value_or_default::<ButtonColorProperty>(values),
        size: value_or_default::<ButtonSizeProperty>(values),
        width: value_or_default::<WidthProperty>(values),
        height: value_or_default::<HeightProperty>(values),
        icon_placement: value_or_default::<ButtonIconPlacementProperty>(values),
        interaction_state: value_or_default::<ButtonInteractionStateProperty>(values),
        loading: value_or_default::<ButtonLoadingProperty>(values),
        disabled: value_or_default::<ButtonDisabledProperty>(values),
        element: UiResolvedElementStyle {
            background_color: value_or_default::<BackgroundColorProperty>(values),
            foreground_color: value_or_default::<ForegroundColorProperty>(values),
            border_color: value_or_default::<BorderColorProperty>(values),
            border_width: value_or_default::<BorderWidthProperty>(values),
            corner_radius: value_or_default::<CornerRadiusProperty>(values),
            width: value_or_default::<WidthProperty>(values),
            height: value_or_default::<HeightProperty>(values),
            opacity: value_or_default::<OpacityProperty>(values).clamp(0.0, 1.0),
        },
    }
}

fn value_or_default<P: StyleProperty>(values: &BTreeMap<String, Value>) -> P::Value {
    P::extract_values(values).unwrap_or_else(P::default_value)
}

macro_rules! forward_extract_to_values {
    () => {
        fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
            Self::extract_values(&sheet.self_values)
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundColorProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ForegroundColorProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct BorderColorProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct BorderWidthProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct CornerRadiusProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct WidthProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct HeightProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct OpacityProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonVariantProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonColorProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonSizeProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonIconPlacementProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonInteractionStateProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonLoadingProperty;
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonDisabledProperty;

impl StyleProperty for BackgroundColorProperty {
    type Value = Option<UiStyleColor>;

    fn default_value() -> Self::Value {
        None
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        color_value(values, "background")
            .or_else(|| color_value(values, "background_color"))
            .map(Some)
    }

    forward_extract_to_values!();
}

impl StyleProperty for ForegroundColorProperty {
    type Value = Option<UiStyleColor>;

    fn default_value() -> Self::Value {
        None
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        color_value(values, "foreground")
            .or_else(|| color_value(values, "foreground_color"))
            .or_else(|| color_value(values, "fg"))
            .or_else(|| color_value(values, "color"))
            .map(Some)
    }

    forward_extract_to_values!();
}

impl StyleProperty for BorderColorProperty {
    type Value = Option<UiStyleColor>;

    fn default_value() -> Self::Value {
        None
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        color_value(values, "border")
            .or_else(|| color_value(values, "border_color"))
            .or_else(|| color_value(values, "outline"))
            .map(Some)
    }

    forward_extract_to_values!();
}

impl StyleProperty for BorderWidthProperty {
    type Value = f32;

    fn default_value() -> Self::Value {
        0.0
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        table_number(values, "border", "width").or_else(|| number_value(values, "border_width"))
    }

    forward_extract_to_values!();
}

impl StyleProperty for CornerRadiusProperty {
    type Value = f32;

    fn default_value() -> Self::Value {
        0.0
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        table_number(values, "border", "radius")
            .or_else(|| number_value(values, "corner_radius"))
            .or_else(|| number_value(values, "radius"))
    }

    forward_extract_to_values!();
}

impl StyleProperty for WidthProperty {
    type Value = StyleDimension;

    fn default_value() -> Self::Value {
        StyleDimension::Auto
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        dimension_value(values, "width")
    }

    forward_extract_to_values!();
}

impl StyleProperty for HeightProperty {
    type Value = StyleDimension;

    fn default_value() -> Self::Value {
        StyleDimension::Auto
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        dimension_value(values, "height")
    }

    forward_extract_to_values!();
}

impl StyleProperty for OpacityProperty {
    type Value = f32;

    fn default_value() -> Self::Value {
        1.0
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        number_value(values, "opacity")
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonVariantProperty {
    type Value = ButtonVariant;

    fn default_value() -> Self::Value {
        ButtonVariant::Text
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        string_value(values, "button_variant")
            .or_else(|| string_value(values, "variant"))
            .and_then(parse_button_variant)
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonColorProperty {
    type Value = ButtonColor;

    fn default_value() -> Self::Value {
        ButtonColor::Primary
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        string_value(values, "button_color")
            .or_else(|| string_value(values, "color"))
            .and_then(parse_button_color)
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonSizeProperty {
    type Value = ButtonSize;

    fn default_value() -> Self::Value {
        ButtonSize::Medium
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        string_value(values, "button_size")
            .or_else(|| string_value(values, "size"))
            .or_else(|| string_value(values, "density"))
            .and_then(parse_button_size)
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonIconPlacementProperty {
    type Value = ButtonIconPlacement;

    fn default_value() -> Self::Value {
        ButtonIconPlacement::None
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        string_value(values, "icon_placement")
            .or_else(|| string_value(values, "button_icon_placement"))
            .and_then(parse_icon_placement)
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonInteractionStateProperty {
    type Value = ButtonInteractionState;

    fn default_value() -> Self::Value {
        ButtonInteractionState::Normal
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        string_value(values, "button_interaction_state")
            .or_else(|| string_value(values, "interaction_state"))
            .and_then(parse_interaction_state)
            .or_else(|| {
                bool_value(values, "loading")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Loading)
            })
            .or_else(|| {
                bool_value(values, "disabled")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Disabled)
            })
            .or_else(|| {
                bool_value(values, "pressed")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Pressed)
            })
            .or_else(|| {
                bool_value(values, "dragging")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Hover)
            })
            .or_else(|| {
                bool_value(values, "focused")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Focused)
            })
            .or_else(|| {
                bool_value(values, "hovered")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Hover)
            })
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonLoadingProperty {
    type Value = bool;

    fn default_value() -> Self::Value {
        false
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        bool_value(values, "loading")
    }

    forward_extract_to_values!();
}

impl StyleProperty for ButtonDisabledProperty {
    type Value = bool;

    fn default_value() -> Self::Value {
        false
    }

    fn extract_values(values: &BTreeMap<String, Value>) -> Option<Self::Value> {
        bool_value(values, "disabled")
    }

    forward_extract_to_values!();
}

fn value<'a>(values: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a Value> {
    values.get(key)
}

fn string_value<'a>(values: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    value(values, key).and_then(Value::as_str)
}

fn bool_value(values: &BTreeMap<String, Value>, key: &str) -> Option<bool> {
    value(values, key).and_then(Value::as_bool)
}

fn number_value(values: &BTreeMap<String, Value>, key: &str) -> Option<f32> {
    number_from_value(value(values, key)?)
}

fn table_number(values: &BTreeMap<String, Value>, table: &str, key: &str) -> Option<f32> {
    value(values, table)
        .and_then(Value::as_table)
        .and_then(|table| table.get(key))
        .and_then(number_from_value)
}

fn number_from_value(value: &Value) -> Option<f32> {
    match value {
        Value::Integer(value) => Some(*value as f32),
        Value::Float(value) => Some(*value as f32),
        _ => None,
    }
}

fn color_value(values: &BTreeMap<String, Value>, key: &str) -> Option<UiStyleColor> {
    let value = value(values, key)?;
    match value {
        Value::String(raw) => parse_style_color(raw),
        Value::Array(values) if values.len() == 4 => {
            let channels = values
                .iter()
                .map(number_from_value)
                .collect::<Option<Vec<_>>>()?;
            Some(UiStyleColor::Rgba(UiRgbaColor::new(
                channels[0],
                channels[1],
                channels[2],
                channels[3],
            )))
        }
        Value::Table(table) => table
            .get("color")
            .and_then(Value::as_str)
            .and_then(parse_style_color),
        _ => None,
    }
}

fn parse_style_color(raw: &str) -> Option<UiStyleColor> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }
    if value.eq_ignore_ascii_case("inherit") {
        return Some(UiStyleColor::Inherit);
    }
    if value.eq_ignore_ascii_case("transparent") {
        return Some(UiStyleColor::Transparent);
    }
    if let Some(color) = parse_hex_color(value) {
        return Some(UiStyleColor::Rgba(color));
    }
    Some(UiStyleColor::Role(value.to_string()))
}

fn parse_hex_color(raw: &str) -> Option<UiRgbaColor> {
    let hex = raw.strip_prefix('#')?;
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match hex.len() {
        6 => Some(UiRgbaColor::from_u8(
            channel(0..2)?,
            channel(2..4)?,
            channel(4..6)?,
            255,
        )),
        8 => Some(UiRgbaColor::from_u8(
            channel(0..2)?,
            channel(2..4)?,
            channel(4..6)?,
            channel(6..8)?,
        )),
        _ => None,
    }
}

fn dimension_value(values: &BTreeMap<String, Value>, key: &str) -> Option<StyleDimension> {
    let value = value(values, key)?;
    match value {
        Value::Integer(value) => Some(StyleDimension::Fixed(*value as f32)),
        Value::Float(value) => Some(StyleDimension::Fixed(*value as f32)),
        Value::String(raw) => parse_dimension(raw),
        _ => None,
    }
}

fn parse_dimension(raw: &str) -> Option<StyleDimension> {
    let value = raw.trim();
    if value.eq_ignore_ascii_case("auto") {
        Some(StyleDimension::Auto)
    } else if value.eq_ignore_ascii_case("fill") || value.eq_ignore_ascii_case("full") {
        Some(StyleDimension::Fill)
    } else if let Ok(number) = value.parse::<f32>() {
        Some(StyleDimension::Fixed(number))
    } else if !value.is_empty() {
        Some(StyleDimension::Style(value.to_string()))
    } else {
        None
    }
}

fn parse_button_variant(raw: &str) -> Option<ButtonVariant> {
    let value = raw.trim();
    if value.eq_ignore_ascii_case("default") {
        Some(ButtonVariant::Default)
    } else if value.eq_ignore_ascii_case("text") {
        Some(ButtonVariant::Text)
    } else if ["contained", "primary", "filled"]
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
    {
        Some(ButtonVariant::Contained)
    } else if ["outlined", "outline"]
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
    {
        Some(ButtonVariant::Outlined)
    } else {
        None
    }
}

fn parse_button_color(raw: &str) -> Option<ButtonColor> {
    let value = raw.trim();
    for (names, color) in [
        (&["default"][..], ButtonColor::Default),
        (&["inherit"][..], ButtonColor::Inherit),
        (&["primary", "accent"][..], ButtonColor::Primary),
        (&["secondary"][..], ButtonColor::Secondary),
        (&["success"][..], ButtonColor::Success),
        (&["error", "danger"][..], ButtonColor::Error),
        (&["info"][..], ButtonColor::Info),
        (&["warning"][..], ButtonColor::Warning),
    ] {
        if names
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
        {
            return Some(color);
        }
    }
    if value.starts_with('#') {
        parse_hex_color(value).map(ButtonColor::Custom)
    } else if value.is_empty() {
        None
    } else {
        Some(ButtonColor::Style(value.to_string()))
    }
}

fn parse_button_size(raw: &str) -> Option<ButtonSize> {
    let value = raw.trim();
    for (names, size) in [
        (&["small", "compact"][..], ButtonSize::Small),
        (&["medium", "default"][..], ButtonSize::Medium),
        (&["large", "prominent"][..], ButtonSize::Large),
    ] {
        if names
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
        {
            return Some(size);
        }
    }
    None
}

fn parse_icon_placement(raw: &str) -> Option<ButtonIconPlacement> {
    let value = raw.trim();
    for (names, placement) in [
        (&["none"][..], ButtonIconPlacement::None),
        (
            &["start", "before", "leading"][..],
            ButtonIconPlacement::Start,
        ),
        (&["end", "after", "trailing"][..], ButtonIconPlacement::End),
        (
            &["icon_only", "icon-only", "only"][..],
            ButtonIconPlacement::IconOnly,
        ),
    ] {
        if names
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
        {
            return Some(placement);
        }
    }
    None
}

fn parse_interaction_state(raw: &str) -> Option<ButtonInteractionState> {
    let value = raw.trim();
    for (names, state) in [
        (&["normal", "default"][..], ButtonInteractionState::Normal),
        (&["hover", "hovered"][..], ButtonInteractionState::Hover),
        (
            &["pressed", "press", "active"][..],
            ButtonInteractionState::Pressed,
        ),
        (&["focused", "focus"][..], ButtonInteractionState::Focused),
        (&["disabled"][..], ButtonInteractionState::Disabled),
        (&["loading"][..], ButtonInteractionState::Loading),
    ] {
        if names
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
        {
            return Some(state);
        }
    }
    None
}
