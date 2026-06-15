use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPackInputs {
    pub trim_report: ZrPackTrimReport,
    pub pack_assets: Vec<ZrPackInputAsset>,
    pub diagnostics: Vec<String>,
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

    pub fn pack_inputs(&self, manifest_dir: &Path) -> Result<ExportPackInputs, String> {
        let trim_report = ZrPackTrimPlanner::trim(self.trim_config(), self.trim_inputs());
        let mut diagnostics = trim_report.diagnostics.clone();
        if trim_report.has_missing_dependencies() {
            diagnostics
                .push("pack stage stopped because asset dependencies are missing".to_string());
        }

        let mut pack_assets = Vec::new();
        for path in &trim_report.included_assets {
            let entry = self
                .assets
                .iter()
                .find(|asset| asset.path == *path)
                .ok_or_else(|| format!("included asset {path} was not found in manifest"))?;
            let source = entry
                .source
                .as_ref()
                .ok_or_else(|| format!("included asset {path} is missing source"))?;
            let source = source_path(manifest_dir, source);
            let bytes = std::fs::read(&source).map_err(|error| {
                format!("failed to read asset source {}: {error}", source.display())
            })?;
            pack_assets.push(ZrPackInputAsset::new(path.clone(), bytes));
        }

        Ok(ExportPackInputs {
            trim_report,
            pack_assets,
            diagnostics,
        })
    }
}

fn source_path(manifest_dir: &Path, source: &Path) -> PathBuf {
    if source.is_absolute() {
        source.to_path_buf()
    } else {
        manifest_dir.join(source)
    }
}
