use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialDependencySet, RenderMaterialFallbackPolicy, RenderMaterialValidationError,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialReadinessReport {
    pub material_name: Option<String>,
    pub dependencies: RenderMaterialDependencySet,
    pub fallback_policy: RenderMaterialFallbackPolicy,
    pub validation_errors: Vec<RenderMaterialValidationError>,
    pub fallback_usages: Vec<RenderMaterialFallbackUsage>,
}

impl RenderMaterialReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.validation_errors.is_empty() && self.fallback_usages.is_empty()
    }

    pub fn uses_fallback(&self) -> bool {
        !self.fallback_usages.is_empty()
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
}
