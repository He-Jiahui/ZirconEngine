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
    let scope = Arc::new(StyleSheetScope::new(UiV2ResolvedStyle {
        self_values: values.clone(),
        slot: BTreeMap::new(),
    }));
    ButtonStyleFields::default().resolve(&[Arc::downgrade(&scope)])
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

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        color_value(sheet, "background")
            .or_else(|| color_value(sheet, "background_color"))
            .map(Some)
    }
}

impl StyleProperty for ForegroundColorProperty {
    type Value = Option<UiStyleColor>;

    fn default_value() -> Self::Value {
        None
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        color_value(sheet, "foreground")
            .or_else(|| color_value(sheet, "foreground_color"))
            .or_else(|| color_value(sheet, "fg"))
            .or_else(|| color_value(sheet, "color"))
            .map(Some)
    }
}

impl StyleProperty for BorderColorProperty {
    type Value = Option<UiStyleColor>;

    fn default_value() -> Self::Value {
        None
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        color_value(sheet, "border")
            .or_else(|| color_value(sheet, "border_color"))
            .or_else(|| color_value(sheet, "outline"))
            .map(Some)
    }
}

impl StyleProperty for BorderWidthProperty {
    type Value = f32;

    fn default_value() -> Self::Value {
        0.0
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        table_number(sheet, "border", "width").or_else(|| number_value(sheet, "border_width"))
    }
}

impl StyleProperty for CornerRadiusProperty {
    type Value = f32;

    fn default_value() -> Self::Value {
        0.0
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        table_number(sheet, "border", "radius")
            .or_else(|| number_value(sheet, "corner_radius"))
            .or_else(|| number_value(sheet, "radius"))
    }
}

impl StyleProperty for WidthProperty {
    type Value = StyleDimension;

    fn default_value() -> Self::Value {
        StyleDimension::Auto
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        dimension_value(sheet, "width")
    }
}

impl StyleProperty for HeightProperty {
    type Value = StyleDimension;

    fn default_value() -> Self::Value {
        StyleDimension::Auto
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        dimension_value(sheet, "height")
    }
}

impl StyleProperty for OpacityProperty {
    type Value = f32;

    fn default_value() -> Self::Value {
        1.0
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        number_value(sheet, "opacity")
    }
}

impl StyleProperty for ButtonVariantProperty {
    type Value = ButtonVariant;

    fn default_value() -> Self::Value {
        ButtonVariant::Text
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        string_value(sheet, "button_variant")
            .or_else(|| string_value(sheet, "variant"))
            .and_then(parse_button_variant)
    }
}

impl StyleProperty for ButtonColorProperty {
    type Value = ButtonColor;

    fn default_value() -> Self::Value {
        ButtonColor::Primary
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        string_value(sheet, "button_color")
            .or_else(|| string_value(sheet, "color"))
            .and_then(parse_button_color)
    }
}

impl StyleProperty for ButtonSizeProperty {
    type Value = ButtonSize;

    fn default_value() -> Self::Value {
        ButtonSize::Medium
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        string_value(sheet, "button_size")
            .or_else(|| string_value(sheet, "size"))
            .or_else(|| string_value(sheet, "density"))
            .and_then(parse_button_size)
    }
}

impl StyleProperty for ButtonIconPlacementProperty {
    type Value = ButtonIconPlacement;

    fn default_value() -> Self::Value {
        ButtonIconPlacement::None
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        string_value(sheet, "icon_placement")
            .or_else(|| string_value(sheet, "button_icon_placement"))
            .and_then(parse_icon_placement)
    }
}

impl StyleProperty for ButtonInteractionStateProperty {
    type Value = ButtonInteractionState;

    fn default_value() -> Self::Value {
        ButtonInteractionState::Normal
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        string_value(sheet, "button_interaction_state")
            .or_else(|| string_value(sheet, "interaction_state"))
            .and_then(parse_interaction_state)
            .or_else(|| {
                bool_value(sheet, "loading")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Loading)
            })
            .or_else(|| {
                bool_value(sheet, "disabled")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Disabled)
            })
            .or_else(|| {
                bool_value(sheet, "pressed")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Pressed)
            })
            .or_else(|| {
                bool_value(sheet, "focused")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Focused)
            })
            .or_else(|| {
                bool_value(sheet, "hovered")
                    .filter(|value| *value)
                    .map(|_| ButtonInteractionState::Hover)
            })
    }
}

impl StyleProperty for ButtonLoadingProperty {
    type Value = bool;

    fn default_value() -> Self::Value {
        false
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        bool_value(sheet, "loading")
    }
}

impl StyleProperty for ButtonDisabledProperty {
    type Value = bool;

    fn default_value() -> Self::Value {
        false
    }

    fn extract(sheet: &UiV2ResolvedStyle) -> Option<Self::Value> {
        bool_value(sheet, "disabled")
    }
}

fn value<'a>(sheet: &'a UiV2ResolvedStyle, key: &str) -> Option<&'a Value> {
    sheet.self_values.get(key)
}

fn string_value(sheet: &UiV2ResolvedStyle, key: &str) -> Option<String> {
    value(sheet, key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn bool_value(sheet: &UiV2ResolvedStyle, key: &str) -> Option<bool> {
    value(sheet, key).and_then(Value::as_bool)
}

fn number_value(sheet: &UiV2ResolvedStyle, key: &str) -> Option<f32> {
    number_from_value(value(sheet, key)?)
}

fn table_number(sheet: &UiV2ResolvedStyle, table: &str, key: &str) -> Option<f32> {
    value(sheet, table)
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

fn color_value(sheet: &UiV2ResolvedStyle, key: &str) -> Option<UiStyleColor> {
    let value = value(sheet, key)?;
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

fn dimension_value(sheet: &UiV2ResolvedStyle, key: &str) -> Option<StyleDimension> {
    let value = value(sheet, key)?;
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

fn parse_button_variant(raw: String) -> Option<ButtonVariant> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "default" => Some(ButtonVariant::Default),
        "text" => Some(ButtonVariant::Text),
        "contained" | "primary" | "filled" => Some(ButtonVariant::Contained),
        "outlined" | "outline" => Some(ButtonVariant::Outlined),
        _ => None,
    }
}

fn parse_button_color(raw: String) -> Option<ButtonColor> {
    let value = raw.trim();
    let normalized = value.to_ascii_lowercase();
    match normalized.as_str() {
        "default" => Some(ButtonColor::Default),
        "inherit" => Some(ButtonColor::Inherit),
        "primary" | "accent" => Some(ButtonColor::Primary),
        "secondary" => Some(ButtonColor::Secondary),
        "success" => Some(ButtonColor::Success),
        "error" | "danger" => Some(ButtonColor::Error),
        "info" => Some(ButtonColor::Info),
        "warning" => Some(ButtonColor::Warning),
        _ if value.starts_with('#') => parse_hex_color(value).map(ButtonColor::Custom),
        _ if !value.is_empty() => Some(ButtonColor::Style(value.to_string())),
        _ => None,
    }
}

fn parse_button_size(raw: String) -> Option<ButtonSize> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "small" | "compact" => Some(ButtonSize::Small),
        "medium" | "default" => Some(ButtonSize::Medium),
        "large" | "prominent" => Some(ButtonSize::Large),
        _ => None,
    }
}

fn parse_icon_placement(raw: String) -> Option<ButtonIconPlacement> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "none" => Some(ButtonIconPlacement::None),
        "start" | "before" | "leading" => Some(ButtonIconPlacement::Start),
        "end" | "after" | "trailing" => Some(ButtonIconPlacement::End),
        "icon_only" | "icon-only" | "only" => Some(ButtonIconPlacement::IconOnly),
        _ => None,
    }
}

fn parse_interaction_state(raw: String) -> Option<ButtonInteractionState> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "normal" | "default" => Some(ButtonInteractionState::Normal),
        "hover" | "hovered" => Some(ButtonInteractionState::Hover),
        "pressed" | "press" | "active" => Some(ButtonInteractionState::Pressed),
        "focused" | "focus" => Some(ButtonInteractionState::Focused),
        "disabled" => Some(ButtonInteractionState::Disabled),
        "loading" => Some(ButtonInteractionState::Loading),
        _ => None,
    }
}
