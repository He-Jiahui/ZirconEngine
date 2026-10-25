use zircon_runtime::asset::assets::{ModelAsset, SceneMeshInstanceAsset};

use super::{StaticMeshInstanceObservation, StaticMeshSource, StaticMeshSourceError};

pub struct StaticMeshSourceAdapter;

impl StaticMeshSourceAdapter {
    /// Reads original typed payloads supplied together by the existing source/provider owner.
    /// This checks base primitive correspondence only, never inferring source nodes/skins,
    /// asset UUIDs, material classes or static/skinned admission from locator text.
    pub fn read(
        model: &ModelAsset,
        instance: Option<&SceneMeshInstanceAsset>,
    ) -> Result<StaticMeshSource, StaticMeshSourceError> {
        let instance = match instance {
            None => StaticMeshInstanceObservation::Unknown,
            Some(instance) => {
                if instance.model.locator != model.uri {
                    return Err(StaticMeshSourceError::ModelMismatch {
                        expected: model.uri.clone(),
                        actual: instance.model.locator.clone(),
                    });
                }
                // A legacy first/fallback material never fills missing per-primitive bindings.
                if instance.primitives.len() != model.primitives.len() {
                    return Err(StaticMeshSourceError::BindingCountMismatch {
                        expected: model.primitives.len(),
                        actual: instance.primitives.len(),
                    });
                }
                for (primitive_index, (primitive, binding)) in model
                    .primitives
                    .iter()
                    .zip(&instance.primitives)
                    .enumerate()
                {
                    let mesh = primitive
                        .mesh
                        .as_ref()
                        .ok_or(StaticMeshSourceError::MissingMeshReference { primitive_index })?;
                    if mesh != &binding.mesh {
                        return Err(StaticMeshSourceError::MeshReferenceMismatch {
                            primitive_index,
                            expected: Box::new(mesh.clone()),
                            actual: Box::new(binding.mesh.clone()),
                        });
                    }
                }
                StaticMeshInstanceObservation::Known(Box::new(instance.clone()))
            }
        };
        Ok(StaticMeshSource {
            geometry: model.overview(),
            instance,
        })
    }
}
