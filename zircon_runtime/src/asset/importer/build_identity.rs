use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const ASSET_IMPORT_RECIPE_SCHEMA_VERSION: u32 = 1;
const ASSET_IMPORT_ACTION_KEY_SCHEMA_VERSION: u32 = 1;
const PROJECT_IMPORT_BUILD_PROFILE: &str = "project_import";

#[derive(Clone, Debug, PartialEq)]
pub struct AssetImportRecipe {
    schema_version: u32,
    settings: BTreeMap<String, AssetImportRecipeValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AssetImportRecipeValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(String),
    Array(Vec<Self>),
    Table(BTreeMap<String, Self>),
}

impl Default for AssetImportRecipe {
    fn default() -> Self {
        Self::from_legacy_settings(toml::Table::new())
    }
}

impl AssetImportRecipe {
    /// Migrates the v7 sidecar settings payload into the current recipe envelope.
    pub fn from_legacy_settings(settings: toml::Table) -> Self {
        Self {
            schema_version: ASSET_IMPORT_RECIPE_SCHEMA_VERSION,
            settings: settings
                .into_iter()
                .map(|(key, value)| (key, AssetImportRecipeValue::from_toml(value)))
                .collect(),
        }
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn settings(&self) -> &BTreeMap<String, AssetImportRecipeValue> {
        &self.settings
    }

    pub fn setting(&self, name: &str) -> Option<&AssetImportRecipeValue> {
        self.settings.get(name)
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut encoded = b"zircon.asset.import.recipe".to_vec();
        encoded.extend_from_slice(&self.schema_version.to_le_bytes());
        encode_recipe_table(&mut encoded, &self.settings);
        encoded
    }
}

impl AssetImportRecipeValue {
    fn from_toml(value: toml::Value) -> Self {
        match value {
            toml::Value::String(value) => Self::String(value),
            toml::Value::Integer(value) => Self::Integer(value),
            toml::Value::Float(value) => Self::Float(value),
            toml::Value::Boolean(value) => Self::Boolean(value),
            toml::Value::Datetime(value) => Self::Datetime(value.to_string()),
            toml::Value::Array(values) => {
                Self::Array(values.into_iter().map(Self::from_toml).collect())
            }
            toml::Value::Table(table) => Self::Table(
                table
                    .into_iter()
                    .map(|(key, value)| (key, Self::from_toml(value)))
                    .collect(),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetImportBuildContext {
    target_platform: String,
    build_profile: String,
    engine_abi_version: u32,
    toolchain_identity: String,
}

impl AssetImportBuildContext {
    pub fn new(
        target_platform: impl Into<String>,
        build_profile: impl Into<String>,
        engine_abi_version: u32,
        toolchain_identity: impl Into<String>,
    ) -> Self {
        Self {
            target_platform: target_platform.into(),
            build_profile: build_profile.into(),
            engine_abi_version,
            toolchain_identity: toolchain_identity.into(),
        }
    }

    pub(crate) fn project_import(toolchain_identity: impl Into<String>) -> Self {
        Self::new(
            env!("ZR_ASSET_IMPORT_BUILD_TARGET"),
            PROJECT_IMPORT_BUILD_PROFILE,
            zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8,
            format!(
                "{};build={}",
                toolchain_identity.into(),
                env!("ZR_ASSET_IMPORT_BUILD_ID")
            ),
        )
    }

    pub fn target_platform(&self) -> &str {
        &self.target_platform
    }

    pub fn build_profile(&self) -> &str {
        &self.build_profile
    }

    pub const fn engine_abi_version(&self) -> u32 {
        self.engine_abi_version
    }

    pub fn toolchain_identity(&self) -> &str {
        &self.toolchain_identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssetImportBuildIdentity {
    function_identity: String,
    recipe: AssetImportRecipe,
    input_digest: String,
    build_context: AssetImportBuildContext,
    action_key: String,
}

impl AssetImportBuildIdentity {
    pub(crate) fn new(
        function_identity: impl Into<String>,
        recipe: AssetImportRecipe,
        input_digest: impl Into<String>,
        build_context: AssetImportBuildContext,
    ) -> Self {
        let function_identity = function_identity.into();
        let input_digest = input_digest.into();
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"zircon.asset.import.action.v1");
        update_hash_field(
            &mut hasher,
            b"schema",
            &ASSET_IMPORT_ACTION_KEY_SCHEMA_VERSION.to_le_bytes(),
        );
        update_hash_field(&mut hasher, b"function", function_identity.as_bytes());
        update_hash_field(&mut hasher, b"recipe", &recipe.canonical_bytes());
        update_hash_field(&mut hasher, b"inputs", input_digest.as_bytes());
        update_hash_field(
            &mut hasher,
            b"target",
            build_context.target_platform.as_bytes(),
        );
        update_hash_field(
            &mut hasher,
            b"profile",
            build_context.build_profile.as_bytes(),
        );
        update_hash_field(
            &mut hasher,
            b"engine_abi",
            &build_context.engine_abi_version.to_le_bytes(),
        );
        update_hash_field(
            &mut hasher,
            b"toolchain",
            build_context.toolchain_identity.as_bytes(),
        );
        let action_key = format!("blake3:{}", hasher.finalize().to_hex());
        Self {
            function_identity,
            recipe,
            input_digest,
            build_context,
            action_key,
        }
    }

    pub fn function_identity(&self) -> &str {
        &self.function_identity
    }

    pub fn recipe(&self) -> &AssetImportRecipe {
        &self.recipe
    }

    pub fn input_digest(&self) -> &str {
        &self.input_digest
    }

    pub fn build_context(&self) -> &AssetImportBuildContext {
        &self.build_context
    }

    pub fn action_key(&self) -> &str {
        &self.action_key
    }

    pub(crate) fn into_context_parts(self) -> (AssetImportRecipe, AssetImportBuildContext, String) {
        (self.recipe, self.build_context, self.action_key)
    }
}

fn update_hash_field(hasher: &mut blake3::Hasher, label: &[u8], bytes: &[u8]) {
    hasher.update(&(label.len() as u64).to_le_bytes());
    hasher.update(label);
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

pub(crate) fn canonical_import_input_digest(
    source_identity: &str,
    source_bytes: &[u8],
    snapshots: &BTreeMap<PathBuf, Vec<u8>>,
    source_root: Option<&Path>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"zircon.asset.import.inputs.v1");
    update_hash_field(&mut hasher, b"source_identity", source_identity.as_bytes());
    update_hash_field(&mut hasher, b"primary", source_bytes);
    hasher.update(&(snapshots.len() as u64).to_le_bytes());
    for (path, bytes) in snapshots {
        let relative = source_root
            .and_then(|root| path.strip_prefix(root).ok())
            .unwrap_or(path);
        hasher.update(&(relative.components().count() as u64).to_le_bytes());
        for component in relative.components() {
            update_os_component_hash(&mut hasher, component.as_os_str());
        }
        update_hash_field(&mut hasher, b"snapshot", bytes);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn update_os_component_hash(hasher: &mut blake3::Hasher, component: &std::ffi::OsStr) {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        update_hash_field(hasher, b"path_component_unix", component.as_bytes());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt as _;
        let encoded = component
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        update_hash_field(hasher, b"path_component_windows", &encoded);
    }
    #[cfg(not(any(unix, windows)))]
    {
        update_hash_field(
            hasher,
            b"path_component_utf8",
            component.to_string_lossy().as_bytes(),
        );
    }
}

fn encode_recipe_table(encoded: &mut Vec<u8>, table: &BTreeMap<String, AssetImportRecipeValue>) {
    encoded.push(b't');
    encoded.extend_from_slice(&(table.len() as u64).to_le_bytes());
    for (key, value) in table {
        encode_bytes(encoded, key.as_bytes());
        encode_recipe_value(encoded, value);
    }
}

fn encode_recipe_value(encoded: &mut Vec<u8>, value: &AssetImportRecipeValue) {
    match value {
        AssetImportRecipeValue::String(value) => {
            encoded.push(b's');
            encode_bytes(encoded, value.as_bytes());
        }
        AssetImportRecipeValue::Integer(value) => {
            encoded.push(b'i');
            encoded.extend_from_slice(&value.to_le_bytes());
        }
        AssetImportRecipeValue::Float(value) => {
            encoded.push(b'f');
            encoded.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        AssetImportRecipeValue::Boolean(value) => {
            encoded.push(b'b');
            encoded.push(u8::from(*value));
        }
        AssetImportRecipeValue::Datetime(value) => {
            encoded.push(b'd');
            encode_bytes(encoded, value.as_bytes());
        }
        AssetImportRecipeValue::Array(values) => {
            encoded.push(b'a');
            encoded.extend_from_slice(&(values.len() as u64).to_le_bytes());
            for value in values {
                encode_recipe_value(encoded, value);
            }
        }
        AssetImportRecipeValue::Table(table) => encode_recipe_table(encoded, table),
    }
}

fn encode_bytes(encoded: &mut Vec<u8>, bytes: &[u8]) {
    encoded.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    encoded.extend_from_slice(bytes);
}

#[cfg(test)]
#[path = "tests/build_identity.rs"]
mod tests;
