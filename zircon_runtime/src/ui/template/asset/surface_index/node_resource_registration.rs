use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::{
    template::{UiResourceFallbackMode, UiResourceFallbackPolicy, UiResourceKind},
    tree::UiTree,
};

use super::{UiAssetSurfaceIndex, UiAssetSurfaceNodeResourceRegistrationReport};

impl UiAssetSurfaceIndex {
    pub fn record_tree_node_resources(
        &mut self,
        tree: &UiTree,
    ) -> UiAssetSurfaceNodeResourceRegistrationReport {
        let tree_id = tree.tree_id.clone();
        self.remove_surface_node_assets(&tree_id);
        let mut report = UiAssetSurfaceNodeResourceRegistrationReport {
            tree_id: tree_id.clone(),
            ..Default::default()
        };

        for (node_id, node) in &tree.nodes {
            let mut collector = NodeResourceCollector::default();
            if let Some(metadata) = &node.template_metadata {
                collector.collect_map(&metadata.attributes, "attributes");
                collector.collect_map(&metadata.slot_attributes, "slot_attributes");
                collector.collect_map(&metadata.style_overrides, "style_overrides");
            }

            let resources = collector.finish();
            if resources.is_empty() {
                report.nodes_without_resources.push(*node_id);
                continue;
            }

            report.nodes_registered += 1;
            report.resource_uris_registered += resources.len();
            self.record_node_assets(tree_id.clone(), *node_id, resources);
        }

        report
    }
}

/// Projects resource URIs from already-instantiated retained metadata.
///
/// Compile-time document collection owns strict schema diagnostics. Runtime metadata can contain
/// ordinary TOML tables, so this projection stays tolerant and ignores non-resource values.
#[derive(Default)]
struct NodeResourceCollector {
    uris: Vec<String>,
    seen: BTreeSet<String>,
}

impl NodeResourceCollector {
    fn collect_map(&mut self, values: &BTreeMap<String, Value>, root: &str) {
        for (key, value) in values {
            self.collect_value(value, &format!("{root}.{key}"));
        }
    }

    fn collect_value(&mut self, value: &Value, path: &str) {
        match value {
            Value::String(uri) if has_supported_resource_scheme(uri) => {
                let kind = UiResourceKind::infer_from_path_and_uri(path, uri);
                self.push_resource_uri(kind, uri, &UiResourceFallbackPolicy::default());
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    self.collect_value(value, &format!("{path}[{index}]"));
                }
            }
            Value::Table(table) if is_resource_table(table) => {
                self.collect_resource_table(table, path);
            }
            Value::Table(table) => {
                for (key, value) in table {
                    self.collect_value(value, &format!("{path}.{key}"));
                }
            }
            _ => {}
        }
    }

    fn collect_resource_table(&mut self, table: &toml::map::Map<String, Value>, path: &str) {
        let Some(Value::String(uri)) = table.get("uri") else {
            return;
        };
        if !has_supported_resource_scheme(uri) {
            return;
        }
        let kind = table
            .get("kind")
            .and_then(Value::as_str)
            .and_then(resource_kind_from_name)
            .unwrap_or_else(|| UiResourceKind::infer_from_path_and_uri(path, uri));
        let fallback = fallback_policy_from_table(table);
        self.push_resource_uri(kind, uri, &fallback);
    }

    fn push_resource_uri(
        &mut self,
        kind: UiResourceKind,
        uri: &str,
        fallback: &UiResourceFallbackPolicy,
    ) {
        let trimmed = uri.trim();
        if !trimmed.is_empty() && self.seen.insert(trimmed.to_string()) {
            self.uris.push(trimmed.to_string());
        }
        if let Some(fallback_uri) = fallback.uri.as_deref() {
            let fallback_uri = fallback_uri.trim();
            if !fallback_uri.is_empty()
                && has_supported_resource_scheme(fallback_uri)
                && (fallback.mode != UiResourceFallbackMode::Placeholder
                    || UiResourceKind::infer_from_path_and_uri("", fallback_uri) == kind)
                && self.seen.insert(fallback_uri.to_string())
            {
                self.uris.push(fallback_uri.to_string());
            }
        }
    }

    fn finish(self) -> Vec<String> {
        self.uris
    }
}

fn is_resource_table(table: &toml::map::Map<String, Value>) -> bool {
    table.contains_key("uri")
        || table.get("kind").is_some_and(
            |kind| matches!(kind, Value::String(kind) if resource_kind_from_name(kind).is_some()),
        )
        || matches!(table.get("fallback"), Some(Value::Table(_)))
}

fn fallback_policy_from_table(table: &toml::map::Map<String, Value>) -> UiResourceFallbackPolicy {
    let Some(Value::Table(fallback)) = table.get("fallback") else {
        return UiResourceFallbackPolicy::default();
    };

    let mode = fallback
        .get("mode")
        .and_then(Value::as_str)
        .map(fallback_mode_from_name)
        .unwrap_or_default();
    let uri = fallback
        .get("uri")
        .and_then(Value::as_str)
        .map(str::to_string);
    UiResourceFallbackPolicy { mode, uri }
}

fn fallback_mode_from_name(value: &str) -> UiResourceFallbackMode {
    match value {
        "placeholder" => UiResourceFallbackMode::Placeholder,
        "optional" => UiResourceFallbackMode::Optional,
        _ => UiResourceFallbackMode::None,
    }
}

fn resource_kind_from_name(value: &str) -> Option<UiResourceKind> {
    match value {
        "font" => Some(UiResourceKind::Font),
        "image" => Some(UiResourceKind::Image),
        "media" => Some(UiResourceKind::Media),
        "generic_asset" => Some(UiResourceKind::GenericAsset),
        _ => None,
    }
}

fn has_supported_resource_scheme(uri: &str) -> bool {
    uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://")
}
