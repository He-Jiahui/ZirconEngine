use std::collections::BTreeMap;
use std::sync::Arc;

use toml::Value;
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonInteractionState, ButtonVariant, StyleDimension, UiStyleColor,
};
use zircon_runtime_interface::ui::v2::UiV2ResolvedStyle;

use crate::ui::style::{
    resolve_button_style_from_values, ButtonStyleFields, ButtonVariantProperty, StyleField,
    StyleSheetScope,
};

#[test]
fn material_button_style_resolves_near_scope_over_far_scope() {
    let far = scope([
        ("button_variant", "contained"),
        ("button_color", "secondary"),
    ]);
    let near = scope([("button_variant", "outlined"), ("button_color", "primary")]);
    let style =
        ButtonStyleFields::default().resolve(&[Arc::downgrade(&far), Arc::downgrade(&near)]);

    assert_eq!(style.variant, ButtonVariant::Outlined);
    assert_eq!(style.color, ButtonColor::Primary);
}

#[test]
fn material_button_style_uses_override_before_cascade_and_normalizes_default_variant() {
    let scope = scope([("button_variant", "contained"), ("button_color", "warning")]);
    let fields = ButtonStyleFields {
        variant: StyleField::<ButtonVariantProperty>::override_value(ButtonVariant::Default),
        ..ButtonStyleFields::default()
    };

    let style = fields.resolve(&[Arc::downgrade(&scope)]);

    assert_eq!(style.variant, ButtonVariant::Text);
    assert_eq!(style.color, ButtonColor::Warning);
}

#[test]
fn material_button_style_skips_expired_weak_scopes_and_uses_defaults() {
    let expired = {
        let scope = scope([("button_variant", "contained")]);
        Arc::downgrade(&scope)
    };

    let style = ButtonStyleFields::default().resolve(&[expired]);

    assert_eq!(style.variant, ButtonVariant::Text);
    assert_eq!(style.color, ButtonColor::Primary);
    assert_eq!(style.width, StyleDimension::Auto);
}

#[test]
fn material_button_style_resolves_typed_values_from_v2_style_values() {
    let values = BTreeMap::from([
        (
            "button_variant".to_string(),
            Value::String("DEFAULT".to_string()),
        ),
        (
            "button_color".to_string(),
            Value::String("#0c2238cc".to_string()),
        ),
        (
            "button_size".to_string(),
            Value::String("compact".to_string()),
        ),
        (
            "icon_placement".to_string(),
            Value::String("leading".to_string()),
        ),
        ("pressed".to_string(), Value::Boolean(true)),
        (
            "background".to_string(),
            Value::String("#102030".to_string()),
        ),
        (
            "fg".to_string(),
            Value::String("material.primary".to_string()),
        ),
        (
            "border".to_string(),
            Value::Table(toml::map::Map::from_iter([
                (
                    "color".to_string(),
                    Value::String("transparent".to_string()),
                ),
                ("width".to_string(), Value::Float(2.0)),
                ("radius".to_string(), Value::Float(12.0)),
            ])),
        ),
        ("width".to_string(), Value::String("fill".to_string())),
        ("height".to_string(), Value::Integer(40)),
        ("opacity".to_string(), Value::Float(1.5)),
    ]);

    let style = resolve_button_style_from_values(&values);

    assert_eq!(style.variant, ButtonVariant::Text);
    assert!(
        matches!(style.color, ButtonColor::Custom(color) if color.to_u8() == [12, 34, 56, 204])
    );
    assert_eq!(style.interaction_state, ButtonInteractionState::Pressed);
    assert_eq!(style.width, StyleDimension::Fill);
    assert_eq!(style.height, StyleDimension::Fixed(40.0));
    assert!(
        matches!(style.element.background_color, Some(UiStyleColor::Rgba(color)) if color.to_u8() == [16, 32, 48, 255])
    );
    assert_eq!(
        style.element.foreground_color,
        Some(UiStyleColor::Role("material.primary".to_string()))
    );
    assert_eq!(style.element.border_color, Some(UiStyleColor::Transparent));
    assert_eq!(style.element.border_width, 2.0);
    assert_eq!(style.element.corner_radius, 12.0);
    assert_eq!(style.element.opacity, 1.0);
}

fn scope<'a, const N: usize>(values: [(&'a str, &'a str); N]) -> Arc<StyleSheetScope> {
    Arc::new(StyleSheetScope::new(UiV2ResolvedStyle {
        self_values: values
            .into_iter()
            .map(|(key, value)| (key.to_string(), Value::String(value.to_string())))
            .collect(),
        slot: BTreeMap::new(),
    }))
}
