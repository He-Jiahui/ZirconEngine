use std::collections::{BTreeMap, HashMap};

use crate::asset::AssetReference;
use crate::core::framework::render::RenderMaterialDiagnosticSource;

use super::{
    CompiledRenderPipeline, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity,
};

/// Compiled render graph plus authoring diagnostics collected from feature assets.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderPipelineCompileReport {
    pub pipeline: CompiledRenderPipeline,
    pub diagnostics: Vec<RendererFeatureContractDiagnostic>,
}

impl RenderPipelineCompileReport {
    /// Groups report diagnostics by runtime feature name for renderer-data tooling.
    pub fn diagnostics_by_feature(
        &self,
    ) -> BTreeMap<&str, Vec<&RendererFeatureContractDiagnostic>> {
        let mut diagnostics = BTreeMap::new();
        for diagnostic in &self.diagnostics {
            diagnostics
                .entry(diagnostic.feature())
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    /// Groups diagnostics that are owned by or directly point at a material asset.
    pub fn diagnostics_by_material(
        &self,
    ) -> HashMap<AssetReference, Vec<&RendererFeatureContractDiagnostic>> {
        let mut diagnostics = HashMap::with_capacity(self.diagnostics.len());
        for diagnostic in &self.diagnostics {
            let Some(material) = diagnostic.material_reference() else {
                continue;
            };
            diagnostics
                .entry(material.clone())
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    /// Groups classified diagnostics by their repair/source family.
    pub fn diagnostics_by_source(
        &self,
    ) -> BTreeMap<RenderMaterialDiagnosticSource, Vec<&RendererFeatureContractDiagnostic>> {
        let mut diagnostics = BTreeMap::new();
        for diagnostic in &self.diagnostics {
            let Some(source) = diagnostic.source() else {
                continue;
            };
            diagnostics
                .entry(source)
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    /// Groups diagnostics by triage severity for editor/report filtering.
    pub fn diagnostics_by_severity(
        &self,
    ) -> BTreeMap<RendererFeatureContractDiagnosticSeverity, Vec<&RendererFeatureContractDiagnostic>>
    {
        let mut diagnostics = BTreeMap::new();
        for diagnostic in &self.diagnostics {
            diagnostics
                .entry(diagnostic.severity())
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    /// Groups diagnostics that are owned by or directly point at shader assets.
    pub fn diagnostics_by_shader(
        &self,
    ) -> HashMap<AssetReference, Vec<&RendererFeatureContractDiagnostic>> {
        let mut diagnostics = HashMap::new();
        for diagnostic in &self.diagnostics {
            for shader in diagnostic.shader_references() {
                diagnostics
                    .entry(shader.clone())
                    .or_insert_with(Vec::new)
                    .push(diagnostic);
            }
        }
        diagnostics
    }
}
