use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use zircon_editor::ui::material_editor::{
    MaterialEditorDiagnosticRow, MaterialEditorProjection, MaterialEditorPropertyRow,
    MaterialEditorTextureSlotRow,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialTextureSlotValue, ShaderAsset, ShaderMaterialPropertyAsset,
    ShaderTextureSlotAsset, ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialDiagnosticSource, RenderMaterialPropertyValue,
};

#[derive(Debug, PartialEq)]
pub enum MaterialDocumentProjectionError {
    ShaderLocatorMismatch {
        expected: AssetReference,
        actual: AssetUri,
    },
    DuplicatePropertyName(String),
    DuplicateTextureSlotName(String),
}

impl fmt::Display for MaterialDocumentProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShaderLocatorMismatch { expected, actual } => write!(
                formatter,
                "material authoring shader `{expected}` does not match supplied shader `{actual}`"
            ),
            Self::DuplicatePropertyName(name) => {
                write!(formatter, "material shader repeats property key `{name}`")
            }
            Self::DuplicateTextureSlotName(name) => {
                write!(
                    formatter,
                    "material shader repeats texture slot key `{name}`"
                )
            }
        }
    }
}

impl Error for MaterialDocumentProjectionError {}

/// One proposed change to an exact local property key, before a document write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialDocumentPropertyEdit<'a> {
    Set(&'a RenderMaterialPropertyValue),
    ClearOverride,
}

#[derive(Debug, PartialEq)]
pub enum MaterialDocumentPropertyEditError {
    Source(MaterialDocumentProjectionError),
    SchemaUnavailable,
    EmptyPropertyName,
    UndeclaredProperty(String),
    TypeMismatch {
        name: String,
        expected: MaterialPropertyKind,
        actual: &'static str,
    },
    NonFiniteValue(String),
}

impl fmt::Display for MaterialDocumentPropertyEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => fmt::Display::fmt(error, formatter),
            Self::SchemaUnavailable => {
                formatter.write_str("material property Set requires the current shader schema")
            }
            Self::EmptyPropertyName => {
                formatter.write_str("cannot set an empty material property key")
            }
            Self::UndeclaredProperty(name) => {
                write!(
                    formatter,
                    "material shader does not declare property `{name}`"
                )
            }
            Self::TypeMismatch {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "material property `{name}` requires `{expected}`, received `{actual}`"
            ),
            Self::NonFiniteValue(name) => {
                write!(formatter, "new material property `{name}` must be finite")
            }
        }
    }
}

impl Error for MaterialDocumentPropertyEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

/// A borrow-only domain proposal for the original Document transaction owner.
///
/// This retains no cloned document, previous TOML value, undo record or clock.
/// The host must pin its real view/document/request/revision and shader handles,
/// charge the raw previous entry and new payload, then prepare/apply one inverse
/// delta under its existing permit/history protocol. This value grants no write.
/// No-op/CAS comparisons must use the exact authored values, not f32 projections.
/// ClearOverride removes only local presence; it does not write a schema default
/// or prove that a required property's parent-chain value resolves. A reset that
/// retains an override must instead resolve its real default and propose Set.
/// Existing diagnostics and unrelated invalid authored entries remain intact.
#[derive(Debug)]
pub struct MaterialDocumentPropertyEditPlan<'a> {
    document: &'a ZMaterialDocument,
    name: &'a str,
    property: Option<&'a ShaderMaterialPropertyAsset>,
    edit: MaterialDocumentPropertyEdit<'a>,
}

impl<'a> MaterialDocumentPropertyEditPlan<'a> {
    pub fn source_document(&self) -> &'a ZMaterialDocument {
        self.document
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn property_definition(&self) -> Option<&'a ShaderMaterialPropertyAsset> {
        self.property
    }

    pub fn edit(&self) -> MaterialDocumentPropertyEdit<'a> {
        self.edit
    }

    pub fn was_overridden(&self) -> bool {
        self.document.overrides.contains_key(self.name)
    }

    pub fn will_be_overridden(&self) -> bool {
        matches!(self.edit, MaterialDocumentPropertyEdit::Set(_))
    }
}

/// Checks a typed property proposal without mutating or flattening its source.
///
/// Set accepts only a unique declared key and its exact runtime property kind
/// (Color uses Vec4). It does not coerce strings/integers, trim keys or clamp HDR
/// components. A missing schema, ambiguous target or non-finite new value fails.
/// ClearOverride can repair unknown/empty keys or invalid values without a schema.
/// A supplied shader must still match, and a duplicated target remains ambiguous.
/// Other schema entries are not validated by this single-key preflight.
pub fn plan_material_document_property_edit<'a>(
    document: &'a ZMaterialDocument,
    shader: Option<&'a ShaderAsset>,
    name: &'a str,
    edit: MaterialDocumentPropertyEdit<'a>,
) -> Result<MaterialDocumentPropertyEditPlan<'a>, MaterialDocumentPropertyEditError> {
    let mut property = None;
    if let Some(shader) = shader {
        check_shader_locator(document, shader)
            .map_err(MaterialDocumentPropertyEditError::Source)?;
        for candidate in &shader.property_schema {
            if candidate.name == name {
                if property.is_some() {
                    return Err(MaterialDocumentPropertyEditError::Source(
                        MaterialDocumentProjectionError::DuplicatePropertyName(name.into()),
                    ));
                }
                property = Some(candidate);
            }
        }
    }
    if let MaterialDocumentPropertyEdit::Set(value) = edit {
        if name.is_empty() {
            return Err(MaterialDocumentPropertyEditError::EmptyPropertyName);
        }
        if shader.is_none() {
            return Err(MaterialDocumentPropertyEditError::SchemaUnavailable);
        }
        let declaration = property
            .ok_or_else(|| MaterialDocumentPropertyEditError::UndeclaredProperty(name.into()))?;
        let actual = typed_property_kind(value);
        if actual != Some(declaration.kind)
            && !(declaration.kind == MaterialPropertyKind::Color
                && actual == Some(MaterialPropertyKind::Vec4))
        {
            return Err(MaterialDocumentPropertyEditError::TypeMismatch {
                name: name.into(),
                expected: declaration.kind,
                actual: actual.map_or("string", MaterialPropertyKind::token),
            });
        }
        let finite = match value {
            RenderMaterialPropertyValue::Float { value } => value.is_finite(),
            RenderMaterialPropertyValue::Vec2 { value } => value.iter().all(|v| v.is_finite()),
            RenderMaterialPropertyValue::Vec3 { value } => value.iter().all(|v| v.is_finite()),
            RenderMaterialPropertyValue::Vec4 { value } => value.iter().all(|v| v.is_finite()),
            _ => true,
        };
        if !finite {
            return Err(MaterialDocumentPropertyEditError::NonFiniteValue(
                name.into(),
            ));
        }
    }
    Ok(MaterialDocumentPropertyEditPlan {
        document,
        name,
        property,
        edit,
    })
}

fn typed_property_kind(value: &RenderMaterialPropertyValue) -> Option<MaterialPropertyKind> {
    Some(match value {
        RenderMaterialPropertyValue::Bool { .. } => MaterialPropertyKind::Bool,
        RenderMaterialPropertyValue::Float { .. } => MaterialPropertyKind::Float,
        RenderMaterialPropertyValue::Int { .. } => MaterialPropertyKind::Int,
        RenderMaterialPropertyValue::UInt { .. } => MaterialPropertyKind::UInt,
        RenderMaterialPropertyValue::Vec2 { .. } => MaterialPropertyKind::Vec2,
        RenderMaterialPropertyValue::Vec3 { .. } => MaterialPropertyKind::Vec3,
        RenderMaterialPropertyValue::Vec4 { .. } => MaterialPropertyKind::Vec4,
        RenderMaterialPropertyValue::String { .. } => return None,
    })
}

/// Set keeps local presence even when both reference and fallback are empty.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialDocumentTextureEdit<'a> {
    Set(&'a MaterialTextureSlotValue),
    ClearOverride,
}

#[derive(Debug, PartialEq)]
pub enum MaterialDocumentTextureEditError {
    Source(MaterialDocumentProjectionError),
    SchemaUnavailable,
    EmptySlotName,
    UndeclaredSlot(String),
    NonFiniteTransform(String),
}

impl fmt::Display for MaterialDocumentTextureEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => fmt::Display::fmt(error, formatter),
            Self::SchemaUnavailable => {
                formatter.write_str("material texture Set requires the current shader schema")
            }
            Self::EmptySlotName => formatter.write_str("cannot set an empty material texture key"),
            Self::UndeclaredSlot(name) => {
                write!(
                    formatter,
                    "material shader does not declare texture slot `{name}`"
                )
            }
            Self::NonFiniteTransform(name) => {
                write!(
                    formatter,
                    "new texture transform for `{name}` must be finite"
                )
            }
        }
    }
}

impl Error for MaterialDocumentTextureEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

/// A borrow-only proposal for the original Document transaction owner.
///
/// Set borrows the complete authored slot, including its reference identity,
/// fallback, optional transform and UV channel. An explicitly empty slot stays a
/// local override; ClearOverride removes local presence instead. Neither action
/// resolves a parent, chooses a default texture or grants a document write.
/// The host must validate real texture handles/dimensions, sampler/UV readiness
/// and the shader contract, then charge the exact old/new payload and apply its
/// original one-permit inverse delta/history/CAS flow. URI equality alone does
/// not prove payload identity or ResourceManager generation. Required-binding
/// diagnostics cannot be cleared by a successful local authoring preflight.
#[derive(Debug)]
pub struct MaterialDocumentTextureEditPlan<'a> {
    document: &'a ZMaterialDocument,
    name: &'a str,
    slot: Option<&'a ShaderTextureSlotAsset>,
    edit: MaterialDocumentTextureEdit<'a>,
}

impl<'a> MaterialDocumentTextureEditPlan<'a> {
    pub fn source_document(&self) -> &'a ZMaterialDocument {
        self.document
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn slot_definition(&self) -> Option<&'a ShaderTextureSlotAsset> {
        self.slot
    }

    pub fn edit(&self) -> MaterialDocumentTextureEdit<'a> {
        self.edit
    }

    pub fn was_overridden(&self) -> bool {
        self.document.textures.contains_key(self.name)
    }

    pub fn will_be_overridden(&self) -> bool {
        matches!(self.edit, MaterialDocumentTextureEdit::Set(_))
    }
}

/// Checks one exact local texture key without cloning or modifying the source.
///
/// Set requires a unique declaration and finite new UV components. No fallback,
/// alias, URI, UV channel or finite transform component is normalized or clamped.
/// ClearOverride can repair unknown/empty keys and invalid existing transforms
/// without a schema. A supplied shader must match and its target cannot be
/// ambiguous. Unrelated schema entries are not validated by this single-key plan.
pub fn plan_material_document_texture_edit<'a>(
    document: &'a ZMaterialDocument,
    shader: Option<&'a ShaderAsset>,
    name: &'a str,
    edit: MaterialDocumentTextureEdit<'a>,
) -> Result<MaterialDocumentTextureEditPlan<'a>, MaterialDocumentTextureEditError> {
    let mut slot = None;
    if let Some(shader) = shader {
        check_shader_locator(document, shader).map_err(MaterialDocumentTextureEditError::Source)?;
        for candidate in &shader.texture_slots {
            if candidate.name == name {
                if slot.is_some() {
                    return Err(MaterialDocumentTextureEditError::Source(
                        MaterialDocumentProjectionError::DuplicateTextureSlotName(name.into()),
                    ));
                }
                slot = Some(candidate);
            }
        }
    }
    if let MaterialDocumentTextureEdit::Set(value) = edit {
        if name.is_empty() {
            return Err(MaterialDocumentTextureEditError::EmptySlotName);
        }
        if shader.is_none() {
            return Err(MaterialDocumentTextureEditError::SchemaUnavailable);
        }
        if slot.is_none() {
            return Err(MaterialDocumentTextureEditError::UndeclaredSlot(
                name.into(),
            ));
        }
        if let Some(transform) = value.transform {
            if !transform
                .scale
                .iter()
                .chain(transform.offset.iter())
                .all(|v| v.is_finite())
                || !transform.rotation.is_finite()
            {
                return Err(MaterialDocumentTextureEditError::NonFiniteTransform(
                    name.into(),
                ));
            }
        }
    }
    Ok(MaterialDocumentTextureEditPlan {
        document,
        name,
        slot,
        edit,
    })
}

fn check_shader_locator(
    document: &ZMaterialDocument,
    shader: &ShaderAsset,
) -> Result<(), MaterialDocumentProjectionError> {
    if document.shader.locator != shader.uri {
        return Err(MaterialDocumentProjectionError::ShaderLocatorMismatch {
            expected: document.shader.clone(),
            actual: shader.uri.clone(),
        });
    }
    Ok(())
}

/// Projects local authoring presence into the existing editor property rows.
///
/// An effective MaterialAsset can contain inherited values. Its maps cannot
/// determine the override checkboxes of the original ZMaterialDocument. This
/// projection reads that original document, preserving explicit default values,
/// empty texture slots and unknown local keys for repair. Shader groups/labels
/// and defaults keep their source order; no authored key is normalized.
///
/// A shader default is schema metadata, not a resolved inherited value. When the
/// document has a parent, the host must separately resolve its current effective
/// values; it must not display a default as proof of the parent's current value.
/// No material is flattened, reconstructed, compiled, mutated or saved here.
///
/// URI equality rejects an unrelated shader but does not prove UUID, payload or
/// load generation. The original host must bind document/view/request/revision
/// and ResourceManager handles before publishing these rows. This function does
/// not replace runtime type, dependency, shader contract or GPU readiness checks.
pub fn project_authored_material_document(
    document: &ZMaterialDocument,
    shader: Option<&ShaderAsset>,
) -> Result<MaterialEditorProjection, MaterialDocumentProjectionError> {
    if let Some(shader) = shader {
        check_shader_locator(document, shader)?;
    }

    let mut properties = Vec::with_capacity(
        shader
            .map_or(0, |shader| shader.property_schema.len())
            .saturating_add(document.overrides.len()),
    );
    let mut texture_slots = Vec::with_capacity(
        shader
            .map_or(0, |shader| shader.texture_slots.len())
            .saturating_add(document.textures.len()),
    );
    let mut property_names = BTreeSet::new();
    let mut texture_names = BTreeSet::new();

    if let Some(shader) = shader {
        for property in &shader.property_schema {
            if !property_names.insert(property.name.as_str()) {
                return Err(MaterialDocumentProjectionError::DuplicatePropertyName(
                    property.name.clone(),
                ));
            }
            let local = document.overrides.get(&property.name);
            properties.push(MaterialEditorPropertyRow {
                name: property.name.clone(),
                kind: Some(property.kind.to_string()),
                group: property.editor.get("group").cloned(),
                label: property.editor.get("label").cloned(),
                default_value: property.default.clone(),
                override_value: local.cloned(),
                is_overridden: local.is_some(),
            });
        }
        for slot in &shader.texture_slots {
            if !texture_names.insert(slot.name.as_str()) {
                return Err(MaterialDocumentProjectionError::DuplicateTextureSlotName(
                    slot.name.clone(),
                ));
            }
            let local = document.textures.get(&slot.name);
            texture_slots.push(MaterialEditorTextureSlotRow {
                name: slot.name.clone(),
                kind: Some(slot.kind.clone()),
                group: slot.group.clone(),
                label: slot.label.clone(),
                default_fallback: slot.default.clone(),
                reference: local.and_then(|value| value.reference.clone()),
                fallback: local.and_then(|value| value.fallback.clone()),
                is_overridden: local.is_some(),
            });
        }
    }

    for (name, value) in &document.overrides {
        if !property_names.contains(name.as_str()) {
            properties.push(MaterialEditorPropertyRow {
                name: name.clone(),
                kind: None,
                group: None,
                label: None,
                default_value: None,
                override_value: Some(value.clone()),
                is_overridden: true,
            });
        }
    }
    for (name, value) in &document.textures {
        if !texture_names.contains(name.as_str()) {
            texture_slots.push(MaterialEditorTextureSlotRow {
                name: name.clone(),
                kind: None,
                group: None,
                label: None,
                default_fallback: None,
                reference: value.reference.clone(),
                fallback: value.fallback.clone(),
                is_overridden: true,
            });
        }
    }

    let mut diagnostics: Vec<_> = document
        .validation_diagnostics
        .iter()
        .map(|message| MaterialEditorDiagnosticRow {
            source: None,
            path: "material.validation_diagnostics".into(),
            message: message.clone(),
        })
        .collect();
    if let Some(shader) = shader {
        diagnostics.extend(shader.validation_diagnostics.iter().map(|message| {
            MaterialEditorDiagnosticRow {
                source: message
                    .starts_with("wgsl_capture ")
                    .then_some(RenderMaterialDiagnosticSource::WgslCapture),
                path: "shader.validation_diagnostics".into(),
                message: message.clone(),
            }
        }));
    } else {
        diagnostics.push(MaterialEditorDiagnosticRow {
            source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
            path: "shader".into(),
            message: "shader schema is not available for this authoring projection".into(),
        });
    }

    Ok(MaterialEditorProjection {
        material_name: document.name.clone(),
        shader_reference: document.shader.clone(),
        properties,
        texture_slots,
        diagnostics,
    })
}
