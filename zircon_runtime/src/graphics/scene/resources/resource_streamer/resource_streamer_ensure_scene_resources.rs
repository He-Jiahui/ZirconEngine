use crate::graphics::types::{EditorOrRuntimeFrame, GraphicsError};

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_scene_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<(), GraphicsError> {
        for mesh in &frame.scene.scene.meshes {
            self.ensure_model(device, mesh.model)?;
            self.ensure_material(device, queue, texture_layout, mesh.material)?;
        }
        Ok(())
    }
}
