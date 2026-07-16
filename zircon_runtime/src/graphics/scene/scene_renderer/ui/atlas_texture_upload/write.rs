use crate::text::atlas::GlyphAtlasUploadCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GlyphAtlasTextureUploadWrite {
    pub(super) origin_x: u32,
    pub(super) origin_y: u32,
    pub(super) origin_layer: u32,
    pub(super) source_offset: u64,
    pub(super) bytes_per_row: u32,
    pub(super) rows_per_image: u32,
    pub(super) extent_width: u32,
    pub(super) extent_height: u32,
    pub(super) extent_layers: u32,
}

pub(super) fn glyph_atlas_texture_upload_write(
    command: GlyphAtlasUploadCommand,
) -> GlyphAtlasTextureUploadWrite {
    GlyphAtlasTextureUploadWrite {
        origin_x: command.rect.x,
        origin_y: command.rect.y,
        origin_layer: command.page_key.page_index,
        source_offset: command.source_offset,
        bytes_per_row: command.bytes_per_row,
        rows_per_image: command.rows_per_image,
        extent_width: command.rect.width,
        extent_height: command.rect.height,
        extent_layers: 1,
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn write_glyph_atlas_texture_upload_command(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    bytes: &[u8],
    command: GlyphAtlasUploadCommand,
) {
    let write = glyph_atlas_texture_upload_write(command);
    write_glyph_atlas_texture_upload_bytes(queue, texture, bytes, write);
}

pub(super) fn write_glyph_atlas_texture_upload_bytes(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    bytes: &[u8],
    write: GlyphAtlasTextureUploadWrite,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: write.origin_x,
                y: write.origin_y,
                z: write.origin_layer,
            },
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: write.source_offset,
            bytes_per_row: Some(write.bytes_per_row),
            rows_per_image: Some(write.rows_per_image),
        },
        wgpu::Extent3d {
            width: write.extent_width,
            height: write.extent_height,
            depth_or_array_layers: write.extent_layers,
        },
    );
}
