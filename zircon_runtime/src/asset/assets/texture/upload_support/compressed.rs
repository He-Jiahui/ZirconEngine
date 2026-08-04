use crate::core::framework::render::RenderImageDimension;

use super::super::TextureAsset;
use super::{
    TextureUploadCompressionFamily, TextureUploadPlan, TextureUploadReadiness,
    TextureUploadSupport, div_ceil, ready, unsupported,
};
pub(super) fn compressed_plan_readiness(
    texture: &TextureAsset,
    bytes: &[u8],
    plan: TextureUploadPlan,
    support: TextureUploadSupport,
) -> TextureUploadReadiness {
    if let Some(reason) = unsupported_container_shape_reason(texture, &plan) {
        return unsupported(reason);
    }
    if let Some(reason) = unsupported_feature_reason(texture, &plan, support) {
        return unsupported(reason);
    }
    if !plan.subresources.is_empty() {
        if let Some(reason) = compressed_subresource_reason(texture, bytes, &plan) {
            return unsupported(reason);
        }
        return ready(plan);
    }
    if bytes.len() <= plan.data_offset {
        return unsupported(format!(
            "container texture payload format {} has no image data after {} byte header",
            plan.format, plan.data_offset
        ));
    }
    let Some(required_bytes) = compressed_required_len(texture, &plan) else {
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

fn compressed_required_len(texture: &TextureAsset, plan: &TextureUploadPlan) -> Option<usize> {
    let descriptor = texture.render_image_descriptor();
    let layer_count = descriptor.depth_or_array_layers.max(1);
    let uploaded_mip_count = 1;
    let mut total = 0_u64;
    for mip_level in 0..uploaded_mip_count {
        let block_columns = div_ceil(
            mip_extent(texture.width.max(1), mip_level),
            plan.block_width.max(1),
        );
        let block_rows = div_ceil(
            mip_extent(texture.height.max(1), mip_level),
            plan.block_height.max(1),
        );
        let level_bytes = u64::from(block_columns)
            .checked_mul(u64::from(block_rows))?
            .checked_mul(u64::from(layer_count))?
            .checked_mul(u64::from(plan.bytes_per_block))?;
        total = total.checked_add(level_bytes)?;
    }
    usize::try_from(total).ok()
}

fn unsupported_container_shape_reason(
    texture: &TextureAsset,
    plan: &TextureUploadPlan,
) -> Option<String> {
    let descriptor = texture.render_image_descriptor();
    let has_subresource_layout = !plan.subresources.is_empty();
    if descriptor.dimension == RenderImageDimension::D1 {
        return Some("compressed texture 1d upload is not implemented".to_string());
    }
    if descriptor.mip_count > 1 && !has_subresource_layout {
        return Some("compressed texture mip-chain upload is not implemented".to_string());
    }
    if descriptor.dimension == RenderImageDimension::D2
        && (descriptor.array_layer_count > 1 || descriptor.depth_or_array_layers > 1)
        && !has_subresource_layout
    {
        return Some("compressed texture array/cubemap upload is not implemented".to_string());
    }
    if descriptor.dimension == RenderImageDimension::Cube {
        if texture.width != texture.height {
            return Some("compressed cube texture upload requires square faces".to_string());
        }
        if descriptor.array_layer_count == 0
            || descriptor.depth_or_array_layers != descriptor.array_layer_count
            || descriptor.array_layer_count % 6 != 0
        {
            return Some(
                "compressed cube texture upload requires a non-zero multiple of six faces"
                    .to_string(),
            );
        }
    }
    None
}

fn compressed_subresource_reason(
    texture: &TextureAsset,
    bytes: &[u8],
    plan: &TextureUploadPlan,
) -> Option<String> {
    let descriptor = texture.render_image_descriptor();
    let mip_count = descriptor.mip_count.max(1);
    let layer_count = descriptor.depth_or_array_layers.max(1);
    let Some(expected_count) = mip_count
        .checked_mul(layer_count)
        .and_then(|count| usize::try_from(count).ok())
    else {
        return Some(format!(
            "container texture payload format {} compressed subresource count overflows",
            plan.format
        ));
    };
    if plan.subresources.len() != expected_count {
        return Some(format!(
            "container texture payload format {} declares {} compressed subresources but needs {}",
            plan.format,
            plan.subresources.len(),
            expected_count
        ));
    }
    let mut seen = vec![false; expected_count];
    for subresource in &plan.subresources {
        if subresource.mip_level >= mip_count || subresource.array_layer >= layer_count {
            return Some(format!(
                "container texture payload format {} has an out-of-range compressed subresource",
                plan.format
            ));
        }
        let Some(slot) = usize::try_from(subresource.mip_level)
            .ok()
            .and_then(|mip_level| {
                usize::try_from(layer_count)
                    .ok()
                    .and_then(|layer_count| mip_level.checked_mul(layer_count))
            })
            .and_then(|base| {
                usize::try_from(subresource.array_layer)
                    .ok()
                    .and_then(|array_layer| base.checked_add(array_layer))
            })
        else {
            return Some(format!(
                "container texture payload format {} compressed subresource index overflows",
                plan.format
            ));
        };
        let Some(seen_slot) = seen.get_mut(slot) else {
            return Some(format!(
                "container texture payload format {} compressed subresource index is invalid",
                plan.format
            ));
        };
        if *seen_slot {
            return Some(format!(
                "container texture payload format {} duplicates a compressed subresource",
                plan.format
            ));
        }
        let Some(expected) =
            compressed_subresource_data_length(texture, plan, subresource.mip_level)
        else {
            return Some(format!(
                "container texture payload format {} compressed subresource size overflows",
                plan.format
            ));
        };
        if subresource.data_length != expected.data_length
            || subresource.bytes_per_row != expected.bytes_per_row
            || subresource.block_rows != expected.block_rows
        {
            return Some(format!(
                "container texture payload format {} has an invalid compressed subresource layout",
                plan.format
            ));
        }
        let Some(data_end) = subresource.data_offset.checked_add(subresource.data_length) else {
            return Some(format!(
                "container texture payload format {} compressed subresource range overflows",
                plan.format
            ));
        };
        if bytes.get(subresource.data_offset..data_end).is_none() {
            return Some(format!(
                "container texture payload format {} is missing compressed subresource bytes",
                plan.format
            ));
        }
        *seen_slot = true;
    }
    if seen.into_iter().all(|present| present) {
        None
    } else {
        Some(format!(
            "container texture payload format {} omits a compressed subresource",
            plan.format
        ))
    }
}

#[derive(Clone, Copy)]
struct CompressedSubresourceLayout {
    data_length: usize,
    bytes_per_row: u32,
    block_rows: u32,
}

fn compressed_subresource_data_length(
    texture: &TextureAsset,
    plan: &TextureUploadPlan,
    mip_level: u32,
) -> Option<CompressedSubresourceLayout> {
    let block_columns = div_ceil(
        mip_extent(texture.width.max(1), mip_level),
        plan.block_width.max(1),
    );
    let block_rows = div_ceil(
        mip_extent(texture.height.max(1), mip_level),
        plan.block_height.max(1),
    );
    let bytes_per_row = block_columns.checked_mul(plan.bytes_per_block)?;
    let data_length =
        usize::try_from(u64::from(bytes_per_row).checked_mul(u64::from(block_rows))?).ok()?;
    Some(CompressedSubresourceLayout {
        data_length,
        bytes_per_row,
        block_rows,
    })
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

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 { 1 } else { shifted }
}
