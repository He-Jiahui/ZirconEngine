use crate::core::framework::render::{ShadingModelDescriptor, ShadingModelId};
use crate::graphics::material::{ShadingModelIncludeSourceError, ShadingModelIncludeSourceSet};

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn shading_model_descriptor(
        &self,
        id: ShadingModelId,
    ) -> Option<&ShadingModelDescriptor> {
        self.shading_model_registry.get(id)
    }

    pub(crate) fn shading_model_descriptor_for_pipeline_key(
        &self,
        key: &super::super::PipelineKey,
    ) -> Option<&ShadingModelDescriptor> {
        self.shading_model_descriptor(key.shading_model_id)
    }

    pub(crate) fn shading_model_descriptors(&self) -> Vec<ShadingModelDescriptor> {
        self.shading_model_registry.descriptors().cloned().collect()
    }

    pub(crate) fn shading_model_include_source_set(
        &self,
    ) -> Result<ShadingModelIncludeSourceSet, ShadingModelIncludeSourceError> {
        let asset_manager =
            self.asset_manager()
                .map_err(|error| ShadingModelIncludeSourceError::LoadShader {
                    token: "project_asset_manager".to_string(),
                    locator: "runtime://manager/project-assets".to_string(),
                    message: error.to_string(),
                })?;
        ShadingModelIncludeSourceSet::from_project_asset_manager(
            asset_manager.as_ref(),
            &self.shading_model_descriptors(),
        )
    }
}
