use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiChildMount, UiNodeDefinition, UiStyleDeclarationBlock,
};

pub(super) fn visit_resource_uris<'a>(
    document: &'a UiAssetDocument,
    mut visit: impl FnMut(&'a str),
) {
    for reference in &document.imports.resources {
        visit_uri(&reference.uri, &mut visit);
        if let Some(uri) = reference.fallback.uri.as_deref() {
            visit_uri(uri, &mut visit);
        }
    }
    for value in document.tokens.values() {
        visit_value(value, &mut visit);
    }
    if let Some(root) = &document.root {
        visit_node(root, &mut visit);
    }
    for component in document.components.values() {
        visit_node(&component.root, &mut visit);
    }
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            visit_declaration_block(&rule.set, &mut visit);
        }
    }
}

fn visit_node<'a>(node: &'a UiNodeDefinition, visit: &mut impl FnMut(&'a str)) {
    visit_values(node.props.values(), visit);
    visit_values(node.params.values(), visit);
    if let Some(layout) = &node.layout {
        visit_values(layout.values(), visit);
    }
    visit_declaration_block(&node.style_overrides, visit);
    for child in &node.children {
        visit_child(child, visit);
    }
}

fn visit_child<'a>(child: &'a UiChildMount, visit: &mut impl FnMut(&'a str)) {
    visit_values(child.slot.values(), visit);
    visit_node(&child.node, visit);
}

fn visit_declaration_block<'a>(
    block: &'a UiStyleDeclarationBlock,
    visit: &mut impl FnMut(&'a str),
) {
    visit_values(block.self_values.values(), visit);
    visit_values(block.slot.values(), visit);
}

fn visit_values<'a>(values: impl Iterator<Item = &'a Value>, visit: &mut impl FnMut(&'a str)) {
    for value in values {
        visit_value(value, visit);
    }
}

fn visit_value<'a>(value: &'a Value, visit: &mut impl FnMut(&'a str)) {
    match value {
        Value::String(uri) => visit_uri(uri, visit),
        Value::Array(values) => visit_values(values.iter(), visit),
        Value::Table(table) => visit_values(table.values(), visit),
        _ => {}
    }
}

fn visit_uri<'a>(uri: &'a str, visit: &mut impl FnMut(&'a str)) {
    if uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://") {
        visit(uri);
    }
}

#[cfg(test)]
#[path = "tests/resource_references_optimization_tests.rs"]
mod optimization_tests;
