use crate::core::framework::render::RenderImageDimension;

use super::super::TextureAsset;
use super::{
    div_ceil, ready, unsupported, TextureUploadCompressionFamily, TextureUploadPlan,
    TextureUploadReadiness, TextureUploadSupport,
};
pub(super) fn compressed_plan_readiness(
    texture: &TextureAsset,
    bytes: &[u8],
    plan: TextureUploadPlan,
    support: TextureUploadSupport,
) -> TextureUploadReadiness {
    if let Some(reason) = unsupported_container_shape_reason(texture) {
        return unsupported(reason);
    }
    if let Some(reason) = unsupported_feature_reason(texture, &plan, support) {
        return unsupported(reason);
    }
    if bytes.len() <= plan.data_offset {
        return unsupported(format!(
            "container texture payload format {} has no image data after {} byte header",
            plan.format, plan.data_offset
        ));
    }
    let Some(required_bytes) = compressed_mip0_required_len(texture, &plan) else {
        return unsupported(format!(
            "container texture payload format {} upload size overflows",
            plan.format
        ));
    };
    let available_bytes = bytes.len() - plan.data_offset;
    if let Some(data_length) = plan.data_length {
        if available_bytes < data_length {
            return unsupported(format!(
                "container texture payload format {} declares {} image bytes but only {} are available",
                plan.format, data_length, available_bytes
            ));
        }
        if data_length < required_bytes {
            return unsupported(format!(
                "container texture payload format {} declares {} image bytes but needs at least {}",
                plan.format, data_length, required_bytes
            ));
        }
    }
    if available_bytes < required_bytes {
        return unsupported(format!(
            "container texture payload format {} has {} image bytes but needs at least {}",
            plan.format, available_bytes, required_bytes
        ));
    }
    ready(plan)
}

fn compressed_mip0_required_len(texture: &TextureAsset, plan: &TextureUploadPlan) -> Option<usize> {
    let descriptor = texture.render_image_descriptor();
    let block_columns = div_ceil(texture.width.max(1), plan.block_width.max(1));
    let block_rows = div_ceil(texture.height.max(1), plan.block_height.max(1));
    let layer_count = descriptor.depth_or_array_layers.max(1);
    let bytes = block_columns
        .checked_mul(block_rows)?
        .checked_mul(layer_count)?
        .checked_mul(plan.bytes_per_block)?;
    usize::try_from(bytes).ok()
}

fn unsupported_container_shape_reason(texture: &TextureAsset) -> Option<String> {
    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension == RenderImageDimension::D1 {
        return Some("compressed texture 1d upload is not implemented".to_string());
    }
    if descriptor.mip_count > 1 {
        return Some("compressed texture mip-chain upload is not implemented".to_string());
    }
    if descriptor.dimension == RenderImageDimension::D2
        && (descriptor.array_layer_count > 1 || descriptor.depth_or_array_layers > 1)
    {
        return Some("compressed texture array/cubemap upload is not implemented".to_string());
    }
    None
}

fn unsupported_feature_reason(
    texture: &TextureAsset,
    plan: &TextureUploadPlan,
    support: TextureUploadSupport,
) -> Option<String> {
    match plan.compression {
        TextureUploadCompressionFamily::Uncompressed => None,
        TextureUploadCompressionFamily::Bc if !support.bc => {
            Some("gpu device does not support BC compressed textures".to_string())
        }
        TextureUploadCompressionFamily::Bc
            if texture.render_image_descriptor().dimension == RenderImageDimension::D3
                && !support.bc_sliced_3d =>
        {
            Some("gpu device does not support BC sliced 3d textures".to_string())
        }
        TextureUploadCompressionFamily::Bc => None,
        TextureUploadCompressionFamily::Etc2 if !support.etc2 => {
            Some("gpu device does not support ETC2 compressed textures".to_string())
        }
        TextureUploadCompressionFamily::Etc2
            if texture.render_image_descriptor().dimension == RenderImageDimension::D3 =>
        {
            Some("compressed texture ETC2 3d upload is not implemented".to_string())
        }
        TextureUploadCompressionFamily::Etc2 => None,
        TextureUploadCompressionFamily::Astc if !support.astc_ldr => {
            Some("gpu device does not support ASTC compressed textures".to_string())
        }
        TextureUploadCompressionFamily::Astc
            if (plan.block_depth > 1
                || texture.render_image_descriptor().dimension == RenderImageDimension::D3)
                && !support.astc_sliced_3d =>
        {
            Some("gpu device does not support ASTC sliced 3d textures".to_string())
        }
        TextureUploadCompressionFamily::Astc if plan.block_depth > 1 => {
            Some("astc 3d block payload upload is not implemented".to_string())
        }
        TextureUploadCompressionFamily::Astc => None,
    }
}
