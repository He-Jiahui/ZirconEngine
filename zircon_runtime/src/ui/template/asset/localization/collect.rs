use toml::Value;

use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiLocalizationDependency, UiLocalizationDiagnostic,
    UiLocalizationDiagnosticSeverity, UiLocalizationReport, UiLocalizationTextCandidate,
    UiLocalizedTextRef, UiTextDirection,
};

pub fn collect_document_localization_report(document: &UiAssetDocument) -> UiLocalizationReport {
    let mut report = UiLocalizationReport::default();
    for node in document.iter_nodes() {
        collect_values(
            &format!("nodes.{}.props", node.node_id),
            &node.props,
            &mut report,
        );
        if let Some(layout) = &node.layout {
            collect_values(
                &format!("nodes.{}.layout", node.node_id),
                layout,
                &mut report,
            );
        }
        collect_values(
            &format!("nodes.{}.params", node.node_id),
            &node.params,
            &mut report,
        );
    }
    for stylesheet in &document.stylesheets {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            let rule_path = match rule.id.as_deref() {
                Some(rule_id) => format!("stylesheets.{}.rules.{rule_id}", stylesheet.id),
                None => format!("stylesheets.{}.rules[{rule_index}]", stylesheet.id),
            };
            collect_values(
                &format!("{rule_path}.set.self"),
                &rule.set.self_values,
                &mut report,
            );
            collect_values(
                &format!("{rule_path}.set.slot"),
                &rule.set.slot,
                &mut report,
            );
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
    path_prefix: &str,
    values: &std::collections::BTreeMap<String, Value>,
    report: &mut UiLocalizationReport,
) {
    for (key, value) in values {
        collect_value(&format!("{path_prefix}.{key}"), value, report);
    }
}

fn collect_value(path: &str, value: &Value, report: &mut UiLocalizationReport) {
    match value {
        Value::String(text) if is_text_path(path) => {
            report
                .extraction_candidates
                .push(UiLocalizationTextCandidate {
                    path: path.to_string(),
                    text: text.clone(),
                });
        }
        Value::Table(table) => {
            if let Some(reference) = localized_text_ref(table) {
                if let Some(message) = reference.validate(path) {
                    report.diagnostics.push(UiLocalizationDiagnostic {
                        severity: UiLocalizationDiagnosticSeverity::Error,
                        path: path.to_string(),
                        message,
                    });
                    return;
                }
                report.dependencies.push(UiLocalizationDependency {
                    path: path.to_string(),
                    reference,
                    direction: text_direction(table),
                });
                return;
            }
            for (key, nested) in table {
                collect_value(&format!("{path}.{key}"), nested, report);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_value(&format!("{path}[{index}]"), item, report);
            }
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
