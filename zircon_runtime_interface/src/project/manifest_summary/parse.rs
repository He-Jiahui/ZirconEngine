use crate::serialization::Loaded;
use serde::Deserialize;

use crate::project::{validate_project_name, ProjectGuid, ProjectTemplateReceipt, RelPath};

use super::{
    load_project_manifest_value_from_toml_str, summary::ensure_document_size,
    ProjectManifestSummary, ProjectManifestSummaryError, MAX_PROJECT_ASSET_ROOTS,
};

/// 按顺序限长、迁移、反序列化摘要输入投影并检查其轻量字段约束。
///
/// # 参数
/// document 是已在内存中的 TOML 文本；长度按 UTF-8 字节计。
///
/// # 返回值
/// [`crate::serialization::Loaded`] 中携带摘要和迁移前的格式来源，供 Editor 等调用方另行处理旧版决策。
///
/// # 约束
/// 仅验证共享摘要所需字段；它不读取文件、不检查场景或资产是否存在，也不替代 Runtime 完整 manifest 校验。
///
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

/// TOML 迁移后反序列化用的窄输入投影；部分字段只为校验而读取，不进入公开摘要。
///
#[derive(Deserialize)]
struct SummaryDocument {
    /// 使用共享跨平台项目名规则校验的目录名称。
    name: String,
    #[serde(default)]
    /// 可选 semver 要求；此投影只检查语法，不比较当前运行引擎。
    engine_version_req: Option<String>,
    #[serde(default)]
    /// 可选创建来源凭据；校验时其项目 GUID 与引擎要求必须和清单一致。
    template_receipt: Option<ProjectTemplateReceipt>,
    /// 非空场景引用文本；摘要层不解析 URI，也不检查目标资源是否存在。
    default_scene: String,
    /// 迁移后的结构版本；旧版在此处已由共享迁移链规范化。
    format_version: u32,
    #[serde(default)]
    /// 当前格式要求持久化身份；迁移旧版可暂时没有 GUID。
    project_guid: Option<ProjectGuid>,
    #[serde(default = "default_asset_roots")]
    /// 仅供路径合法性和根集合校验；字段不复制到轻量摘要。
    asset_roots: Vec<RelPath>,
    #[serde(default)]
    /// 反序列化时校验为项目相对路径，随后只保留其校验结果而不投影。
    settings: Option<RelPath>,
    /// 必须能解码为 u32；摘要不保存资产库版本值。
    library_version: u32,
}

/// 在摘要投影前完成名称、版本要求、模板来源、身份和资产根集合校验。
///
impl SummaryDocument {
    /// 根据迁移来源执行字段关系检查；仅未发生迁移的输入要求已有项目 GUID。
    ///
    /// # 参数
    /// requires_persisted_project_guid 由 migrated_from 是否为 None 决定。
    ///
    ///
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

    /// 投影共享展示和预检字段；资产根、settings 与 library_version 有意不随摘要暴露。
    ///
    ///
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

/// 限制根数，并拒绝相同或存在路径组件包含关系的资产根。
///
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

/// 按斜线边界判断组件祖先关系，使 a/child 属于 a 而 a-b 不属于 a。
///
fn is_descendant(ancestor: &RelPath, candidate: &RelPath) -> bool {
    candidate
        .as_str()
        .strip_prefix(ancestor.as_str())
        .is_some_and(|suffix| suffix.starts_with('/'))
}

/// 解析可选的 semver 要求文本；只判断语法，不判断它是否匹配某个运行引擎。
///
/// # 参数
/// None 表示清单未声明 engine_version_req；Some 文本直接交给 semver parser。
///
/// # 返回值
/// 成功只表示约束语法有效；Some 无法解析时保留原文并返回 InvalidEngineVersionReq。
///
/// # 参考
/// Runtime 完整清单通过 zircon_runtime/src/asset/project/manifest/validation.rs 复用此检查。
///
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

/// serde 缺省 asset_roots 时采用与旧版迁移一致的项目资产目录。
///
fn default_asset_roots() -> Vec<RelPath> {
    vec![RelPath::project_assets()]
}
