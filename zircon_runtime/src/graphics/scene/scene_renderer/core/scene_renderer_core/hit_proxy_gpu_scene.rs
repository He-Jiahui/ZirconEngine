use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::scene_renderer::mesh::skinning::{
    create_empty_skinned_joint_palette_arena_buffer, skinned_joint_palette_arena_min_binding_size,
};
use crate::graphics::scene::scene_renderer::mesh::{
    HIT_PROXY_TOKEN_FORMAT, HIT_PROXY_WORLD_NORMAL_FORMAT, HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT,
};

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneHitProxyResources {
    gpu_scene: Option<GpuScene>,
    targets: Option<SceneHitProxyTargets>,
    next_readback_frame_index: u64,
}

pub(in crate::graphics::scene::scene_renderer::core) struct SceneHitProxyTargets {
    pub(in crate::graphics::scene::scene_renderer::core) token: wgpu::Texture,
    pub(in crate::graphics::scene::scene_renderer::core) token_view: wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::core) world_position_depth: wgpu::Texture,
    pub(in crate::graphics::scene::scene_renderer::core) world_position_depth_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::core) world_normal: wgpu::Texture,
    pub(in crate::graphics::scene::scene_renderer::core) world_normal_view: wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::core) _depth: wgpu::Texture,
    pub(in crate::graphics::scene::scene_renderer::core) depth_view: wgpu::TextureView,
}

impl SceneHitProxyResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn allocate_readback_frame_index(
        &mut self,
    ) -> Option<u64> {
        let frame_index = self.next_readback_frame_index;
        self.next_readback_frame_index = frame_index.checked_add(1)?;
        Some(frame_index)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn parts(
        &mut self,
        device: &wgpu::Device,
    ) -> (&mut GpuScene, &SceneHitProxyTargets) {
        let gpu_scene = self.gpu_scene.get_or_insert_with(|| {
            GpuScene::new(
                device,
                create_empty_skinned_joint_palette_arena_buffer(device),
                skinned_joint_palette_arena_min_binding_size(),
            )
        });
        let targets = self
            .targets
            .get_or_insert_with(|| SceneHitProxyTargets::new(device));
        (gpu_scene, targets)
    }
}

impl SceneHitProxyTargets {
    fn new(device: &wgpu::Device) -> Self {
        let (token, token_view) = create_target(
            device,
            "zircon-hit-proxy-token",
            HIT_PROXY_TOKEN_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let (world_position_depth, world_position_depth_view) = create_target(
            device,
            "zircon-hit-proxy-world-position-depth",
            HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let (world_normal, world_normal_view) = create_target(
            device,
            "zircon-hit-proxy-world-normal",
            HIT_PROXY_WORLD_NORMAL_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let (depth, depth_view) = create_target(
            device,
            "zircon-hit-proxy-depth",
            super::super::DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        Self {
            token,
            token_view,
            world_position_depth,
            world_position_depth_view,
            world_normal,
            world_normal_view,
            _depth: depth,
            depth_view,
        }
    }
}

fn create_target(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(test)]
mod tests {
    #[test]
    fn hit_proxy_gpu_scene_is_lazy_and_does_not_tax_normal_viewports() {
        let source = include_str!("hit_proxy_gpu_scene.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("hit-proxy GPU scene test boundary");

        assert!(source.contains("gpu_scene: Option<GpuScene>"));
        assert!(source.contains("targets: Option<SceneHitProxyTargets>"));
        assert!(source.contains("frame_index.checked_add(1)?"));
        assert!(source.contains("get_or_insert_with"));
        assert!(source.contains("width: 1"));
        assert!(source.contains("height: 1"));
        assert!(!source.contains("impl Default for GpuScene"));
    }
}
