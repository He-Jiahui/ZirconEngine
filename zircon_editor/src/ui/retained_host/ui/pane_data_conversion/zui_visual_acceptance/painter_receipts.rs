use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

type SourceIdentity = (String, String, String);

pub(super) fn merge_into_geometry(geometry: &mut Value, painter: &Value) -> Vec<String> {
    let mut issues = Vec::new();
    let Some(styles) = painter.get("styles").and_then(Value::as_array) else {
        issues.push("raw painter evidence has no style receipt array".to_owned());
        geometry["assetAudit"] = incomplete_asset_audit();
        mark_semantic_incomplete(geometry);
        return issues;
    };

    let mut by_identity = BTreeMap::<SourceIdentity, &Value>::new();
    for style in styles {
        let Some(identity) = source_identity(style) else {
            issues.push("painter style receipt has incomplete authored identity".to_owned());
            continue;
        };
        if by_identity.insert(identity.clone(), style).is_some() {
            issues.push(format!(
                "painter style identity is duplicated: {}#{}@{}",
                identity.0, identity.1, identity.2
            ));
        }
    }

    let Some(semantic_nodes) = geometry
        .pointer_mut("/layout/semanticNodes")
        .and_then(Value::as_array_mut)
    else {
        issues.push("geometry has no semantic node array for painter receipts".to_owned());
        geometry["assetAudit"] = incomplete_asset_audit();
        mark_semantic_incomplete(geometry);
        return issues;
    };

    let mut consumed = BTreeSet::new();
    for node in semantic_nodes {
        let Some(identity) = source_identity(node) else {
            issues.push("semantic node has incomplete authored identity".to_owned());
            continue;
        };
        let Some(style) = by_identity.get(&identity).copied() else {
            issues.push(format!(
                "semantic node {}#{}@{} has no painter style receipt",
                identity.0, identity.1, identity.2
            ));
            continue;
        };
        consumed.insert(identity.clone());
        if node.get("nodeId") != style.get("nodeId") {
            issues.push(format!(
                "painter generated node id differs for {}#{}@{}",
                identity.0, identity.1, identity.2
            ));
            continue;
        }
        if node.get("controlId") != style.get("controlId") {
            issues.push(format!(
                "painter control id differs for {}#{}@{}",
                identity.0, identity.1, identity.2
            ));
            continue;
        }
        node["effectiveStyle"] = style
            .get("effectiveStyle")
            .cloned()
            .unwrap_or_else(|| json!({"complete": false, "properties": {}}));
        node["styleInventory"] = style
            .get("styleInventory")
            .cloned()
            .unwrap_or_else(|| incomplete_style_inventory());
    }

    for identity in by_identity.keys() {
        if !consumed.contains(identity) {
            issues.push(format!(
                "painter style receipt has no semantic node: {}#{}@{}",
                identity.0, identity.1, identity.2
            ));
        }
    }
    merge_host_node_style_fields(geometry, &by_identity, &mut issues);

    match painter.get("assetAudit") {
        Some(audit) if audit.is_object() => geometry["assetAudit"] = audit.clone(),
        _ => {
            issues.push("raw painter evidence has no media asset audit".to_owned());
            geometry["assetAudit"] = incomplete_asset_audit();
        }
    }
    if let Some(painter_issues) = painter.get("issues").and_then(Value::as_array) {
        issues.extend(
            painter_issues
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned),
        );
    }
    if !issues.is_empty() {
        mark_semantic_incomplete(geometry);
    }
    issues
}

pub(super) fn merge_runtime_fingerprints(paint: &Value, text: &Value) -> (Vec<Value>, Vec<String>) {
    let mut pairs = BTreeMap::<String, String>::new();
    let mut conflicts = BTreeSet::new();
    let mut issues = Vec::new();
    for evidence in [paint, text] {
        let Some(fingerprints) = evidence
            .get("runtimeAssetFingerprints")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for fingerprint in fingerprints {
            let Some(pair) = fingerprint.as_array() else {
                issues.push("runtime asset fingerprint is not an array".to_owned());
                continue;
            };
            let (Some(path), Some(sha256)) = (
                pair.first().and_then(Value::as_str),
                pair.get(1).and_then(Value::as_str),
            ) else {
                issues.push("runtime asset fingerprint is not a string pair".to_owned());
                continue;
            };
            if pair.len() != 2 || path.is_empty() || sha256.len() != 64 {
                issues.push("runtime asset fingerprint has invalid path or SHA-256".to_owned());
                continue;
            }
            if conflicts.contains(path) {
                continue;
            }
            match pairs.get(path) {
                Some(previous) if previous != sha256 => {
                    pairs.remove(path);
                    conflicts.insert(path.to_owned());
                    issues.push(format!(
                        "runtime asset path {path} has conflicting captured hashes"
                    ));
                }
                None => {
                    pairs.insert(path.to_owned(), sha256.to_owned());
                }
                _ => {}
            }
        }
    }
    (
        pairs
            .into_iter()
            .map(|(path, sha256)| json!([path, sha256]))
            .collect(),
        issues,
    )
}

fn merge_host_node_style_fields(
    geometry: &mut Value,
    by_identity: &BTreeMap<SourceIdentity, &Value>,
    issues: &mut Vec<String>,
) {
    let Some(host_nodes) = geometry.get_mut("nodes").and_then(Value::as_array_mut) else {
        return;
    };
    for node in host_nodes {
        let Some(identity) = source_identity(node) else {
            continue;
        };
        let Some(style) = by_identity.get(&identity).copied() else {
            continue;
        };
        if node.get("nodeId") != style.get("nodeId") {
            issues.push(format!(
                "host node generated id differs from painter for {}#{}@{}",
                identity.0, identity.1, identity.2
            ));
            continue;
        }
        node["effectiveStyle"] = style
            .get("effectiveStyle")
            .cloned()
            .unwrap_or_else(|| json!({"complete": false, "properties": {}}));
        node["styleInventory"] = style
            .get("styleInventory")
            .cloned()
            .unwrap_or_else(|| incomplete_style_inventory());
    }
}

fn source_identity(value: &Value) -> Option<SourceIdentity> {
    let source_path = value.get("sourcePath")?.as_str()?;
    let source_node_id = value.get("sourceNodeId")?.as_str()?;
    let instance_path = value.get("instancePath")?.as_str()?;
    (!source_path.is_empty() && !source_node_id.is_empty() && !instance_path.is_empty()).then(
        || {
            (
                source_path.to_owned(),
                source_node_id.to_owned(),
                instance_path.to_owned(),
            )
        },
    )
}

fn mark_semantic_incomplete(geometry: &mut Value) {
    if let Some(audit) = geometry.get_mut("semanticAudit") {
        audit["complete"] = json!(false);
    }
}

fn incomplete_asset_audit() -> Value {
    json!({"complete": false, "resources": []})
}

fn incomplete_style_inventory() -> Value {
    json!({
        "version": 1,
        "complete": false,
        "properties": {
            "foregroundColor": null,
            "backgroundColor": null,
            "borderColor": null,
            "borderWidth": null,
            "borderRadius": null,
            "opacity": null,
            "boxShadow": null,
        }
    })
}

#[cfg(test)]
#[path = "tests/painter_receipts.rs"]
mod tests;
