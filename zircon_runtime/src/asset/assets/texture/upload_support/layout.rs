use super::TextureUploadCompressionFamily;
#[derive(Clone, Copy)]
pub(super) struct CompressedFormatLayout {
    pub(super) compression: TextureUploadCompressionFamily,
    pub(super) block_width: u32,
    pub(super) block_height: u32,
    pub(super) block_depth: u32,
    pub(super) bytes_per_block: u32,
}

fn bc_layout(bytes_per_block: u32) -> CompressedFormatLayout {
    CompressedFormatLayout {
        compression: TextureUploadCompressionFamily::Bc,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
    }
}

fn etc2_layout(bytes_per_block: u32) -> CompressedFormatLayout {
    CompressedFormatLayout {
        compression: TextureUploadCompressionFamily::Etc2,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
    }
}

fn astc_layout(block_width: u32, block_height: u32) -> CompressedFormatLayout {
    CompressedFormatLayout {
        compression: TextureUploadCompressionFamily::Astc,
        block_width,
        block_height,
        block_depth: 1,
        bytes_per_block: 16,
    }
}

pub(super) fn ktx_gl_compressed_layout(gl_internal_format: u32) -> Option<CompressedFormatLayout> {
    match gl_internal_format {
        // S3TC / BC1, BC2, BC3 including sRGB variants.
        0x83f0 | 0x83f1 | 0x8c4c | 0x8c4d => Some(bc_layout(8)),
        0x83f2 | 0x83f3 | 0x8c4e | 0x8c4f => Some(bc_layout(16)),
        // RGTC / BPTC forms of BC4, BC5, BC6H, and BC7.
        0x8dbb | 0x8dbc => Some(bc_layout(8)),
        0x8dbd | 0x8dbe | 0x8e8c | 0x8e8d | 0x8e8e | 0x8e8f => Some(bc_layout(16)),
        // ETC2 / EAC formats accepted by wgpu's ETC2 family.
        0x9274 | 0x9275 | 0x9276 | 0x9277 | 0x9270 | 0x9271 => Some(etc2_layout(8)),
        0x9278 | 0x9279 | 0x9272 | 0x9273 => Some(etc2_layout(16)),
        0x93b0..=0x93bd | 0x93d0..=0x93dd => {
            let (block_width, block_height) = ktx_gl_astc_block(gl_internal_format)?;
            Some(astc_layout(block_width, block_height))
        }
        _ => None,
    }
}

fn ktx_gl_astc_block(gl_internal_format: u32) -> Option<(u32, u32)> {
    let index = if (0x93b0..=0x93bd).contains(&gl_internal_format) {
        gl_internal_format - 0x93b0
    } else if (0x93d0..=0x93dd).contains(&gl_internal_format) {
        gl_internal_format - 0x93d0
    } else {
        return None;
    };
    astc_2d_block_by_index(index)
}

pub(super) fn ktx2_vk_compressed_layout(vk_format: u32) -> Option<CompressedFormatLayout> {
    match vk_format {
        // VK_FORMAT_BC1_*_BLOCK
        131..=134 => Some(bc_layout(8)),
        // VK_FORMAT_BC2/BC3_*_BLOCK
        135..=138 => Some(bc_layout(16)),
        // VK_FORMAT_BC4_*_BLOCK
        139 | 140 => Some(bc_layout(8)),
        // VK_FORMAT_BC5/BC6H/BC7_*_BLOCK
        141..=146 => Some(bc_layout(16)),
        // VK_FORMAT_ETC2_* and VK_FORMAT_EAC_*_BLOCK
        147..=150 | 153 | 154 => Some(etc2_layout(8)),
        151 | 152 | 155 | 156 => Some(etc2_layout(16)),
        157..=184 => {
            let (block_width, block_height) = ktx2_astc_block(vk_format)?;
            Some(astc_layout(block_width, block_height))
        }
        _ => None,
    }
}

fn ktx2_astc_block(vk_format: u32) -> Option<(u32, u32)> {
    if !(157..=184).contains(&vk_format) {
        return None;
    }
    astc_2d_block_by_index((vk_format - 157) / 2)
}

fn astc_2d_block_by_index(index: u32) -> Option<(u32, u32)> {
    Some(match index {
        0 => (4, 4),
        1 => (5, 4),
        2 => (5, 5),
        3 => (6, 5),
        4 => (6, 6),
        5 => (8, 5),
        6 => (8, 6),
        7 => (8, 8),
        8 => (10, 5),
        9 => (10, 6),
        10 => (10, 8),
        11 => (10, 10),
        12 => (12, 10),
        13 => (12, 12),
        _ => return None,
    })
}

pub(super) fn is_supported_astc_block(width: u32, height: u32, depth: u32) -> bool {
    if depth == 1 {
        return matches!(
            (width, height),
            (4, 4)
                | (5, 4)
                | (5, 5)
                | (6, 5)
                | (6, 6)
                | (8, 5)
                | (8, 6)
                | (8, 8)
                | (10, 5)
                | (10, 6)
                | (10, 8)
                | (10, 10)
                | (12, 10)
                | (12, 12)
        );
    }

    // ASTC 3D block dimensions are codec-defined, not runtime policy.
    matches!(
        (width, height, depth),
        (3, 3, 3)
            | (4, 3, 3)
            | (4, 4, 3)
            | (4, 4, 4)
            | (5, 4, 4)
            | (5, 5, 4)
            | (5, 5, 5)
            | (6, 5, 5)
            | (6, 6, 5)
            | (6, 6, 6)
    )
}
