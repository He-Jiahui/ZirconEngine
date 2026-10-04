use std::collections::VecDeque;

use crate::graphics::resource_identity::SampledTextureIdentity;

const MAX_TAA_RESOLVE_BIND_GROUPS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TaaResolveBindGroupKey {
    scene_color: SampledTextureIdentity,
    scene_depth: SampledTextureIdentity,
    scene_velocity: SampledTextureIdentity,
    history_previous: SampledTextureIdentity,
    reactive_mask: SampledTextureIdentity,
}

impl TaaResolveBindGroupKey {
    pub(crate) const fn new(
        scene_color: SampledTextureIdentity,
        scene_depth: SampledTextureIdentity,
        scene_velocity: SampledTextureIdentity,
        history_previous: SampledTextureIdentity,
        reactive_mask: SampledTextureIdentity,
    ) -> Self {
        Self {
            scene_color,
            scene_depth,
            scene_velocity,
            history_previous,
            reactive_mask,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TaaResolveFrameTargetKey {
    scene_color: SampledTextureIdentity,
    scene_depth: SampledTextureIdentity,
    scene_velocity: SampledTextureIdentity,
}

impl From<TaaResolveBindGroupKey> for TaaResolveFrameTargetKey {
    fn from(key: TaaResolveBindGroupKey) -> Self {
        Self {
            scene_color: key.scene_color,
            scene_depth: key.scene_depth,
            scene_velocity: key.scene_velocity,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TaaResolveHistoryPairKey {
    first: SampledTextureIdentity,
    second: SampledTextureIdentity,
}

impl TaaResolveHistoryPairKey {
    const fn new(first: SampledTextureIdentity, second: SampledTextureIdentity) -> Self {
        if first.get() <= second.get() {
            Self { first, second }
        } else {
            Self {
                first: second,
                second: first,
            }
        }
    }
}

struct TaaResolveBindGroupEntry {
    key: TaaResolveBindGroupKey,
    bind_group: wgpu::BindGroup,
}

#[derive(Default)]
/// TAA 绑定组按五个采样身份缓存；场景目标或双缓冲历史身份变化会清空缓存，历史 pair 忽略前后顺序以适配 ping-pong，局部 MRU 队列上限为 8。
pub(crate) struct TaaResolveBindGroupCache {
    entries: VecDeque<TaaResolveBindGroupEntry>,
    frame_target: Option<TaaResolveFrameTargetKey>,
    history_pair: Option<TaaResolveHistoryPairKey>,
}

pub(crate) struct PreparedTaaResolveBindGroup {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) created: bool,
}

impl TaaResolveBindGroupCache {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.frame_target = None;
        self.history_pair = None;
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        key: TaaResolveBindGroupKey,
        history_current_identity: SampledTextureIdentity,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        scene_velocity_view: &wgpu::TextureView,
        taa_history_previous_view: &wgpu::TextureView,
        params_buffer: &wgpu::Buffer,
        taa_reactive_mask_view: &wgpu::TextureView,
    ) -> PreparedTaaResolveBindGroup {
        let frame_target = TaaResolveFrameTargetKey::from(key);
        let history_pair =
            TaaResolveHistoryPairKey::new(key.history_previous, history_current_identity);
        if self
            .frame_target
            .is_some_and(|current| current != frame_target)
            || self
                .history_pair
                .is_some_and(|current| current != history_pair)
        {
            self.entries.clear();
        }
        self.frame_target = Some(frame_target);
        self.history_pair = Some(history_pair);
        if let Some(entry) = self.entries.back().filter(|entry| entry.key == key) {
            return PreparedTaaResolveBindGroup {
                bind_group: entry.bind_group.clone(),
                created: false,
            };
        }
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            let entry = self
                .entries
                .remove(index)
                .expect("located TAA resolve bind group must remain cached");
            self.entries.push_back(entry);
            return PreparedTaaResolveBindGroup {
                bind_group: self
                    .entries
                    .back()
                    .expect("cached TAA resolve bind group")
                    .bind_group
                    .clone(),
                created: false,
            };
        }

        if self.entries.len() >= MAX_TAA_RESOLVE_BIND_GROUPS {
            self.entries.pop_front();
        }
        self.entries.push_back(TaaResolveBindGroupEntry {
            key,
            bind_group: create_bind_group(
                device,
                layout,
                scene_color_view,
                scene_depth_view,
                scene_velocity_view,
                taa_history_previous_view,
                params_buffer,
                taa_reactive_mask_view,
            ),
        });
        PreparedTaaResolveBindGroup {
            bind_group: self
                .entries
                .back()
                .expect("prepared TAA resolve bind group")
                .bind_group
                .clone(),
            created: true,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    scene_color_view: &wgpu::TextureView,
    scene_depth_view: &wgpu::TextureView,
    scene_velocity_view: &wgpu::TextureView,
    taa_history_previous_view: &wgpu::TextureView,
    params_buffer: &wgpu::Buffer,
    taa_reactive_mask_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-taa-resolve-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(scene_color_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(scene_velocity_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(taa_history_previous_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(taa_reactive_mask_view),
            },
        ],
    })
}

#[cfg(test)]
#[path = "taa_resolve_bind_group_cache/tests/mru_tests.rs"]
mod mru_tests;

#[cfg(test)]
#[path = "tests/taa_resolve_bind_group_cache.rs"]
mod tests;
