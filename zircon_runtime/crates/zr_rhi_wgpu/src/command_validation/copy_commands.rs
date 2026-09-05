use zr_rhi::{BufferUsage, CommandListCommand, RhiError, TextureUsage};

use super::super::device::DeterministicRhiContractDeviceState;
use super::super::render_pass_validation::ActiveRenderPass;
use super::super::resource_validation::{ensure_buffer_usage, ensure_texture_usage};
use super::super::texture_copy::{
    texture_copy_layout, texture_to_texture_copy_layouts, texture_upload_byte_len,
    validate_texture_copy_destination_aspect, validate_texture_copy_source_aspect,
};

/// Handles the transfer subset independently from the render/compute state
/// machine. Both validation and deterministic execution consume the same
/// region/aspect contract as the production WGPU encoder.
pub(super) fn validate(
    state: &DeterministicRhiContractDeviceState,
    command: &CommandListCommand,
    active_render_pass: &Option<ActiveRenderPass>,
    active_compute_pass: bool,
) -> Result<bool, RhiError> {
    match command {
        CommandListCommand::CopyBufferToBuffer {
            source,
            destination,
            source_offset,
            destination_offset,
            size,
        } => {
            ensure_no_active_pass(
                active_render_pass,
                active_compute_pass,
                "copy_buffer_to_buffer",
            )?;
            let source_buffer = state
                .buffers
                .get(source)
                .ok_or(RhiError::UnknownBuffer(source.diagnostic_id()))?;
            let destination_buffer = state
                .buffers
                .get(destination)
                .ok_or(RhiError::UnknownBuffer(destination.diagnostic_id()))?;
            ensure_buffer_usage(
                source.diagnostic_id(),
                &source_buffer.desc,
                BufferUsage::COPY_SRC,
            )?;
            ensure_buffer_usage(
                destination.diagnostic_id(),
                &destination_buffer.desc,
                BufferUsage::COPY_DST,
            )?;
            let source_end = source_offset.saturating_add(*size);
            let destination_end = destination_offset.saturating_add(*size);
            if source_end > source_buffer.desc.size_bytes
                || destination_end > destination_buffer.desc.size_bytes
            {
                return Err(RhiError::BufferCopyOutOfRange {
                    source_buffer: source.diagnostic_id(),
                    destination_buffer: destination.diagnostic_id(),
                    source_offset: *source_offset,
                    destination_offset: *destination_offset,
                    size: *size,
                });
            }
        }
        CommandListCommand::CopyBufferToTexture {
            source,
            destination,
            source_offset,
            bytes_per_row,
            region,
        } => {
            ensure_no_active_pass(
                active_render_pass,
                active_compute_pass,
                "copy_buffer_to_texture",
            )?;
            let source_buffer = state
                .buffers
                .get(source)
                .ok_or(RhiError::UnknownBuffer(source.diagnostic_id()))?;
            let destination_texture = state
                .textures
                .get(destination)
                .ok_or(RhiError::UnknownTexture(destination.diagnostic_id()))?;
            ensure_buffer_usage(
                source.diagnostic_id(),
                &source_buffer.desc,
                BufferUsage::COPY_SRC,
            )?;
            ensure_texture_usage(
                destination.diagnostic_id(),
                &destination_texture.desc,
                TextureUsage::COPY_DST,
            )?;
            validate_texture_copy_destination_aspect(
                *destination,
                &destination_texture.desc,
                *region,
            )?;
            let Some(layout) = texture_copy_layout(&destination_texture.desc, *region) else {
                return Err(RhiError::BufferToTextureCopyOutOfRange {
                    source_buffer: source.diagnostic_id(),
                    destination_texture: destination.diagnostic_id(),
                    source_offset: *source_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                });
            };
            let row_size = layout.copy_row_bytes;
            let Some(copy_size) = texture_upload_byte_len(*region, *bytes_per_row, row_size) else {
                return Err(RhiError::BufferToTextureCopyOutOfRange {
                    source_buffer: source.diagnostic_id(),
                    destination_texture: destination.diagnostic_id(),
                    source_offset: *source_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                });
            };
            if *bytes_per_row < row_size
                || source_offset.saturating_add(copy_size) > source_buffer.desc.size_bytes
                || layout.last_copy_end > destination_texture.contents.len() as u64
            {
                return Err(RhiError::BufferToTextureCopyOutOfRange {
                    source_buffer: source.diagnostic_id(),
                    destination_texture: destination.diagnostic_id(),
                    source_offset: *source_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                });
            }
        }
        CommandListCommand::CopyTextureToBuffer {
            source,
            destination,
            destination_offset,
            bytes_per_row,
            region,
        } => {
            ensure_no_active_pass(
                active_render_pass,
                active_compute_pass,
                "copy_texture_to_buffer",
            )?;
            let source_texture = state
                .textures
                .get(source)
                .ok_or(RhiError::UnknownTexture(source.diagnostic_id()))?;
            let destination_buffer = state
                .buffers
                .get(destination)
                .ok_or(RhiError::UnknownBuffer(destination.diagnostic_id()))?;
            ensure_texture_usage(
                source.diagnostic_id(),
                &source_texture.desc,
                TextureUsage::COPY_SRC,
            )?;
            ensure_buffer_usage(
                destination.diagnostic_id(),
                &destination_buffer.desc,
                BufferUsage::COPY_DST,
            )?;
            validate_texture_copy_source_aspect(*source, &source_texture.desc, *region)?;
            let Some(layout) = texture_copy_layout(&source_texture.desc, *region) else {
                return Err(RhiError::TextureToBufferCopyOutOfRange {
                    source_texture: source.diagnostic_id(),
                    destination_buffer: destination.diagnostic_id(),
                    destination_offset: *destination_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                });
            };
            let row_size = layout.copy_row_bytes;
            let Some(copy_size) = texture_upload_byte_len(*region, *bytes_per_row, row_size) else {
                return Err(RhiError::TextureToBufferCopyOutOfRange {
                    source_texture: source.diagnostic_id(),
                    destination_buffer: destination.diagnostic_id(),
                    destination_offset: *destination_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                });
            };
            if *bytes_per_row < row_size
                || destination_offset.saturating_add(copy_size) > destination_buffer.desc.size_bytes
                || layout.last_copy_end > source_texture.contents.len() as u64
            {
                return Err(RhiError::TextureToBufferCopyOutOfRange {
                    source_texture: source.diagnostic_id(),
                    destination_buffer: destination.diagnostic_id(),
                    destination_offset: *destination_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                });
            }
        }
        CommandListCommand::CopyTextureToTexture {
            source,
            destination,
            source_region,
            destination_region,
        } => {
            ensure_no_active_pass(
                active_render_pass,
                active_compute_pass,
                "copy_texture_to_texture",
            )?;
            let source_texture = state
                .textures
                .get(source)
                .ok_or(RhiError::UnknownTexture(source.diagnostic_id()))?;
            let destination_texture = state
                .textures
                .get(destination)
                .ok_or(RhiError::UnknownTexture(destination.diagnostic_id()))?;
            ensure_texture_usage(
                source.diagnostic_id(),
                &source_texture.desc,
                TextureUsage::COPY_SRC,
            )?;
            ensure_texture_usage(
                destination.diagnostic_id(),
                &destination_texture.desc,
                TextureUsage::COPY_DST,
            )?;
            texture_to_texture_copy_layouts(
                *source,
                &source_texture.desc,
                *destination,
                &destination_texture.desc,
                *source_region,
                *destination_region,
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn execute(
    state: &mut DeterministicRhiContractDeviceState,
    command: &CommandListCommand,
) -> Result<bool, RhiError> {
    match command {
        CommandListCommand::CopyBufferToBuffer {
            source,
            destination,
            source_offset,
            destination_offset,
            size,
        } => {
            let source_start = *source_offset as usize;
            let source_end = source_start + *size as usize;
            let destination_start = *destination_offset as usize;
            let destination_end = destination_start + *size as usize;
            if source == destination {
                state
                    .buffers
                    .get_mut(source)
                    .ok_or(RhiError::UnknownBuffer(source.diagnostic_id()))?
                    .contents
                    .copy_within(source_start..source_end, destination_start);
            } else {
                let [source_buffer, destination_buffer] =
                    state.buffers.get_disjoint_mut([source, destination]);
                let source_buffer =
                    source_buffer.ok_or(RhiError::UnknownBuffer(source.diagnostic_id()))?;
                let destination_buffer = destination_buffer
                    .ok_or(RhiError::UnknownBuffer(destination.diagnostic_id()))?;
                destination_buffer.contents[destination_start..destination_end]
                    .copy_from_slice(&source_buffer.contents[source_start..source_end]);
            }
        }
        CommandListCommand::CopyBufferToTexture {
            source,
            destination,
            source_offset,
            bytes_per_row,
            region,
        } => {
            let (buffers, textures) = (&state.buffers, &mut state.textures);
            let source_contents = &buffers
                .get(source)
                .ok_or(RhiError::UnknownBuffer(source.diagnostic_id()))?
                .contents;
            let destination_texture = textures
                .get_mut(destination)
                .ok_or(RhiError::UnknownTexture(destination.diagnostic_id()))?;
            let layout =
                texture_copy_layout(&destination_texture.desc, *region).ok_or_else(|| {
                    RhiError::BufferToTextureCopyOutOfRange {
                        source_buffer: source.diagnostic_id(),
                        destination_texture: destination.diagnostic_id(),
                        source_offset: *source_offset,
                        bytes_per_row: *bytes_per_row,
                        mip_level: region.mip_level,
                        origin_x: region.origin_x,
                        origin_y: region.origin_y,
                        origin_z: region.origin_z,
                        width: region.width,
                        height: region.height,
                        depth_or_array_layers: region.depth_or_array_layers,
                    }
                })?;
            let row_size = layout.copy_row_bytes as usize;
            let source_offset = *source_offset as usize;
            let bytes_per_row = *bytes_per_row as usize;
            for slice in 0..region.depth_or_array_layers as usize {
                for row in 0..region.height as usize {
                    let source_row = slice * region.height as usize + row;
                    let source_start = source_offset + source_row * bytes_per_row;
                    let source_end = source_start + row_size;
                    let destination_start = layout.offset as usize
                        + slice * layout.slice_stride as usize
                        + row * layout.row_stride as usize;
                    let destination_end = destination_start + row_size;
                    destination_texture.contents[destination_start..destination_end]
                        .copy_from_slice(&source_contents[source_start..source_end]);
                }
            }
        }
        CommandListCommand::CopyTextureToBuffer {
            source,
            destination,
            destination_offset,
            bytes_per_row,
            region,
        } => {
            let (textures, buffers) = (&state.textures, &mut state.buffers);
            let source_texture = textures
                .get(source)
                .ok_or(RhiError::UnknownTexture(source.diagnostic_id()))?;
            let layout = texture_copy_layout(&source_texture.desc, *region).ok_or_else(|| {
                RhiError::TextureToBufferCopyOutOfRange {
                    source_texture: source.diagnostic_id(),
                    destination_buffer: destination.diagnostic_id(),
                    destination_offset: *destination_offset,
                    bytes_per_row: *bytes_per_row,
                    mip_level: region.mip_level,
                    origin_x: region.origin_x,
                    origin_y: region.origin_y,
                    origin_z: region.origin_z,
                    width: region.width,
                    height: region.height,
                    depth_or_array_layers: region.depth_or_array_layers,
                }
            })?;
            let row_size = layout.copy_row_bytes as usize;
            let destination_offset = *destination_offset as usize;
            let bytes_per_row = *bytes_per_row as usize;
            let destination_buffer = buffers
                .get_mut(destination)
                .ok_or(RhiError::UnknownBuffer(destination.diagnostic_id()))?;
            for slice in 0..region.depth_or_array_layers as usize {
                for row in 0..region.height as usize {
                    let source_start = layout.offset as usize
                        + slice * layout.slice_stride as usize
                        + row * layout.row_stride as usize;
                    let source_end = source_start + row_size;
                    let destination_row = slice * region.height as usize + row;
                    let destination_start = destination_offset + destination_row * bytes_per_row;
                    let destination_end = destination_start + row_size;
                    destination_buffer.contents[destination_start..destination_end]
                        .copy_from_slice(&source_texture.contents[source_start..source_end]);
                }
            }
        }
        CommandListCommand::CopyTextureToTexture {
            source,
            destination,
            source_region,
            destination_region,
        } => {
            let [source_texture, destination_texture] =
                state.textures.get_disjoint_mut([source, destination]);
            let source_texture =
                source_texture.ok_or(RhiError::UnknownTexture(source.diagnostic_id()))?;
            let destination_texture =
                destination_texture.ok_or(RhiError::UnknownTexture(destination.diagnostic_id()))?;
            let (source_layout, destination_layout) = texture_to_texture_copy_layouts(
                *source,
                &source_texture.desc,
                *destination,
                &destination_texture.desc,
                *source_region,
                *destination_region,
            )?;
            let row_size = source_layout.copy_row_bytes as usize;
            for slice in 0..source_region.depth_or_array_layers as usize {
                for row in 0..source_region.height as usize {
                    let source_start = source_layout.offset as usize
                        + slice * source_layout.slice_stride as usize
                        + row * source_layout.row_stride as usize;
                    let source_end = source_start + row_size;
                    let destination_start = destination_layout.offset as usize
                        + slice * destination_layout.slice_stride as usize
                        + row * destination_layout.row_stride as usize;
                    let destination_end = destination_start + row_size;
                    destination_texture.contents[destination_start..destination_end]
                        .copy_from_slice(&source_texture.contents[source_start..source_end]);
                }
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn ensure_no_active_pass(
    active_render_pass: &Option<ActiveRenderPass>,
    active_compute_pass: bool,
    command: &str,
) -> Result<(), RhiError> {
    if active_render_pass.is_some() {
        Err(RhiError::InvalidRenderPass {
            reason: format!("{command} cannot be recorded inside an active render pass"),
        })
    } else if active_compute_pass {
        Err(RhiError::InvalidComputePass {
            reason: format!("{command} cannot be recorded inside an active compute pass"),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_copy_execution_does_not_clone_whole_source_resources() {
        let source = include_str!("copy_commands.rs");
        let source = source.split("mod tests {").next().unwrap();
        let compact = source.split_whitespace().collect::<String>();

        assert!(
            !compact.contains("contents[source_start..source_end].to_vec();"),
            "buffer-to-buffer execution must not allocate a temporary byte vector"
        );
        assert!(
            !compact.contains(".contents.clone();"),
            "buffer-to-texture execution must borrow source contents instead of cloning the whole buffer"
        );
        assert!(
            !compact.contains("letsource_texture=state.textures.get(source).ok_or(RhiError::UnknownTexture(source.diagnostic_id()))?.clone();"),
            "texture-to-buffer execution must borrow the source texture instead of cloning the whole resource"
        );
    }

    #[test]
    fn copy_command_validation_stays_in_its_child_owner() {
        let parent = include_str!("../command_validation.rs");
        let copy_commands = include_str!("copy_commands.rs");

        assert!(parent.contains("mod copy_commands;"));
        assert!(parent.contains("copy_commands::validate"));
        assert!(parent.contains("copy_commands::execute"));
        assert!(copy_commands.contains("pub(super) fn validate("));
        assert!(copy_commands.contains("pub(super) fn execute("));
        assert!(parent.lines().count() <= 800);
        assert!(copy_commands.lines().count() <= 800);
    }
}
