use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use crate::core::asset::{AssetTypeId, AssetTypeIdError, AssetTypeRegistryError};
use crate::core::commands::{EditorCommandMenuPath, EditorCommandRegistryError};
use crate::core::editor_operation::{EditorOperationPath, EditorOperationPathError};

use super::validate_contribution_id;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorExtensionRegistryError {
    DuplicateContribution {
        kind: &'static str,
        id: String,
    },
    InvalidContributionId {
        kind: &'static str,
        id: String,
    },
    InvalidDescriptorSchemaVersion {
        kind: &'static str,
        id: String,
        version: u32,
    },
    InvalidUiDocument {
        kind: &'static str,
        document: String,
    },
    MissingUiTemplate {
        template_id: String,
    },
    MissingLocalizationBundle {
        page_id: String,
        bundle_id: String,
    },
    UnknownLocalizationKey {
        page_id: String,
        bundle_id: String,
        key: String,
    },
    CommandLocalization {
        command_id: EditorOperationPath,
        detail: String,
    },
    UnknownExtensionOwner {
        owner_id: String,
    },
    ToolResourceKindOwnerMismatch {
        owner_id: String,
        kind: String,
        expected_prefix: String,
    },
    ToolResourceKindRequiresPluginSource {
        kind: String,
    },
    StaleContributionHandle {
        owner_id: String,
    },
    InvalidAssetImporterExtensions(String),
    InvalidMenuPath(String),
    CommandViewTargetConflict {
        command_id: EditorOperationPath,
        view_id: String,
    },
    MenuCapabilitiesRequireContributedCommand {
        command_id: EditorOperationPath,
    },
    MissingViewportOverlayProvider {
        provider_id: String,
    },
    SceneMode(String),
    ViewportOverlayProvider(String),
    Command(EditorCommandRegistryError),
    OperationPath(EditorOperationPathError),
    AssetTypeId(AssetTypeIdError),
    AssetTypeRegistry(AssetTypeRegistryError),
    ToolScheduler(String),
    View(String),
}

impl fmt::Display for EditorExtensionRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateContribution { kind, id } => {
                write!(formatter, "editor {kind} {id} already registered")
            }
            Self::InvalidContributionId { kind, id } => {
                write!(formatter, "editor {kind} id `{id}` is invalid")
            }
            Self::InvalidDescriptorSchemaVersion { kind, id, version } => write!(
                formatter,
                "editor {kind} `{id}` schema version `{version}` is invalid"
            ),
            Self::InvalidUiDocument { kind, document } => write!(
                formatter,
                "editor {kind} `{document}` must reference a supported editor UI asset"
            ),
            Self::MissingUiTemplate { template_id } => write!(
                formatter,
                "editor UI template pane data source references missing template `{template_id}`"
            ),
            Self::MissingLocalizationBundle { page_id, bundle_id } => write!(
                formatter,
                "editor settings page `{page_id}` references missing localization bundle `{bundle_id}`"
            ),
            Self::UnknownLocalizationKey {
                page_id,
                bundle_id,
                key,
            } => write!(
                formatter,
                "editor settings page `{page_id}` references unknown key `{key}` in localization bundle `{bundle_id}`"
            ),
            Self::CommandLocalization { command_id, detail } => write!(
                formatter,
                "editor command `{command_id}` localization is invalid: {detail}"
            ),
            Self::UnknownExtensionOwner { owner_id } => {
                write!(
                    formatter,
                    "editor extension owner `{owner_id}` is not registered"
                )
            }
            Self::ToolResourceKindOwnerMismatch {
                owner_id,
                kind,
                expected_prefix,
            } => write!(
                formatter,
                "editor extension owner `{owner_id}` cannot register tool resource kind `{}`; expected namespace `{expected_prefix}`",
                kind
            ),
            Self::ToolResourceKindRequiresPluginSource { kind } => write!(
                formatter,
                "editor builtin contribution cannot register tool resource kind `{kind}`; a plugin contribution source is required"
            ),
            Self::StaleContributionHandle { owner_id } => write!(
                formatter,
                "editor contribution handle for owner `{owner_id}` is stale or no longer registered"
            ),
            Self::InvalidAssetImporterExtensions(id) => {
                write!(
                    formatter,
                    "editor asset importer `{id}` must declare at least one source extension"
                )
            }
            Self::InvalidMenuPath(path) => {
                write!(formatter, "editor menu item path `{path}` is invalid")
            }
            Self::CommandViewTargetConflict {
                command_id,
                view_id,
            } => write!(
                formatter,
                "editor command {command_id} does not open extension view {view_id}"
            ),
            Self::MenuCapabilitiesRequireContributedCommand { command_id } => write!(
                formatter,
                "editor menu capability constraints for {command_id} must be owned by a command contributed by the same extension"
            ),
            Self::MissingViewportOverlayProvider { provider_id } => write!(
                formatter,
                "editor scene mode references unregistered overlay provider `{provider_id}`"
            ),
            Self::SceneMode(error) => {
                write!(formatter, "editor scene mode registration failed: {error}")
            }
            Self::ViewportOverlayProvider(error) => {
                write!(
                    formatter,
                    "editor viewport overlay provider registration failed: {error}"
                )
            }
            Self::Command(error) => write!(formatter, "{error}"),
            Self::OperationPath(error) => write!(formatter, "{error}"),
            Self::AssetTypeId(error) => write!(formatter, "{error}"),
            Self::AssetTypeRegistry(error) => write!(formatter, "{error}"),
            Self::ToolScheduler(error) => {
                write!(formatter, "editor tool scheduler operation failed: {error}")
            }
            Self::View(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for EditorExtensionRegistryError {}

impl From<AssetTypeIdError> for EditorExtensionRegistryError {
    fn from(error: AssetTypeIdError) -> Self {
        Self::AssetTypeId(error)
    }
}

impl From<AssetTypeRegistryError> for EditorExtensionRegistryError {
    fn from(error: AssetTypeRegistryError) -> Self {
        Self::AssetTypeRegistry(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawerDescriptor {
    id: String,
    display_name: String,
}

impl DrawerDescriptor {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EditorMenuItemDescriptor {
    menu_path: EditorCommandMenuPath,
    #[serde(skip)]
    stable_path: String,
    operation: EditorOperationPath,
    #[serde(default)]
    priority: i32,
    #[serde(default = "default_menu_item_enabled")]
    enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl<'de> Deserialize<'de> for EditorMenuItemDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct SerializedEditorMenuItemDescriptor {
            menu_path: EditorCommandMenuPath,
            operation: EditorOperationPath,
            #[serde(default)]
            priority: i32,
            #[serde(default = "default_menu_item_enabled")]
            enabled: bool,
            #[serde(default)]
            required_capabilities: Vec<String>,
        }

        let descriptor = SerializedEditorMenuItemDescriptor::deserialize(deserializer)?;
        Ok(Self::new(descriptor.menu_path, descriptor.operation)
            .with_priority(descriptor.priority)
            .with_enabled(descriptor.enabled)
            .with_required_capabilities(descriptor.required_capabilities))
    }
}

impl EditorMenuItemDescriptor {
    pub fn new(menu_path: EditorCommandMenuPath, operation: EditorOperationPath) -> Self {
        let stable_path = menu_path.stable_path();
        Self {
            menu_path,
            stable_path,
            operation,
            priority: 0,
            enabled: true,
            required_capabilities: Vec::new(),
        }
    }

    pub fn builtin(operation: EditorOperationPath, root_id: &str, group_ids: &[&str]) -> Self {
        let menu_path = EditorCommandMenuPath::builtin(&operation, root_id, group_ids);
        Self::new(menu_path, operation)
    }

    pub fn for_operation(operation: EditorOperationPath) -> Self {
        let mut segments = operation.as_str().split('.');
        let root = segments.next().unwrap_or("commands");
        match segments.next() {
            Some(group) => Self::builtin(operation.clone(), root, &[group]),
            None => Self::builtin(operation.clone(), root, &[]),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let capabilities = capabilities.into_iter();
        let (lower_bound, _) = capabilities.size_hint();
        self.required_capabilities.reserve(lower_bound);
        self.required_capabilities
            .extend(capabilities.map(Into::into));
        self.required_capabilities.sort_unstable();
        self.required_capabilities.dedup();
        self
    }

    pub fn path(&self) -> &str {
        &self.stable_path
    }

    pub fn menu_path(&self) -> &EditorCommandMenuPath {
        &self.menu_path
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

fn default_menu_item_enabled() -> bool {
    true
}

pub(super) fn validate_menu_item_path(
    descriptor: &EditorMenuItemDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    if descriptor.menu_path().segments().len() < MIN_MENU_PATH_SEGMENTS
        || descriptor.path() != descriptor.menu_path().stable_path()
    {
        return Err(EditorExtensionRegistryError::InvalidMenuPath(
            descriptor.path().to_owned(),
        ));
    }
    Ok(())
}

const MIN_MENU_PATH_SEGMENTS: usize = 2;

fn is_invalid_ui_document(document: &str) -> bool {
    document.trim().is_empty() || document.trim() != document
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImporterDescriptor {
    id: String,
    display_name: String,
    operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    source_extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_type: Option<AssetTypeId>,
    #[serde(default)]
    priority: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl AssetImporterDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        operation: EditorOperationPath,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            operation,
            source_extensions: Vec::new(),
            output_type: None,
            priority: 0,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_source_extension(mut self, extension: impl AsRef<str>) -> Self {
        push_normalized_extension(&mut self.source_extensions, extension.as_ref());
        self
    }

    pub fn with_source_extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for extension in extensions {
            push_normalized_extension(&mut self.source_extensions, extension.as_ref());
        }
        self
    }

    pub fn with_output_type(mut self, output_type: AssetTypeId) -> Self {
        self.output_type = Some(output_type);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let capabilities = capabilities.into_iter();
        let (lower_bound, _) = capabilities.size_hint();
        self.required_capabilities.reserve(lower_bound);
        self.required_capabilities
            .extend(capabilities.map(Into::into));
        self.required_capabilities.sort_unstable();
        self.required_capabilities.dedup();
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn source_extensions(&self) -> &[String] {
        &self.source_extensions
    }

    pub fn output_type(&self) -> Option<&AssetTypeId> {
        self.output_type.as_ref()
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

fn push_normalized_extension(extensions: &mut Vec<String>, extension: &str) {
    let extension = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    if !extension.is_empty() {
        if !extensions.windows(2).all(|pair| pair[0] <= pair[1]) {
            extensions.sort_unstable();
            extensions.dedup();
        }
        if let Err(index) = extensions.binary_search(&extension) {
            extensions.insert(index, extension);
        }
    }
}

pub(super) fn validate_asset_importer(
    descriptor: &AssetImporterDescriptor,
) -> Result<(), EditorExtensionRegistryError> {
    validate_contribution_id("asset importer", &descriptor.id)?;
    if descriptor.source_extensions.is_empty() {
        return Err(
            EditorExtensionRegistryError::InvalidAssetImporterExtensions(descriptor.id.clone()),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_descriptor_rejects_the_retired_shortcut_owner() {
        let operation = EditorOperationPath::parse("fixture.editor.command").unwrap();
        let descriptor = EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(&operation, "tools", &[]),
            operation,
        );
        let mut serialized = serde_json::to_value(descriptor).unwrap();
        serialized
            .as_object_mut()
            .unwrap()
            .insert("shortcut".to_string(), serde_json::json!("Ctrl+Alt+R"));

        assert!(serde_json::from_value::<EditorMenuItemDescriptor>(serialized).is_err());
    }

    #[test]
    fn menu_descriptor_rebuilds_its_stable_path_from_typed_segments() {
        let operation = EditorOperationPath::parse("fixture.editor.command").unwrap();
        let descriptor = EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(&operation, "tools", &["fixture"]),
            operation,
        );
        let serialized = serde_json::to_value(&descriptor).unwrap();

        assert!(serialized.get("stable_path").is_none());
        let restored: EditorMenuItemDescriptor = serde_json::from_value(serialized).unwrap();
        assert_eq!(restored.path(), "tools/fixture/fixture.editor.command");
    }

    #[test]
    fn menu_descriptor_rejects_the_retired_string_path_payload() {
        let operation = EditorOperationPath::parse("fixture.editor.command").unwrap();
        let descriptor = EditorMenuItemDescriptor::new(
            EditorCommandMenuPath::builtin(&operation, "tools", &["fixture"]),
            operation,
        );
        let mut serialized = serde_json::to_value(descriptor).unwrap();
        serialized.as_object_mut().unwrap().insert(
            "stable_path".to_string(),
            serde_json::json!("Tools/Fixture/Fixture Command"),
        );

        assert!(serde_json::from_value::<EditorMenuItemDescriptor>(serialized).is_err());
    }

    #[test]
    fn descriptor_validation_avoids_short_lived_scan_allocations() {
        let source = include_str!("contribution_descriptors.rs");
        let menu_collection = ["split('/')", ".collect::<Vec<_>>()"].concat();
        let binding_clone = ["EditorOperationPath::parse", "(binding.clone())"].concat();
        assert!(!source.contains(&menu_collection));
        assert!(!source.contains(&binding_clone));

        let extension_body = source
            .split("fn push_normalized_extension")
            .nth(1)
            .and_then(|body| body.split("pub(super) fn validate_asset_importer").next())
            .expect("extension normalization body should remain available");
        assert!(extension_body.contains("binary_search"));
        assert!(extension_body.contains("extensions.windows(2)"));
    }
}

#[cfg(test)]
#[path = "contribution_descriptors/optimization_tests.rs"]
mod optimization_tests;
