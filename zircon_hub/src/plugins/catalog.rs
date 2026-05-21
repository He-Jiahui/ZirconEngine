use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::HubError;

const PLUGINS_DIR: &str = "zircon_plugins";
const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
const PROJECT_PLUGIN_DIRS: &[&str] = &["Plugins", "plugins"];
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];
pub const PROJECT_PLUGIN_SCOPE: &str = "Project";
pub const ENGINE_PLUGIN_SCOPE: &str = "Engine";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub maturity: String,
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
    default_packaging: Vec<String>,
    #[serde(default)]
    modules: Vec<PluginManifestModule>,
}

#[derive(Debug, Deserialize)]
struct PluginManifestModule {
    name: Option<String>,
}

pub fn discover_plugin_catalog<I>(repo_roots: I) -> Result<Vec<PluginCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    discover_plugin_catalog_with_project_roots(Vec::<PathBuf>::new(), repo_roots)
}

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
        let manifest_key = normalized_path_key(&manifest_path);
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
            let manifest_key = normalized_path_key(&path);
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
    let fallback_id = package_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("plugin")
        .to_string();
    let id = non_empty_or(manifest.id, fallback_id);
    let display_name = non_empty_or(manifest.display_name, id.clone());
    Ok(PluginCatalogEntry {
        id,
        display_name,
        description: manifest.description.unwrap_or_default(),
        category: manifest
            .category
            .unwrap_or_else(|| "uncategorized".to_string()),
        maturity: manifest.maturity.unwrap_or_else(|| "unknown".to_string()),
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

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn normalized_path_key(path: &Path) -> String {
    let value = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    if cfg!(target_os = "windows") {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

fn non_empty_or(value: Option<String>, fallback: String) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn discover_plugin_catalog_reads_manifest_metadata() {
        let repo_root = temp_repo_root("catalog");
        let plugin_root = repo_root.join(PLUGINS_DIR).join("demo");
        fs::create_dir_all(&plugin_root).unwrap();
        fs::write(
            plugin_root.join(PLUGIN_MANIFEST_FILE),
            r#"
id = "demo"
display_name = "Demo Plugin"
description = "Demo plugin description."
category = "runtime"
maturity = "beta"
default_packaging = ["native_dynamic", "library_embed"]

[[modules]]
name = "demo.runtime"

[[modules]]
name = "demo.editor"
"#,
        )
        .unwrap();

        let entries = discover_plugin_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "demo");
        assert_eq!(entries[0].scope, ENGINE_PLUGIN_SCOPE);
        assert_eq!(entries[0].display_name, "Demo Plugin");
        assert_eq!(
            entries[0].default_packaging,
            vec!["native_dynamic".to_string(), "library_embed".to_string()]
        );
        assert_eq!(entries[0].module_count, 2);
    }

    #[test]
    fn discover_plugin_catalog_falls_back_to_next_root() {
        let missing_root = temp_repo_root("catalog-missing");
        let repo_root = temp_repo_root("catalog-fallback");
        let plugin_root = repo_root.join(PLUGINS_DIR).join("fallback");
        fs::create_dir_all(&plugin_root).unwrap();
        fs::write(plugin_root.join(PLUGIN_MANIFEST_FILE), "id = \"fallback\"").unwrap();

        let entries = discover_plugin_catalog([missing_root.clone(), repo_root.clone()]).unwrap();
        fs::remove_dir_all(missing_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries[0].id, "fallback");
        assert_eq!(entries[0].scope, ENGINE_PLUGIN_SCOPE);
    }

    #[test]
    fn discover_plugin_catalog_reads_project_and_engine_scopes() {
        let project_root = temp_repo_root("catalog-project-scope");
        let repo_root = temp_repo_root("catalog-engine-scope");
        let project_plugin_root = project_root.join("Plugins").join("project_runtime");
        let engine_plugin_root = repo_root.join(PLUGINS_DIR).join("engine_runtime");
        fs::create_dir_all(&project_plugin_root).unwrap();
        fs::create_dir_all(&engine_plugin_root).unwrap();
        fs::write(
            project_plugin_root.join(PLUGIN_MANIFEST_FILE),
            "id = \"project_runtime\"\ndisplay_name = \"Project Runtime\"",
        )
        .unwrap();
        fs::write(
            engine_plugin_root.join(PLUGIN_MANIFEST_FILE),
            "id = \"engine_runtime\"\ndisplay_name = \"Engine Runtime\"",
        )
        .unwrap();

        let entries =
            discover_plugin_catalog_with_project_roots([project_root.clone()], [repo_root.clone()])
                .unwrap();
        fs::remove_dir_all(project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.id == "project_runtime" && entry.scope == PROJECT_PLUGIN_SCOPE));
        assert!(entries
            .iter()
            .any(|entry| entry.id == "engine_runtime" && entry.scope == ENGINE_PLUGIN_SCOPE));
    }

    fn temp_repo_root(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("zircon-hub-{label}-{now}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
