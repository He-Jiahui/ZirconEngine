use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

use super::diagnostic::PluginDiagnostic;
use super::manifest_sync::{
    discover_manifest_declarations, read_sdk_api_version, read_workspace_version,
    synchronize_manifest_file, SyncMode, SyncOutcome,
};
use super::validate::{validate_native_artifact, validate_plugin_manifest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCheckReport {
    pub checked_manifests: usize,
    pub diagnostics: Vec<PluginDiagnostic>,
}

pub fn check_plugin_workspace(repo_root: &Path) -> Result<PluginCheckReport, PluginCheckError> {
    check_plugin_workspace_with_artifact_root(repo_root, None)
}

pub fn check_plugin_workspace_with_artifact_root(
    repo_root: &Path,
    artifact_root: Option<&Path>,
) -> Result<PluginCheckReport, PluginCheckError> {
    let plugin_root = repo_root.join("zircon_plugins");
    let workspace_path = plugin_root.join("Cargo.toml");
    let workspace: Value = fs::read_to_string(&workspace_path)?.parse()?;
    let workspace_members = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or_else(|| PluginCheckError::new("zircon_plugins Cargo.toml is missing members"))?
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();
    let runtime_catalog_source =
        fs::read_to_string(plugin_root.join("first_party_runtime_catalog/src/lib.rs"))?;
    let runtime_catalog_cargo: Value =
        fs::read_to_string(plugin_root.join("first_party_runtime_catalog/Cargo.toml"))?.parse()?;
    let editor_catalog_source =
        fs::read_to_string(plugin_root.join("first_party_editor_catalog/src/catalog.rs"))?;
    let editor_catalog_cargo: Value =
        fs::read_to_string(plugin_root.join("first_party_editor_catalog/Cargo.toml"))?.parse()?;
    let app_cargo: Value = fs::read_to_string(repo_root.join("zircon_app/Cargo.toml"))?.parse()?;
    let mut manifest_paths = Vec::new();
    collect_plugin_manifests(&plugin_root, &mut manifest_paths)?;
    manifest_paths.sort();
    let mut diagnostics = Vec::new();

    for manifest_path in &manifest_paths {
        let Some(package_root) = manifest_path.parent() else {
            return Err(PluginCheckError::new(format!(
                "plugin manifest path `{}` has no package root",
                manifest_path.display()
            )));
        };
        let display_path = relative_path(repo_root, manifest_path);
        let manifest_text = fs::read_to_string(manifest_path)?;
        for mut diagnostic in validate_plugin_manifest(&manifest_text, Some(package_root)) {
            diagnostic.message = format!("{display_path}: {}", diagnostic.message);
            diagnostics.push(diagnostic);
        }
        if let (Some(artifact_root), Some(artifact_path)) = (
            artifact_root,
            native_artifact_path(&manifest_text, artifact_root),
        ) {
            for mut diagnostic in validate_native_artifact(&manifest_text, &artifact_path) {
                diagnostic.message = format!("{display_path}: {}", diagnostic.message);
                diagnostics.push(diagnostic);
            }
        }
        for owner in ["runtime", "editor", "dist", "native"] {
            let cargo_path = package_root.join(owner).join("Cargo.toml");
            if !cargo_path.is_file() {
                continue;
            }
            let member = relative_path(&plugin_root, &package_root.join(owner));
            if !workspace_members.contains(&member) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.workspace.member.missing",
                    format!("{display_path}: crate `{member}` is not a plugin workspace member"),
                    "Run `cargo zircon plugin new` for new packages or add the missing member structurally.",
                ));
            }
        }
        let package_relative = relative_path(&plugin_root, package_root);
        let expected_include = format!("../../{package_relative}/plugin.toml");
        if !runtime_catalog_source.contains(&expected_include) {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.catalog.manifest.missing",
                format!("{display_path}: runtime catalog does not include this manifest"),
                "Add the package through cargo-zircon catalog wiring.",
            ));
        }
    }

    validate_catalog_wiring(
        "runtime",
        "zircon_first_party_runtime_catalog",
        &runtime_catalog_cargo,
        &runtime_catalog_source,
        &app_cargo,
        &mut diagnostics,
    );
    validate_catalog_wiring(
        "editor",
        "zircon_first_party_editor_catalog",
        &editor_catalog_cargo,
        &editor_catalog_source,
        &app_cargo,
        &mut diagnostics,
    );

    let version = read_workspace_version(repo_root)?;
    let sdk_api_version = read_sdk_api_version(repo_root)?;
    for owner in discover_manifest_declarations(repo_root)? {
        if synchronize_manifest_file(
            owner.declaration_path(),
            owner.manifest_path(),
            &version,
            &sdk_api_version,
            SyncMode::Check,
        )? == SyncOutcome::Drift
        {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.manifest.declaration_drift",
                format!(
                    "{} differs from its Rust PluginDeclaration projection",
                    relative_path(repo_root, owner.manifest_path())
                ),
                format!(
                    "Run `cargo zircon plugin sync-manifest {}` and commit the generated snapshot.",
                    owner.package_id()
                ),
            ));
        }
    }

    Ok(PluginCheckReport {
        checked_manifests: manifest_paths.len(),
        diagnostics,
    })
}

fn native_artifact_path(manifest_text: &str, artifact_root: &Path) -> Option<PathBuf> {
    let manifest: Value = manifest_text.parse().ok()?;
    let dist_crate = manifest
        .get("distribution")?
        .get("dist_crate")?
        .as_str()?
        .replace('-', "_");
    let file_name = if cfg!(target_os = "windows") {
        format!("{dist_crate}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{dist_crate}.dylib")
    } else {
        format!("lib{dist_crate}.so")
    };
    Some(artifact_root.join(file_name))
}

fn validate_catalog_wiring(
    owner: &str,
    catalog_crate: &str,
    catalog_cargo: &Value,
    catalog_source: &str,
    app_cargo: &Value,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let Some(features) = catalog_cargo.get("features").and_then(Value::as_table) else {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.catalog.features.missing",
            format!("{owner} catalog is missing [features]"),
            "Restore the generated catalog feature table.",
        ));
        return;
    };
    let Some(dependencies) = catalog_cargo.get("dependencies").and_then(Value::as_table) else {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.catalog.dependencies.missing",
            format!("{owner} catalog is missing [dependencies]"),
            "Restore the generated catalog dependency table.",
        ));
        return;
    };
    let Some(app_features) = app_cargo.get("features").and_then(Value::as_table) else {
        diagnostics.push(PluginDiagnostic::new(
            "plugin.app.features.missing",
            "zircon_app is missing [features]",
            "Restore the app feature table before wiring plugins.",
        ));
        return;
    };

    for (feature, entries) in features {
        for entry in entries
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if let Some(dependency) = entry.strip_prefix("dep:") {
                if !dependencies.contains_key(dependency) {
                    diagnostics.push(PluginDiagnostic::new(
                        "plugin.catalog.feature_dependency.missing",
                        format!(
                            "{owner} catalog feature `{feature}` references missing dependency `{dependency}`"
                        ),
                        "Regenerate the catalog Cargo wiring through cargo zircon plugin new.",
                    ));
                }
            } else if !entry.contains('/') && !features.contains_key(entry) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.catalog.feature_reference.missing",
                    format!(
                        "{owner} catalog feature `{feature}` references missing catalog feature `{entry}`"
                    ),
                    "Restore the generated feature or remove the stale reference.",
                ));
            }
        }
    }

    let registered_crates = catalog_registration_crates(catalog_source);
    for registered in &registered_crates {
        if !dependencies.contains_key(registered) {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.catalog.registration_dependency.missing",
                format!("{owner} catalog registration `{registered}` has no Cargo dependency"),
                "Regenerate the catalog dependency and feature wiring.",
            ));
        }
    }

    let catalog_reference_prefix = format!("{catalog_crate}/");
    for (app_feature, entries) in app_features {
        for entry in entries
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let Some(catalog_feature) = entry.strip_prefix(&catalog_reference_prefix) else {
                continue;
            };
            if !features.contains_key(catalog_feature) {
                diagnostics.push(PluginDiagnostic::new(
                    "plugin.app.catalog_feature.invalid",
                    format!(
                        "zircon_app feature `{app_feature}` references missing {owner} catalog feature `{catalog_feature}`"
                    ),
                    "Regenerate the app and catalog feature wiring together.",
                ));
            }
        }
    }

    for (dependency, definition) in dependencies {
        let expected_suffix = format!("_{owner}");
        let is_optional_plugin = dependency.starts_with("zircon_plugin_")
            && dependency.ends_with(&expected_suffix)
            && definition
                .as_table()
                .and_then(|table| table.get("optional"))
                .and_then(Value::as_bool)
                == Some(true);
        if !is_optional_plugin {
            continue;
        }
        let dependency_reference = format!("dep:{dependency}");
        let dependency_features = features
            .iter()
            .filter_map(|(feature, entries)| {
                entries
                    .as_array()
                    .is_some_and(|entries| {
                        entries
                            .iter()
                            .any(|entry| entry.as_str() == Some(dependency_reference.as_str()))
                    })
                    .then_some(feature.as_str())
            })
            .collect::<Vec<_>>();
        if dependency_features.is_empty() {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.catalog.dependency_feature.missing",
                format!("{owner} catalog dependency `{dependency}` is not enabled by a feature"),
                "Generate one catalog feature containing the dependency's dep: entry.",
            ));
            continue;
        }
        if !registered_crates.contains(dependency) {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.catalog.registration.missing",
                format!("{owner} catalog dependency `{dependency}` has no registration branch"),
                "Regenerate the catalog registration branch through cargo zircon plugin new.",
            ));
        } else if !dependency_features
            .iter()
            .any(|feature| catalog_source.contains(&format!("#[cfg(feature = \"{feature}\")]")))
        {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.catalog.registration_feature.invalid",
                format!(
                    "{owner} catalog registration `{dependency}` is not guarded by its dependency feature"
                ),
                "Regenerate the registration branch and its cfg feature together.",
            ));
        }
        let app_reaches_dependency = app_features.values().any(|entries| {
            entries.as_array().is_some_and(|entries| {
                entries.iter().filter_map(Value::as_str).any(|entry| {
                    dependency_features
                        .iter()
                        .any(|feature| entry == format!("{catalog_reference_prefix}{feature}"))
                })
            })
        });
        if !app_reaches_dependency {
            diagnostics.push(PluginDiagnostic::new(
                "plugin.app.catalog_feature.missing",
                format!(
                    "zircon_app has no feature enabling {owner} catalog dependency `{dependency}`"
                ),
                "Regenerate the zircon_app feature through cargo zircon plugin new.",
            ));
        }
    }
}

fn catalog_registration_crates(source: &str) -> HashSet<String> {
    const PREFIX: &str = "zircon_plugin_";
    const SUFFIX: &str = "::plugin_registration()";
    let mut crates = HashSet::new();
    for line in source.lines() {
        let Some(start) = line.find(PREFIX) else {
            continue;
        };
        let Some(end) = line[start..].find(SUFFIX) else {
            continue;
        };
        crates.insert(line[start..start + end].to_string());
    }
    crates
}

fn collect_plugin_manifests(
    directory: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), PluginCheckError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|value| value.to_str()) == Some("target") {
                continue;
            }
            collect_plugin_manifests(&path, paths)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some("plugin.toml") {
            paths.push(path);
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Debug)]
pub struct PluginCheckError {
    message: String,
}

impl PluginCheckError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PluginCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PluginCheckError {}

impl From<std::io::Error> for PluginCheckError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<toml::de::Error> for PluginCheckError {
    fn from(error: toml::de::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<super::manifest_sync::ManifestSyncError> for PluginCheckError {
    fn from(error: super::manifest_sync::ManifestSyncError) -> Self {
        Self::new(error.to_string())
    }
}
