use std::collections::HashMap;

use zr_rhi::{
    RenderResourceHandleAllocator, RhiError, TextureHandle, TextureViewDesc, TextureViewHandle,
};

use crate::texture_view::validate_texture_view_desc;

use super::DeterministicRhiContractDeviceState;

impl DeterministicRhiContractDeviceState {
    pub(super) fn destroy_texture(
        &mut self,
        handles: &RenderResourceHandleAllocator,
        handle: TextureHandle,
    ) -> Result<(), RhiError> {
        if self.surface_owned_textures.contains(&handle) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: handle.diagnostic_id(),
            });
        }
        if !self.textures.contains_key(&handle) {
            return Err(RhiError::UnknownTexture(handle.diagnostic_id()));
        }
        let live_views = self.texture_view_counts.get(&handle).copied().unwrap_or(0);
        if live_views != 0 {
            return Err(RhiError::TextureHasLiveViews {
                texture: handle.diagnostic_id(),
                live_views,
            });
        }
        self.textures.remove(&handle);
        handles.release_texture(handle)?;
        Ok(())
    }

    pub(super) fn create_texture_view(
        &mut self,
        handles: &RenderResourceHandleAllocator,
        desc: &TextureViewDesc,
    ) -> Result<TextureViewHandle, RhiError> {
        if self.surface_owned_textures.contains(&desc.texture) {
            return Err(RhiError::SurfaceOwnedTexture {
                texture: desc.texture.diagnostic_id(),
            });
        }
        let texture = self
            .textures
            .get(&desc.texture)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownTexture(desc.texture.diagnostic_id()))?;
        validate_texture_view_desc(&texture, desc)?;
        let handle = handles.allocate_texture_view()?;
        self.texture_views.insert(handle, desc.clone());
        let count = self.texture_view_counts.entry(desc.texture).or_insert(0);
        *count = count.saturating_add(1);
        Ok(handle)
    }

    pub(super) fn texture_view_desc(
        &self,
        handle: TextureViewHandle,
    ) -> Result<TextureViewDesc, RhiError> {
        self.texture_views
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))
    }

    pub(super) fn destroy_texture_view(
        &mut self,
        handles: &RenderResourceHandleAllocator,
        handle: TextureViewHandle,
    ) -> Result<(), RhiError> {
        if self.surface_owned_texture_views.contains(&handle) {
            return Err(RhiError::SurfaceOwnedTextureView {
                view: handle.diagnostic_id(),
            });
        }
        let Some(view) = self.texture_views.remove(&handle) else {
            return Err(RhiError::UnknownTextureView(handle.diagnostic_id()));
        };
        decrement_texture_view_count(&mut self.texture_view_counts, view.texture);
        handles.release_texture_view(handle)?;
        Ok(())
    }
}

fn decrement_texture_view_count(
    texture_view_counts: &mut HashMap<TextureHandle, u32>,
    texture: TextureHandle,
) {
    let Some(count) = texture_view_counts.get_mut(&texture) else {
        return;
    };
    if *count <= 1 {
        texture_view_counts.remove(&texture);
    } else {
        *count -= 1;
    }
}
