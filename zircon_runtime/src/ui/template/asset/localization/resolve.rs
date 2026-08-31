use std::collections::{BTreeSet, HashMap, HashSet};

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiLocalizationDependency, UiLocalizationDiagnostic, UiLocalizationDiagnosticSeverity,
    UiLocalizationReport,
};

const DEFAULT_LOCALIZATION_TABLE: &str = "default";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiLocalizationTableCatalog {
    tables: HashMap<String, HashMap<String, UiLocalizationTableEntry>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct UiLocalizationTableEntry {
    source_uri: Option<String>,
    keys: HashSet<String>,
}

impl UiLocalizationTableCatalog {
    pub fn register_table_keys<I, S>(
        &mut self,
        locale: impl Into<String>,
        table: impl Into<String>,
        source_uri: Option<String>,
        keys: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let entry = UiLocalizationTableEntry {
            source_uri,
            keys: keys
                .into_iter()
                .map(Into::into)
                .filter(|key: &String| !key.trim().is_empty())
                .collect(),
        };
        let _ = self
            .tables
            .entry(locale.into())
            .or_default()
            .insert(table.into(), entry);
        self
    }
}

pub fn validate_localization_report_against_catalog(
    report: &UiLocalizationReport,
    locale: &str,
    catalog: &UiLocalizationTableCatalog,
) -> Vec<UiLocalizationDiagnostic> {
    let locale = locale.trim();
    if locale.is_empty() {
        return Vec::new();
    }
    let locale_tables = catalog.tables.get(locale);
    let mut emitted_diagnostics = HashSet::new();
    let mut diagnostics = report
        .dependencies
        .iter()
        .filter_map(|dependency| {
            validate_dependency(
                locale,
                dependency,
                locale_tables,
                &mut emitted_diagnostics,
            )
        })
        .collect::<Vec<_>>();
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn localization_table_keys_from_toml_str(
    source: &str,
) -> Result<BTreeSet<String>, toml::de::Error> {
    let value = Value::Table(toml::from_str(source)?);
    let mut keys = BTreeSet::new();
    let mut path = String::new();
    collect_locale_keys(&mut path, &value, &mut keys);
    Ok(keys)
}

fn validate_dependency<'dependency>(
    locale: &str,
    dependency: &'dependency UiLocalizationDependency,
    locale_tables: Option<&HashMap<String, UiLocalizationTableEntry>>,
    emitted_diagnostics: &mut HashSet<(&'dependency str, &'dependency str, &'dependency str, bool)>,
) -> Option<UiLocalizationDiagnostic> {
    let table_name = dependency
        .reference
        .table
        .as_deref()
        .unwrap_or(DEFAULT_LOCALIZATION_TABLE);
    let Some(table) = locale_tables.and_then(|tables| tables.get(table_name)) else {
        let identity = (
            dependency.path.as_str(),
            dependency.reference.key.as_str(),
            table_name,
            dependency.reference.fallback.is_some(),
        );
        if !emitted_diagnostics.insert(identity) {
            return None;
        }
        return Some(UiLocalizationDiagnostic::new(
            "missing_locale_table",
            UiLocalizationDiagnosticSeverity::Error,
            dependency.path.clone(),
            format!(
                "locale table {locale}/{table_name} is not registered for key {}",
                dependency.reference.key
            ),
        ));
    };
    if table.keys.contains(&dependency.reference.key) {
        return None;
    }
    let identity = (
        dependency.path.as_str(),
        dependency.reference.key.as_str(),
        table_name,
        dependency.reference.fallback.is_some(),
    );
    if !emitted_diagnostics.insert(identity) {
        return None;
    }
    let message = match table.source_uri.as_deref() {
        Some(source_uri) => format!(
            "locale key {} is missing from {locale}/{table_name} in {source_uri}",
            dependency.reference.key
        ),
        None => format!(
            "locale key {} is missing from {locale}/{table_name}",
            dependency.reference.key
        ),
    };
    Some(UiLocalizationDiagnostic::new(
        "missing_locale_key",
        missing_ref_severity(dependency),
        dependency.path.clone(),
        message,
    ))
}

fn missing_ref_severity(dependency: &UiLocalizationDependency) -> UiLocalizationDiagnosticSeverity {
    if dependency.reference.fallback.is_some() {
        UiLocalizationDiagnosticSeverity::Warning
    } else {
        UiLocalizationDiagnosticSeverity::Error
    }
}

fn collect_locale_keys(path: &mut String, value: &Value, keys: &mut BTreeSet<String>) {
    match value {
        Value::Table(table) => {
            let prefix_len = path.len();
            for (key, value) in table {
                path.truncate(prefix_len);
                if prefix_len > 0 {
                    path.push('.');
                }
                path.push_str(key);
                collect_locale_keys(path, value, keys);
            }
            path.truncate(prefix_len);
        }
        Value::Array(_) => {}
        _ if !path.is_empty() => {
            let _ = keys.insert(path.clone());
        }
        _ => {}
    }
}

#[cfg(test)]
mod performance_tests;
