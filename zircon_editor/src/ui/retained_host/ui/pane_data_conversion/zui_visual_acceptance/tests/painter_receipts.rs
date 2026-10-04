use super::{merge_into_geometry, merge_runtime_fingerprints};
use serde_json::json;

fn geometry() -> serde_json::Value {
    json!({
        "semanticAudit": {"complete": true, "expectedNodeCount": 1},
        "layout": {"semanticNodes": [{
            "nodeId": "generated-title",
            "sourcePath": "main.zui",
            "sourceNodeId": "title",
            "instancePath": "[]",
            "controlId": "title",
            "effectiveStyle": {"complete": false, "properties": {}},
            "styleInventory": {"complete": false},
        }]},
        "nodes": [{
            "nodeId": "generated-title",
            "sourcePath": "main.zui",
            "sourceNodeId": "title",
            "instancePath": "[]",
            "controlId": "title",
        }],
    })
}

fn painter() -> serde_json::Value {
    json!({
        "styles": [{
            "nodeId": "generated-title",
            "sourcePath": "main.zui",
            "sourceNodeId": "title",
            "instancePath": "[]",
            "controlId": "title",
            "effectiveStyle": {"complete": true, "properties": {"opacity": 1.0}},
            "styleInventory": {"version": 1, "complete": true},
        }],
        "assetAudit": {"complete": true, "resources": []},
        "runtimeAssetFingerprints": [["assets/font.ttf", "a"]],
        "issues": [],
    })
}

#[test]
fn painter_receipts_join_on_authored_identity_and_keep_generated_ids() {
    let mut geometry = geometry();
    assert!(merge_into_geometry(&mut geometry, &painter()).is_empty());
    let node = &geometry["layout"]["semanticNodes"][0];
    assert_eq!(node["nodeId"], "generated-title");
    assert_eq!(node["sourceNodeId"], "title");
    assert_eq!(node["effectiveStyle"]["complete"], true);
    assert_eq!(geometry["assetAudit"]["complete"], true);
    assert_eq!(geometry["semanticAudit"]["complete"], true);
}

#[test]
fn missing_painter_owner_fails_geometry_audit_closed() {
    let mut geometry = geometry();
    let mut painter = painter();
    painter["styles"] = json!([]);
    let issues = merge_into_geometry(&mut geometry, &painter);
    assert_eq!(issues.len(), 1);
    assert_eq!(geometry["semanticAudit"]["complete"], false);
    assert_eq!(
        geometry["layout"]["semanticNodes"][0]["styleInventory"]["complete"],
        false
    );
}

#[test]
fn runtime_fingerprints_merge_fonts_and_media_by_repo_path() {
    let paint = json!({"runtimeAssetFingerprints": [["assets/icon.svg", "a".repeat(64)]]});
    let text = json!({"runtimeAssetFingerprints": [
        ["assets/font.ttf", "b".repeat(64)], ["assets/icon.svg", "a".repeat(64)]
    ]});
    assert_eq!(
        serde_json::Value::Array(merge_runtime_fingerprints(&paint, &text).0),
        json!([
            ["assets/font.ttf", "b".repeat(64)],
            ["assets/icon.svg", "a".repeat(64)]
        ])
    );
}

#[test]
fn runtime_fingerprint_conflicts_are_diagnosed_and_removed() {
    let paint = json!({"runtimeAssetFingerprints": [["assets/icon.svg", "a".repeat(64)]]});
    let text = json!({"runtimeAssetFingerprints": [["assets/icon.svg", "b".repeat(64)]]});
    let (fingerprints, issues) = merge_runtime_fingerprints(&paint, &text);
    assert!(fingerprints.is_empty());
    assert_eq!(issues.len(), 1);
}
