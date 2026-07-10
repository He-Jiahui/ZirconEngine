use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiChildMount, UiNodeDefinition, UiStyleDeclarationBlock,
};

pub(super) fn collect_resource_uris(document: &UiAssetDocument) -> Vec<String> {
    let mut uris = Vec::new();
    for reference in &document.imports.resources {
        push_uri(&reference.uri, &mut uris);
        if let Some(uri) = reference.fallback.uri.as_deref() {
            push_uri(uri, &mut uris);
        }
    }
    for value in document.tokens.values() {
        collect_value(value, &mut uris);
    }
    if let Some(root) = &document.root {
        collect_node(root, &mut uris);
    }
    for component in document.components.values() {
        collect_node(&component.root, &mut uris);
    }
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            collect_declaration_block(&rule.set, &mut uris);
        }
    }
    uris
}

fn collect_node(node: &UiNodeDefinition, uris: &mut Vec<String>) {
    collect_values(node.props.values(), uris);
    collect_values(node.params.values(), uris);
    if let Some(layout) = &node.layout {
        collect_values(layout.values(), uris);
    }
    collect_declaration_block(&node.style_overrides, uris);
    for child in &node.children {
        collect_child(child, uris);
    }
}

fn collect_child(child: &UiChildMount, uris: &mut Vec<String>) {
    collect_values(child.slot.values(), uris);
    collect_node(&child.node, uris);
}

fn collect_declaration_block(block: &UiStyleDeclarationBlock, uris: &mut Vec<String>) {
    collect_values(block.self_values.values(), uris);
    collect_values(block.slot.values(), uris);
}

fn collect_values<'a>(values: impl Iterator<Item = &'a Value>, uris: &mut Vec<String>) {
    for value in values {
        collect_value(value, uris);
    }
}

fn collect_value(value: &Value, uris: &mut Vec<String>) {
    match value {
        Value::String(uri) => push_uri(uri, uris),
        Value::Array(values) => collect_values(values.iter(), uris),
        Value::Table(table) => collect_values(table.values(), uris),
        _ => {}
    }
}

fn push_uri(uri: &str, uris: &mut Vec<String>) {
    if uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://") {
        uris.push(uri.to_string());
    }
}
