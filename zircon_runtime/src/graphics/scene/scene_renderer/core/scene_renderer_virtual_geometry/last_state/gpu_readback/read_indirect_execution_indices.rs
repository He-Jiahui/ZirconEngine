#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::graphics::backend::read_buffer_u32s;
#[cfg(test)]
use crate::graphics::types::GraphicsError;

use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_indirect_execution_draw_ref_indices(
        &self,
    ) -> Result<Vec<u32>, GraphicsError> {
        let execution_authority_records =
            self.read_last_virtual_geometry_indirect_execution_authority_records()?;
        if !execution_authority_records.is_empty() {
            return Ok(execution_authority_records
                .into_iter()
                .map(|record| record.draw_ref_index)
                .collect());
        }
        if let Some(draw_ref_indices) = draw_ref_indices_from_execution_args_and_authority(self)? {
            return Ok(draw_ref_indices);
        }
        if let Some(draw_ref_indices) =
            draw_ref_indices_from_execution_args_and_shared_submission_tokens(self)?
        {
            return Ok(draw_ref_indices);
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
fn draw_ref_indices_from_execution_args_and_authority(
    renderer: &SceneRenderer,
) -> Result<Option<Vec<u32>>, GraphicsError> {
    let execution_tokens = read_execution_submission_tokens(renderer)?;
    if execution_tokens.is_empty() {
        return Ok(None);
    }

    let authority_by_submission_token = renderer
        .read_last_virtual_geometry_indirect_authority_records()?
        .into_iter()
        .map(|record| {
            (
                (record.submission_index.min(0xffff) << 16) | record.draw_ref_rank.min(0xffff),
                record.draw_ref_index,
            )
        })
        .collect::<HashMap<_, _>>();
    if authority_by_submission_token.is_empty() {
        return Ok(None);
    }

    let draw_ref_indices = execution_tokens
        .into_iter()
        .filter_map(|submission_token| {
            authority_by_submission_token
                .get(&submission_token)
                .copied()
        })
        .collect::<Vec<_>>();
    if draw_ref_indices.is_empty() {
        return Ok(None);
    }

    Ok(Some(draw_ref_indices))
}

#[cfg(test)]
fn draw_ref_indices_from_execution_args_and_shared_submission_tokens(
    renderer: &SceneRenderer,
) -> Result<Option<Vec<u32>>, GraphicsError> {
    let execution_tokens = read_execution_submission_tokens(renderer)?;
    if execution_tokens.is_empty() {
        return Ok(None);
    }

    let mut shared_draw_ref_index_by_token = renderer
        .read_last_virtual_geometry_indirect_submission_tokens()?
        .into_iter()
        .enumerate()
        .map(|(draw_ref_index, submission_token)| (submission_token, draw_ref_index as u32))
        .collect::<HashMap<_, _>>();
    if shared_draw_ref_index_by_token.is_empty() {
        shared_draw_ref_index_by_token = renderer
            .read_last_virtual_geometry_indirect_args_with_instances()?
            .into_iter()
            .enumerate()
            .map(
                |(draw_ref_index, (_first_index, _index_count, submission_token))| {
                    (submission_token, draw_ref_index as u32)
                },
            )
            .collect::<HashMap<_, _>>();
    }
    if shared_draw_ref_index_by_token.is_empty() {
        return Ok(None);
    }

    let draw_ref_indices = execution_tokens
        .into_iter()
        .filter_map(|submission_token| {
            shared_draw_ref_index_by_token
                .get(&submission_token)
                .copied()
        })
        .collect::<Vec<_>>();
    if draw_ref_indices.is_empty() {
        return Ok(None);
    }

    Ok(Some(draw_ref_indices))
}

#[cfg(test)]
fn read_execution_submission_tokens(renderer: &SceneRenderer) -> Result<Vec<u32>, GraphicsError> {
    let execution_submission_tokens =
        renderer.read_last_virtual_geometry_indirect_execution_submission_tokens()?;
    if !execution_submission_tokens.is_empty() {
        return Ok(execution_submission_tokens);
    }

    const INDIRECT_ARGS_WORD_COUNT: usize = 5;

    let Some(buffer) = renderer
        .last_virtual_geometry_indirect_execution_args_buffer
        .as_ref()
    else {
        return Ok(Vec::new());
    };
    if renderer.last_virtual_geometry_indirect_draw_count == 0 {
        return Ok(Vec::new());
    }

    let staging = renderer
        .backend
        .device
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-indirect-execution-args-readback"),
            size: (renderer.last_virtual_geometry_indirect_draw_count as u64)
                * (std::mem::size_of::<u32>() as u64)
                * INDIRECT_ARGS_WORD_COUNT as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    let mut encoder =
        renderer
            .backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-vg-indirect-execution-args-readback-encoder"),
            });
    encoder.copy_buffer_to_buffer(
        buffer,
        0,
        &staging,
        0,
        (renderer.last_virtual_geometry_indirect_draw_count as u64)
            * (std::mem::size_of::<u32>() as u64)
            * INDIRECT_ARGS_WORD_COUNT as u64,
    );
    renderer.backend.queue.submit([encoder.finish()]);

    Ok(read_buffer_u32s(
        &renderer.backend.device,
        &staging,
        (renderer.last_virtual_geometry_indirect_draw_count as usize) * INDIRECT_ARGS_WORD_COUNT,
    )?
    .chunks_exact(INDIRECT_ARGS_WORD_COUNT)
    .map(|chunk| chunk[4])
    .collect())
}
