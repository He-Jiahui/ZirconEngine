use std::fmt::Write as _;

use toml::Value;

use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiLocalizationDependency, UiLocalizationDiagnostic,
    UiLocalizationDiagnosticSeverity, UiLocalizationReport, UiLocalizationTextCandidate,
    UiLocalizedTextRef, UiTextDirection,
};

pub fn collect_document_localization_report(document: &UiAssetDocument) -> UiLocalizationReport {
    let mut report = UiLocalizationReport::default();
    let mut path = String::new();
    for node in document.iter_nodes() {
        path.clear();
        path.push_str("nodes.");
        path.push_str(&node.node_id);
        let node_prefix_len = path.len();

        path.push_str(".props");
        collect_values(&mut path, &node.props, &mut report);
        if let Some(layout) = &node.layout {
            path.truncate(node_prefix_len);
            path.push_str(".layout");
            collect_values(&mut path, layout, &mut report);
        }
        path.truncate(node_prefix_len);
        path.push_str(".params");
        collect_values(&mut path, &node.params, &mut report);
    }
    for stylesheet in &document.stylesheets {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            path.clear();
            path.push_str("stylesheets.");
            path.push_str(&stylesheet.id);
            path.push_str(".rules");
            match rule.id.as_deref() {
                Some(rule_id) => {
                    path.push('.');
                    path.push_str(rule_id);
                }
                None => write!(path, "[{rule_index}]").expect("writing to String cannot fail"),
            }
            let rule_prefix_len = path.len();

            path.push_str(".set.self");
            collect_values(&mut path, &rule.set.self_values, &mut report);
            path.truncate(rule_prefix_len);
            path.push_str(".set.slot");
            collect_values(&mut path, &rule.set.slot, &mut report);
        }
    }
    report.dependencies.sort();
    report.diagnostics.sort();
    report.extraction_candidates.sort();
    report
}

pub fn validate_document_localization(document: &UiAssetDocument) -> Result<(), UiAssetError> {
    let report = collect_document_localization_report(document);
    if let Some(diagnostic) = report.diagnostics.first() {
        return Err(UiAssetError::InvalidDocument {
            asset_id: document.asset.id.clone(),
            detail: diagnostic.message.clone(),
        });
    }
    Ok(())
}

fn collect_values(
    path: &mut String,
    values: &std::collections::BTreeMap<String, Value>,
    report: &mut UiLocalizationReport,
) {
    let prefix_len = path.len();
    for (key, value) in values {
        path.truncate(prefix_len);
        path.push('.');
        path.push_str(key);
        collect_value(path, value, report);
    }
    path.truncate(prefix_len);
}

fn collect_value(path: &mut String, value: &Value, report: &mut UiLocalizationReport) {
    match value {
        Value::String(text) if is_text_path(path) => {
            report
                .extraction_candidates
                .push(UiLocalizationTextCandidate {
                    path: path.clone(),
                    text: text.clone(),
                });
        }
        Value::Table(table) => {
            if let Some(reference) = localized_text_ref(table) {
                if let Some(message) = reference.validate(path.as_str()) {
                    report.diagnostics.push(UiLocalizationDiagnostic::new(
                        "empty_localized_text_key",
                        UiLocalizationDiagnosticSeverity::Error,
                        path.as_str(),
                        message,
                    ));
                    return;
                }
                report.dependencies.push(UiLocalizationDependency {
                    path: path.clone(),
                    reference,
                    direction: text_direction(table),
                });
                return;
            }
            let prefix_len = path.len();
            for (key, nested) in table {
                path.truncate(prefix_len);
                path.push('.');
                path.push_str(key);
                collect_value(path, nested, report);
            }
            path.truncate(prefix_len);
        }
        Value::Array(items) => {
            let prefix_len = path.len();
            for (index, item) in items.iter().enumerate() {
                path.truncate(prefix_len);
                write!(path, "[{index}]").expect("writing to String cannot fail");
                collect_value(path, item, report);
            }
            path.truncate(prefix_len);
        }
        _ => {}
    }
}

fn localized_text_ref(table: &toml::map::Map<String, Value>) -> Option<UiLocalizedTextRef> {
    let key = table.get("text_key")?.as_str()?.to_string();
    Some(UiLocalizedTextRef {
        key,
        table: table
            .get("table")
            .and_then(Value::as_str)
            .map(str::to_string),
        fallback: table
            .get("fallback")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn text_direction(table: &toml::map::Map<String, Value>) -> UiTextDirection {
    match table.get("direction").and_then(Value::as_str) {
        Some("ltr") | Some("left_to_right") => UiTextDirection::LeftToRight,
        Some("rtl") | Some("right_to_left") => UiTextDirection::RightToLeft,
        _ => UiTextDirection::Auto,
    }
}

fn is_text_path(path: &str) -> bool {
    path.ends_with(".text") || path.ends_with(".label") || path.ends_with(".title")
}

#[cfg(test)]
mod performance_tests;
