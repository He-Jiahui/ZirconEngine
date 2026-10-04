use super::{evaluate, semantic_node_is_paint_visible};
use serde_json::{json, Value};

fn complete_style_inventory() -> serde_json::Value {
    json!({
        "version": 1,
        "complete": true,
        "properties": {
            "foregroundColor": "rgba(255,255,255,1)",
            "backgroundColor": "rgba(0,0,0,1)",
            "borderColor": "rgba(17,17,17,1)",
            "borderWidth": 0,
            "borderRadius": 0,
            "opacity": 1,
            "boxShadow": {"complete": true, "layers": []}
        }
    })
}

#[test]
fn missing_painter_receipts_keep_capture_pending() {
    let geometry = json!({
        "case": {"id": "normal"}, "caseSha256": "sha", "coordinateSpace": "logical",
        "semanticAudit": {"complete": true, "expectedNodeCount": 1},
        "assetAudit": {"complete": false, "resources": []},
        "layout": {"semanticNodes": [{
            "nodeId": "generated-title", "sourceNodeId": "title", "sourcePath": "main.zui",
            "instancePath": "[]", "controlId": "Title",
            "parentNodeId": null, "parentSourcePath": null,
            "parentSourceNodeId": null, "parentInstancePath": null,
            "component": "Label", "visible": true, "detached": false,
            "clip": false, "clipBounds": null,
            "text": "Workbench", "bounds": {"x": 0, "y": 0, "width": 100, "height": 20},
            "effectiveStyle": {"complete": false, "properties": {}}
        }]}
    });
    let text = json!({
        "case": {"id": "normal"}, "caseSha256": "sha", "coordinateSpace": "logical",
        "nodes": [], "fontAudit": {"loaded": false}, "runtimeAssetFingerprints": []
    });
    let raw = json!({"schema": "dev.zircon.editor.text-paint-evidence"});

    let readiness = evaluate(&geometry, &[], &text, &raw);
    assert_eq!(readiness["status"], "pending");
    assert!(readiness["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason
            .as_str()
            .unwrap()
            .contains("no native measured text record")));
    assert!(readiness["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason.as_str().unwrap().contains("asset readiness")));

    let mut unmeasured_text = text;
    unmeasured_text["nodes"] = json!([{
        "sourcePath": "main.zui", "sourceNodeId": "title", "instancePath": "[]",
        "text": "Workbench", "layout": {"lines": []}, "fonts": []
    }]);
    let readiness = evaluate(&geometry, &[], &unmeasured_text, &raw);
    assert_eq!(readiness["status"], "pending");
    assert!(readiness["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason.as_str().unwrap().contains("logical line records")));
}

#[test]
fn complete_status_requires_linked_text_style_and_font_receipts() {
    let font_sha = "a".repeat(64);
    let active_tokens = json!({"id": "test-token-set"});
    let active_tokens_sha = super::super::contract::canonical_hash(&active_tokens).unwrap();
    let style = json!({
        "complete": true,
        "properties": {
            "foregroundColor": "rgba(255,255,255,1)",
            "backgroundColor": "rgba(0,0,0,1)",
            "borderColor": "rgba(17,17,17,1)",
            "borderWidth": 0, "borderRadius": 0, "opacity": 1
        }
    });
    let geometry = json!({
        "case": {"id": "normal", "data": {"workbenchPresentation": {}}},
        "caseSha256": "sha", "coordinateSpace": "logical",
        "hostOverlayAudit": {"complete": true, "windows": []},
        "semanticAudit": {"complete": true, "expectedNodeCount": 1},
        "assetAudit": {"complete": true, "resources": []},
        "sourceIdentityProvenance": {
        "sourceMapFingerprint": ["map.json", font_sha.clone()],
        "entrySourceFingerprint": ["main.zui", font_sha.clone()],
        "sources": [
            ["main.zui", font_sha.clone()],
            ["zircon_editor/assets/ui/editor/host/workbench_shell.zui", font_sha.clone()],
            ["zircon_editor/assets/ui/editor/windows/workbench_window.zui", font_sha.clone()]
        ],
        "runtimeLoadedSources": {
            "complete": true,
            "documentIds": [
                "res://ui/editor/host/workbench_shell.zui",
                "res://ui/editor/windows/workbench_window.zui"
            ],
            "files": [
                {
                    "assetId": "res://ui/editor/host/workbench_shell.zui",
                    "sourcePath": "zircon_editor/assets/ui/editor/host/workbench_shell.zui",
                    "resourceUri": "res://ui/editor/host/workbench_shell.zui",
                    "physicalPath": "C:/fixture/assets/ui/editor/host/workbench_shell.zui",
                    "sha256": font_sha.clone(),
                    "catalogMatches": true,
                    "currentFileMatches": true
                },
                {
                    "assetId": "res://ui/editor/windows/workbench_window.zui",
                    "sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui",
                    "resourceUri": "res://ui/editor/windows/workbench_window.zui",
                    "physicalPath": "C:/fixture/assets/ui/editor/windows/workbench_window.zui",
                    "sha256": font_sha.clone(),
                    "catalogMatches": true,
                    "currentFileMatches": true
                }
            ],
            "unresolvedImports": []
        }
    },
    "dependencyFingerprints": [
        {"sourcePath": "zircon_editor/assets/ui/editor/host/workbench_shell.zui", "sha256": font_sha.clone()},
        {"sourcePath": "zircon_editor/assets/ui/editor/windows/workbench_window.zui", "sha256": font_sha.clone()}
    ],
    "activeDesignTokens": {
        "complete": true,
        "sha256": active_tokens_sha,
        "tokens": active_tokens
    },
    "layout": {"semanticNodes": [{
        "nodeId": "generated-title", "sourceNodeId": "title", "sourcePath": "main.zui",
        "instancePath": "[]", "controlId": "Title",
        "parentNodeId": null, "parentSourcePath": null,
        "parentSourceNodeId": null, "parentInstancePath": null,
        "component": "Label", "visible": true, "detached": false,
        "clip": false, "clipBounds": null,
        "text": "Workbench", "bounds": {"x": 0, "y": 0, "width": 100, "height": 20},
        "effectiveStyle": style,
        "styleInventory": complete_style_inventory()
    }]}
    });
    let text = json!({
        "case": {"id": "normal"}, "caseSha256": "sha", "coordinateSpace": "logical",
        "nodes": [{
            "nodeId": "generated-title", "sourceNodeId": "title",
            "sourcePath": "main.zui", "instancePath": "[]",
            "text": "Workbench", "layout": {
                "font_size": 14, "font_family": "Fira Sans", "font_weight": 400,
                "line_height": 20, "letter_spacing": 0,
                "lines": [{"text": "Workbench", "frame": {"x": 0, "y": 0, "width": 80, "height": 20}}]},
            "fonts": [{"familyName": "Fira Sans", "postScriptName": "FiraSans-Regular",
                "faceIndex": 0, "glyphCount": 9, "resourcePath": "font.ttf", "sha256": font_sha.clone()}]
        }],
        "runtimeAssetFingerprints": [["font.ttf", font_sha]],
        "fontAudit": {"loaded": true}
    });
    let raw = json!({"schema": "dev.zircon.editor.text-paint-evidence"});

    let readiness = evaluate(&geometry, &[], &text, &raw);
    assert_eq!(readiness["status"], "passed");
    assert_eq!(readiness["reasons"], json!([]));

    let mut missing_loaded_sources = geometry.clone();
    missing_loaded_sources["sourceIdentityProvenance"]
        .as_object_mut()
        .unwrap()
        .remove("runtimeLoadedSources");
    let missing_loaded_sources_readiness = evaluate(&missing_loaded_sources, &[], &text, &raw);
    assert_eq!(missing_loaded_sources_readiness["status"], "pending");
    assert!(missing_loaded_sources_readiness["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason
            .as_str()
            .unwrap()
            .contains("runtime-loaded source closure")));

    let mut missing_token_snapshot = geometry.clone();
    missing_token_snapshot
        .as_object_mut()
        .unwrap()
        .remove("activeDesignTokens");
    let missing_token_readiness = evaluate(&missing_token_snapshot, &[], &text, &raw);
    assert_eq!(missing_token_readiness["status"], "pending");

    let mut offscreen_geometry = geometry.clone();
    offscreen_geometry["windowMetrics"] = json!({
        "logicalSize": {"width": 100, "height": 100}
    });
    offscreen_geometry["layout"]["semanticNodes"][0]["bounds"]["x"] = json!(110);
    offscreen_geometry["layout"]["semanticNodes"][0]["effectiveStyle"] = json!({
        "complete": false, "properties": {}
    });
    offscreen_geometry["layout"]["semanticNodes"][0]["styleInventory"]["complete"] = json!(false);
    offscreen_geometry["layout"]["semanticNodes"][0]["styleInventory"]["properties"]["boxShadow"] =
        Value::Null;
    let mut no_offscreen_text = text.clone();
    no_offscreen_text["nodes"] = json!([]);
    let offscreen_readiness = evaluate(&offscreen_geometry, &[], &no_offscreen_text, &raw);
    assert_eq!(offscreen_readiness["status"], "passed");

    let mut missing_overlay_audit = geometry.clone();
    missing_overlay_audit
        .as_object_mut()
        .unwrap()
        .remove("hostOverlayAudit");
    let readiness = evaluate(&missing_overlay_audit, &[], &text, &raw);
    assert_eq!(readiness["status"], "pending");
    assert!(readiness["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason.as_str().unwrap().contains("host floating overlays")));
}

#[test]
fn paint_visibility_includes_viewport_and_effective_clip_intersection() {
    let geometry = json!({
        "windowMetrics": {"logicalSize": {"width": 100, "height": 80}}
    });
    let node = json!({
        "visible": true, "bounds": {"x": 90, "y": 10, "width": 20, "height": 20},
        "clip": false, "clipBounds": null
    });
    assert!(semantic_node_is_paint_visible(&node, &geometry));

    let mut outside_viewport = node.clone();
    outside_viewport["bounds"]["x"] = json!(101);
    assert!(!semantic_node_is_paint_visible(
        &outside_viewport,
        &geometry
    ));

    let mut outside_clip = node.clone();
    outside_clip["clip"] = json!(true);
    outside_clip["clipBounds"] = json!({
        "x": 0, "y": 0, "width": 80, "height": 80
    });
    assert!(!semantic_node_is_paint_visible(&outside_clip, &geometry));

    let mut zero_width_clip = node.clone();
    zero_width_clip["clip"] = json!(true);
    zero_width_clip["clipBounds"] = json!({
        "x": 100, "y": 0, "width": 0, "height": 80
    });
    assert!(!semantic_node_is_paint_visible(&zero_width_clip, &geometry));

    let mut zero_height_clip = node;
    zero_height_clip["clip"] = json!(true);
    zero_height_clip["clipBounds"] = json!({
        "x": 0, "y": 20, "width": 100, "height": 0
    });
    assert!(!semantic_node_is_paint_visible(
        &zero_height_clip,
        &geometry
    ));
}
