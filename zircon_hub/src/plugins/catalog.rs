use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::HubError;

const PLUGINS_DIR: &str = "zircon_plugins";
const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub maturity: String,
    pub default_packaging: Vec<String>,
    pub module_count: usize,
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
    for repo_root in repo_roots {
        let plugins_root = repo_root.join(PLUGINS_DIR);
        if !plugins_root.is_dir() {
            continue;
        }
        return discover_plugin_catalog_in_root(&plugins_root);
    }
    Ok(Vec::new())
}

fn discover_plugin_catalog_in_root(
    plugins_root: &Path,
) -> Result<Vec<PluginCatalogEntry>, HubError> {
    let mut entries = Vec::new();
    collect_plugin_manifests(plugins_root, &mut entries)?;
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(entries)
}

fn collect_plugin_manifests(
    directory: &Path,
    entries: &mut Vec<PluginCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_plugin_manifests(&path, entries)?;
        } else if file_type.is_file()
            && path.file_name().and_then(|name| name.to_str()) == Some(PLUGIN_MANIFEST_FILE)
        {
            entries.push(read_plugin_manifest(&path)?);
        }
    }
    Ok(())
}

fn read_plugin_manifest(manifest_path: &Path) -> Result<PluginCatalogEntry, HubError> {
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
        package_root,
        manifest_path: manifest_path.to_path_buf(),
    })
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
