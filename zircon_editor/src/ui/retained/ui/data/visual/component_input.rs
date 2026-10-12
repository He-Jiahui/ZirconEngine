use std::collections::HashSet;

use serde_json::{Map, Value};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

const MAX_TEXT_OVERRIDE_BYTES: usize = 4096;

pub(super) fn validate(data: &Value) -> Result<(), String> {
    let object = data.as_object().ok_or("review data must be an object")?;
    if object.is_empty() {
        return Ok(());
    }
    if let Some(selector) = object.get("workbenchState") {
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "workbenchState" | "workbenchPresentation"))
        {
            return Err(
                "workbench state review data cannot be combined with component input or unknown data".into(),
            );
        }
        if object
            .get("workbenchPresentation")
            .is_some_and(|presentation| !presentation.is_object())
        {
            return Err("workbenchPresentation must be an object".into());
        }
        let selector = selector
            .as_object()
            .ok_or("workbenchState must be an object")?;
        if selector.keys().any(|key| {
            !matches!(
                key.as_str(),
                "sourcePath"
                    | "controlId"
                    | "sourceNodeId"
                    | "instancePath"
                    | "textOverrides"
                    | "scrollTarget"
            )
        }) {
            return Err("workbenchState contains an unsupported selector property".into());
        }
        for key in ["sourcePath", "controlId"] {
            if selector
                .get(key)
                .and_then(Value::as_str)
                .map_or(true, str::is_empty)
            {
                return Err(format!("workbenchState requires a nonempty {key}"));
            }
        }
        if selector
            .get("sourceNodeId")
            .is_some_and(|value| value.as_str().map_or(true, str::is_empty))
        {
            return Err("workbenchState sourceNodeId must be a nonempty string".into());
        }
        if selector
            .get("instancePath")
            .is_some_and(|value| value.as_str().map_or(true, str::is_empty))
        {
            return Err("workbenchState instancePath must be a nonempty string".into());
        }
        if let Some(target) = selector.get("scrollTarget") {
            let target = target
                .as_object()
                .ok_or("workbenchState scrollTarget must be an object")?;
            if target.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "sourcePath" | "controlId" | "sourceNodeId" | "instancePath"
                )
            }) {
                return Err("workbenchState scrollTarget contains an unsupported property".into());
            }
            for key in ["sourcePath", "controlId"] {
                if target
                    .get(key)
                    .and_then(Value::as_str)
                    .map_or(true, str::is_empty)
                {
                    return Err(format!(
                        "workbenchState scrollTarget requires a nonempty {key}"
                    ));
                }
            }
            for key in ["sourceNodeId", "instancePath"] {
                if target
                    .get(key)
                    .is_some_and(|value| value.as_str().map_or(true, str::is_empty))
                {
                    return Err(format!(
                        "workbenchState scrollTarget {key} must be a nonempty string"
                    ));
                }
            }
        }
        if let Some(overrides) = selector.get("textOverrides") {
            validate_text_overrides(overrides)?;
        }
        return Ok(());
    }
    if let Some(presentation) = object.get("workbenchPresentation") {
        if object.len() != 1 || !presentation.is_object() {
            return Err(
                "workbenchPresentation must be an object and cannot be mixed with other data"
                    .into(),
            );
        }
        return Ok(());
    }
    let input = object
        .get("componentInput")
        .and_then(Value::as_object)
        .filter(|_| object.len() == 1)
        .ok_or("review data requires an explicit business host adapter")?;
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

pub(super) fn has_text_overrides(data: &Value) -> bool {
    data.get("workbenchState")
        .and_then(Value::as_object)
        .and_then(|state| state.get("textOverrides"))
        .and_then(Value::as_array)
        .is_some_and(|overrides| !overrides.is_empty())
}

fn validate_text_overrides(value: &Value) -> Result<(), String> {
    let overrides = value
        .as_array()
        .filter(|overrides| !overrides.is_empty())
        .ok_or("workbenchState textOverrides must be a nonempty array")?;
    let mut targets = HashSet::with_capacity(overrides.len());
    for override_value in overrides {
        let object = override_value
            .as_object()
            .ok_or("workbenchState text override must be an object")?;
        if object.keys().any(|key| {
            !matches!(
                key.as_str(),
                "sourcePath" | "sourceNodeId" | "instancePath" | "controlId" | "property" | "value"
            )
        }) {
            return Err("workbenchState text override contains an unsupported property".into());
        }
        let required_string = |key: &str| {
            object
                .get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("workbenchState text override requires a nonempty {key}"))
        };
        let source_path = required_string("sourcePath")?;
        let source_node_id = required_string("sourceNodeId")?;
        let control_id = required_string("controlId")?;
        let instance_path = object
            .get("instancePath")
            .map(|_| required_string("instancePath"))
            .transpose()?
            .unwrap_or("");
        if required_string("property")? != "text" {
            return Err("workbenchState text override property must be text".into());
        }
        let text = required_string("value")?;
        if text.trim().is_empty()
            || text.len() > MAX_TEXT_OVERRIDE_BYTES
            || text.chars().any(char::is_control)
        {
            return Err(format!(
                "workbenchState text override value must contain 1..={MAX_TEXT_OVERRIDE_BYTES} non-control UTF-8 bytes"
            ));
        }
        if !targets.insert((source_path, source_node_id, instance_path, control_id)) {
            return Err("workbenchState text override targets must be unique".into());
        }
    }
    Ok(())
}

pub(super) fn props(document: &UiV2AssetDocument, data: &Value) -> Result<Value, String> {
    validate(data)?;
    let Some(input) = data.get("componentInput").and_then(Value::as_object) else {
        return Ok(Value::Object(Map::new()));
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
    let key = match root.component.as_str() {
        "InputField" => "value",
        "SearchField" => "query",
        _ => return Err("component input needs a declared text field".into()),
    };
    if !input.contains_key(key)
        || input
            .keys()
            .any(|candidate| candidate != key && candidate != "validation_level")
    {
        return Err("component input differs from its declared value property".into());
    }
    Ok(Value::Object(input.clone()))
}

#[cfg(test)]
include!("tests/component_input_cases.rs");
