use crate::serialization::Loaded;
use serde::Deserialize;
use std::collections::BTreeSet;

use crate::project::RelPath;

use super::{
    ProjectManifestSummary, ProjectManifestSummaryError, load_project_manifest_value_from_toml_str,
};

pub(super) fn parse_str(
    document: &str,
) -> Result<Loaded<ProjectManifestSummary>, ProjectManifestSummaryError> {
    let loaded = load_project_manifest_value_from_toml_str(document)?;
    let parsed: SummaryDocument = serde_json::from_value(loaded.value)
        .map_err(|source| ProjectManifestSummaryError::InvalidShape { source })?;
    parsed.validate()?;
    Ok(Loaded {
        value: parsed.into_summary(),
        migrated_from: loaded.migrated_from,
    })
}

#[derive(Deserialize)]
struct SummaryDocument {
    name: String,
    #[serde(default)]
    engine_version_req: Option<String>,
    default_scene: String,
    format_version: u32,
    #[serde(default = "default_asset_roots")]
    asset_roots: Vec<RelPath>,
    #[serde(default)]
    settings: Option<RelPath>,
    #[serde(alias = "schema_version")]
    library_version: u32,
}

impl SummaryDocument {
    fn validate(&self) -> Result<(), ProjectManifestSummaryError> {
        if self.name.trim().is_empty() {
            return Err(ProjectManifestSummaryError::InvalidValue {
                message: "name cannot be empty".to_string(),
            });
        }
        if self.default_scene.trim().is_empty() {
            return Err(ProjectManifestSummaryError::InvalidValue {
                message: "default_scene cannot be empty".to_string(),
            });
        }
        validate_engine_version_req(self.engine_version_req.as_deref())?;
        if self.asset_roots.is_empty() {
            return Err(ProjectManifestSummaryError::InvalidValue {
                message: "asset_roots cannot be empty".to_string(),
            });
        }
        let mut roots = BTreeSet::new();
        for root in &self.asset_roots {
            if !roots.insert(root.as_str()) {
                return Err(ProjectManifestSummaryError::InvalidValue {
                    message: format!("duplicate normalized asset root {root}"),
                });
            }
        }
        for (index, left) in self.asset_roots.iter().enumerate() {
            for right in self.asset_roots.iter().skip(index + 1) {
                if roots_overlap(left, right) {
                    return Err(ProjectManifestSummaryError::InvalidValue {
                        message: format!("asset roots {left} and {right} overlap"),
                    });
                }
            }
        }
        let _ = &self.settings;
        let _ = self.library_version;
        Ok(())
    }

    fn into_summary(self) -> ProjectManifestSummary {
        ProjectManifestSummary {
            name: self.name,
            engine_version_req: self.engine_version_req,
            default_scene: self.default_scene,
            format_version: self.format_version,
        }
    }
}

fn roots_overlap(left: &RelPath, right: &RelPath) -> bool {
    is_descendant(left, right) || is_descendant(right, left)
}

fn is_descendant(ancestor: &RelPath, candidate: &RelPath) -> bool {
    candidate
        .as_str()
        .strip_prefix(ancestor.as_str())
        .is_some_and(|suffix| suffix.starts_with('/'))
}

pub fn validate_engine_version_req(
    requirement: Option<&str>,
) -> Result<(), ProjectManifestSummaryError> {
    let Some(requirement) = requirement else {
        return Ok(());
    };
    semver::VersionReq::parse(requirement).map_err(|source| {
        ProjectManifestSummaryError::InvalidEngineVersionReq {
            value: requirement.to_string(),
            source,
        }
    })?;
    Ok(())
}

fn default_asset_roots() -> Vec<RelPath> {
    vec![RelPath::project_assets()]
}
