use super::super::{
    append_rgb_as_rgba16f_texels, append_rgba16f_texels, source_cubemap_face_mip_offset,
    source_cubemap_mip_size, CubemapFace, SourceCubemapIrradianceCube, SourceCubemapMipChain,
};
use super::{SourceCubemapUploadArtifact, SourceCubemapUploadMip};

const RGBA16F_BYTES_PER_TEXEL: u32 = 8;
const UPLOAD_ROW_ALIGNMENT: u32 = 256;

pub fn build_source_cubemap_upload_artifact(
    mip_chain: &SourceCubemapMipChain,
    irradiance_cube: Option<&SourceCubemapIrradianceCube>,
) -> SourceCubemapUploadArtifact {
    SourceCubemapUploadArtifact::new(
        encode_cubemap_mips(
            mip_chain.source_face_size(),
            mip_chain.source_mip_count(),
            mip_chain.source_texels(),
        ),
        encode_cubemap_mips(
            mip_chain.pmrem_face_size(),
            mip_chain.pmrem_mip_count(),
            mip_chain.pmrem_texels(),
        ),
        irradiance_cube.map(encode_irradiance_mip),
    )
}

fn encode_cubemap_mips(
    face_size: u32,
    mip_count: u32,
    texels: &[[f32; 4]],
) -> Vec<SourceCubemapUploadMip> {
    let mut mips = Vec::with_capacity(mip_count as usize);
    for mip_level in 0..mip_count {
        let mip_face_size = source_cubemap_mip_size(face_size, mip_level);
        let texels_per_face = mip_face_size as usize * mip_face_size as usize;
        let bytes_per_row = padded_bytes_per_row(mip_face_size);
        let mut bytes = Vec::with_capacity(
            bytes_per_row as usize * mip_face_size as usize * CubemapFace::ALL.len(),
        );
        for face in CubemapFace::ALL {
            let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level);
            let face_texels = &texels[offset..offset + texels_per_face];
            for row in face_texels.chunks_exact(mip_face_size as usize) {
                append_rgba16f_texels(&mut bytes, row);
                bytes.resize(bytes.len() + bytes_per_row as usize - row.len() * 8, 0);
            }
        }
        mips.push(SourceCubemapUploadMip::new(
            mip_level,
            mip_face_size,
            bytes_per_row,
            bytes,
        ));
    }
    mips
}

fn encode_irradiance_mip(irradiance_cube: &SourceCubemapIrradianceCube) -> SourceCubemapUploadMip {
    let face_size = irradiance_cube.face_size();
    let bytes_per_row = padded_bytes_per_row(face_size);
    let mut bytes =
        Vec::with_capacity(bytes_per_row as usize * face_size as usize * CubemapFace::ALL.len());
    for face_texels in irradiance_cube
        .texels()
        .chunks_exact(face_size as usize * face_size as usize)
    {
        for row in face_texels.chunks_exact(face_size as usize) {
            append_rgb_as_rgba16f_texels(&mut bytes, row, 1.0);
            bytes.resize(bytes.len() + bytes_per_row as usize - row.len() * 8, 0);
        }
    }
    SourceCubemapUploadMip::new(0, face_size, bytes_per_row, bytes)
}

fn padded_bytes_per_row(face_size: u32) -> u32 {
    face_size
        .saturating_mul(RGBA16F_BYTES_PER_TEXEL)
        .max(1)
        .div_ceil(UPLOAD_ROW_ALIGNMENT)
        .saturating_mul(UPLOAD_ROW_ALIGNMENT)
}
