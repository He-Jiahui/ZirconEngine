use std::collections::BTreeMap;
use std::sync::Arc;

use super::{InspectorField, InspectorRegistrationError};

const ASSET_REFERENCE_MARKERS: [&str; 21] = [
    "asset",
    "asset_handle",
    "asset_id",
    "asset_locator",
    "asset_reference",
    "audio",
    "curve_asset",
    "font",
    "material",
    "mesh",
    "prefab",
    "render_target",
    "script",
    "shader",
    "sprite",
    "texture",
    "tile_set",
    "ui_document",
    "ui_theme",
    "video",
    "world",
];
const NUMBER_FIELD_TYPE_ALIASES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "number",
];
const BUILTIN_FIELD_TYPE_ALIASES: &[&str] = &[
    "bool",
    "boolean",
    "color",
    "enum",
    "asset_reference",
    "curve",
];

/// Stable field-editor families understood by retained and reflected inspector surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldEditorKind {
    Auto,
    Numeric,
    Boolean,
    Color,
    Enum,
    AssetReference,
    CurvePlaceholder,
}

impl FieldEditorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Numeric => "numeric",
            Self::Boolean => "boolean",
            Self::Color => "color",
            Self::Enum => "enum",
            Self::AssetReference => "asset_reference",
            Self::CurvePlaceholder => "curve_placeholder",
        }
    }
}

/// A resolved type-level editor. Asset reference editors retain their allow-list in the instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldEditorInstance {
    kind: FieldEditorKind,
    asset_reference_markers: Arc<[&'static str]>,
}

impl FieldEditorInstance {
    pub fn automatic() -> Self {
        Self::new(FieldEditorKind::Auto)
    }

    pub fn new(kind: FieldEditorKind) -> Self {
        Self {
            kind,
            asset_reference_markers: Arc::from([]),
        }
    }

    fn asset_reference() -> Self {
        Self {
            kind: FieldEditorKind::AssetReference,
            asset_reference_markers: Arc::from(ASSET_REFERENCE_MARKERS),
        }
    }

    pub const fn kind(&self) -> FieldEditorKind {
        self.kind
    }

    pub fn asset_reference_markers(&self) -> &[&'static str] {
        &self.asset_reference_markers
    }
}

/// Input passed to a field-editor factory without exposing mutable inspector state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldEditorInit {
    field: InspectorField,
}

impl FieldEditorInit {
    pub fn field(&self) -> &InspectorField {
        &self.field
    }
}

pub type FieldEditorFactory = fn(FieldEditorInit) -> FieldEditorInstance;

/// Type-keyed definition contributed into a ticket-owned field editor catalog.
#[derive(Clone)]
pub struct FieldEditorDefinition {
    type_name: String,
    make: FieldEditorFactory,
}

impl FieldEditorDefinition {
    pub fn new(type_name: impl Into<String>, make: FieldEditorFactory) -> Self {
        Self {
            type_name: type_name.into(),
            make,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn validate(&self) -> Result<(), InspectorRegistrationError> {
        if !super::is_stable_type_name(&self.type_name) {
            return Err(InspectorRegistrationError::InvalidTypeName(
                self.type_name.clone(),
            ));
        }
        Ok(())
    }

    fn make(&self, field: InspectorField) -> FieldEditorInstance {
        (self.make)(FieldEditorInit { field })
    }
}

/// Registration-owned type editor catalog. Lookups never rebuild the catalog or field metadata.
#[derive(Clone, Default)]
pub struct FieldEditorContainer {
    definitions: BTreeMap<String, FieldEditorDefinition>,
}

impl FieldEditorContainer {
    pub fn builtin() -> Self {
        Self {
            definitions: BTreeMap::from([
                (
                    "number".to_owned(),
                    FieldEditorDefinition::new("number", numeric_editor),
                ),
                (
                    "bool".to_owned(),
                    FieldEditorDefinition::new("bool", boolean_editor),
                ),
                (
                    "color".to_owned(),
                    FieldEditorDefinition::new("color", color_editor),
                ),
                (
                    "enum".to_owned(),
                    FieldEditorDefinition::new("enum", enum_editor),
                ),
                (
                    "asset_reference".to_owned(),
                    FieldEditorDefinition::new("asset_reference", asset_reference_editor),
                ),
                (
                    "curve".to_owned(),
                    FieldEditorDefinition::new("curve", curve_placeholder_editor),
                ),
            ]),
        }
    }

    pub fn register(
        &mut self,
        definition: FieldEditorDefinition,
    ) -> Result<(), InspectorRegistrationError> {
        let type_name = definition.type_name.clone();
        definition.validate()?;
        if is_builtin_field_editor_alias(&type_name)
            && normalize_field_type_name(&type_name) != type_name
        {
            return Err(InspectorRegistrationError::NonCanonicalFieldEditorType(
                type_name,
            ));
        }
        if self.definitions.contains_key(&type_name) {
            return Err(InspectorRegistrationError::DuplicateFieldEditor(type_name));
        }
        self.definitions.insert(type_name, definition);
        Ok(())
    }

    pub fn with_contributions(
        definitions: impl IntoIterator<Item = FieldEditorDefinition>,
    ) -> Result<Self, InspectorRegistrationError> {
        let mut container = Self::builtin();
        for definition in definitions {
            container.register(definition)?;
        }
        Ok(container)
    }

    pub fn definition(&self, type_name: &str) -> Option<&FieldEditorDefinition> {
        if let Some(definition) = self.definitions.get(type_name) {
            return Some(definition);
        }
        // Qualified reflection identities belong to their contributing
        // plugin. A missing or revoked exact definition must not silently
        // select a similarly named built-in editor.
        (!is_qualified_field_type_name(type_name))
            .then(|| self.definitions.get(normalize_field_type_name(type_name)))
            .flatten()
    }

    pub fn resolve(&self, field: InspectorField) -> FieldEditorInstance {
        self.definition(field.type_name())
            .map(|definition| definition.make(field))
            .unwrap_or_else(FieldEditorInstance::automatic)
    }

    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        self.definitions.keys().map(String::as_str)
    }
}

fn normalize_field_type_name(type_name: &str) -> &str {
    if NUMBER_FIELD_TYPE_ALIASES
        .iter()
        .any(|alias| type_name.eq_ignore_ascii_case(alias))
    {
        return "number";
    }
    if type_name.eq_ignore_ascii_case("bool") || type_name.eq_ignore_ascii_case("boolean") {
        return "bool";
    }
    if ends_with_ignore_ascii_case(type_name, "color") {
        return "color";
    }
    if ends_with_ignore_ascii_case(type_name, "enum") {
        return "enum";
    }
    if contains_ignore_ascii_case(type_name, "asset")
        || ends_with_ignore_ascii_case(type_name, "resource")
    {
        return "asset_reference";
    }
    if ends_with_ignore_ascii_case(type_name, "curve") {
        return "curve";
    }
    type_name
}

fn is_builtin_field_editor_alias(type_name: &str) -> bool {
    NUMBER_FIELD_TYPE_ALIASES
        .iter()
        .chain(BUILTIN_FIELD_TYPE_ALIASES)
        .any(|alias| type_name.eq_ignore_ascii_case(alias))
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .as_bytes()
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix.as_bytes()))
}

fn contains_ignore_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn is_qualified_field_type_name(type_name: &str) -> bool {
    type_name.contains('.') || type_name.contains("::")
}

fn numeric_editor(_init: FieldEditorInit) -> FieldEditorInstance {
    FieldEditorInstance::new(FieldEditorKind::Numeric)
}

fn boolean_editor(_init: FieldEditorInit) -> FieldEditorInstance {
    FieldEditorInstance::new(FieldEditorKind::Boolean)
}

fn color_editor(_init: FieldEditorInit) -> FieldEditorInstance {
    FieldEditorInstance::new(FieldEditorKind::Color)
}

fn enum_editor(_init: FieldEditorInit) -> FieldEditorInstance {
    FieldEditorInstance::new(FieldEditorKind::Enum)
}

fn asset_reference_editor(_init: FieldEditorInit) -> FieldEditorInstance {
    FieldEditorInstance::asset_reference()
}

fn curve_placeholder_editor(_init: FieldEditorInit) -> FieldEditorInstance {
    FieldEditorInstance::new(FieldEditorKind::CurvePlaceholder)
}

#[cfg(test)]
mod performance_contract_tests {
    use super::*;

    #[test]
    fn field_type_normalization_borrows_ascii_aliases_and_preserves_qualified_types() {
        for (type_name, expected) in [
            ("I64", "number"),
            ("BoOlEaN", "bool"),
            ("LinearCOLOR", "color"),
            ("EditorENUM", "enum"),
            ("TextureASSET", "asset_reference"),
            ("GpuRESOURCE", "asset_reference"),
            ("AnimationCURVE", "curve"),
            ("OpaqueCustomRecord", "OpaqueCustomRecord"),
        ] {
            assert_eq!(normalize_field_type_name(type_name), expected);
        }

        let editors = FieldEditorContainer::builtin();
        assert!(editors.definition("Plugin.TextureAsset").is_none());
        assert!(editors.definition("plugin::LinearColor").is_none());
    }
}
