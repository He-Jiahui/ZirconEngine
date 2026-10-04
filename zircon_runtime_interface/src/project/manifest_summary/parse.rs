use crate::serialization::Loaded;
use serde::Deserialize;

use crate::project::{validate_project_name, ProjectGuid, ProjectTemplateReceipt, RelPath};

use super::{
    load_project_manifest_value_from_toml_str, summary::ensure_document_size,
    ProjectManifestSummary, ProjectManifestSummaryError, MAX_PROJECT_ASSET_ROOTS,
};

pub(super) fn parse_str(
    document: &str,
) -> Result<Loaded<ProjectManifestSummary>, ProjectManifestSummaryError> {
    ensure_document_size(document.len())?;
    // 共享适配器先完成结构版本迁移；摘要保留其来源，以区分旧格式与当前准入要求。
    let loaded = load_project_manifest_value_from_toml_str(document)?;
    let parsed: SummaryDocument = serde_json::from_value(loaded.value)
        .map_err(|source| ProjectManifestSummaryError::InvalidShape { source })?;
    parsed.validate(loaded.migrated_from.is_none())?;
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
    #[serde(default)]
    template_receipt: Option<ProjectTemplateReceipt>,
    default_scene: String,
    format_version: u32,
    #[serde(default)]
    project_guid: Option<ProjectGuid>,
    #[serde(default = "default_asset_roots")]
    asset_roots: Vec<RelPath>,
    #[serde(default)]
    settings: Option<RelPath>,
    library_version: u32,
}

impl SummaryDocument {
    fn validate(
        &self,
        requires_persisted_project_guid: bool,
    ) -> Result<(), ProjectManifestSummaryError> {
        validate_project_name(&self.name)
            .map_err(|source| ProjectManifestSummaryError::InvalidProjectName { source })?;
        if self.default_scene.trim().is_empty() {
            return Err(ProjectManifestSummaryError::InvalidValue {
                message: "default_scene cannot be empty".to_string(),
            });
        }
        validate_engine_version_req(self.engine_version_req.as_deref())?;
        // receipt 复述项目 GUID 和模板引擎范围；拒绝清单编辑后与创建来源脱节。
        if let Some(receipt) = &self.template_receipt {
            if self.project_guid != Some(receipt.project_guid()) {
                return Err(ProjectManifestSummaryError::InvalidValue {
                    message: "template receipt project_guid must match the project manifest"
                        .to_string(),
                });
            }
            if self.engine_version_req.as_deref() != receipt.descriptor().engine_version_req() {
                return Err(ProjectManifestSummaryError::InvalidValue {
                    message: "template receipt engine requirement must match the project manifest"
                        .to_string(),
                });
            }
        }
        // 只有已迁移的旧格式允许缺少 GUID；当前格式必须有可持续追踪的项目身份。
        if requires_persisted_project_guid && self.project_guid.is_none() {
            return Err(ProjectManifestSummaryError::InvalidValue {
                message: "project_guid is required by the current project manifest format"
                    .to_string(),
            });
        }
        if self.asset_roots.is_empty() {
            return Err(ProjectManifestSummaryError::InvalidValue {
                message: "asset_roots cannot be empty".to_string(),
            });
        }
        validate_asset_roots(&self.asset_roots)?;
        // 摘要省略 settings，但其值仍由 RelPath 校验；library_version 保留 u32 的结构校验。
        let _ = &self.settings;
        let _ = self.library_version;
        Ok(())
    }

    fn into_summary(self) -> ProjectManifestSummary {
        ProjectManifestSummary {
            name: self.name,
            engine_version_req: self.engine_version_req,
            template_receipt: self.template_receipt,
            default_scene: self.default_scene,
            format_version: self.format_version,
            project_guid: self.project_guid,
        }
    }
}

fn validate_asset_roots(asset_roots: &[RelPath]) -> Result<(), ProjectManifestSummaryError> {
    if asset_roots.len() > MAX_PROJECT_ASSET_ROOTS {
        return Err(ProjectManifestSummaryError::TooManyAssetRoots {
            max: MAX_PROJECT_ASSET_ROOTS,
            found: asset_roots.len(),
        });
    }

    // 按路径组件排序后只需比较相邻项，并保留 a 与 a-b 的边界语义。
    let mut ordered = asset_roots.iter().collect::<Vec<_>>();
    ordered.sort_unstable_by(|left, right| left.as_str().split('/').cmp(right.as_str().split('/')));
    for pair in ordered.windows(2) {
        let ancestor = pair[0];
        let descendant = pair[1];
        if ancestor == descendant {
            return Err(ProjectManifestSummaryError::DuplicateAssetRoot {
                root: ancestor.to_string(),
            });
        }
        if is_descendant(ancestor, descendant) {
            return Err(ProjectManifestSummaryError::OverlappingAssetRoots {
                ancestor: ancestor.to_string(),
                descendant: descendant.to_string(),
            });
        }
    }
    Ok(())
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
