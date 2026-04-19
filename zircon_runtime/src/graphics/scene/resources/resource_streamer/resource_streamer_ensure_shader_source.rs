use crate::core::resource::{AssetReference, ResourceId};

use crate::graphics::types::GraphicsError;

use super::super::fallback_shader_uri;
use super::super::prepared::PreparedShader;
use super::super::runtime::ShaderRuntime;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_shader_source(
        &mut self,
        reference: &AssetReference,
    ) -> Result<(ResourceId, u64), GraphicsError> {
        let uri = &reference.locator;
        let shader_id = self
            .asset_manager
            .resolve_asset_id(uri)
            .or_else(|| self.asset_manager.resolve_asset_id(&fallback_shader_uri()))
            .ok_or_else(|| GraphicsError::Asset(format!("missing shader resource for {uri}")))?;
        let revision = self.resource_revision(shader_id)?;

        if self
            .shaders
            .get(&shader_id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok((shader_id, revision));
        }

        let shader = self
            .asset_manager
            .load_shader_asset(shader_id)
            .or_else(|_| {
                self.asset_manager
                    .load_shader_asset_by_uri(&fallback_shader_uri())
            })
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        self.shaders.insert(
            shader_id,
            PreparedShader {
                revision,
                runtime: ShaderRuntime {
                    source: shader.source,
                },
            },
        );
        Ok((shader_id, revision))
    }
}
