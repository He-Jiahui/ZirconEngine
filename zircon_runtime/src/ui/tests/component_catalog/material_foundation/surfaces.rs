use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::assert_has_prop;
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_app_bar(registry);
    assert_paper(registry);
    assert_card(registry);
    assert_card_action_area(registry);
    assert_card_actions(registry);
    assert_card_content(registry);
    assert_card_header(registry);
    assert_card_media(registry);
    assert_toolbar(registry);
}

fn assert_app_bar(registry: &UiComponentDescriptorRegistry) {
    let app_bar = registry.descriptor("AppBar").expect("AppBar descriptor");
    assert_enum_options(
        app_bar,
        "position",
        &["absolute", "fixed", "relative", "static", "sticky"],
    );
    assert_enum_options(
        app_bar,
        "color",
        &[
            "default",
            "inherit",
            "primary",
            "secondary",
            "transparent",
            "error",
            "info",
            "success",
            "warning",
        ],
    );
    assert_eq!(
        app_bar
            .prop("position")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Enum("fixed".to_string())),
        "AppBar should default position to local MUI AppBar.js"
    );
    assert_eq!(
        app_bar
            .prop("elevation")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Float(4.0)),
        "AppBar should default elevation to local MUI AppBar.js"
    );
    for prop in ["enableColorOnDark", "square"] {
        assert_has_prop(app_bar, prop);
    }
}

fn assert_paper(registry: &UiComponentDescriptorRegistry) {
    let paper = registry.descriptor("Paper").expect("Paper descriptor");
    assert_enum_options(paper, "variant", &["elevation", "outlined"]);
    for prop in ["component", "elevation", "square"] {
        assert_has_prop(paper, prop);
    }
    assert_eq!(
        paper
            .prop("variant")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Enum("elevation".to_string())),
        "Paper should default variant to local MUI Paper.js"
    );
    assert_eq!(
        paper
            .prop("square")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Bool(false)),
        "Paper should default square to local MUI Paper.js"
    );
}

fn assert_card(registry: &UiComponentDescriptorRegistry) {
    let card = registry.descriptor("Card").expect("Card descriptor");
    assert_enum_options(card, "variant", &["elevation", "outlined"]);
    assert_has_prop(card, "raised");
    assert_eq!(
        card.prop("raised")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Bool(false)),
        "Card should default raised to local MUI Card.js"
    );
    for slot_name in ["media", "content", "actions"] {
        assert!(
            card.slot_schema.iter().any(|slot| slot.name == slot_name),
            "Card missing MUI composition slot `{slot_name}`"
        );
    }
}

fn assert_card_action_area(registry: &UiComponentDescriptorRegistry) {
    let card_action_area = registry
        .descriptor("CardActionArea")
        .expect("CardActionArea descriptor");
    assert_has_prop(card_action_area, "focusVisibleClassName");
    for slot_name in ["content", "focusHighlight"] {
        assert!(
            card_action_area
                .slot_schema
                .iter()
                .any(|slot| slot.name == slot_name),
            "CardActionArea missing MUI slot `{slot_name}`"
        );
    }
}

fn assert_card_actions(registry: &UiComponentDescriptorRegistry) {
    let card_actions = registry
        .descriptor("CardActions")
        .expect("CardActions descriptor");
    assert_eq!(
        card_actions
            .prop("disableSpacing")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Bool(false)),
        "CardActions should default disableSpacing to local MUI CardActions.js"
    );
}

fn assert_card_content(registry: &UiComponentDescriptorRegistry) {
    let card_content = registry
        .descriptor("CardContent")
        .expect("CardContent descriptor");
    assert_has_prop(card_content, "component");
    assert!(card_content.slot_schema("content").is_some());
}

fn assert_card_header(registry: &UiComponentDescriptorRegistry) {
    let card_header = registry
        .descriptor("CardHeader")
        .expect("CardHeader descriptor");
    for prop in [
        "component",
        "title",
        "subheader",
        "avatar",
        "action",
        "disableTypography",
    ] {
        assert_has_prop(card_header, prop);
    }
    for slot_name in ["avatar", "content", "title", "subheader", "action"] {
        assert!(
            card_header
                .slot_schema
                .iter()
                .any(|slot| slot.name == slot_name),
            "CardHeader missing MUI slot `{slot_name}`"
        );
    }
}

fn assert_card_media(registry: &UiComponentDescriptorRegistry) {
    let card_media = registry
        .descriptor("CardMedia")
        .expect("CardMedia descriptor");
    for prop in ["component", "image", "src"] {
        assert_has_prop(card_media, prop);
    }
}

fn assert_toolbar(registry: &UiComponentDescriptorRegistry) {
    let toolbar = registry.descriptor("Toolbar").expect("Toolbar descriptor");
    assert_enum_options(toolbar, "variant", &["regular", "dense"]);
    assert_eq!(
        toolbar
            .prop("variant")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Enum("regular".to_string())),
        "Toolbar should default variant to local MUI Toolbar.js"
    );
    assert_has_prop(toolbar, "disableGutters");
}
