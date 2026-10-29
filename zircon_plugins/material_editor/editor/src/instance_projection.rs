use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use zircon_editor::ui::material_editor::{
    MaterialEditorDiagnosticRow, MaterialEditorProjection, MaterialEditorPropertyRow,
    MaterialEditorTextureSlotRow,
};
use zircon_runtime::asset::{AssetReference, AssetUri, ShaderAsset, ZMaterialDocument};
use zircon_runtime::core::framework::render::RenderMaterialDiagnosticSource;

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
        if document.shader.locator != shader.uri {
            return Err(MaterialDocumentProjectionError::ShaderLocatorMismatch {
                expected: document.shader.clone(),
                actual: shader.uri.clone(),
            });
        }
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
