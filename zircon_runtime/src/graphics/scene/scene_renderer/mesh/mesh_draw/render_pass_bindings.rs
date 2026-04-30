use crate::graphics::scene::resources::PipelineKey;

use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn pipeline_key(&self) -> &PipelineKey {
        &self.pipeline_key
    }

    pub(crate) fn bind_model<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        pass.set_bind_group(1, &self.model_bind_group, &[]);
    }

    pub(crate) fn bind_texture<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        pass.set_bind_group(2, &self.texture.bind_group, &[]);
    }

    pub(crate) fn bind_geometry_buffers<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    }

    pub(crate) fn record_indexed_draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        if let Some(indirect_args_buffer) = &self.indirect_args_buffer {
            pass.draw_indexed_indirect(indirect_args_buffer, self.indirect_args_offset);
        } else {
            pass.draw_indexed(
                self.first_index..(self.first_index + self.draw_index_count),
                0,
                0..1,
            );
        }
    }
}
