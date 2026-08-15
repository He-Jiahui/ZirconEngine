mod templates;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};

use super::manifest_sync::{
    engine_compatibility_range, read_sdk_api_version, read_workspace_version, synchronize_manifest,
    PluginDeclarationProjection,
};
use templates::render_package_files;

const RUNTIME_REGISTRATION_MARKER: &str = "    // @cargo-zircon:runtime-registration-end";
const EDITOR_REGISTRATION_MARKER: &str = "    // @cargo-zircon:editor-registration-end";
const STATIC_MANIFEST_MARKER: &str = "    // @cargo-zircon:static-manifest-end";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginKind {
    Importer,
    System,
    Editor,
}

impl PluginKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "importer" => Some(Self::Importer),
            "system" => Some(Self::System),
            "editor" => Some(Self::Editor),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewPluginOptions<'a> {
    pub repo_root: &'a Path,
    pub id: &'a str,
    pub kind: PluginKind,
    pub native: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldReport {
    pub package_id: String,
    pub created_paths: Vec<PathBuf>,
}

pub fn scaffold_plugin(options: &NewPluginOptions<'_>) -> Result<ScaffoldReport, ScaffoldError> {
    validate_plugin_id(options.id)?;
    let plugin_root = options.repo_root.join("zircon_plugins");
    let package_root = plugin_root.join(options.id);
    if package_root.exists() {
        return Err(ScaffoldError::new(format!(
            "plugin package `{}` already exists at {}",
            options.id,
            package_root.display()
        )));
    }

    let workspace_path = plugin_root.join("Cargo.toml");
    let catalog_cargo_path = plugin_root.join("first_party_runtime_catalog/Cargo.toml");
    let catalog_source_path = plugin_root.join("first_party_runtime_catalog/src/lib.rs");
    let editor_catalog_cargo_path = plugin_root.join("first_party_editor_catalog/Cargo.toml");
    let editor_catalog_source_path = plugin_root.join("first_party_editor_catalog/src/catalog.rs");
    let app_cargo_path = options.repo_root.join("zircon_app/Cargo.toml");
    let workspace_original = fs::read_to_string(&workspace_path)?;
    let catalog_cargo_original = fs::read_to_string(&catalog_cargo_path)?;
    let catalog_source_original = fs::read_to_string(&catalog_source_path)?;
    let editor_catalog_cargo_original = fs::read_to_string(&editor_catalog_cargo_path)?;
    let editor_catalog_source_original = fs::read_to_string(&editor_catalog_source_path)?;
    let app_cargo_original = fs::read_to_string(&app_cargo_path)?;
    require_marker(&catalog_source_original, STATIC_MANIFEST_MARKER)?;
    if options.kind != PluginKind::Editor {
        require_marker(&catalog_source_original, RUNTIME_REGISTRATION_MARKER)?;
    } else {
        require_marker(&editor_catalog_source_original, EDITOR_REGISTRATION_MARKER)?;
    }

    let package_version = read_workspace_version(options.repo_root)?;
    let sdk_api_version = read_sdk_api_version(options.repo_root)?;
    let engine_compatibility = engine_compatibility_range(&package_version)?;
    let mut package_files = render_package_files(
        options,
        &package_version,
        &sdk_api_version,
        &engine_compatibility,
    );
    synchronize_scaffold_manifest(
        &mut package_files,
        options,
        &package_version,
        &sdk_api_version,
    )?;
    let workspace_updated = wire_plugin_workspace(
        &workspace_original,
        options.id,
        options.kind,
        options.native,
    )?;
    let catalog_cargo_updated = if options.kind == PluginKind::Editor {
        catalog_cargo_original.clone()
    } else {
        wire_runtime_catalog_cargo(&catalog_cargo_original, options.id)?
    };
    let catalog_source_updated =
        wire_catalog_source(&catalog_source_original, options.id, options.kind)?;
    let editor_catalog_cargo_updated = if options.kind == PluginKind::Editor {
        wire_editor_catalog_cargo(&editor_catalog_cargo_original, options.id)?
    } else {
        editor_catalog_cargo_original.clone()
    };
    let editor_catalog_source_updated = if options.kind == PluginKind::Editor {
        wire_editor_catalog_source(&editor_catalog_source_original, options.id)?
    } else {
        editor_catalog_source_original.clone()
    };
    let app_cargo_updated = wire_app_cargo(&app_cargo_original, options.id, options.kind)?;

    let result: Result<ScaffoldReport, ScaffoldError> = (|| {
        let mut created_paths = Vec::new();
        for (relative_path, contents) in package_files {
            let path = package_root.join(relative_path);
            let parent = path.parent().ok_or_else(|| {
                ScaffoldError::new(format!(
                    "generated template path `{}` has no parent directory",
                    path.display()
                ))
            })?;
            fs::create_dir_all(parent)?;
            fs::write(&path, contents)?;
            created_paths.push(path);
        }
        fs::write(&workspace_path, workspace_updated)?;
        fs::write(&catalog_cargo_path, catalog_cargo_updated)?;
        fs::write(&catalog_source_path, catalog_source_updated)?;
        fs::write(&editor_catalog_cargo_path, editor_catalog_cargo_updated)?;
        fs::write(&editor_catalog_source_path, editor_catalog_source_updated)?;
        fs::write(&app_cargo_path, app_cargo_updated)?;
        Ok(ScaffoldReport {
            package_id: options.id.to_string(),
            created_paths,
        })
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&package_root);
        let _ = fs::write(&workspace_path, workspace_original);
        let _ = fs::write(&catalog_cargo_path, catalog_cargo_original);
        let _ = fs::write(&catalog_source_path, catalog_source_original);
        let _ = fs::write(&editor_catalog_cargo_path, editor_catalog_cargo_original);
        let _ = fs::write(&editor_catalog_source_path, editor_catalog_source_original);
        let _ = fs::write(&app_cargo_path, app_cargo_original);
    }
    result
}

fn synchronize_scaffold_manifest(
    files: &mut [(PathBuf, String)],
    options: &NewPluginOptions<'_>,
    package_version: &str,
    sdk_api_version: &str,
) -> Result<(), ScaffoldError> {
    let owner = if options.kind == PluginKind::Editor {
        "editor"
    } else {
        "runtime"
    };
    let declaration_path = PathBuf::from(format!("{owner}/src/capability.rs"));
    let declaration_source = files
        .iter()
        .find(|(path, _)| path == &declaration_path)
        .map(|(_, contents)| contents.as_str())
        .ok_or_else(|| ScaffoldError::new("scaffold is missing declaration source"))?;
    let declaration = PluginDeclarationProjection::parse(declaration_source)?;
    let manifest = files
        .iter()
        .find(|(path, _)| path == Path::new("plugin.toml"))
        .map(|(_, contents)| contents.as_str())
        .ok_or_else(|| ScaffoldError::new("scaffold is missing plugin.toml"))?;
    let synchronized =
        synchronize_manifest(manifest, &declaration, package_version, sdk_api_version)?;
    let (_, manifest) = files
        .iter_mut()
        .find(|(path, _)| path == Path::new("plugin.toml"))
        .ok_or_else(|| ScaffoldError::new("scaffold is missing plugin.toml"))?;
    *manifest = synchronized;
    Ok(())
}

fn wire_plugin_workspace(
    source: &str,
    id: &str,
    kind: PluginKind,
    native: bool,
) -> Result<String, ScaffoldError> {
    let mut document: DocumentMut = source.parse()?;
    let members = document
        .get_mut("workspace")
        .and_then(Item::as_table_mut)
        .and_then(|workspace| workspace.get_mut("members"))
        .and_then(Item::as_array_mut)
        .ok_or_else(|| ScaffoldError::new("zircon_plugins workspace is missing members"))?;
    let owner = match kind {
        PluginKind::Editor => "editor",
        PluginKind::Importer | PluginKind::System => "runtime",
    };
    append_unique_string(members, format!("{id}/{owner}"));
    if native {
        append_unique_string(members, format!("{id}/dist"));
    }
    Ok(document.to_string())
}

fn wire_runtime_catalog_cargo(source: &str, id: &str) -> Result<String, ScaffoldError> {
    let mut document: DocumentMut = source.parse()?;
    let dependency_name = format!("zircon_plugin_{id}_runtime");
    let feature_dependency = format!("dep:{dependency_name}");
    let feature_name = format!("{}-runtime-plugin", id.replace('_', "-"));
    let features = document
        .get_mut("features")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| ScaffoldError::new("runtime catalog is missing [features]"))?;
    ensure_table_key_absent(features, &feature_name, "runtime catalog feature")?;
    let base_features = features
        .get_mut("base-runtime-plugins")
        .and_then(Item::as_array_mut)
        .ok_or_else(|| {
            ScaffoldError::new("runtime catalog is missing the base-runtime-plugins feature array")
        })?;
    append_unique_string(base_features, feature_name.clone());
    let mut feature_values = Array::new();
    feature_values.push(feature_dependency);
    features.insert(&feature_name, Item::Value(Value::Array(feature_values)));

    let dependencies = document
        .get_mut("dependencies")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| ScaffoldError::new("runtime catalog is missing [dependencies]"))?;
    ensure_table_key_absent(dependencies, &dependency_name, "runtime catalog dependency")?;
    let mut dependency = InlineTable::new();
    dependency.insert("path", Value::from(format!("../{id}/runtime")));
    dependency.insert("optional", Value::from(true));
    dependencies.insert(
        &dependency_name,
        Item::Value(Value::InlineTable(dependency)),
    );
    Ok(document.to_string())
}

fn wire_editor_catalog_cargo(source: &str, id: &str) -> Result<String, ScaffoldError> {
    let mut document: DocumentMut = source.parse()?;
    let dependency_name = format!("zircon_plugin_{id}_editor");
    let feature_name = format!("{}-editor-plugin", id.replace('_', "-"));
    let features = document
        .get_mut("features")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| ScaffoldError::new("editor catalog is missing [features]"))?;
    ensure_table_key_absent(features, &feature_name, "editor catalog feature")?;
    let mut feature_values = Array::new();
    feature_values.push(format!("dep:{dependency_name}"));
    features.insert(&feature_name, Item::Value(Value::Array(feature_values)));
    let dependencies = document
        .get_mut("dependencies")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| ScaffoldError::new("editor catalog is missing [dependencies]"))?;
    ensure_table_key_absent(dependencies, &dependency_name, "editor catalog dependency")?;
    let mut dependency = InlineTable::new();
    dependency.insert("path", Value::from(format!("../{id}/editor")));
    dependency.insert("optional", Value::from(true));
    dependencies.insert(
        &dependency_name,
        Item::Value(Value::InlineTable(dependency)),
    );
    Ok(document.to_string())
}

fn wire_app_cargo(source: &str, id: &str, kind: PluginKind) -> Result<String, ScaffoldError> {
    let mut document: DocumentMut = source.parse()?;
    let owner = if kind == PluginKind::Editor {
        "editor"
    } else {
        "runtime"
    };
    let id_feature = id.replace('_', "-");
    let catalog = format!("zircon_first_party_{owner}_catalog");
    let feature_name = format!("first-party-{id_feature}-{owner}-plugin");
    let catalog_feature = format!("{catalog}/{id_feature}-{owner}-plugin");
    let features = document
        .get_mut("features")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| ScaffoldError::new("zircon_app is missing [features]"))?;
    ensure_table_key_absent(features, &feature_name, "zircon_app feature")?;
    let mut feature_values = Array::new();
    feature_values.push(format!("dep:{catalog}"));
    feature_values.push(catalog_feature);
    features.insert(&feature_name, Item::Value(Value::Array(feature_values)));
    Ok(document.to_string())
}

fn wire_catalog_source(source: &str, id: &str, kind: PluginKind) -> Result<String, ScaffoldError> {
    let manifest_entry = format!("    (\"{id}\", include_str!(\"../../{id}/plugin.toml\")),\n");
    let mut updated = insert_before_marker(source, STATIC_MANIFEST_MARKER, &manifest_entry)?;
    if kind != PluginKind::Editor {
        let feature_name = format!("{}-runtime-plugin", id.replace('_', "-"));
        let registration = format!(
            "    #[cfg(feature = \"{feature_name}\")]\n    if _id.key() == \"{id}\" {{\n        return Some(zircon_plugin_{id}_runtime::plugin_registration());\n    }}\n"
        );
        updated = insert_before_marker(&updated, RUNTIME_REGISTRATION_MARKER, &registration)?;
    }
    Ok(updated)
}

fn wire_editor_catalog_source(source: &str, id: &str) -> Result<String, ScaffoldError> {
    let feature_name = format!("{}-editor-plugin", id.replace('_', "-"));
    let registration = format!(
        "    #[cfg(feature = \"{feature_name}\")]\n    if _plugin_id.key() == \"{id}\" {{\n        return Some(zircon_plugin_{id}_editor::plugin_registration());\n    }}\n"
    );
    insert_before_marker(source, EDITOR_REGISTRATION_MARKER, &registration)
}

fn insert_before_marker(
    source: &str,
    marker: &str,
    insertion: &str,
) -> Result<String, ScaffoldError> {
    let offset = source.find(marker).ok_or_else(|| {
        ScaffoldError::new(format!("catalog source is missing marker `{marker}`"))
    })?;
    let mut output = String::with_capacity(source.len() + insertion.len());
    output.push_str(&source[..offset]);
    output.push_str(insertion);
    output.push_str(&source[offset..]);
    Ok(output)
}

fn require_marker(source: &str, marker: &str) -> Result<(), ScaffoldError> {
    if source.contains(marker) {
        Ok(())
    } else {
        Err(ScaffoldError::new(format!(
            "runtime catalog source is missing generator marker `{marker}`"
        )))
    }
}

fn append_unique_string(values: &mut Array, value: String) {
    if values
        .iter()
        .any(|entry| entry.as_str() == Some(value.as_str()))
    {
        return;
    }
    values.push(value);
}

fn ensure_table_key_absent(
    table: &toml_edit::Table,
    key: &str,
    owner: &str,
) -> Result<(), ScaffoldError> {
    if table.contains_key(key) {
        return Err(ScaffoldError::new(format!(
            "{owner} `{key}` already exists; refusing to overwrite it"
        )));
    }
    Ok(())
}

fn validate_plugin_id(id: &str) -> Result<(), ScaffoldError> {
    let valid = !id.is_empty()
        && id.as_bytes()[0].is_ascii_lowercase()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !id.ends_with('_')
        && !id.contains("__");
    if valid {
        Ok(())
    } else {
        Err(ScaffoldError::new(format!(
            "plugin id `{id}` must be lowercase snake case"
        )))
    }
}

#[derive(Debug)]
pub struct ScaffoldError {
    message: String,
}

impl ScaffoldError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ScaffoldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ScaffoldError {}

impl From<std::io::Error> for ScaffoldError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<toml::de::Error> for ScaffoldError {
    fn from(error: toml::de::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<toml::ser::Error> for ScaffoldError {
    fn from(error: toml::ser::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<toml_edit::TomlError> for ScaffoldError {
    fn from(error: toml_edit::TomlError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<super::manifest_sync::ManifestSyncError> for ScaffoldError {
    fn from(error: super::manifest_sync::ManifestSyncError) -> Self {
        Self::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::wire_runtime_catalog_cargo;

    #[test]
    fn runtime_catalog_wiring_rejects_an_existing_feature_without_overwriting_it() {
        let source = r#"[features]
base-runtime-plugins = []
demo-probe-runtime-plugin = ["manual-contract"]

[dependencies]
"#;

        let error = wire_runtime_catalog_cargo(source, "demo_probe").unwrap_err();

        assert!(error
            .to_string()
            .contains("runtime catalog feature `demo-probe-runtime-plugin` already exists"));
    }
}
