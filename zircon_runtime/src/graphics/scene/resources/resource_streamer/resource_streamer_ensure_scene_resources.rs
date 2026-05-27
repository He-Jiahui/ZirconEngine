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
            self.ensure_model(device, mesh.model)?;
            self.ensure_material(device, queue, texture_layout, mesh.material)?;
            self.last_material_count += 1;
            if let Some(report) = self.material_readiness_report(&mesh.material.id()) {
                let is_ready = report.is_ready();
                let uses_fallback = report.uses_fallback();
                let validation_error_count = report.validation_errors.len();
                let diagnostic_count = report.diagnostics.len();
                if is_ready {
                    self.last_material_ready_count += 1;
                }
                if uses_fallback {
                    self.last_material_fallback_count += 1;
                }
                self.last_material_validation_error_count += validation_error_count;
                self.last_material_diagnostic_count += diagnostic_count;
            }
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
}
