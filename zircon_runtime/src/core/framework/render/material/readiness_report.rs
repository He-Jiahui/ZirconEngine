use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialDependencySet, RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy,
    RenderMaterialPropertyUniformSummary, RenderMaterialValidationError,
};
use crate::core::resource::{AssetReference, ResourceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialFallbackUsage {
    pub reason: RenderMaterialFallbackReason,
    pub fallback_policy: RenderMaterialFallbackPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum RenderMaterialFallbackReason {
    Material {
        material: ResourceId,
    },
    Shader {
        reference: AssetReference,
    },
    Texture {
        slot: String,
        reference: AssetReference,
    },
    Validation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialReadinessDiagnostic {
    pub source: RenderMaterialDiagnosticSource,
    pub path: String,
    pub diagnostic: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialReadinessReport {
    pub material_name: Option<String>,
    pub dependencies: RenderMaterialDependencySet,
    pub fallback_policy: RenderMaterialFallbackPolicy,
    pub validation_errors: Vec<RenderMaterialValidationError>,
    pub fallback_usages: Vec<RenderMaterialFallbackUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uniform_summary: Option<RenderMaterialPropertyUniformSummary>,
    #[serde(default)]
    pub diagnostics: Vec<RenderMaterialReadinessDiagnostic>,
}

impl RenderMaterialReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.validation_errors.is_empty() && self.fallback_usages.is_empty()
    }

    pub fn uses_fallback(&self) -> bool {
        !self.fallback_usages.is_empty()
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn push_validation_error_once(&mut self, error: RenderMaterialValidationError) {
        if !self.validation_errors.contains(&error) {
            self.validation_errors.push(error);
        }
    }

    pub fn push_fallback_usage_once(&mut self, usage: RenderMaterialFallbackUsage) {
        if !self.fallback_usages.contains(&usage) {
            self.fallback_usages.push(usage);
        }
    }

    pub fn push_diagnostic_once(&mut self, diagnostic: RenderMaterialReadinessDiagnostic) {
        if !self.diagnostics.contains(&diagnostic) {
            self.diagnostics.push(diagnostic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::ResourceLocator;

    #[test]
    fn material_readiness_report_deduplicates_material_uniform_diagnostics() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("res://shaders/uniform_material.zshader")
                .expect("valid shader locator"),
        );
        let mut report = RenderMaterialReadinessReport {
            material_name: Some("UniformMaterial".to_string()),
            dependencies: RenderMaterialDependencySet::new(shader),
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
            validation_errors: Vec::new(),
            fallback_usages: Vec::new(),
            uniform_summary: None,
            diagnostics: Vec::new(),
        };
        let diagnostic = RenderMaterialReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::MaterialUniform,
            path: "uniform.debug_label".to_string(),
            diagnostic: "material property debug_label cannot be encoded into the renderer uniform payload: unsupported property type".to_string(),
        };

        report.push_diagnostic_once(diagnostic.clone());
        report.push_diagnostic_once(diagnostic);

        assert!(report.is_ready());
        assert!(report.has_diagnostics());
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].source,
            RenderMaterialDiagnosticSource::MaterialUniform
        );
        assert_eq!(report.diagnostics[0].path, "uniform.debug_label");
    }
}
