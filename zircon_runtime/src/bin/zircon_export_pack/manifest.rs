use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::pack::{
    ZrPackInputAsset, ZrPackTrimConfig, ZrPackTrimInputAsset, ZrPackTrimPlanner, ZrPackTrimReport,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportAssetPackManifest {
    pub roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_filter: Option<String>,
    #[serde(default)]
    pub assets: Vec<ExportAssetPackEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportAssetPackEntry {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<PathBuf>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportPackInputs {
    pub trim_report: ZrPackTrimReport,
    pub pack_assets: Vec<ZrPackInputAsset>,
    pub diagnostics: Vec<String>,
    pub asset_source_errors: Vec<String>,
}

impl ExportAssetPackManifest {
    pub fn trim_config(&self) -> ZrPackTrimConfig {
        let mut config = ZrPackTrimConfig::new(self.roots.clone());
        config.asset_filter = self.asset_filter.clone();
        config
    }

    pub fn trim_inputs(&self) -> Vec<ZrPackTrimInputAsset> {
        self.assets
            .iter()
            .map(|asset| {
                let mut input = ZrPackTrimInputAsset::new(asset.path.clone());
                input.dependencies = asset.dependencies.clone();
                input.labels = asset.labels.clone();
                input
            })
            .collect()
    }

    pub fn pack_inputs(&self, manifest_dir: &Path) -> ExportPackInputs {
        let trim_report = ZrPackTrimPlanner::trim(self.trim_config(), self.trim_inputs());
        let mut diagnostics = trim_report.diagnostics.clone();
        if trim_report.has_missing_dependencies() {
            diagnostics
                .push("pack stage stopped because asset dependencies are missing".to_string());
        }

        let mut pack_assets = Vec::new();
        let mut asset_source_errors = Vec::new();
        let mut entries_by_path = HashMap::with_capacity(self.assets.len());
        for entry in &self.assets {
            entries_by_path.entry(entry.path.as_str()).or_insert(entry);
        }
        for path in &trim_report.included_assets {
            let Some(entry) = entries_by_path.get(path.as_str()) else {
                let diagnostic = format!("included asset {path} was not found in manifest");
                diagnostics.push(diagnostic.clone());
                asset_source_errors.push(diagnostic);
                continue;
            };
            let Some(source) = entry.source.as_ref() else {
                let diagnostic = format!("included asset {path} is missing source");
                diagnostics.push(diagnostic.clone());
                asset_source_errors.push(diagnostic);
                continue;
            };
            let source = source_path(manifest_dir, source);
            let bytes = match std::fs::read(&source) {
                Ok(bytes) => bytes,
                Err(error) => {
                    let diagnostic =
                        format!("failed to read asset source {}: {error}", source.display());
                    diagnostics.push(diagnostic.clone());
                    asset_source_errors.push(diagnostic);
                    continue;
                }
            };
            pack_assets.push(ZrPackInputAsset::new(path.clone(), bytes));
        }

        ExportPackInputs {
            trim_report,
            pack_assets,
            diagnostics,
            asset_source_errors,
        }
    }
}

fn source_path(manifest_dir: &Path, source: &Path) -> PathBuf {
    if source.is_absolute() {
        source.to_path_buf()
    } else {
        manifest_dir.join(source)
    }
}

#[cfg(test)]
mod performance_contract_tests {
    #[test]
    fn included_assets_use_a_manifest_path_index() {
        let source = include_str!("manifest.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;
        let function = source.split("pub fn pack_inputs").nth(1).unwrap();
        let function = function.split("fn source_path").next().unwrap();

        assert!(function.contains("HashMap::with_capacity(self.assets.len())"));
        assert!(function.contains("entries_by_path.get(path.as_str())"));
        assert!(!function.contains("self.assets.iter().find"));
    }
}
