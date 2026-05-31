use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_scene_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &ViewportRenderFrame,
    ) -> Result<(), GraphicsError> {
        self.last_material_count = 0;
        self.last_material_ready_count = 0;
        self.last_material_fallback_count = 0;
        self.last_material_validation_error_count = 0;
        self.last_material_diagnostic_count = 0;
        self.last_sprite_count = 0;
        self.last_sprite_ready_count = 0;
        self.last_sprite_texture_fallback_count = 0;
        for mesh in frame.meshes() {
            let direct_mesh_ready = mesh
                .mesh
                .map(|mesh| self.ensure_mesh(device, mesh).is_ok())
                .unwrap_or(false);
            if !direct_mesh_ready {
                self.ensure_model(device, mesh.model)?;
            }
            self.ensure_material(device, queue, texture_layout, mesh.material)?;
            self.record_material_summary(mesh.material.id());
        }
        for sprite in frame.sprites() {
            self.last_sprite_count += 1;
            if self
                .ensure_sprite_texture(device, queue, texture_layout, sprite.image.id())
                .is_ok()
            {
                self.last_sprite_ready_count += 1;
            } else {
                self.last_sprite_texture_fallback_count += 1;
            }
        }
        Ok(())
    }

    fn record_material_summary(&mut self, material_id: crate::core::resource::ResourceId) {
        self.last_material_count += 1;
        if let Some(summary) = self.material_readiness_summary(&material_id) {
            if summary.is_ready {
                self.last_material_ready_count += 1;
            }
            if summary.uses_fallback {
                self.last_material_fallback_count += 1;
            }
            self.last_material_validation_error_count += summary.validation_error_count;
            self.last_material_diagnostic_count += summary.diagnostic_count;
        }
    }
}
