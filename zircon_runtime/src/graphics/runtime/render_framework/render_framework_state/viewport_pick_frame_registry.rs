use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::core::framework::render::{
    RenderFrameExtract, RenderViewportHandle, RenderViewportPickRequest, ShaderQualityTier,
};
use crate::core::math::UVec2;
use crate::graphics::scene::MeshHitProxyTokenSource;
use crate::graphics::{ViewportRenderFrame, ViewportRenderRegion};

use super::{ViewportHitProxyIdentity, ViewportHitProxyTable};

const MAX_RETAINED_VIEWPORT_PICK_FRAME_GENERATIONS: usize = 3;

#[derive(Clone, Debug)]
pub(in crate::graphics::runtime::render_framework) struct ViewportPickFrameSnapshot {
    viewport: RenderViewportHandle,
    viewport_size: UVec2,
    generation: u64,
    source_extract: Arc<RenderFrameExtract>,
    visible_stable_instance_keys: Arc<[u64]>,
    hit_proxies: ViewportHitProxyTable,
    render_region: ViewportRenderRegion,
    shader_quality: ShaderQualityTier,
    texture_mip_bias: u8,
    texture_max_anisotropy: u8,
    virtual_geometry_enabled: bool,
}

impl ViewportPickFrameSnapshot {
    pub(in crate::graphics::runtime::render_framework) fn from_rendered_frame(
        viewport: RenderViewportHandle,
        generation: u64,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
    ) -> Self {
        let visible_stable_instance_keys = visible_stable_instance_keys(frame);
        let hit_proxies =
            ViewportHitProxyTable::from_rendered_frame(frame, &visible_stable_instance_keys);
        Self {
            viewport,
            viewport_size: frame.viewport_size,
            generation,
            source_extract: Arc::clone(&frame.extract),
            visible_stable_instance_keys: visible_stable_instance_keys.into(),
            hit_proxies,
            render_region: frame.render_region(),
            shader_quality: frame.shader_quality(),
            texture_mip_bias: frame.texture_mip_bias(),
            texture_max_anisotropy: frame.texture_max_anisotropy(),
            virtual_geometry_enabled,
        }
    }

    pub(in crate::graphics::runtime::render_framework) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(in crate::graphics::runtime::render_framework) fn world_generation(&self) -> u64 {
        self.source_extract.world.generation()
    }

    pub(in crate::graphics::runtime::render_framework) fn source_extract(
        &self,
    ) -> &Arc<RenderFrameExtract> {
        &self.source_extract
    }

    pub(in crate::graphics::runtime::render_framework) fn render_frame(
        &self,
    ) -> ViewportRenderFrame {
        let mut frame = ViewportRenderFrame::from_shared_extract(
            Arc::clone(&self.source_extract),
            self.viewport_size,
        )
        .with_shader_quality(self.shader_quality)
        .with_texture_mip_bias(self.texture_mip_bias)
        .with_texture_max_anisotropy(self.texture_max_anisotropy);
        frame.render_region = self.render_region;
        frame
    }

    pub(in crate::graphics::runtime::render_framework) fn visible_stable_instance_keys(
        &self,
    ) -> &[u64] {
        &self.visible_stable_instance_keys
    }

    pub(in crate::graphics::runtime::render_framework) fn hit_proxy_token_for_instance(
        &self,
        stable_instance_key: u64,
    ) -> Option<u32> {
        self.hit_proxies.token_for_instance(stable_instance_key)
    }

    pub(in crate::graphics::runtime::render_framework) fn resolve_hit_proxy_token(
        &self,
        token: u32,
    ) -> Option<ViewportHitProxyIdentity> {
        self.hit_proxies.resolve(token)
    }

    pub(in crate::graphics::runtime::render_framework) const fn render_region(
        &self,
    ) -> ViewportRenderRegion {
        self.render_region
    }

    pub(in crate::graphics::runtime::render_framework) const fn shader_quality(
        &self,
    ) -> ShaderQualityTier {
        self.shader_quality
    }

    pub(in crate::graphics::runtime::render_framework) const fn texture_mip_bias(&self) -> u8 {
        self.texture_mip_bias
    }

    pub(in crate::graphics::runtime::render_framework) const fn texture_max_anisotropy(
        &self,
    ) -> u8 {
        self.texture_max_anisotropy
    }

    pub(in crate::graphics::runtime::render_framework) const fn virtual_geometry_enabled(
        &self,
    ) -> bool {
        self.virtual_geometry_enabled
    }

    pub(in crate::graphics::runtime::render_framework) fn matches_request(
        &self,
        request: RenderViewportPickRequest,
    ) -> bool {
        request.is_valid()
            && self.viewport == request.viewport
            && self.viewport_size == request.viewport_size
            && self.generation == request.frame_generation
    }
}

impl MeshHitProxyTokenSource for ViewportPickFrameSnapshot {
    fn token_for_instance(&self, stable_instance_key: u64) -> Option<u32> {
        self.hit_proxy_token_for_instance(stable_instance_key)
    }
}

#[derive(Default)]
pub(in crate::graphics::runtime::render_framework) struct ViewportPickFrameRegistry {
    by_viewport: HashMap<RenderViewportHandle, VecDeque<Arc<ViewportPickFrameSnapshot>>>,
}

impl ViewportPickFrameRegistry {
    pub(in crate::graphics::runtime::render_framework) fn publish(
        &mut self,
        snapshot: ViewportPickFrameSnapshot,
    ) {
        let frames = self.by_viewport.entry(snapshot.viewport).or_default();
        frames.retain(|frame| frame.generation != snapshot.generation);
        frames.push_back(Arc::new(snapshot));
        while frames.len() > MAX_RETAINED_VIEWPORT_PICK_FRAME_GENERATIONS {
            frames.pop_front();
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn resolve(
        &self,
        viewport: RenderViewportHandle,
        generation: u64,
    ) -> Option<Arc<ViewportPickFrameSnapshot>> {
        self.by_viewport
            .get(&viewport)?
            .iter()
            .find(|frame| frame.generation == generation)
            .cloned()
    }

    pub(in crate::graphics::runtime::render_framework) fn remove(
        &mut self,
        viewport: RenderViewportHandle,
    ) {
        self.by_viewport.remove(&viewport);
    }
}

fn visible_stable_instance_keys(frame: &ViewportRenderFrame) -> Vec<u64> {
    let Some(visibility) = frame.frame_visibility() else {
        return frame
            .meshes()
            .iter()
            .map(|mesh| mesh.stable_instance_key)
            .collect();
    };
    let Some(main_view) = visibility.main_view() else {
        return Vec::new();
    };
    main_view
        .visible
        .iter()
        .filter_map(|index| visibility.stable_instance_keys.get(*index as usize))
        .copied()
        .collect()
}
