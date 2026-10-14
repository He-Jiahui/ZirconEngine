use std::collections::BTreeSet;

use crate::project::{validate_project_name, ProjectGuid, ProjectManifestSummary, RelPath};

use super::embedded::{EmbeddedProjectTemplateEntry, RENDERABLE_EMPTY_ENTRIES};
use super::{
    project_template_descriptor, ProjectTemplateDescriptor, ProjectTemplateId,
    ProjectTemplatePackError, RenderedProjectTemplate, RenderedProjectTemplateEntry,
};

const PROJECT_MANIFEST_PATH: &str = "zircon-project.toml";

/// Renders the versioned template pack without consulting a source checkout at runtime.
/// 返回拥有数据的模板项；清单只改写项目身份与模板引擎范围，描述符摘要仍标识原始嵌入包。
///
/// # 参数
/// - `id`：要渲染的内置模板稳定 ID。
/// - `project_name`：新项目名称，按项目名称校验器的现有规则检查。
///
/// # 返回值
/// 返回包含规范描述符、解析后的清单摘要和拥有文件字节的渲染结果；名称、模板布局或清单不满足约束时返回错误。
///
/// # 生命周期
/// 结果不借用模板源码目录，可交由 Editor 创建事务写入 staging；本函数自身不访问项目目标路径。
pub fn render_project_template(
    id: ProjectTemplateId,
    project_name: &str,
) -> Result<RenderedProjectTemplate, ProjectTemplatePackError> {
    validate_project_name(project_name)
        .map_err(|source| ProjectTemplatePackError::InvalidProjectName { source })?;
    let source = match id {
        ProjectTemplateId::RenderableEmpty => RENDERABLE_EMPTY_ENTRIES,
    };
    let descriptor = project_template_descriptor(id);
    let mut entries = render_entries(source)?;
    let manifest = entries
        .iter_mut()
        .find(|entry| entry.path.as_str() == PROJECT_MANIFEST_PATH)
        .ok_or(ProjectTemplatePackError::MissingManifest)?;
    // 只有身份与引擎要求来自新项目/当前模板版本，其余清单内容继续来自嵌入包。
    rewrite_manifest_identity(
        &mut manifest.bytes,
        project_name,
        ProjectGuid::new(),
        descriptor.engine_version_req(),
    )?;
    validate_manifest_requirements(&manifest.bytes, descriptor)?;
    // 统一解析渲染后的清单，让返回摘要与真正写出的 manifest 字节对应。
    let summary = ProjectManifestSummary::parse_toml_bytes(&manifest.bytes)?.value;
    Ok(RenderedProjectTemplate {
        descriptor,
        summary,
        entries,
    })
}

fn validate_manifest_requirements(
    bytes: &[u8],
    descriptor: ProjectTemplateDescriptor,
) -> Result<(), ProjectTemplatePackError> {
    // 从渲染清单的 provider selections 中逐目标核对 descriptor 契约，不在此加载 provider。
    let source = std::str::from_utf8(bytes)
        .map_err(|source| ProjectTemplatePackError::ManifestUtf8 { source })?;
    let manifest = toml::from_str::<toml::Table>(source)
        .map_err(|source| ProjectTemplatePackError::ManifestToml { source })?;
    let selections = manifest
        .get("plugins")
        .and_then(|plugins| plugins.get("selections"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| ProjectTemplatePackError::ManifestRequirements {
            reason: "missing plugins.selections".to_string(),
        })?;

    // 描述符逐目标声明必需 provider；渲染前保证清单没有弱化这份准入契约。
    for requirement in descriptor.target_requirements() {
        let target = match requirement.target() {
            crate::runtime_build_set::ZrRuntimeModuleCompositionTargetV1::ClientRuntime => {
                "client_runtime"
            }
            crate::runtime_build_set::ZrRuntimeModuleCompositionTargetV1::ServerRuntime => {
                "server_runtime"
            }
            crate::runtime_build_set::ZrRuntimeModuleCompositionTargetV1::EditorHost => {
                "editor_host"
            }
        };
        for provider in requirement.required_runtime_providers() {
            // 重复匹配也属于无效选择，不能让任一条启用项掩盖同 ID 的另一条弱化项。
            let mut matching = selections.iter().filter(|selection| {
                selection.get("id").and_then(toml::Value::as_str) == Some(provider)
            });
            let Some(selection) = matching.next() else {
                return Err(missing_provider_requirement(target, provider, 0));
            };
            if matching.next().is_some() {
                return Err(missing_provider_requirement(
                    target,
                    provider,
                    2 + matching.count(),
                ));
            }
            let enabled = selection
                .get("enabled")
                .and_then(toml::Value::as_bool)
                .unwrap_or(true);
            let required = selection
                .get("required")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false);
            let supports_target = selection
                .get("target_modes")
                .and_then(toml::Value::as_array)
                // 与 Runtime 选择器一致：省略或留空 target_modes 表示默认支持全部目标。
                .is_none_or(|targets| {
                    targets.is_empty()
                        || targets
                            .iter()
                            .any(|candidate| candidate.as_str() == Some(target))
                });
            if !enabled || !required || !supports_target {
                return Err(ProjectTemplatePackError::ManifestRequirements {
                    reason: format!(
                        "provider {provider} must be enabled, required, and available for target {target}"
                    ),
                });
            }
        }
    }
    Ok(())
}

fn missing_provider_requirement(
    target: &str,
    provider: &str,
    found: usize,
) -> ProjectTemplatePackError {
    ProjectTemplatePackError::ManifestRequirements {
        reason: format!(
            "target {target} requires exactly one {provider} provider selection, found {found}"
        ),
    }
}

fn render_entries(
    source: &[EmbeddedProjectTemplateEntry],
) -> Result<Vec<RenderedProjectTemplateEntry>, ProjectTemplatePackError> {
    // 先逐项校验路径并拒绝重复，再检查目录前缀冲突，确保返回布局可逐文件写入。
    let mut paths = BTreeSet::new();
    let mut entries = Vec::with_capacity(source.len());
    for source_entry in source {
        let entry = render_entry(source_entry)?;
        if !paths.insert(entry.path.as_str().to_string()) {
            return Err(ProjectTemplatePackError::DuplicateEntryPath {
                path: entry.path.to_string(),
            });
        }
        entries.push(entry);
    }
    // 同名路径已在上一步拒绝；再排除文件同时充当目录的布局以便创建端逐项落盘。
    for path in &paths {
        let mut ancestor = path.as_str();
        while let Some((parent, _)) = ancestor.rsplit_once('/') {
            if paths.contains(parent) {
                return Err(ProjectTemplatePackError::EntryPathConflictsWithFile {
                    path: path.clone(),
                    ancestor: parent.to_string(),
                });
            }
            ancestor = parent;
        }
    }
    Ok(entries)
}

fn render_entry(
    entry: &EmbeddedProjectTemplateEntry,
) -> Result<RenderedProjectTemplateEntry, ProjectTemplatePackError> {
    // RelPath 只提供词法相对路径约束；创建端还要把它限制在 staging 根目录内。
    Ok(RenderedProjectTemplateEntry {
        path: RelPath::parse(entry.path)?,
        bytes: entry.bytes.to_vec(),
    })
}

fn rewrite_manifest_identity(
    bytes: &mut Vec<u8>,
    project_name: &str,
    project_guid: ProjectGuid,
    engine_version_req: Option<&str>,
) -> Result<(), ProjectTemplatePackError> {
    let source = std::str::from_utf8(bytes)
        .map_err(|source| ProjectTemplatePackError::ManifestUtf8 { source })?;
    let mut manifest = toml::from_str::<toml::Table>(source)
        .map_err(|source| ProjectTemplatePackError::ManifestToml { source })?;
    // 用 TOML 值只替换身份字段，保留模板其余结构化配置，不做文本替换。
    manifest.insert(
        "name".to_string(),
        toml::Value::String(project_name.to_string()),
    );
    manifest.insert(
        "project_guid".to_string(),
        toml::Value::String(project_guid.to_string()),
    );
    if let Some(engine_version_req) = engine_version_req {
        manifest.insert(
            "engine_version_req".to_string(),
            toml::Value::String(engine_version_req.to_string()),
        );
    }
    // 重新编码 TOML 后字节布局可能变化；描述符摘要仍基于原始嵌入 manifest。
    *bytes = toml::to_string_pretty(&manifest)
        .map_err(|source| ProjectTemplatePackError::ManifestEncode { source })?
        .into_bytes();
    Ok(())
}

#[cfg(test)]
#[path = "tests/render.rs"]
mod tests;
