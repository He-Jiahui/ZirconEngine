use crate::asset::{ShaderAsset, ShaderRuntimeSourceKind};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialReadinessDiagnostic,
    RenderMaterialReadinessReport, RenderMaterialValidationError,
};

use super::MaterialAsset;

pub(super) fn push_shader_readiness_validation_errors(
    report: &mut RenderMaterialReadinessReport,
    shader: &ShaderAsset,
) {
    let readiness = shader.readiness_report();
    if readiness.runtime_source.source_kind == ShaderRuntimeSourceKind::Unavailable {
        report
            .push_validation_error_once(RenderMaterialValidationError::MissingRuntimeShaderSource);
    }

    for entry in readiness.entry_points {
        if let Some(diagnostic) = entry.diagnostic {
            report.push_validation_error_once(
                RenderMaterialValidationError::ShaderReadinessDiagnostic {
                    source: RenderMaterialDiagnosticSource::ShaderReadiness,
                    path: format!("entry_points.{}", entry.name),
                    diagnostic,
                },
            );
        }
    }

    for definition in readiness.shader_defs {
        if let Some(diagnostic) = definition.diagnostic {
            let path_name = if definition.normalized_name.is_empty() {
                "<empty>".to_string()
            } else {
                definition.normalized_name
            };
            report.push_validation_error_once(
                RenderMaterialValidationError::ShaderReadinessDiagnostic {
                    source: RenderMaterialDiagnosticSource::ShaderReadiness,
                    path: format!("shader_defs.{path_name}"),
                    diagnostic,
                },
            );
        }
    }

    for diagnostic in readiness.validation_diagnostics {
        report.push_validation_error_once(RenderMaterialValidationError::MissingWgslCapture {
            source: RenderMaterialDiagnosticSource::WgslCapture,
            path: "shader.validation_diagnostics".to_string(),
            name: diagnostic,
        });
    }
}

pub(super) fn material_readiness_diagnostics(
    material: &MaterialAsset,
) -> Vec<RenderMaterialReadinessDiagnostic> {
    material
        .validation_diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| RenderMaterialReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::MaterialAsset,
            path: format!("material.validation_diagnostics[{index}]"),
            diagnostic: diagnostic.clone(),
        })
        .collect()
}
