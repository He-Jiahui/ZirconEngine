#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_submission_tokens(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        read_submission_tokens_buffer(
            self,
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_submission_buffer()
                .as_deref(),
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_args_count(),
            "zircon-vg-indirect-submission-tokens",
        )
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_submission_tokens(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        read_submission_tokens_buffer(
            self,
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_execution_submission_buffer()
                .as_deref(),
            self.advanced_plugin_outputs
                .virtual_geometry_indirect_draw_count(),
            "zircon-vg-indirect-execution-submission-tokens",
        )
    }
}

#[cfg(test)]
fn read_submission_tokens_buffer(
    renderer: &SceneRenderer,
    buffer: Option<&wgpu::Buffer>,
    token_count: u32,
    label_prefix: &str,
) -> Result<Vec<u32>, GraphicsError> {
    let Some(buffer) = buffer else {
        return Ok(Vec::new());
    };
    if token_count == 0 {
        return Ok(Vec::new());
    }

    let staging = renderer
        .backend
        .device
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{label_prefix}-readback")),
            size: (token_count as u64) * (std::mem::size_of::<u32>() as u64),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    let mut encoder =
        renderer
            .backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("{label_prefix}-readback-encoder")),
            });
    encoder.copy_buffer_to_buffer(
        buffer,
        0,
        &staging,
        0,
        (token_count as u64) * (std::mem::size_of::<u32>() as u64),
    );
    renderer.backend.queue.submit([encoder.finish()]);
    read_buffer_u32s(&renderer.backend.device, &staging, token_count as usize)
}
