use std::error::Error;
use std::fmt;

use zircon_runtime::asset::{AssetReference, AssetUri, ShaderAsset};
use zircon_runtime::core::framework::render::RenderMaterialValidationError;

use super::{
    compile_material_graph_with_diagnostics, MaterialAsset, MaterialGraphAsset,
    MaterialGraphDiagnostic,
};

/// Graph and shader failures retain their own source vocabulary. Shader paths
/// must not be presented as graph node IDs or synthesized pin locations.
#[derive(Debug, PartialEq)]
pub enum MaterialGraphShaderCompileError {
    Graph(Vec<MaterialGraphDiagnostic>),
    ShaderLocatorMismatch {
        expected: AssetReference,
        actual: AssetUri,
    },
    ShaderImportDiagnostics(Vec<String>),
    ShaderContract(Vec<RenderMaterialValidationError>),
}

impl fmt::Display for MaterialGraphShaderCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graph(diagnostics) => write!(
                formatter,
                "material graph compilation reported {} diagnostics",
                diagnostics.len()
            ),
            Self::ShaderLocatorMismatch { expected, actual } => write!(
                formatter,
                "material graph shader `{expected}` does not match supplied shader `{actual}`"
            ),
            Self::ShaderImportDiagnostics(diagnostics) => write!(
                formatter,
                "material graph shader import reported {} diagnostics",
                diagnostics.len()
            ),
            Self::ShaderContract(diagnostics) => write!(
                formatter,
                "material graph output violates {} shader contract requirements",
                diagnostics.len()
            ),
        }
    }
}

impl Error for MaterialGraphShaderCompileError {}

/// Compiles the existing graph backend, then checks its material against the
/// supplied shader's actual Runtime contract and retained import diagnostics.
/// Neither source is mutated. The legacy graph-only compile API stays unchanged.
///
/// The host must resolve the full shader reference through its ResourceManager
/// and bind this borrowed shader to that original active version, document and
/// request. URI equality below catches a wrong source, but does not prove UUID,
/// payload identity, remove/recreate ABA, texture residency or shader readiness.
/// This function neither generates GPU shader code nor publishes a material,
/// writes a source file, acquires a mutation permit or marks a document saved.
pub fn compile_material_graph_with_shader_contract(
    graph: &MaterialGraphAsset,
    shader: &ShaderAsset,
) -> Result<MaterialAsset, MaterialGraphShaderCompileError> {
    let material = compile_material_graph_with_diagnostics(graph)
        .map_err(MaterialGraphShaderCompileError::Graph)?;
    if material.shader.locator != shader.uri {
        return Err(MaterialGraphShaderCompileError::ShaderLocatorMismatch {
            expected: material.shader,
            actual: shader.uri.clone(),
        });
    }
    if !shader.validation_diagnostics.is_empty() {
        return Err(MaterialGraphShaderCompileError::ShaderImportDiagnostics(
            shader.validation_diagnostics.clone(),
        ));
    }
    let diagnostics = material.shader_contract_diagnostics(shader);
    if diagnostics.is_empty() {
        Ok(material)
    } else {
        Err(MaterialGraphShaderCompileError::ShaderContract(diagnostics))
    }
}
