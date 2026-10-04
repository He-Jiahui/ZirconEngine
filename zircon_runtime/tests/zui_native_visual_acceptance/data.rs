use std::collections::BTreeMap;

use serde_json::Value;
use toml::Value as TomlValue;
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

const COLLECTION_REVIEW_CASES: [&str; 6] = [
    "empty",
    "one",
    "normal",
    "overflow",
    "narrow",
    "long-locale",
];

/// The catalog deliberately keeps business data narrow: component previews may
/// provide the declared text/query value and its validation tone, while views
/// and fixtures must use their source-owned deterministic data.  Keeping this
/// envelope explicit prevents a review case from becoming an arbitrary source
/// mutation channel.
pub(super) fn validate(data: &Value, host: &str) -> Result<(), String> {
    let object = data
        .as_object()
        .ok_or("native review data must be an object")?;
    if object.is_empty() {
        return Ok(());
    }
    if host == "fixture"
        && object.len() == 1
        && object
            .get("collectionCase")
            .and_then(Value::as_str)
            .is_some_and(|case| COLLECTION_REVIEW_CASES.contains(&case))
    {
        return Ok(());
    }
    if host != "component" {
        return Err("native nonempty review data requires a component host".into());
    }
    let input = object
        .get("componentInput")
        .and_then(Value::as_object)
        .filter(|_| object.len() == 1)
        .ok_or("native review data requires an explicit componentInput adapter")?;
    for (key, value) in input {
        if !matches!(key.as_str(), "value" | "query" | "validation_level") || !value.is_string() {
            return Err(format!("unsupported component input property: {key}"));
        }
        if key == "validation_level" && !matches!(value.as_str(), Some("normal" | "error")) {
            return Err("unsupported component validation level".into());
        }
    }
    Ok(())
}

fn toml_strings(value: Option<&TomlValue>) -> Vec<String> {
    value
        .and_then(TomlValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

fn long_collection_label(value: &str) -> String {
    let mut parts = value.split('|').map(ToOwned::to_owned).collect::<Vec<_>>();
    let label = "A deliberately long localized collection label for overflow coverage";
    if parts.len() > 1 {
        parts[1] = label.to_owned();
    } else if let Some(first) = parts.first_mut() {
        *first = label.to_owned();
    }
    parts.join("|")
}

/// Apply source-declared collection review data to a cloned native preview.
/// This mirrors the Penpot bridge adapter and deliberately runs before
/// compilation so layout/text measurement sees the same transient values.
pub(super) fn apply_collection_case(
    document: &mut UiV2AssetDocument,
    data: &Value,
) -> Result<(), String> {
    let Some(case) = data.get("collectionCase").and_then(Value::as_str) else {
        return Ok(());
    };
    if !COLLECTION_REVIEW_CASES.contains(&case) {
        return Err(format!("unsupported native collection review case: {case}"));
    }

    let owners = document
        .nodes
        .iter()
        .filter_map(|(node_id, node)| {
            let cases = toml_strings(node.props.get("collection_test_cases"));
            cases.iter().any(|candidate| candidate == case).then(|| {
                let direct = node
                    .children
                    .iter()
                    .filter_map(|child| document.nodes.get(&child.node).map(|_| child.node.clone()))
                    .collect::<Vec<_>>();
                let explicit = direct
                    .iter()
                    .filter(|child_id| {
                        document
                            .nodes
                            .get(*child_id)
                            .and_then(|child| child.props.get("collection_item"))
                            .and_then(TomlValue::as_str)
                            .is_some()
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let item_ids = if !explicit.is_empty() {
                    explicit
                } else {
                    let item_count = toml_strings(node.props.get("collection_items")).len();
                    direct
                        .iter()
                        .filter(|child_id| {
                            document.nodes.get(*child_id).is_some_and(|child| {
                                matches!(child.component.as_str(), "Button" | "Chip" | "Paper")
                                    && child
                                        .props
                                        .get("text")
                                        .and_then(TomlValue::as_str)
                                        .is_some_and(|text| !text.starts_with("Add"))
                            })
                        })
                        .take(item_count)
                        .cloned()
                        .collect::<Vec<_>>()
                };
                (node_id.clone(), item_ids)
            })
        })
        .collect::<Vec<_>>();

    for (owner_id, item_ids) in owners {
        let visible_count = match case {
            "empty" => 0,
            "one" => 1,
            _ => item_ids.len(),
        };
        {
            let Some(owner) = document.nodes.get_mut(&owner_id) else {
                continue;
            };
            owner.props.insert(
                "collection_review_case".into(),
                TomlValue::String(case.into()),
            );
            let original = toml_strings(owner.props.get("collection_items"));
            match case {
                "empty" => {
                    owner
                        .props
                        .insert("collection_items".into(), TomlValue::Array(Vec::new()));
                }
                "one" => {
                    owner.props.insert(
                        "collection_items".into(),
                        TomlValue::Array(
                            original
                                .iter()
                                .take(1)
                                .cloned()
                                .map(TomlValue::String)
                                .collect(),
                        ),
                    );
                }
                "overflow" => {
                    let visible_limit = owner
                        .props
                        .get("visible_limit")
                        .and_then(TomlValue::as_integer)
                        .unwrap_or(3)
                        .max(1) as usize;
                    let mut expanded = original.clone();
                    expanded.extend(original.iter().take(visible_limit).cloned());
                    owner.props.insert(
                        "collection_items".into(),
                        TomlValue::Array(expanded.into_iter().map(TomlValue::String).collect()),
                    );
                    owner
                        .props
                        .insert("collection_overflow".into(), TomlValue::Boolean(true));
                }
                "long-locale" => {
                    owner.props.insert(
                        "collection_items".into(),
                        TomlValue::Array(
                            original
                                .iter()
                                .map(|value| TomlValue::String(long_collection_label(value)))
                                .collect(),
                        ),
                    );
                }
                "normal" | "narrow" => {}
                _ => unreachable!(),
            }
            if case == "narrow" {
                owner.props.insert(
                    "collection_review_width".into(),
                    TomlValue::String("narrow".into()),
                );
            }
        }
        for (index, child_id) in item_ids.iter().enumerate() {
            let Some(child) = document.nodes.get_mut(child_id) else {
                continue;
            };
            if index < visible_count {
                child.props.remove("visibility");
            } else {
                child
                    .props
                    .insert("visibility".into(), TomlValue::String("collapsed".into()));
            }
            if case == "long-locale" {
                let text = child
                    .props
                    .get("text")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned);
                if let Some(text) = text {
                    child.props.insert(
                        "text".into(),
                        TomlValue::String(long_collection_label(&text)),
                    );
                }
                let item = child
                    .props
                    .get("collection_item")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned);
                if let Some(item) = item {
                    child.props.insert(
                        "collection_item".into(),
                        TomlValue::String(long_collection_label(&item)),
                    );
                }
            }
        }
    }
    Ok(())
}

/// Resolve the component-input adapter into the mount props consumed by the
/// real V2 instancer.  Applying the value before compilation is important: the
/// compiler then measures the same text that the renderer paints.
pub(super) fn component_mount_props(
    document: &UiV2AssetDocument,
    data: &Value,
) -> Result<BTreeMap<String, TomlValue>, String> {
    validate(data, "component")?;
    let Some(input) = data.get("componentInput").and_then(Value::as_object) else {
        return Ok(BTreeMap::new());
    };
    if document.asset.kind != UiV2AssetKind::Component || document.components.len() != 1 {
        return Err("component input needs one exported component".into());
    }
    let definition = document
        .components
        .values()
        .next()
        .ok_or("missing component export")?;
    let root = document
        .nodes
        .get(&definition.root)
        .ok_or("missing component root")?;
    let value_key = match root.component.as_str() {
        "InputField" => "value",
        "SearchField" => "query",
        _ => return Err("component input needs a declared text field".into()),
    };
    if !input.contains_key(value_key)
        || input
            .keys()
            .any(|candidate| candidate != value_key && candidate != "validation_level")
    {
        return Err("component input differs from its declared value property".into());
    }
    Ok(input
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                TomlValue::String(value.as_str().unwrap_or_default().to_owned()),
            )
        })
        .collect())
}
