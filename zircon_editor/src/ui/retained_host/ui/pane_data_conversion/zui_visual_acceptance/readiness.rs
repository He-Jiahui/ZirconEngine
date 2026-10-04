use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

const REQUIRED_STYLE_PROPERTIES: &[&str] = &[
    "foregroundColor",
    "backgroundColor",
    "borderColor",
    "borderWidth",
    "borderRadius",
    "opacity",
];

const REQUIRED_STYLE_INVENTORY_PROPERTIES: &[&str] = &[
    "foregroundColor",
    "backgroundColor",
    "borderColor",
    "borderWidth",
    "borderRadius",
    "opacity",
    "boxShadow",
];

pub(super) fn evaluate(
    geometry: &Value,
    geometry_issues: &[String],
    text: &Value,
    raw_painter_text: &Value,
) -> Value {
    let mut reasons = Vec::new();
    let semantic_nodes = geometry
        .pointer("/layout/semanticNodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if geometry["coordinateSpace"] != "logical" {
        reasons.push("geometry is not in logical coordinates".to_owned());
    }
    if geometry.get("case").is_none() || geometry["caseSha256"].as_str().is_none() {
        reasons.push("geometry does not carry the catalog case and canonical hash".to_owned());
    }
    if geometry
        .pointer("/case/data/workbenchPresentation")
        .is_some()
        && (geometry["hostOverlayAudit"]["complete"].as_bool() != Some(true)
            || geometry["hostOverlayAudit"]["windows"].as_array().is_none())
    {
        reasons.push("workbench host floating overlays lack a complete audit".to_owned());
    }
    if geometry
        .pointer("/case/data/workbenchPresentation")
        .is_some()
    {
        if !runtime_loaded_sources_are_complete(
            geometry.pointer("/sourceIdentityProvenance/runtimeLoadedSources"),
            geometry.get("sourceIdentityProvenance"),
            geometry.get("dependencyFingerprints"),
        ) {
            reasons
                .push("workbench runtime-loaded source closure is incomplete or stale".to_owned());
        }
        if !active_design_tokens_are_complete(geometry.get("activeDesignTokens")) {
            reasons
                .push("workbench active design-token snapshot is incomplete or stale".to_owned());
        }
    }
    if semantic_nodes.is_empty() {
        reasons.push("geometry has no source-identified semantic nodes".to_owned());
    }
    if geometry["semanticAudit"]["complete"].as_bool() != Some(true)
        || geometry["semanticAudit"]["expectedNodeCount"].as_u64()
            != Some(semantic_nodes.len() as u64)
    {
        reasons.push(
            "geometry semantic audit is incomplete or does not match emitted nodes".to_owned(),
        );
    }
    if !source_provenance_is_complete(
        geometry.get("sourceIdentityProvenance"),
        &semantic_nodes,
        geometry.get("dependencyFingerprints"),
    ) {
        reasons.push("semantic source identities lack matching file fingerprints".to_owned());
    }
    for issue in geometry_issues {
        reasons.push(format!("geometry identity incomplete: {issue}"));
    }
    let mut semantic_ids = BTreeSet::new();
    for (index, node) in semantic_nodes.iter().enumerate() {
        let identity_complete = [
            "nodeId",
            "sourcePath",
            "sourceNodeId",
            "instancePath",
            "component",
        ]
        .iter()
        .all(|key| {
            node.get(*key)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
        });
        let has_parent = node.as_object().is_some_and(|object| {
            object.contains_key("parentNodeId")
                && object.contains_key("parentSourcePath")
                && object.contains_key("parentSourceNodeId")
                && object.contains_key("parentInstancePath")
        });
        let has_control_identity = node.get("controlId").is_some();
        let bounds_complete = ["x", "y", "width", "height"].iter().all(|key| {
            node.pointer(&format!("/bounds/{key}"))
                .and_then(Value::as_f64)
                .is_some()
        });
        if !identity_complete
            || !has_parent
            || !has_control_identity
            || !bounds_complete
            || !clip_is_well_formed(node)
            || node["visible"].as_bool().is_none()
            || node["detached"].as_bool().is_none()
            || !node
                .as_object()
                .is_some_and(|object| object.contains_key("text"))
        {
            reasons.push(format!(
                "semantic node {index} lacks required identity or logical geometry fields"
            ));
        }
        let source_key = (
            node["sourcePath"].as_str().unwrap_or_default().to_owned(),
            node["sourceNodeId"].as_str().unwrap_or_default().to_owned(),
            node["instancePath"].as_str().unwrap_or_default().to_owned(),
        );
        if !semantic_ids.insert(source_key.clone()) {
            reasons.push(format!(
                "semantic source identity is duplicated: {}#{}@{}",
                source_key.0, source_key.1, source_key.2
            ));
        }
        let paint_visible = semantic_node_is_paint_visible(node, geometry);
        if paint_visible && !style_is_complete(node.get("effectiveStyle")) {
            reasons.push(format!(
                "semantic node {index} has no complete resolved painter style receipt"
            ));
        }
        if paint_visible && !style_inventory_is_complete(node.get("styleInventory")) {
            reasons.push(format!(
                "semantic node {index} has an incomplete final style inventory"
            ));
        }
    }

    let expected_text_nodes = semantic_nodes
        .iter()
        .filter_map(|node| {
            (semantic_node_is_paint_visible(node, geometry))
                .then(|| node["text"].as_str())
                .flatten()
                .filter(|value| !value.is_empty())
                .map(|text| {
                    (
                        (
                            node["sourcePath"].as_str().unwrap_or_default().to_owned(),
                            node["sourceNodeId"].as_str().unwrap_or_default().to_owned(),
                            node["instancePath"].as_str().unwrap_or_default().to_owned(),
                        ),
                        text,
                    )
                })
        })
        .collect::<BTreeMap<_, _>>();
    let text_nodes = text.get("nodes").and_then(Value::as_array);
    let mut observed_text_nodes = BTreeSet::new();
    if let Some(text_nodes) = text_nodes {
        for node in text_nodes {
            let Some(source_path) = node.get("sourcePath").and_then(Value::as_str) else {
                reasons.push("native text node has no authored source path".to_owned());
                continue;
            };
            let Some(source_node_id) = node.get("sourceNodeId").and_then(Value::as_str) else {
                reasons.push("native text node has no authored source node id".to_owned());
                continue;
            };
            let Some(instance_path) = node.get("instancePath").and_then(Value::as_str) else {
                reasons.push(format!(
                    "native text node {source_node_id} has no authored instance path"
                ));
                continue;
            };
            let identity = (
                source_path.to_owned(),
                source_node_id.to_owned(),
                instance_path.to_owned(),
            );
            if !observed_text_nodes.insert(identity.clone()) {
                reasons.push(format!("native text node {source_node_id} is duplicated"));
                continue;
            }
            let Some(expected_text) = expected_text_nodes.get(&identity) else {
                reasons.push(format!(
                    "native text node {source_node_id} has no visible semantic text source"
                ));
                continue;
            };
            if node.get("text").and_then(Value::as_str) != Some(*expected_text) {
                reasons.push(format!(
                    "native text node {source_node_id} differs from semantic text"
                ));
            }
            let layout = &node["layout"];
            if layout["font_size"].as_f64().is_none() {
                reasons.push(format!(
                    "native text node {source_node_id} has no measured logical font size"
                ));
            }
            if layout["font_family"].as_str().map_or(true, str::is_empty)
                || layout.get("font_weight").map_or(true, Value::is_null)
                || layout["line_height"].as_f64().is_none()
                || layout["letter_spacing"].as_f64().is_none()
            {
                reasons.push(format!(
                    "native text node {source_node_id} has incomplete normalized font metrics"
                ));
            }
            let lines = layout["lines"].as_array();
            if lines.map_or(true, Vec::is_empty) {
                reasons.push(format!(
                    "native text node {source_node_id} has no measured logical line records"
                ));
            } else if let Some(lines) = lines {
                for line in lines {
                    let frame = &line["frame"];
                    let has_line_text = line.get("text").and_then(Value::as_str).is_some();
                    let has_frame = ["x", "y", "width", "height"]
                        .iter()
                        .all(|key| frame.get(*key).and_then(Value::as_f64).is_some());
                    if !has_line_text || !has_frame {
                        reasons.push(format!(
                            "native text node {source_node_id} has an incomplete line measurement"
                        ));
                        break;
                    }
                }
            }
            if !fonts_are_complete(node.get("fonts"), text.get("runtimeAssetFingerprints")) {
                reasons.push(format!(
                    "native text node {source_node_id} has no file-backed used-font receipt"
                ));
            }
        }
    } else if !expected_text_nodes.is_empty() {
        reasons.push("native text artifact has no logical node array".to_owned());
    }
    for node_id in expected_text_nodes.keys() {
        if !observed_text_nodes.contains(node_id) {
            reasons.push(format!(
                "visible semantic text node {node_id:?} has no native measured text record"
            ));
        }
    }
    if !expected_text_nodes.is_empty()
        && text
            .get("runtimeAssetFingerprints")
            .and_then(Value::as_array)
            .map_or(true, Vec::is_empty)
    {
        reasons
            .push("no file-backed runtime font fingerprints are linked to painted text".to_owned());
    }
    if !expected_text_nodes.is_empty()
        && text_nodes.map_or(true, |nodes| {
            nodes.iter().all(|node| {
                node.get("fonts")
                    .and_then(Value::as_array)
                    .map_or(true, Vec::is_empty)
            })
        })
    {
        reasons.push("raw painter font usage is not joined to semantic text nodes".to_owned());
    }
    if text.get("case").is_none()
        || text["caseSha256"].as_str().is_none()
        || text["coordinateSpace"] != "logical"
    {
        reasons.push(
            "native text artifact lacks the full catalog case or logical coordinate envelope"
                .to_owned(),
        );
    }
    if text["fontAudit"]["loaded"].as_bool() != Some(true) {
        reasons.push("native font audit does not prove loaded font assets".to_owned());
    }
    if !runtime_fingerprints_are_valid(text.get("runtimeAssetFingerprints")) {
        reasons.push("runtime asset fingerprints are not valid [path, sha256] pairs".to_owned());
    }
    if !asset_audit_is_complete(
        geometry.get("assetAudit"),
        text.get("runtimeAssetFingerprints"),
    ) {
        reasons
            .push("host painter did not provide complete file-backed asset readiness".to_owned());
    }
    if raw_painter_text["schema"].as_str().is_none() {
        reasons.push("raw native painter text evidence is missing".to_owned());
    }

    let complete = reasons.is_empty();
    json!({
        "status": if complete { "passed" } else { "pending" },
        "complete": complete,
        "reasons": reasons,
    })
}

fn runtime_fingerprints_are_valid(fingerprints: Option<&Value>) -> bool {
    let Some(fingerprints) = fingerprints.and_then(Value::as_array) else {
        return false;
    };
    let mut paths = BTreeSet::new();
    fingerprints.iter().all(|fingerprint| {
        let Some(parts) = fingerprint.as_array() else {
            return false;
        };
        if parts.len() != 2 {
            return false;
        }
        let path = parts[0].as_str().unwrap_or_default();
        let sha256 = parts[1].as_str().unwrap_or_default();
        !path.is_empty()
            && !path.contains(['\\', ':'])
            && path
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != "..")
            && sha256.len() == 64
            && sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            && paths.insert(path.to_owned())
    })
}

fn source_provenance_is_complete(
    provenance: Option<&Value>,
    nodes: &[Value],
    dependencies: Option<&Value>,
) -> bool {
    let Some(provenance) = provenance else {
        return false;
    };
    let Some(sources) = provenance.get("sources").and_then(Value::as_array) else {
        return false;
    };
    let Some(dependencies) = dependencies.and_then(Value::as_array) else {
        return false;
    };
    let entry_source = provenance
        .get("entrySourceFingerprint")
        .and_then(Value::as_array);
    let valid_source = |source_path: &str| {
        sources.iter().any(|fingerprint| {
            fingerprint.as_array().is_some_and(|parts| {
                let Some(sha256) = parts.get(1).and_then(Value::as_str) else {
                    return false;
                };
                let cataloged = dependencies.iter().any(|dependency| {
                    dependency.get("sourcePath").and_then(Value::as_str) == Some(source_path)
                        && dependency.get("sha256").and_then(Value::as_str) == Some(sha256)
                });
                let entry = entry_source.is_some_and(|parts| {
                    parts.len() == 2
                        && parts[0].as_str() == Some(source_path)
                        && parts[1].as_str() == Some(sha256)
                });
                parts.len() == 2
                    && parts[0].as_str() == Some(source_path)
                    && sha256.len() == 64
                    && sha256.bytes().all(|b| b.is_ascii_hexdigit())
                    && (cataloged || entry)
            })
        })
    };
    let map_fingerprint_valid = provenance
        .get("sourceMapFingerprint")
        .filter(|value| !value.is_null())
        .map_or(false, |fingerprint| {
            fingerprint.as_array().is_some_and(|parts| {
                parts.len() == 2
                    && parts[0].as_str().is_some_and(|path| !path.is_empty())
                    && parts[1].as_str().is_some_and(|sha| {
                        sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_hexdigit())
                    })
            })
        });
    map_fingerprint_valid
        && nodes.iter().all(|node| {
            node.get("sourcePath")
                .and_then(Value::as_str)
                .is_some_and(|source_path| valid_source(source_path))
        })
}

fn runtime_loaded_sources_are_complete(
    audit: Option<&Value>,
    provenance: Option<&Value>,
    dependencies: Option<&Value>,
) -> bool {
    let Some(audit) = audit else { return false };
    if audit["complete"].as_bool() != Some(true) {
        return false;
    }
    let Some(document_ids) = audit["documentIds"].as_array() else {
        return false;
    };
    let Some(files) = audit["files"].as_array() else {
        return false;
    };
    if files.is_empty() {
        return false;
    }
    if audit["unresolvedImports"]
        .as_array()
        .map_or(true, |items| !items.is_empty())
    {
        return false;
    }
    let required_roots = [
        "res://ui/editor/host/workbench_shell.zui",
        "res://ui/editor/windows/workbench_window.zui",
    ];
    if required_roots.iter().any(|root| {
        !document_ids
            .iter()
            .any(|value| value.as_str() == Some(root))
    }) {
        return false;
    }
    let Some(provenance) = provenance else {
        return false;
    };
    let Some(sources) = provenance["sources"].as_array() else {
        return false;
    };
    let Some(dependencies) = dependencies.and_then(Value::as_array) else {
        return false;
    };
    let entry_source = provenance["entrySourceFingerprint"].as_array();
    for file in files {
        let Some(source_path) = file["sourcePath"].as_str().filter(|path| !path.is_empty()) else {
            return false;
        };
        let Some(_resource_uri) = file["resourceUri"].as_str().filter(|path| !path.is_empty())
        else {
            return false;
        };
        let Some(sha256) = file["sha256"].as_str() else {
            return false;
        };
        if file["assetId"].as_str().map_or(true, str::is_empty)
            || file["physicalPath"].as_str().map_or(true, str::is_empty)
            || file["catalogMatches"].as_bool() != Some(true)
            || file["currentFileMatches"].as_bool() != Some(true)
            || sha256.len() != 64
            || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return false;
        }
        let source_fingerprint_matches = sources.iter().any(|fingerprint| {
            fingerprint.as_array().is_some_and(|parts| {
                parts.len() == 2
                    && parts[0].as_str() == Some(source_path)
                    && parts[1].as_str() == Some(sha256)
            })
        });
        let entry_fingerprint_matches = entry_source.is_some_and(|parts| {
            parts.len() == 2
                && parts[0].as_str() == Some(source_path)
                && parts[1].as_str() == Some(sha256)
        });
        let dependency_fingerprint_matches = dependencies.iter().any(|dependency| {
            dependency["sourcePath"].as_str() == Some(source_path)
                && dependency["sha256"].as_str() == Some(sha256)
        });
        if !(source_fingerprint_matches
            && (entry_fingerprint_matches || dependency_fingerprint_matches))
        {
            return false;
        }
    }
    document_ids.iter().all(|document_id| {
        document_id.as_str().is_some_and(|root| {
            files
                .iter()
                .any(|file| file["resourceUri"].as_str() == Some(root))
        })
    }) && required_roots.iter().all(|root| {
        files
            .iter()
            .any(|file| file["resourceUri"].as_str() == Some(root))
    })
}

fn active_design_tokens_are_complete(receipt: Option<&Value>) -> bool {
    let Some(receipt) = receipt else { return false };
    if receipt["complete"].as_bool() != Some(true) {
        return false;
    }
    let Some(expected_hash) = receipt["sha256"].as_str() else {
        return false;
    };
    if !receipt["tokens"].is_object() {
        return false;
    }
    if expected_hash.len() != 64 || !expected_hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return false;
    }
    super::contract::canonical_hash(&receipt["tokens"])
        .is_ok_and(|actual_hash| actual_hash == expected_hash)
}

fn style_is_complete(style: Option<&Value>) -> bool {
    let Some(style) = style else { return false };
    if style["complete"].as_bool() != Some(true) {
        return false;
    }
    let properties = &style["properties"];
    let Some(properties) = properties.as_object() else {
        return false;
    };
    properties.len() == REQUIRED_STYLE_PROPERTIES.len()
        && REQUIRED_STYLE_PROPERTIES
            .iter()
            .all(|property| properties.contains_key(*property))
        && ["foregroundColor", "backgroundColor", "borderColor"]
            .iter()
            .all(|property| is_css_color_or_absent(&properties[*property]))
        && valid_style_numbers(properties)
}

fn style_inventory_is_complete(style: Option<&Value>) -> bool {
    let Some(style) = style else { return false };
    if style["version"].as_u64() != Some(1) || style["complete"].as_bool() != Some(true) {
        return false;
    }
    let Some(properties) = style.get("properties").and_then(Value::as_object) else {
        return false;
    };
    if properties.len() != REQUIRED_STYLE_INVENTORY_PROPERTIES.len()
        || !REQUIRED_STYLE_INVENTORY_PROPERTIES
            .iter()
            .all(|property| properties.contains_key(*property))
        || !["foregroundColor", "backgroundColor", "borderColor"]
            .iter()
            .all(|property| is_css_color_or_absent(&properties[*property]))
        || !valid_style_numbers(properties)
    {
        return false;
    }
    let shadow = &properties["boxShadow"];
    shadow["complete"].as_bool() == Some(true)
        && shadow["layers"].as_array().is_some_and(|layers| {
            layers.iter().all(|layer| {
                let required = [
                    "offsetX",
                    "offsetY",
                    "blurRadius",
                    "spreadRadius",
                    "radius",
                    "color",
                    "opacity",
                    "inset",
                ];
                layer.as_object().is_some_and(|layer| {
                    layer.len() == required.len()
                        && required.iter().all(|key| layer.contains_key(*key))
                        && [
                            "offsetX",
                            "offsetY",
                            "blurRadius",
                            "spreadRadius",
                            "radius",
                            "opacity",
                        ]
                        .iter()
                        .all(|key| layer[*key].as_f64().is_some())
                        && layer["blurRadius"]
                            .as_f64()
                            .is_some_and(|value| value >= 0.0)
                        && layer["spreadRadius"]
                            .as_f64()
                            .is_some_and(|value| value >= 0.0)
                        && layer["radius"].as_f64().is_some_and(|value| value >= 0.0)
                        && layer["opacity"]
                            .as_f64()
                            .is_some_and(|value| (0.0..=1.0).contains(&value))
                        && is_canonical_rgba(&layer["color"])
                        && layer["inset"].as_bool().is_some()
                })
            })
        })
}

fn is_css_color_or_absent(value: &Value) -> bool {
    value.is_null() || is_canonical_rgba(value)
}

fn is_canonical_rgba(value: &Value) -> bool {
    let Some(color) = value.as_str() else {
        return false;
    };
    let Some(components) = color
        .strip_prefix("rgba(")
        .and_then(|color| color.strip_suffix(')'))
    else {
        return false;
    };
    let components = components.split(',').collect::<Vec<_>>();
    components.len() == 4
        && components[..3]
            .iter()
            .all(|component| component.parse::<u16>().is_ok_and(|value| value <= 255))
        && components[3]
            .parse::<f64>()
            .is_ok_and(|value| value.is_finite() && (0.0..=1.0).contains(&value))
}

fn valid_style_numbers(properties: &serde_json::Map<String, Value>) -> bool {
    properties["borderWidth"]
        .as_f64()
        .is_some_and(|value| value >= 0.0)
        && properties["borderRadius"]
            .as_f64()
            .is_some_and(|value| value >= 0.0)
        && properties["opacity"]
            .as_f64()
            .is_some_and(|value| (0.0..=1.0).contains(&value))
}

fn clip_is_well_formed(node: &Value) -> bool {
    let Some(clipped) = node.get("clip").and_then(Value::as_bool) else {
        return false;
    };
    let Some(bounds) = node.get("clipBounds") else {
        return false;
    };
    if !clipped {
        return bounds.is_null();
    }
    ["x", "y", "width", "height"]
        .iter()
        .all(|key| bounds.get(*key).and_then(Value::as_f64).is_some())
}

fn asset_audit_is_complete(receipt: Option<&Value>, fingerprints: Option<&Value>) -> bool {
    let Some(receipt) = receipt else { return false };
    if receipt["complete"].as_bool() != Some(true) {
        return false;
    }
    let Some(resources) = receipt.get("resources").and_then(Value::as_array) else {
        return false;
    };
    let Some(fingerprints) = fingerprints.and_then(Value::as_array) else {
        return resources.is_empty();
    };
    resources.iter().all(|resource| {
        if resource.get("loaded").and_then(Value::as_bool) != Some(true) {
            return false;
        }
        let kind = resource.get("kind").and_then(Value::as_str);
        let path = resource.get("path").and_then(Value::as_str);
        let sha256 = resource.get("sha256").and_then(Value::as_str);
        let Some((kind, path, sha256)) = kind.zip(path).zip(sha256).map(|((k, p), h)| (k, p, h))
        else {
            return false;
        };
        if kind.is_empty() || path.is_empty() || !valid_sha256(sha256) {
            return false;
        }
        let source_path = resource.get("sourcePath").and_then(Value::as_str);
        let source_node_id = resource.get("sourceNodeId").and_then(Value::as_str);
        let instance_path = resource.get("instancePath").and_then(Value::as_str);
        if source_path.is_none()
            || source_node_id.is_none()
            || instance_path.is_none()
            || source_path.is_some_and(str::is_empty)
            || source_node_id.is_some_and(str::is_empty)
            || instance_path.is_some_and(str::is_empty)
        {
            return false;
        }
        fingerprints.iter().any(|fingerprint| {
            fingerprint.as_array().is_some_and(|parts| {
                parts.len() == 2
                    && parts[0].as_str() == Some(path)
                    && parts[1].as_str() == Some(sha256)
            })
        })
    })
}

fn fonts_are_complete(faces: Option<&Value>, fingerprints: Option<&Value>) -> bool {
    let Some(faces) = faces.and_then(Value::as_array) else {
        return false;
    };
    if faces.is_empty() {
        return false;
    }
    let Some(fingerprints) = fingerprints.and_then(Value::as_array) else {
        return false;
    };
    faces.iter().all(|face| {
        let path = face.get("resourcePath").and_then(Value::as_str);
        let sha256 = face.get("sha256").and_then(Value::as_str);
        let family = face.get("familyName").and_then(Value::as_str);
        let post_script_name = face.get("postScriptName").and_then(Value::as_str);
        let glyph_count = face.get("glyphCount").and_then(Value::as_u64).unwrap_or(0);
        let face_index = face.get("faceIndex").and_then(Value::as_u64);
        let Some((path, sha256, family, post_script_name)) =
            path.zip(sha256).zip(family).zip(post_script_name).map(
                |(((path, sha256), family), post_script_name)| {
                    (path, sha256, family, post_script_name)
                },
            )
        else {
            return false;
        };
        if path.is_empty()
            || !valid_sha256(sha256)
            || family.is_empty()
            || post_script_name.is_empty()
            || glyph_count == 0
            || face_index.is_none()
        {
            return false;
        }
        fingerprints.iter().any(|fingerprint| {
            fingerprint.as_array().is_some_and(|parts| {
                parts.len() == 2
                    && parts[0].as_str() == Some(path)
                    && parts[1].as_str() == Some(sha256)
            })
        })
    })
}

fn valid_sha256(sha256: &str) -> bool {
    sha256.len() == 64 && sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn semantic_node_is_paint_visible(node: &Value, geometry: &Value) -> bool {
    // Keep offscreen nodes in the semantic tree, but require measured paint receipts only
    // where the viewport and effective ancestor clip allow the native painter to emit pixels.
    if node.get("visible").and_then(Value::as_bool) != Some(true) {
        return false;
    }
    let Some(viewport_width) = geometry
        .pointer("/windowMetrics/logicalSize/width")
        .and_then(Value::as_f64)
    else {
        return true;
    };
    let Some(viewport_height) = geometry
        .pointer("/windowMetrics/logicalSize/height")
        .and_then(Value::as_f64)
    else {
        return true;
    };
    let Some(bounds) = node.get("bounds") else {
        return true;
    };
    let Some([x, y, width, height]) = ["x", "y", "width", "height"]
        .map(|key| bounds.get(key).and_then(Value::as_f64))
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|values| <[f64; 4]>::try_from(values).ok())
    else {
        return true;
    };
    if width <= 0.0 || height <= 0.0 || viewport_width <= 0.0 || viewport_height <= 0.0 {
        return false;
    }
    let intersects = |left: f64, top: f64, right: f64, bottom: f64| {
        x + width > left && y + height > top && x < right && y < bottom
    };
    if !intersects(0.0, 0.0, viewport_width, viewport_height) {
        return false;
    }
    if node.get("clip").and_then(Value::as_bool) != Some(true) {
        return true;
    }
    let Some(clip) = node.get("clipBounds") else {
        return true;
    };
    let Some([clip_x, clip_y, clip_width, clip_height]) = ["x", "y", "width", "height"]
        .map(|key| clip.get(key).and_then(Value::as_f64))
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .and_then(|values| <[f64; 4]>::try_from(values).ok())
    else {
        return true;
    };
    if clip_width <= 0.0 || clip_height <= 0.0 {
        return false;
    }
    intersects(clip_x, clip_y, clip_x + clip_width, clip_y + clip_height)
}

#[cfg(test)]
#[path = "tests/readiness.rs"]
mod tests;
