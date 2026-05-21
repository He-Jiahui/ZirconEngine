use std::collections::BTreeSet;

use zircon_runtime::asset::{AssetReference, MaterialAsset, ShaderAsset};
use zircon_runtime::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};

/// Structural authoring projection for one `.zmaterial` and its optional shader contract.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialEditorProjection {
    pub material_name: Option<String>,
    pub shader_reference: AssetReference,
    pub properties: Vec<MaterialEditorPropertyRow>,
    pub texture_slots: Vec<MaterialEditorTextureSlotRow>,
    pub diagnostics: Vec<MaterialEditorDiagnosticRow>,
}

/// One scalar/vector shader property row, including authored override state.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialEditorPropertyRow {
    pub name: String,
    pub kind: Option<String>,
    pub group: Option<String>,
    pub label: Option<String>,
    pub default_value: Option<toml::Value>,
    pub override_value: Option<toml::Value>,
    pub is_overridden: bool,
}

/// One shader texture slot row, including concrete reference or fallback authoring state.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialEditorTextureSlotRow {
    pub name: String,
    pub kind: Option<String>,
    pub group: Option<String>,
    pub label: Option<String>,
    pub default_fallback: Option<String>,
    pub reference: Option<AssetReference>,
    pub fallback: Option<String>,
    pub is_overridden: bool,
}

/// Editor-facing diagnostic row with a stable source document path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialEditorDiagnosticRow {
    pub source: Option<RenderMaterialDiagnosticSource>,
    pub path: String,
    pub message: String,
}

impl MaterialEditorProjection {
    pub fn from_material(material: &MaterialAsset, shader: Option<&ShaderAsset>) -> Self {
        let properties = project_property_rows(material, shader);
        let texture_slots = project_texture_slot_rows(material, shader);
        let diagnostics = project_diagnostic_rows(material, shader);

        Self {
            material_name: material.name.clone(),
            shader_reference: material.shader.clone(),
            properties,
            texture_slots,
            diagnostics,
        }
    }
}

fn project_property_rows(
    material: &MaterialAsset,
    shader: Option<&ShaderAsset>,
) -> Vec<MaterialEditorPropertyRow> {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();

    if let Some(shader) = shader {
        for property in &shader.property_schema {
            let override_value = material.property_overrides().get(&property.name).cloned();
            rows.push(MaterialEditorPropertyRow {
                name: property.name.clone(),
                kind: Some(property.kind.clone()),
                group: property.editor.get("group").cloned(),
                label: property.editor.get("label").cloned(),
                default_value: property.default.clone(),
                is_overridden: override_value.is_some(),
                override_value,
            });
            seen.insert(property.name.clone());
        }
    }

    for (name, value) in material.property_overrides() {
        if seen.contains(name) {
            continue;
        }
        rows.push(MaterialEditorPropertyRow {
            name: name.clone(),
            kind: None,
            group: None,
            label: None,
            default_value: None,
            override_value: Some(value.clone()),
            is_overridden: true,
        });
    }

    rows
}

fn project_texture_slot_rows(
    material: &MaterialAsset,
    shader: Option<&ShaderAsset>,
) -> Vec<MaterialEditorTextureSlotRow> {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();

    if let Some(shader) = shader {
        for slot in &shader.texture_slots {
            let material_slot = material.texture_slots.get(&slot.name);
            rows.push(MaterialEditorTextureSlotRow {
                name: slot.name.clone(),
                kind: Some(slot.kind.clone()),
                group: slot.group.clone(),
                label: slot.label.clone(),
                default_fallback: slot.default.clone(),
                reference: material_slot.and_then(|value| value.reference.clone()),
                fallback: material_slot.and_then(|value| value.fallback.clone()),
                is_overridden: material_slot.is_some(),
            });
            seen.insert(slot.name.clone());
        }
    }

    for (name, value) in &material.texture_slots {
        if seen.contains(name) {
            continue;
        }
        rows.push(MaterialEditorTextureSlotRow {
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

    rows
}

fn project_diagnostic_rows(
    material: &MaterialAsset,
    shader: Option<&ShaderAsset>,
) -> Vec<MaterialEditorDiagnosticRow> {
    let mut rows = material
        .validation_errors()
        .into_iter()
        .map(diagnostic_row_for_error)
        .collect::<Vec<_>>();
    rows.extend(material.validation_diagnostics.iter().map(|diagnostic| {
        MaterialEditorDiagnosticRow {
            source: None,
            path: "material.validation_diagnostics".to_string(),
            message: diagnostic.clone(),
        }
    }));

    if let Some(shader) = shader {
        rows.extend(
            material
                .shader_contract_diagnostics(shader)
                .into_iter()
                .map(diagnostic_row_for_error),
        );
        rows.extend(
            shader
                .validation_diagnostics
                .iter()
                .map(shader_validation_diagnostic_row),
        );
    }

    rows
}

fn shader_validation_diagnostic_row(diagnostic: &String) -> MaterialEditorDiagnosticRow {
    MaterialEditorDiagnosticRow {
        source: diagnostic
            .starts_with("wgsl_capture ")
            .then_some(RenderMaterialDiagnosticSource::WgslCapture),
        path: "shader.validation_diagnostics".to_string(),
        message: diagnostic.clone(),
    }
}

fn diagnostic_row_for_error(error: RenderMaterialValidationError) -> MaterialEditorDiagnosticRow {
    match error {
        RenderMaterialValidationError::InvalidMaskCutoff { cutoff } => {
            MaterialEditorDiagnosticRow {
                source: None,
                path: "overrides.alpha_mode.cutoff".to_string(),
                message: format!("alpha mask cutoff {cutoff} must be finite and within 0.0..=1.0"),
            }
        }
        RenderMaterialValidationError::UnresolvedMaterialReference { material } => {
            MaterialEditorDiagnosticRow {
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: "material".to_string(),
                message: format!("material `{material}` could not be resolved"),
            }
        }
        RenderMaterialValidationError::MissingRuntimeShaderSource => MaterialEditorDiagnosticRow {
            source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
            path: "shader".to_string(),
            message: "shader has no runtime WGSL source".to_string(),
        },
        RenderMaterialValidationError::UnresolvedShaderReference { reference } => {
            MaterialEditorDiagnosticRow {
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: "shader".to_string(),
                message: format!("shader `{}` could not be resolved", reference.locator),
            }
        }
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference } => {
            MaterialEditorDiagnosticRow {
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: format!("textures.{slot}"),
                message: format!("texture `{}` could not be resolved", reference.locator),
            }
        }
        RenderMaterialValidationError::TextureNotUploadReady {
            slot,
            reference,
            reason,
        } => MaterialEditorDiagnosticRow {
            source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
            path: format!("textures.{slot}"),
            message: format!(
                "texture `{}` is not upload-ready: {reason}",
                reference.locator
            ),
        },
        RenderMaterialValidationError::UnknownPropertyOverride { source, path, name } => {
            MaterialEditorDiagnosticRow {
                source: Some(source),
                path,
                message: format!("property override `{name}` is not declared by the shader"),
            }
        }
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } => MaterialEditorDiagnosticRow {
            source: Some(source),
            path,
            message: format!("property override `{name}` must match shader type `{expected}`"),
        },
        RenderMaterialValidationError::MissingRequiredProperty { source, path, name } => {
            MaterialEditorDiagnosticRow {
                source: Some(source),
                path,
                message: format!("required shader property `{name}` needs a material override"),
            }
        }
        RenderMaterialValidationError::UnknownTextureSlot { source, path, slot } => {
            MaterialEditorDiagnosticRow {
                source: Some(source),
                path,
                message: format!("texture slot `{slot}` is not declared by the shader"),
            }
        }
        RenderMaterialValidationError::MissingWgslCapture { source, path, name } => {
            MaterialEditorDiagnosticRow {
                source: Some(source),
                path,
                message: format!("shader WGSL does not appear to capture `{name}`"),
            }
        }
    }
}
