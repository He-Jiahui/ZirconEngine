use crate::render_graph::CompiledRenderGraph;
use crate::rhi::{RenderDeviceProfile, SubmissionTicket};

use super::super::TransientResourcePool;
use super::RenderGraphExecutionResources;

impl RenderGraphExecutionResources {
    pub(in crate::graphics::scene::scene_renderer) fn materialize_transient_resources_with_pool(
        &mut self,
        device: &wgpu::Device,
        device_profile: &RenderDeviceProfile,
        graph: &CompiledRenderGraph,
        pool: &mut TransientResourcePool,
    ) -> Result<(), String> {
        self.device_epoch = Some(super::super::RenderPassDeviceEpoch::from_profile(
            device_profile,
        ));
        super::super::materialization::materialize_transient_resources(
            self,
            device,
            device_profile,
            graph,
            pool,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn release_transient_backings_into_pool(
        &mut self,
        pool: &mut TransientResourcePool,
    ) {
        self.clear_transient_binding_metadata();

        for (_, allocation) in std::mem::take(&mut self.owned_textures) {
            pool.release_texture(allocation);
        }

        for (_, allocation) in std::mem::take(&mut self.owned_buffers) {
            pool.release_buffer(allocation);
        }
    }

    /// Transfers frame backings to the pool only after their submit ticket reaches completion.
    pub(in crate::graphics::scene::scene_renderer) fn retire_transient_backings_after_submission(
        &mut self,
        pool: &mut TransientResourcePool,
        ticket: SubmissionTicket,
    ) {
        self.clear_transient_binding_metadata();

        for (_, allocation) in std::mem::take(&mut self.owned_textures) {
            pool.release_texture_after_submission(allocation, ticket);
        }

        for (_, allocation) in std::mem::take(&mut self.owned_buffers) {
            pool.release_buffer_after_submission(allocation, ticket);
        }
    }

    fn clear_transient_binding_metadata(&mut self) {
        self.clear_transient_access_bindings();
        self.clear_persistent_texture_access_bindings();
        self.clear_external_access_bindings();
        self.imported_texture_views.clear();
        self.sampled_texture_identities.clear();
        self.imported_textures.clear();
        self.imported_texture_descs.clear();
        self.buffers.clear();
        self.imported_buffer_descs.clear();
        self.owned_texture_backings.clear();
        self.texture_view_aliases.clear();
        self.buffer_backings.clear();
    }
}
