use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiLocalizationDependency, UiLocalizationDiagnostic, UiLocalizationDiagnosticSeverity,
    UiLocalizationReport,
};

const DEFAULT_LOCALIZATION_TABLE: &str = "default";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiLocalizationTableCatalog {
    tables: BTreeMap<(String, String), UiLocalizationTableEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct UiLocalizationTableEntry {
    source_uri: Option<String>,
    keys: BTreeSet<String>,
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
        let _ = self.tables.insert((locale.into(), table.into()), entry);
        self
    }

    fn table(&self, locale: &str, table: &str) -> Option<&UiLocalizationTableEntry> {
        self.tables.get(&(locale.to_string(), table.to_string()))
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
    let mut diagnostics = report
        .dependencies
        .iter()
        .filter_map(|dependency| validate_dependency(locale, dependency, catalog))
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
    collect_locale_keys("", &value, &mut keys);
    Ok(keys)
}

fn validate_dependency(
    locale: &str,
    dependency: &UiLocalizationDependency,
    catalog: &UiLocalizationTableCatalog,
) -> Option<UiLocalizationDiagnostic> {
    let table_name = dependency
        .reference
        .table
        .as_deref()
        .unwrap_or(DEFAULT_LOCALIZATION_TABLE);
    let Some(table) = catalog.table(locale, table_name) else {
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
    let source = table
        .source_uri
        .as_deref()
        .map(|source_uri| format!(" in {source_uri}"))
        .unwrap_or_default();
    Some(UiLocalizationDiagnostic::new(
        "missing_locale_key",
        missing_ref_severity(dependency),
        dependency.path.clone(),
        format!(
            "locale key {} is missing from {locale}/{table_name}{source}",
            dependency.reference.key
        ),
    ))
}

fn missing_ref_severity(dependency: &UiLocalizationDependency) -> UiLocalizationDiagnosticSeverity {
    if dependency.reference.fallback.is_some() {
        UiLocalizationDiagnosticSeverity::Warning
    } else {
        UiLocalizationDiagnosticSeverity::Error
    }
}

fn collect_locale_keys(prefix: &str, value: &Value, keys: &mut BTreeSet<String>) {
    match value {
        Value::Table(table) => {
            for (key, value) in table {
                let path = join_key(prefix, key);
                collect_locale_keys(&path, value, keys);
            }
        }
        Value::Array(_) => {}
        _ if !prefix.is_empty() => {
            let _ = keys.insert(prefix.to_string());
        }
        _ => {}
    }
}

fn join_key(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}
