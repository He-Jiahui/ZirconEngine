use toml::Value;

use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiInvalidationDiagnostic, UiInvalidationDiagnosticSeverity, UiNodeDefinition,
    UiSelector, UiSelectorToken,
};

pub const LARGE_DOCUMENT_NODE_WARNING_THRESHOLD: usize = 1000;
pub const NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD: usize = 250;
pub const BROAD_SELECTOR_WARNING_THRESHOLD: usize = 50;

pub fn collect_invalidation_diagnostics(
    document: &UiAssetDocument,
) -> Vec<UiInvalidationDiagnostic> {
    let mut diagnostics = Vec::new();
    let node_count = document.iter_nodes().count();
    if node_count >= LARGE_DOCUMENT_NODE_WARNING_THRESHOLD {
        diagnostics.push(warning(
            "large_document",
            format!(
                "ui asset contains {node_count} nodes; incremental invalidation should be used"
            ),
        ));
    }

    let broad_selector_count = document
        .stylesheets
        .iter()
        .flat_map(|stylesheet| stylesheet.rules.iter())
        .filter(|rule| selector_is_broad(&rule.selector))
        .count();
    if broad_selector_count >= BROAD_SELECTOR_WARNING_THRESHOLD {
        diagnostics.push(warning(
            "broad_selector",
            format!(
                "ui asset contains {broad_selector_count} broad selector rules that can increase style invalidation work"
            ),
        ));
    }

    for node in document
        .iter_nodes()
        .filter(|node| node_has_non_virtualized_scroll_child_pressure(node))
    {
        diagnostics.push(warning(
            "non_virtualized_scroll_children",
            format!(
                "ui node `{}` has {} direct children in a ScrollableBox without virtualization",
                node.node_id,
                node.children.len()
            ),
        ));
    }

    diagnostics
}

fn warning(code: impl Into<String>, message: impl Into<String>) -> UiInvalidationDiagnostic {
    UiInvalidationDiagnostic {
        code: code.into(),
        severity: UiInvalidationDiagnosticSeverity::Warning,
        message: message.into(),
    }
}

fn node_has_non_virtualized_scroll_child_pressure(node: &UiNodeDefinition) -> bool {
    node.children.len() >= NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD
        && node_is_scrollable_box(node)
        && !node_declares_virtualization(node)
}

fn node_is_scrollable_box(node: &UiNodeDefinition) -> bool {
    node.widget_type.as_deref() == Some("ScrollableBox")
        || node
            .layout
            .as_ref()
            .and_then(|layout| layout.get("container"))
            .and_then(Value::as_table)
            .and_then(|container| container.get("kind"))
            .and_then(Value::as_str)
            == Some("ScrollableBox")
}

fn node_declares_virtualization(node: &UiNodeDefinition) -> bool {
    node.layout
        .as_ref()
        .and_then(|layout| layout.get("container"))
        .and_then(Value::as_table)
        .is_some_and(|container| container.contains_key("virtualization"))
}

fn selector_is_broad(selector: &str) -> bool {
    UiSelector::parse(selector).ok().is_some_and(|selector| {
        selector.segments.iter().all(|segment| {
            segment
                .tokens
                .iter()
                .all(|token| matches!(token, UiSelectorToken::Type(_) | UiSelectorToken::Host))
        })
    })
}
