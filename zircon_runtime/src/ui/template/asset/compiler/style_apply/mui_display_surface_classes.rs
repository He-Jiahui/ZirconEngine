use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, bool_from_attributes_any, map_attribute,
    map_from_attributes, mui_slot_name, pascal_case, string_attribute_any,
    string_attribute_any_prefer_non_default, string_from_attributes_any, string_from_map,
};

pub(super) fn append_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "Alert" => append_alert_classes(node, prefix),
        "AppBar" => append_app_bar_classes(node, prefix),
        "Avatar" => append_avatar_classes(node, prefix),
        "CardActionArea" => append_card_action_area_classes(node, prefix),
        "CardActions" => append_card_actions_classes(node, prefix),
        "CardMedia" => append_card_media_classes(node, prefix),
        "Chip" => append_chip_classes(node, prefix),
        "Divider" => append_divider_classes(node, prefix),
        "Icon" | "SvgIcon" => append_icon_classes(node, prefix),
        "List" => append_list_classes(node, prefix),
        "Paper" => append_paper_classes(node, prefix),
        "Skeleton" => append_skeleton_classes(node, prefix),
        "Snackbar" => append_snackbar_classes(node, prefix),
        "Table" => append_table_classes(node, prefix),
        "Toolbar" => append_toolbar_classes(node, prefix),
        "Typography" => append_typography_classes(node, prefix),
        _ => return false,
    }
    true
}

pub(super) fn append_slot_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) -> bool {
    match (owner_component, slot_name) {
        ("Alert", "icon" | "message" | "action" | "closeButton" | "closeIcon") => {
            append_alert_slot_classes(child, slot_name)
        }
        ("Badge", "badge") => append_badge_slot_classes(child, owner_attributes),
        ("Chip", "avatar" | "icon" | "label" | "deleteIcon") => {
            append_chip_slot_classes(child, slot_name)
        }
        ("Divider", "wrapper") => append_divider_wrapper_slot_classes(child, owner_attributes),
        _ => return false,
    }
    true
}

fn append_icon_classes(node: &mut UiTemplateNode, prefix: &str) {
    let color = string_attribute_any_prefer_non_default(
        node,
        &[("color", "inherit"), ("mui_color", "primary")],
    )
    .unwrap_or_else(|| "inherit".to_string());
    if color != "inherit" {
        append_class(
            &mut node.classes,
            format!("{prefix}-color{}", pascal_case(&color)),
        );
    }

    let font_size = string_attribute_any_prefer_non_default(
        node,
        &[("fontSize", "medium"), ("font_size", "medium")],
    )
    .unwrap_or_else(|| "medium".to_string());
    append_class(
        &mut node.classes,
        format!("{prefix}-fontSize{}", pascal_case(&font_size)),
    );
}

fn append_alert_classes(node: &mut UiTemplateNode, prefix: &str) {
    let severity = string_attribute_any_prefer_non_default(node, &[("severity", "success")])
        .unwrap_or_else(|| "success".to_string());
    let color =
        string_attribute_any_prefer_non_default(node, &[("color", ""), ("severity", "success")])
            .unwrap_or_else(|| severity.clone());

    append_class(
        &mut node.classes,
        format!("{prefix}-color{}", pascal_case(&color)),
    );
    append_alert_component_variant(node, &severity, &color);
}

fn append_alert_component_variant(node: &mut UiTemplateNode, severity: &str, color: &str) {
    let mut tokens = node
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string());
    append_variant_token(&mut tokens, &variant);
    append_variant_token(&mut tokens, color);
    append_variant_token(&mut tokens, &format!("color{}", pascal_case(color)));
    if alert_has_visible_icon(node) {
        append_variant_token(&mut tokens, "hasIcon");
    }
    if alert_has_action(node) {
        append_variant_token(&mut tokens, "hasAction");
    }
    if alert_has_close_action(node) {
        append_variant_token(&mut tokens, "hasCloseAction");
    }
    if !severity.eq_ignore_ascii_case(color) {
        append_variant_token(&mut tokens, severity);
    }
    if !tokens.is_empty() {
        let _ = node
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

fn alert_has_visible_icon(node: &UiTemplateNode) -> bool {
    !matches!(node.attributes.get("icon"), Some(Value::Boolean(false)))
        && !matches!(
            node.attributes.get("show_icon"),
            Some(Value::Boolean(false))
        )
        && !matches!(node.attributes.get("showIcon"), Some(Value::Boolean(false)))
}

fn alert_has_action(node: &UiTemplateNode) -> bool {
    alert_has_non_empty_attribute(node, &["action"])
        || alert_has_slot(node, "action")
        || alert_has_close_action(node)
}

fn alert_has_close_action(node: &UiTemplateNode) -> bool {
    alert_has_non_empty_attribute(node, &["onClose", "on_close"])
        || alert_has_slot(node, "closeButton")
        || alert_has_slot(node, "closeIcon")
}

fn alert_has_slot(node: &UiTemplateNode, slot_name: &str) -> bool {
    node.children
        .iter()
        .any(|child| mui_slot_name(child).as_deref() == Some(slot_name))
        || node
            .slots
            .get(slot_name)
            .is_some_and(|children| !children.is_empty())
}

fn alert_has_non_empty_attribute(node: &UiTemplateNode, names: &[&str]) -> bool {
    names
        .iter()
        .any(|name| node.attributes.get(*name).is_some_and(non_empty_mui_value))
}

fn append_app_bar_classes(node: &mut UiTemplateNode, prefix: &str) {
    let color = string_attribute_any(node, &["color", "mui_color"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "primary".to_string());
    let position = string_attribute_any(node, &["position"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "fixed".to_string());

    append_class(
        &mut node.classes,
        format!("{prefix}-color{}", pascal_case(&color)),
    );
    append_class(
        &mut node.classes,
        format!("{prefix}-position{}", pascal_case(&position)),
    );
    if position == "fixed" {
        append_class(&mut node.classes, "mui-fixed".to_string());
    }
}

fn append_avatar_classes(node: &mut UiTemplateNode, prefix: &str) {
    let has_image = string_attribute_any(node, &["src", "srcSet", "image", "source", "media"])
        .is_some_and(|value| !value.is_empty());
    if !has_image {
        append_class(&mut node.classes, format!("{prefix}-colorDefault"));
    }
}

fn append_card_action_area_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["focusVisible", "focus_visible", "focused"]) {
        append_class(&mut node.classes, format!("{prefix}-focusVisible"));
        for class_name in super::string_list_attribute(node, "focusVisibleClassName") {
            append_class(&mut node.classes, class_name);
        }
    }
}

fn append_card_actions_classes(node: &mut UiTemplateNode, prefix: &str) {
    if !bool_attribute_any(node, &["disableSpacing", "disable_spacing"]) {
        append_class(&mut node.classes, format!("{prefix}-spacing"));
    }
}

fn append_card_media_classes(node: &mut UiTemplateNode, prefix: &str) {
    let component = string_attribute_any(node, &["component"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "div".to_string());
    if matches!(
        component.as_str(),
        "video" | "audio" | "picture" | "iframe" | "img"
    ) {
        append_class(&mut node.classes, format!("{prefix}-media"));
    }
    if matches!(component.as_str(), "picture" | "img") {
        append_class(&mut node.classes, format!("{prefix}-img"));
    }
}

fn append_chip_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["clickable"]) {
        append_class(&mut node.classes, format!("{prefix}-clickable"));
    }
    if chip_is_deletable(node) {
        append_class(&mut node.classes, format!("{prefix}-deletable"));
    }
    if bool_attribute(node, "disabled") {
        append_class(&mut node.classes, format!("{prefix}-disabled"));
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible", "focused"]) {
        append_class(&mut node.classes, format!("{prefix}-focusVisible"));
    }
    append_chip_component_variant(node);
}

fn append_chip_component_variant(node: &mut UiTemplateNode) {
    let mut tokens = node
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "filled".to_string());
    let size = string_attribute_any(node, &["size", "mui_size"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "medium".to_string());
    let color = string_attribute_any(node, &["color", "mui_color"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());

    for token in [
        variant.clone(),
        size.clone(),
        format!("size{}", pascal_case(&size)),
        color.clone(),
        format!("color{}", pascal_case(&color)),
    ] {
        append_variant_token(&mut tokens, &token);
    }
    if bool_attribute_any(node, &["clickable"]) {
        append_variant_token(&mut tokens, "clickable");
    }
    if chip_is_deletable(node) {
        append_variant_token(&mut tokens, "deletable");
        append_variant_token(&mut tokens, "hasDeleteIcon");
    }
    if bool_attribute_any(node, &["focusVisible", "focus_visible", "focused"]) {
        append_variant_token(&mut tokens, "focusVisible");
    }
    if chip_has_slot(node, "avatar") {
        append_variant_token(&mut tokens, "hasAvatar");
    }
    if chip_has_slot(node, "icon") {
        append_variant_token(&mut tokens, "hasIcon");
    }
    if !tokens.is_empty() {
        let _ = node
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

fn chip_has_slot(node: &UiTemplateNode, slot_name: &str) -> bool {
    node.children
        .iter()
        .any(|child| mui_slot_name(child).as_deref() == Some(slot_name))
        || node
            .slots
            .get(slot_name)
            .is_some_and(|children| !children.is_empty())
}

fn chip_is_deletable(node: &UiTemplateNode) -> bool {
    bool_attribute_any(node, &["deletable"])
        || chip_has_non_empty_attribute(node, &["onDelete", "on_delete"])
}

fn chip_has_non_empty_attribute(node: &UiTemplateNode, names: &[&str]) -> bool {
    names
        .iter()
        .any(|name| node.attributes.get(*name).is_some_and(non_empty_mui_value))
}

fn non_empty_mui_value(value: &Value) -> bool {
    match value {
        Value::String(value) => !value.trim().is_empty(),
        Value::Boolean(value) => *value,
        Value::Array(values) => !values.is_empty(),
        Value::Table(values) => !values.is_empty(),
        Value::Integer(_) | Value::Float(_) | Value::Datetime(_) => true,
    }
}

fn append_divider_classes(node: &mut UiTemplateNode, prefix: &str) {
    let orientation = string_attribute_any(node, &["orientation"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "horizontal".to_string());
    if bool_attribute(node, "absolute") {
        append_class(&mut node.classes, format!("{prefix}-absolute"));
    }
    if orientation == "vertical" {
        append_class(&mut node.classes, format!("{prefix}-vertical"));
    }
    if bool_attribute_any(node, &["flexItem", "flex_item"]) {
        append_class(&mut node.classes, format!("{prefix}-flexItem"));
    }
    if divider_has_children(node) {
        append_class(&mut node.classes, format!("{prefix}-withChildren"));
    }
    let text_align = string_attribute_any(node, &["textAlign", "text_align"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "center".to_string());
    if orientation != "vertical" && matches!(text_align.as_str(), "left" | "right") {
        append_class(
            &mut node.classes,
            format!("{prefix}-textAlign{}", pascal_case(&text_align)),
        );
    }
}

fn divider_has_children(node: &UiTemplateNode) -> bool {
    string_attribute_any(node, &["text", "label"]).is_some_and(|value| !value.is_empty())
        || node
            .children
            .iter()
            .any(|child| mui_slot_name(child).as_deref() == Some("wrapper"))
        || node
            .slots
            .get("wrapper")
            .is_some_and(|children| !children.is_empty())
}

fn append_list_classes(node: &mut UiTemplateNode, prefix: &str) {
    if !bool_attribute_any(node, &["disablePadding", "disable_padding"]) {
        append_class(&mut node.classes, format!("{prefix}-padding"));
    }
    if bool_attribute(node, "dense") {
        append_class(&mut node.classes, format!("{prefix}-dense"));
    }
    let has_subheader = string_attribute_any(node, &["subheader"])
        .is_some_and(|value| !value.is_empty())
        || node
            .children
            .iter()
            .any(|child| mui_slot_name(child).as_deref() == Some("subheader"))
        || node
            .slots
            .get("subheader")
            .is_some_and(|children| !children.is_empty());
    if has_subheader {
        append_class(&mut node.classes, format!("{prefix}-subheader"));
    }
}

fn append_paper_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "elevation".to_string());
    append_class(&mut node.classes, format!("{prefix}-{variant}"));

    if !bool_attribute_any(node, &["square"]) {
        append_class(&mut node.classes, format!("{prefix}-rounded"));
    }
    if variant == "elevation" {
        let elevation = super::int_attribute_any(node, &["elevation"]).unwrap_or(1);
        append_class(&mut node.classes, format!("{prefix}-elevation{elevation}"));
    }
}

fn append_snackbar_classes(node: &mut UiTemplateNode, prefix: &str) {
    let (vertical, horizontal) = snackbar_anchor_origin(node);
    append_class(
        &mut node.classes,
        format!(
            "{prefix}-anchorOrigin{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal)
        ),
    );
}

fn append_skeleton_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant = skeleton_variant(node);
    append_class(&mut node.classes, format!("{prefix}-{variant}"));

    let animation = skeleton_animation(node);
    if let Some(animation) = &animation {
        append_class(&mut node.classes, format!("{prefix}-{animation}"));
    }

    let has_children = skeleton_has_children(node);
    if has_children {
        append_class(&mut node.classes, format!("{prefix}-withChildren"));
        if !has_non_empty_attribute(node, &["width"]) {
            append_class(&mut node.classes, format!("{prefix}-fitContent"));
        }
        if !has_non_empty_attribute(node, &["height"]) {
            append_class(&mut node.classes, format!("{prefix}-heightAuto"));
        }
    }

    append_skeleton_component_variant(node, &variant, animation.as_deref(), has_children);
}

fn skeleton_variant(node: &UiTemplateNode) -> String {
    string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "text".to_string())
}

fn skeleton_animation(node: &UiTemplateNode) -> Option<String> {
    match node.attributes.get("animation") {
        Some(Value::Boolean(false)) => None,
        Some(value) => value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != "false")
            .map(ToOwned::to_owned),
        None => Some("pulse".to_string()),
    }
}

fn skeleton_has_children(node: &UiTemplateNode) -> bool {
    bool_attribute_any(
        node,
        &[
            "hasChildren",
            "has_children",
            "withChildren",
            "with_children",
        ],
    ) || !node.children.is_empty()
        || node.slots.values().any(|children| !children.is_empty())
}

fn append_skeleton_component_variant(
    node: &mut UiTemplateNode,
    variant: &str,
    animation: Option<&str>,
    has_children: bool,
) {
    let mut tokens = node
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    append_variant_token(&mut tokens, variant);
    if let Some(animation) = animation {
        append_variant_token(&mut tokens, animation);
    }
    if has_children {
        append_variant_token(&mut tokens, "withChildren");
        if !has_non_empty_attribute(node, &["width"]) {
            append_variant_token(&mut tokens, "fitContent");
        }
        if !has_non_empty_attribute(node, &["height"]) {
            append_variant_token(&mut tokens, "heightAuto");
        }
    }
    if !tokens.is_empty() {
        let _ = node
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

pub(super) fn append_skeleton_child_metadata(child: &mut UiTemplateNode) {
    let mut tokens = child
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    append_variant_token(&mut tokens, "muiSkeletonChild");
    if !tokens.is_empty() {
        let _ = child
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

fn has_non_empty_attribute(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| {
        node.attributes.get(*name).is_some_and(|value| match value {
            Value::String(value) => !value.trim().is_empty(),
            Value::Array(values) => !values.is_empty(),
            Value::Table(values) => !values.is_empty(),
            Value::Integer(_) | Value::Float(_) | Value::Boolean(_) | Value::Datetime(_) => true,
        })
    })
}

fn snackbar_anchor_origin(node: &UiTemplateNode) -> (String, String) {
    let anchor_origin = map_attribute(node, "anchorOrigin");
    let vertical = string_from_map(&anchor_origin, "vertical")
        .or_else(|| string_attribute_any(node, &["anchor_origin_vertical"]))
        .unwrap_or_else(|| "bottom".to_string());
    let horizontal = string_from_map(&anchor_origin, "horizontal")
        .or_else(|| string_attribute_any(node, &["anchor_origin_horizontal"]))
        .unwrap_or_else(|| "left".to_string());
    (vertical, horizontal)
}

fn append_toolbar_classes(node: &mut UiTemplateNode, prefix: &str) {
    let variant = string_attribute_any(node, &["variant", "mui_variant"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "regular".to_string());
    if !bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-gutters"));
    }
    append_class(&mut node.classes, format!("{prefix}-{variant}"));
}

fn append_table_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["stickyHeader", "sticky_header"]) {
        append_class(&mut node.classes, format!("{prefix}-stickyHeader"));
    }
}

fn append_typography_classes(node: &mut UiTemplateNode, prefix: &str) {
    if let Some(align) = string_attribute_any(node, &["align"])
        .filter(|value| !value.is_empty() && value != "inherit")
    {
        append_class(
            &mut node.classes,
            format!("{prefix}-align{}", pascal_case(&align)),
        );
    }
    if bool_attribute_any(node, &["gutterBottom", "gutter_bottom"]) {
        append_class(&mut node.classes, format!("{prefix}-gutterBottom"));
    }
    if bool_attribute_any(node, &["noWrap", "no_wrap"]) {
        append_class(&mut node.classes, format!("{prefix}-noWrap"));
    }
}

fn append_badge_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    let prefix = "MuiBadge";
    let variant = string_from_attributes_any(owner_attributes, &["variant", "mui_variant"])
        .unwrap_or_else(|| "standard".to_string());
    let color = string_from_attributes_any(owner_attributes, &["color"])
        .unwrap_or_else(|| "default".to_string());
    let overlap = string_from_attributes_any(owner_attributes, &["overlap"])
        .unwrap_or_else(|| "rectangular".to_string());
    let (vertical, horizontal) = badge_anchor_origin(owner_attributes);
    let invisible = badge_slot_invisible(owner_attributes, &variant);

    append_class(&mut child.classes, format!("{prefix}-{variant}"));
    append_class(
        &mut child.classes,
        format!(
            "{prefix}-anchorOrigin{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal)
        ),
    );
    append_class(
        &mut child.classes,
        format!(
            "{prefix}-anchorOrigin{}{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal),
            pascal_case(&overlap)
        ),
    );
    append_class(
        &mut child.classes,
        format!("{prefix}-overlap{}", pascal_case(&overlap)),
    );
    if color != "default" {
        append_class(
            &mut child.classes,
            format!("{prefix}-color{}", pascal_case(&color)),
        );
    }
    if invisible {
        append_class(&mut child.classes, format!("{prefix}-invisible"));
    }
    append_badge_slot_component_variant(
        child,
        &variant,
        &color,
        &overlap,
        &vertical,
        &horizontal,
        invisible,
    );
}

fn badge_slot_invisible(owner_attributes: &BTreeMap<String, Value>, variant: &str) -> bool {
    if bool_from_attributes_any(owner_attributes, &["invisible"]) {
        return true;
    }

    let content = owner_attributes
        .get("badgeContent")
        .or_else(|| owner_attributes.get("badge_content"))
        .or_else(|| owner_attributes.get("value_text"));
    if variant != "dot" && !content.is_some_and(badge_content_present) {
        return true;
    }

    content.is_some_and(|value| {
        badge_content_is_numeric_zero(value)
            && !bool_from_attributes_any(owner_attributes, &["showZero", "show_zero"])
    })
}

fn badge_content_present(value: &Value) -> bool {
    match value {
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Table(values) => !values.is_empty(),
        Value::Integer(_) | Value::Float(_) | Value::Boolean(_) | Value::Datetime(_) => true,
    }
}

fn badge_content_is_numeric_zero(value: &Value) -> bool {
    match value {
        Value::Integer(value) => *value == 0,
        Value::Float(value) => *value == 0.0,
        _ => false,
    }
}

fn append_badge_slot_component_variant(
    child: &mut UiTemplateNode,
    variant: &str,
    color: &str,
    overlap: &str,
    vertical: &str,
    horizontal: &str,
    invisible: bool,
) {
    let mut tokens = child
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    for token in [
        "muiBadgeSlot".to_string(),
        variant.to_string(),
        color.to_string(),
        overlap.to_string(),
        vertical.to_string(),
        horizontal.to_string(),
        format!("overlap{}", pascal_case(overlap)),
        format!(
            "anchorOrigin{}{}",
            pascal_case(vertical),
            pascal_case(horizontal)
        ),
        format!(
            "anchorOrigin{}{}{}",
            pascal_case(vertical),
            pascal_case(horizontal),
            pascal_case(overlap)
        ),
    ] {
        append_variant_token(&mut tokens, &token);
    }
    if invisible {
        append_variant_token(&mut tokens, "invisible");
    }
    if !tokens.is_empty() {
        let _ = child
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

fn append_chip_slot_classes(child: &mut UiTemplateNode, slot_name: &str) {
    let mut tokens = child
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    append_variant_token(&mut tokens, "muiChipSlot");
    append_variant_token(&mut tokens, &format!("chipSlot{}", pascal_case(slot_name)));
    if !tokens.is_empty() {
        let _ = child
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

fn append_alert_slot_classes(child: &mut UiTemplateNode, slot_name: &str) {
    let mut tokens = child
        .attributes
        .get("component_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    append_variant_token(&mut tokens, "muiAlertSlot");
    append_variant_token(&mut tokens, &format!("alertSlot{}", pascal_case(slot_name)));
    if !tokens.is_empty() {
        let _ = child
            .attributes
            .insert("component_variant".to_string(), Value::String(tokens));
    }
}

fn append_variant_token(tokens: &mut String, token: &str) {
    if token.is_empty()
        || tokens
            .split_whitespace()
            .any(|existing| existing.eq_ignore_ascii_case(token))
    {
        return;
    }
    if !tokens.is_empty() {
        tokens.push(' ');
    }
    tokens.push_str(token);
}

fn append_divider_wrapper_slot_classes(
    child: &mut UiTemplateNode,
    owner_attributes: &BTreeMap<String, Value>,
) {
    if string_from_attributes_any(owner_attributes, &["orientation"]).as_deref() == Some("vertical")
    {
        append_class(&mut child.classes, "MuiDivider-wrapperVertical".to_string());
    }
}

fn badge_anchor_origin(owner_attributes: &BTreeMap<String, Value>) -> (String, String) {
    let anchor_origin = map_from_attributes(owner_attributes, "anchorOrigin");
    let vertical = string_from_map(&anchor_origin, "vertical")
        .or_else(|| string_from_attributes_any(owner_attributes, &["anchor_origin_vertical"]))
        .unwrap_or_else(|| "top".to_string());
    let horizontal = string_from_map(&anchor_origin, "horizontal")
        .or_else(|| string_from_attributes_any(owner_attributes, &["anchor_origin_horizontal"]))
        .unwrap_or_else(|| "right".to_string());
    (vertical, horizontal)
}
