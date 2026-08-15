use serde::{Deserialize, Serialize};

use crate::core::asset::AssetTypeId;
use crate::core::editor_operation::EditorOperationPath;

use super::{validate_contribution_id, EditorExtensionRegistryError};

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMenuItemDescriptor {
    path: String,
    operation: EditorOperationPath,
    #[serde(default)]
    priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shortcut: Option<String>,
    #[serde(default = "default_menu_item_enabled")]
    enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl EditorMenuItemDescriptor {
    pub fn new(path: impl Into<String>, operation: EditorOperationPath) -> Self {
        Self {
            path: path.into(),
            operation,
            priority: 0,
            shortcut: None,
            enabled: true,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
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
        self.required_capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn shortcut(&self) -> Option<&str> {
        self.shortcut.as_deref()
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
    let mut segment_count = 0;
    let valid = descriptor.path.split('/').all(|segment| {
        segment_count += 1;
        !segment.trim().is_empty() && segment.trim() == segment
    });
    if !valid || segment_count < MIN_MENU_PATH_SEGMENTS {
        return Err(EditorExtensionRegistryError::InvalidMenuPath(
            descriptor.path.clone(),
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
        self.required_capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self.required_capabilities.sort();
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
            extensions.sort();
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
