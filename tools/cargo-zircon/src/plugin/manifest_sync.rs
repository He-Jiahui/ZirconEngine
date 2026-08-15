use std::fmt;
use std::fs;
use std::path::Path;

use syn::{Expr, Item, Lit};
use toml::map::Map;
use toml::Value;

mod declaration;

pub const GENERATED_MANIFEST_HEADER: &str =
    "# @generated from Rust PluginDeclaration; do not edit by hand.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDeclarationProjection {
    id: String,
    display_name: String,
    category: String,
    module_name: String,
    crate_name: String,
    targets: Vec<String>,
    platforms: Vec<String>,
    capabilities: Vec<String>,
    runtime_capabilities: Vec<String>,
    editor_capabilities: Vec<String>,
    maturity: String,
    packaging: Vec<String>,
    runtime_entry: Option<String>,
    editor_entry: Option<String>,
}

impl PluginDeclarationProjection {
    pub fn parse(source: &str) -> Result<Self, ManifestSyncError> {
        let file = syn::parse_file(source)?;
        file.items
            .into_iter()
            .find_map(|item| match item {
                Item::Macro(item_macro)
                    if item_macro
                        .mac
                        .path
                        .segments
                        .last()
                        .is_some_and(|segment| segment.ident == "declare_plugin") =>
                {
                    Some(syn::parse2(item_macro.mac.tokens).map_err(Into::into))
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                Err(ManifestSyncError::new(
                    "source does not contain zircon_plugin_sdk::declare_plugin!",
                ))
            })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn targets(&self) -> &[String] {
        &self.targets
    }

    pub fn platforms(&self) -> &[String] {
        &self.platforms
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn runtime_capabilities(&self) -> &[String] {
        &self.runtime_capabilities
    }

    pub fn editor_capabilities(&self) -> &[String] {
        &self.editor_capabilities
    }

    pub fn maturity(&self) -> &str {
        &self.maturity
    }

    pub fn packaging(&self) -> &[String] {
        &self.packaging
    }

    pub fn runtime_entry(&self) -> Option<&str> {
        self.runtime_entry.as_deref()
    }

    pub fn editor_entry(&self) -> Option<&str> {
        self.editor_entry.as_deref()
    }

    fn capabilities_for_module_kind(&self, kind: &str) -> &[String] {
        match kind {
            "runtime" => self.runtime_capabilities(),
            "editor" => self.editor_capabilities(),
            _ => self.capabilities(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncMode {
    Check,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    Unchanged,
    Drift,
    Updated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestDeclarationOwner {
    package_id: String,
    declaration_path: std::path::PathBuf,
    manifest_path: std::path::PathBuf,
}

impl ManifestDeclarationOwner {
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn declaration_path(&self) -> &Path {
        &self.declaration_path
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSyncEntry {
    pub package_id: String,
    pub outcome: SyncOutcome,
}

pub fn discover_manifest_declarations(
    repo_root: &Path,
) -> Result<Vec<ManifestDeclarationOwner>, ManifestSyncError> {
    let plugin_root = repo_root.join("zircon_plugins");
    let mut declaration_paths = Vec::new();
    collect_declaration_paths(&plugin_root, &mut declaration_paths)?;
    let mut owners = Vec::new();

    for declaration_path in declaration_paths {
        let Some(src_root) = declaration_path.parent() else {
            continue;
        };
        let Some(crate_root) = src_root.parent() else {
            continue;
        };
        let crate_kind = crate_root.file_name().and_then(|value| value.to_str());
        if !matches!(crate_kind, Some("runtime" | "editor" | "native")) {
            continue;
        }
        let Some(package_root) = crate_root.parent() else {
            continue;
        };
        let manifest_path = package_root.join("plugin.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let source = fs::read_to_string(&declaration_path)?;
        if !source.contains("zircon_plugin_sdk::declare_plugin!") {
            continue;
        }
        let package_id = PluginDeclarationProjection::parse(&source)?.id;
        owners.push(ManifestDeclarationOwner {
            package_id,
            declaration_path,
            manifest_path,
        });
    }

    owners.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    Ok(owners)
}

pub fn synchronize_workspace_manifests(
    repo_root: &Path,
    selector: Option<&str>,
    mode: SyncMode,
) -> Result<Vec<WorkspaceSyncEntry>, ManifestSyncError> {
    let version = read_workspace_version(repo_root)?;
    let sdk_api_version = read_sdk_api_version(repo_root)?;
    let owners = discover_manifest_declarations(repo_root)?;
    let mut results = Vec::new();
    for owner in owners {
        if selector.is_some_and(|selector| selector != owner.package_id()) {
            continue;
        }
        let outcome = synchronize_manifest_file(
            owner.declaration_path(),
            owner.manifest_path(),
            &version,
            &sdk_api_version,
            mode,
        )?;
        results.push(WorkspaceSyncEntry {
            package_id: owner.package_id,
            outcome,
        });
    }
    if let Some(selector) = selector {
        if results.is_empty() {
            return Err(ManifestSyncError::new(format!(
                "no declaration-owned plugin package matched `{selector}`"
            )));
        }
    }
    Ok(results)
}

pub fn synchronize_manifest(
    existing: &str,
    declaration: &PluginDeclarationProjection,
    package_version: &str,
    sdk_api_version: &str,
) -> Result<String, ManifestSyncError> {
    let mut manifest: Value = existing.parse()?;
    let root = manifest
        .as_table_mut()
        .ok_or_else(|| ManifestSyncError::new("plugin manifest root must be a TOML table"))?;

    insert_string(root, "id", declaration.id());
    insert_string(root, "version", package_version);
    insert_string(root, "sdk_api_version", sdk_api_version);
    insert_string(root, "display_name", declaration.display_name());
    insert_string(root, "category", declaration.category());
    insert_string(root, "maturity", declaration.maturity());
    insert_string_array(root, "supported_targets", declaration.targets());
    insert_string_array(root, "supported_platforms", declaration.platforms());
    insert_string_array(root, "capabilities", declaration.capabilities());
    insert_string_array(root, "default_packaging", declaration.packaging());

    synchronize_distribution(root, declaration, package_version)?;
    synchronize_modules(root, declaration)?;

    let serialized = toml::to_string_pretty(&manifest)?;
    Ok(format!("{GENERATED_MANIFEST_HEADER}\n{serialized}"))
}

pub fn synchronize_manifest_file(
    declaration_path: &Path,
    manifest_path: &Path,
    package_version: &str,
    sdk_api_version: &str,
    mode: SyncMode,
) -> Result<SyncOutcome, ManifestSyncError> {
    let source = fs::read_to_string(declaration_path)?;
    let existing = fs::read_to_string(manifest_path)?;
    let declaration = PluginDeclarationProjection::parse(&source)?;
    let synchronized =
        synchronize_manifest(&existing, &declaration, package_version, sdk_api_version)?;
    let existing_value: Value = existing.parse()?;
    let synchronized_value: Value = synchronized.parse()?;
    if existing.starts_with(GENERATED_MANIFEST_HEADER) && existing_value == synchronized_value {
        return Ok(SyncOutcome::Unchanged);
    }
    if mode == SyncMode::Check {
        return Ok(SyncOutcome::Drift);
    }
    fs::write(manifest_path, synchronized)?;
    Ok(SyncOutcome::Updated)
}

fn insert_string(table: &mut Map<String, Value>, key: &str, value: &str) {
    table.insert(key.to_string(), Value::String(value.to_string()));
}

fn insert_string_array(table: &mut Map<String, Value>, key: &str, values: &[String]) {
    table.insert(
        key.to_string(),
        Value::Array(values.iter().cloned().map(Value::String).collect()),
    );
}

fn insert_string_array_values(table: &mut Map<String, Value>, key: &str, values: &[&str]) {
    table.insert(
        key.to_string(),
        Value::Array(
            values
                .iter()
                .map(|value| Value::String((*value).to_string()))
                .collect(),
        ),
    );
}

fn synchronize_distribution(
    root: &mut Map<String, Value>,
    declaration: &PluginDeclarationProjection,
    package_version: &str,
) -> Result<(), ManifestSyncError> {
    let Some(dist_crate) = distribution_crate_name(declaration)? else {
        root.remove("distribution");
        return Ok(());
    };

    let distribution = root
        .entry("distribution")
        .or_insert_with(|| Value::Table(Map::new()))
        .as_table_mut()
        .ok_or_else(|| ManifestSyncError::new("plugin distribution must be a TOML table"))?;
    insert_string_array_values(distribution, "forms", &["dist"]);
    insert_string_array_values(distribution, "default_packaging", &["native_dynamic"]);
    distribution.insert("abi_version".to_string(), Value::Integer(3));
    insert_string(
        distribution,
        "descriptor_symbol",
        "zircon_native_plugin_descriptor_v3",
    );
    insert_string(
        distribution,
        "engine_compat",
        &engine_compatibility_range(package_version)?,
    );
    insert_string(distribution, "dist_crate", &dist_crate);
    if let Some(entry) = declaration.runtime_entry() {
        insert_string(distribution, "runtime_entry", entry);
    } else {
        distribution.remove("runtime_entry");
    }
    if let Some(entry) = declaration.editor_entry() {
        insert_string(distribution, "editor_entry", entry);
    } else {
        distribution.remove("editor_entry");
    }
    Ok(())
}

fn sibling_distribution_crate_name(crate_name: &str) -> Option<String> {
    crate_name
        .strip_suffix("_runtime")
        .or_else(|| crate_name.strip_suffix("_editor"))
        .map(|prefix| format!("{prefix}_dist"))
}

fn distribution_crate_name(
    declaration: &PluginDeclarationProjection,
) -> Result<Option<String>, ManifestSyncError> {
    if !declaration
        .packaging()
        .iter()
        .any(|packaging| packaging == "native_dynamic")
    {
        return Ok(None);
    }
    if let Some(dist_crate) = sibling_distribution_crate_name(declaration.crate_name()) {
        return Ok(Some(dist_crate));
    }
    if declaration.crate_name().ends_with("_native") {
        return Ok(Some(declaration.crate_name().to_string()));
    }
    Err(ManifestSyncError::new(format!(
        "native declaration crate `{}` must end in `_runtime`, `_editor`, or `_native`",
        declaration.crate_name()
    )))
}

pub(super) fn engine_compatibility_range(
    package_version: &str,
) -> Result<String, ManifestSyncError> {
    let mut components = package_version.split('.');
    let major = components
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| {
            ManifestSyncError::new("workspace version has no numeric major component")
        })?;
    let minor = components
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| {
            ManifestSyncError::new("workspace version has no numeric minor component")
        })?;
    let upper = if major == 0 {
        format!(
            "0.{}",
            minor.checked_add(1).ok_or_else(|| {
                ManifestSyncError::new("workspace minor version exceeds supported range")
            })?
        )
    } else {
        format!(
            "{}.0",
            major.checked_add(1).ok_or_else(|| {
                ManifestSyncError::new("workspace major version exceeds supported range")
            })?
        )
    };
    Ok(format!(">={major}.{minor}, <{upper}"))
}

fn synchronize_modules(
    root: &mut Map<String, Value>,
    declaration: &PluginDeclarationProjection,
) -> Result<(), ManifestSyncError> {
    let Some(modules) = root.get_mut("modules").and_then(Value::as_array_mut) else {
        return Err(ManifestSyncError::new(
            "plugin manifest must contain at least one [[modules]] projection",
        ));
    };
    let primary_kind = if declaration.module_name().ends_with(".editor") {
        "editor"
    } else {
        "runtime"
    };
    let desired_dist_crate = distribution_crate_name(declaration)?;
    let owned_sibling_dist_crate = sibling_distribution_crate_name(declaration.crate_name());
    let dist_module_name = format!("{}.dist", declaration.id());
    let separate_dist_crate = desired_dist_crate
        .as_deref()
        .filter(|dist_crate| *dist_crate != declaration.crate_name());
    let has_separate_dist_module = separate_dist_crate.is_some();
    let mut found_primary = false;
    let primary_capabilities = declaration.capabilities_for_module_kind(primary_kind);
    let dist_capabilities =
        if declaration.runtime_entry().is_some() && declaration.editor_entry().is_some() {
            declaration.capabilities()
        } else {
            primary_capabilities
        };

    for module in modules.iter_mut().filter_map(Value::as_table_mut) {
        let matches_primary = module.get("name").and_then(Value::as_str)
            == Some(declaration.module_name())
            || (module.get("crate_name").and_then(Value::as_str) == Some(declaration.crate_name())
                && module.get("kind").and_then(Value::as_str) == Some(primary_kind));
        if matches_primary {
            insert_string(module, "name", declaration.module_name());
            insert_string(module, "kind", primary_kind);
            insert_string(module, "crate_name", declaration.crate_name());
            insert_string_array(module, "target_modes", declaration.targets());
            insert_string_array(module, "capabilities", primary_capabilities);
            found_primary = true;
            continue;
        }
        let is_owned_dist = module.get("kind").and_then(Value::as_str) == Some("native")
            && (module.get("name").and_then(Value::as_str) == Some(dist_module_name.as_str())
                || owned_sibling_dist_crate.as_deref()
                    == module.get("crate_name").and_then(Value::as_str));
        if let Some(dist_crate) = separate_dist_crate.filter(|_| is_owned_dist) {
            insert_string(module, "name", &dist_module_name);
            insert_string(module, "kind", "native");
            insert_string(module, "crate_name", dist_crate);
            insert_string_array(module, "target_modes", declaration.targets());
            insert_string_array(module, "capabilities", dist_capabilities);
        }
    }

    if !found_primary {
        return Err(ManifestSyncError::new(format!(
            "plugin manifest is missing declaration-owned module crate `{}`",
            declaration.crate_name()
        )));
    }
    let found_dist = if has_separate_dist_module {
        let mut kept_dist = false;
        modules.retain(|module| {
            let is_owned_dist = module.get("kind").and_then(Value::as_str) == Some("native")
                && module.get("name").and_then(Value::as_str) == Some(dist_module_name.as_str());
            if !is_owned_dist {
                return true;
            }
            if kept_dist {
                return false;
            }
            kept_dist = true;
            true
        });
        kept_dist
    } else {
        false
    };
    if let Some(dist_crate) = separate_dist_crate {
        if !found_dist {
            let mut module = Map::new();
            insert_string(&mut module, "name", &dist_module_name);
            insert_string(&mut module, "kind", "native");
            insert_string(&mut module, "crate_name", dist_crate);
            insert_string_array(&mut module, "target_modes", declaration.targets());
            insert_string_array(&mut module, "capabilities", dist_capabilities);
            modules.push(Value::Table(module));
        }
    } else {
        modules.retain(|module| {
            let Some(module) = module.as_table() else {
                return true;
            };
            module.get("kind").and_then(Value::as_str) != Some("native")
                || (module.get("name").and_then(Value::as_str) != Some(dist_module_name.as_str())
                    && owned_sibling_dist_crate.as_deref()
                        != module.get("crate_name").and_then(Value::as_str))
        });
    }
    Ok(())
}

fn collect_declaration_paths(
    directory: &Path,
    paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), ManifestSyncError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|value| value.to_str()),
                Some("target" | ".git")
            ) {
                continue;
            }
            collect_declaration_paths(&path, paths)?;
        } else if matches!(
            path.file_name().and_then(|value| value.to_str()),
            Some("capability.rs" | "lib.rs")
        ) {
            paths.push(path);
        }
    }
    Ok(())
}

pub(crate) fn read_workspace_version(repo_root: &Path) -> Result<String, ManifestSyncError> {
    let cargo: Value = fs::read_to_string(repo_root.join("Cargo.toml"))?.parse()?;
    cargo
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("version"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ManifestSyncError::new("root Cargo.toml is missing workspace.package.version")
        })
}

pub(crate) fn read_sdk_api_version(repo_root: &Path) -> Result<String, ManifestSyncError> {
    let defaults_path = repo_root.join("zircon_plugins/plugin_sdk/src/manifest/defaults.rs");
    let file = syn::parse_file(&fs::read_to_string(&defaults_path)?)?;
    file.items
        .into_iter()
        .find_map(|item| match item {
            Item::Const(item_const) if item_const.ident == "SDK_API_VERSION" => {
                match *item_const.expr {
                    Expr::Lit(expression) => match expression.lit {
                        Lit::Str(value) => Some(value.value()),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        })
        .ok_or_else(|| {
            ManifestSyncError::new(format!(
                "{} is missing literal SDK_API_VERSION",
                defaults_path.display()
            ))
        })
}

#[derive(Debug)]
pub struct ManifestSyncError {
    message: String,
}

impl ManifestSyncError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ManifestSyncError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ManifestSyncError {}

impl From<std::io::Error> for ManifestSyncError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<syn::Error> for ManifestSyncError {
    fn from(error: syn::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<toml::de::Error> for ManifestSyncError {
    fn from(error: toml::de::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<toml::ser::Error> for ManifestSyncError {
    fn from(error: toml::ser::Error) -> Self {
        Self::new(error.to_string())
    }
}
