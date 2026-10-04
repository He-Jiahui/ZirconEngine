use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const PLUGINS_DIR: &str = "zircon_plugins";
const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
const PROJECT_PLUGIN_DIRS: &[&str] = &["Plugins", "plugins"];
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];
const EDITOR_CAPABILITY_PREFIX: &[u8] = b"editor.";
pub const PROJECT_PLUGIN_SCOPE: &str = "Project";
pub const ENGINE_PLUGIN_SCOPE: &str = "Engine";

/// 插件清单的 Hub 展示投影；`scope` 与清单路径共同区分项目插件和引擎插件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub maturity: String,
    pub editor_scoped: bool,
    pub default_packaging: Vec<String>,
    pub module_count: usize,
    pub scope: String,
    pub package_root: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    id: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    maturity: Option<String>,
    #[serde(default)]
    supported_targets: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    default_packaging: Vec<String>,
    #[serde(default)]
    modules: Vec<PluginManifestModule>,
}

#[derive(Debug, Deserialize)]
struct PluginManifestModule {
    name: Option<String>,
    kind: Option<String>,
    #[serde(default)]
    target_modes: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

pub fn discover_plugin_catalog<I>(repo_roots: I) -> Result<Vec<PluginCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    discover_plugin_catalog_with_project_roots(Vec::<PathBuf>::new(), repo_roots)
}

/// 刷新项目与源码引擎插件目录；项目根先于引擎根，供 Catalog 和 Editor 视图使用。
/// 清单读取失败会终止本次刷新，调用方应维持并报告原有视图状态。
pub fn discover_plugin_catalog_with_project_roots<P, R>(
    project_roots: P,
    repo_roots: R,
) -> Result<Vec<PluginCatalogEntry>, HubError>
where
    P: IntoIterator<Item = PathBuf>,
    R: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_manifests = HashSet::new();

    for project_root in project_roots {
        if project_root.is_dir() {
            collect_project_plugin_manifests(&project_root, &mut visited_manifests, &mut entries)?;
        }
    }

    for repo_root in repo_roots {
        let plugins_root = repo_root.join(PLUGINS_DIR);
        if !plugins_root.is_dir() {
            continue;
        }
        collect_plugin_manifests(
            &plugins_root,
            ENGINE_PLUGIN_SCOPE,
            &mut visited_manifests,
            &mut entries,
        )?;
        break;
    }
    // BUG: [CR-HUBCORE-0002] 项目与引擎清单仅按路径去重，同 id 仍会进入 Hub 状态；Tauri 原样投影后，Web 的 plugins 唯一性校验抛错，使整份状态在页面消费前被拒为协议失配。
    entries.sort_by(|left, right| {
        scope_rank(&left.scope)
            .cmp(&scope_rank(&right.scope))
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.package_root.cmp(&right.package_root))
    });
    Ok(entries)
}

fn scope_rank(scope: &str) -> usize {
    match scope {
        PROJECT_PLUGIN_SCOPE => 0,
        ENGINE_PLUGIN_SCOPE => 1,
        _ => 2,
    }
}

fn collect_project_plugin_manifests(
    project_root: &Path,
    visited_manifests: &mut HashSet<String>,
    entries: &mut Vec<PluginCatalogEntry>,
) -> Result<(), HubError> {
    let manifest_path = project_root.join(PLUGIN_MANIFEST_FILE);
    if manifest_path.is_file() {
        let manifest_key = project_filesystem_path_key(&manifest_path);
        if visited_manifests.insert(manifest_key) {
            entries.push(read_plugin_manifest(&manifest_path, PROJECT_PLUGIN_SCOPE)?);
        }
    }
    for plugin_dir in PROJECT_PLUGIN_DIRS {
        let root = project_root.join(plugin_dir);
        if root.is_dir() {
            collect_plugin_manifests(&root, PROJECT_PLUGIN_SCOPE, visited_manifests, entries)?;
        }
    }
    Ok(())
}

fn collect_plugin_manifests(
    directory: &Path,
    scope: &str,
    visited_manifests: &mut HashSet<String>,
    entries: &mut Vec<PluginCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_plugin_manifests(&path, scope, visited_manifests, entries)?;
        } else if file_type.is_file()
            && path.file_name().and_then(|name| name.to_str()) == Some(PLUGIN_MANIFEST_FILE)
        {
            let manifest_key = project_filesystem_path_key(&path);
            if visited_manifests.insert(manifest_key) {
                entries.push(read_plugin_manifest(&path, scope)?);
            }
        }
    }
    Ok(())
}

fn read_plugin_manifest(manifest_path: &Path, scope: &str) -> Result<PluginCatalogEntry, HubError> {
    let text = fs::read_to_string(manifest_path)?;
    let manifest = toml::from_str::<PluginManifest>(&text)?;
    let package_root = manifest_path
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf();
    let editor_scoped = plugin_manifest_is_editor_scoped(&manifest);
    let id = non_empty_or_else(manifest.id, || {
        package_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("plugin")
            .to_string()
    });
    let display_name = non_empty_or_else(manifest.display_name, || id.clone());
    Ok(PluginCatalogEntry {
        id,
        display_name,
        description: manifest.description.unwrap_or_default(),
        category: manifest
            .category
            .unwrap_or_else(|| "uncategorized".to_string()),
        maturity: manifest.maturity.unwrap_or_else(|| "unknown".to_string()),
        editor_scoped,
        default_packaging: manifest.default_packaging,
        module_count: manifest
            .modules
            .iter()
            .filter(|module| {
                module
                    .name
                    .as_deref()
                    .is_some_and(|name| !name.trim().is_empty())
            })
            .count(),
        scope: scope.to_string(),
        package_root,
        manifest_path: manifest_path.to_path_buf(),
    })
}

// 编辑器作用域从清单的目标、能力和模块声明推导，避免界面文案决定插件分类。
fn plugin_manifest_is_editor_scoped(manifest: &PluginManifest) -> bool {
    manifest
        .supported_targets
        .iter()
        .any(|target| is_editor_target(target))
        || manifest
            .capabilities
            .iter()
            .any(|capability| is_editor_capability(capability))
        || manifest.modules.iter().any(|module| {
            module
                .kind
                .as_deref()
                .is_some_and(|kind| kind.eq_ignore_ascii_case("editor"))
                || module
                    .target_modes
                    .iter()
                    .any(|mode| is_editor_target(mode))
                || module
                    .capabilities
                    .iter()
                    .any(|capability| is_editor_capability(capability))
        })
}

fn is_editor_target(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("editor") || value.eq_ignore_ascii_case("editor_host")
}

fn is_editor_capability(value: &str) -> bool {
    value
        .trim()
        .as_bytes()
        .get(..EDITOR_CAPABILITY_PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(EDITOR_CAPABILITY_PREFIX))
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn non_empty_or_else(value: Option<String>, fallback: impl FnOnce() -> String) -> String {
    match value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                fallback()
            } else if trimmed.len() == value.len() {
                value
            } else {
                trimmed.to_owned()
            }
        }
        None => fallback(),
    }
}

#[cfg(test)]
#[path = "tests/catalog.rs"]
mod tests;
