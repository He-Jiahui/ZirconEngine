use bytemuck::{bytes_of, cast_slice};
use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::hzb::{
    HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE,
    HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE, HZB_OCCLUSION_DRAW_COUNT_RESOURCE,
    HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE, HZB_OCCLUSION_STATS_RESOURCE,
    HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE,
};
use crate::graphics::scene::scene_renderer::lighting::light_grid_builder::{
    LightGridParams, LIGHT_GRID_EMPTY_ZBIN_HEADER,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES, INDIRECT_COMPACTION_METADATA_STRIDE_BYTES,
    INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES, INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
};

const HZB_NEUTRAL_STORAGE_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC);
const HZB_NEUTRAL_INDIRECT_STORAGE_USAGE: wgpu::BufferUsages =
    HZB_NEUTRAL_STORAGE_USAGE.union(wgpu::BufferUsages::INDIRECT);
const LIGHT_GRID_NEUTRAL_PARAMS_USAGE: wgpu::BufferUsages =
    wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST);
const LIGHT_GRID_NEUTRAL_STORAGE_USAGE: wgpu::BufferUsages =
    wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST);

pub(in crate::graphics::scene::scene_renderer::core) const LIGHT_GRID_PARAMS_NEUTRAL_BACKING: &str =
    "light-grid-params:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const LIGHT_GRID_ZBINS_NEUTRAL_BACKING: &str =
    "light-zbins:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const LIGHT_GRID_TILE_MASKS_NEUTRAL_BACKING:
    &str = "light-tile-masks:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const HZB_INDIRECT_ARGS_NEUTRAL_BACKING: &str =
    "hzb-occlusion-indirect-args:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const HZB_METADATA_NEUTRAL_BACKING: &str =
    "hzb-occlusion-compaction-metadata:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const HZB_COMPACTED_ARGS_NEUTRAL_BACKING:
    &str = "hzb-occlusion-compacted-indirect-args:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const HZB_VISIBLE_INDEX_NEUTRAL_BACKING: &str =
    "hzb-occlusion-visible-instance-index:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const HZB_DRAW_COUNT_NEUTRAL_BACKING: &str =
    "hzb-occlusion-draw-count:neutral";
pub(in crate::graphics::scene::scene_renderer::core) const HZB_STATS_NEUTRAL_BACKING: &str =
    "hzb-occlusion-stats:neutral";

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererNeutralGraphBuffers {
    light_grid: Option<LightGridNeutralBuffers>,
    hzb: Option<HzbNeutralBuffers>,
    plugin: FirstPartyPluginNeutralBuffers,
}

impl SceneRendererNeutralGraphBuffers {
    pub(in crate::graphics::scene::scene_renderer::core) fn light_grid(
        &mut self,
        device: &wgpu::Device,
    ) -> &LightGridNeutralBuffers {
        self.light_grid
            .get_or_insert_with(|| LightGridNeutralBuffers::new(device))
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn hzb(
        &mut self,
        device: &wgpu::Device,
    ) -> &HzbNeutralBuffers {
        self.hzb
            .get_or_insert_with(|| HzbNeutralBuffers::new(device))
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn plugin_buffer(
        &mut self,
        device: &wgpu::Device,
        logical_name: &'static str,
        size: wgpu::BufferAddress,
    ) -> Option<(&wgpu::Buffer, &'static str)> {
        self.plugin.buffer(device, logical_name, size)
    }
}

#[derive(Default)]
struct FirstPartyPluginNeutralBuffers {
    virtual_geometry_feedback: Option<PluginNeutralBuffer>,
}

impl FirstPartyPluginNeutralBuffers {
    fn buffer(
        &mut self,
        device: &wgpu::Device,
        logical_name: &'static str,
        size: wgpu::BufferAddress,
    ) -> Option<(&wgpu::Buffer, &'static str)> {
        let (slot, label, backing): (&mut Option<PluginNeutralBuffer>, &'static str, &'static str) =
            match logical_name {
                "virtual-geometry-feedback" => (
                    &mut self.virtual_geometry_feedback,
                    "zircon-plugin-neutral-virtual-geometry-feedback",
                    "virtual-geometry-feedback:plugin-neutral",
                ),
                _ => return None,
            };
        let buffer = slot.get_or_insert_with(|| PluginNeutralBuffer {
            buffer: zeroed_buffer(device, label, size, PLUGIN_NEUTRAL_USAGE),
        });
        Some((&buffer.buffer, backing))
    }
}

struct PluginNeutralBuffer {
    buffer: wgpu::Buffer,
}

pub(in crate::graphics::scene::scene_renderer::core) struct LightGridNeutralBuffers {
    params: wgpu::Buffer,
    zbins: wgpu::Buffer,
    tile_masks: wgpu::Buffer,
}

impl LightGridNeutralBuffers {
    fn new(device: &wgpu::Device) -> Self {
        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-light-grid-params-neutral"),
            contents: bytes_of(&LightGridParams::disabled()),
            usage: LIGHT_GRID_NEUTRAL_PARAMS_USAGE,
        });
        let zbins = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-light-grid-zbins-neutral"),
            contents: cast_slice(&[LIGHT_GRID_EMPTY_ZBIN_HEADER, 0, 0]),
            usage: LIGHT_GRID_NEUTRAL_STORAGE_USAGE,
        });
        let tile_masks = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-light-grid-tile-masks-neutral"),
            contents: cast_slice(&[0_u32]),
            usage: LIGHT_GRID_NEUTRAL_STORAGE_USAGE,
        });
        Self {
            params,
            zbins,
            tile_masks,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn buffer(
        &self,
        logical_name: &str,
    ) -> Option<(&wgpu::Buffer, &'static str)> {
        match logical_name {
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_GRID_PARAMS => {
                Some((&self.params, LIGHT_GRID_PARAMS_NEUTRAL_BACKING))
            }
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_ZBINS => {
                Some((&self.zbins, LIGHT_GRID_ZBINS_NEUTRAL_BACKING))
            }
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_TILE_MASKS => {
                Some((&self.tile_masks, LIGHT_GRID_TILE_MASKS_NEUTRAL_BACKING))
            }
            _ => None,
        }
    }
}

pub(in crate::graphics::scene::scene_renderer::core) struct HzbNeutralBuffers {
    indirect_args: wgpu::Buffer,
    metadata: wgpu::Buffer,
    compacted_args: wgpu::Buffer,
    visible_index: wgpu::Buffer,
    draw_count: wgpu::Buffer,
    stats: wgpu::Buffer,
}

impl HzbNeutralBuffers {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            indirect_args: neutral_buffer(
                device,
                "zircon-hzb-occlusion-indirect-args-neutral",
                INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
                HZB_NEUTRAL_INDIRECT_STORAGE_USAGE,
            ),
            metadata: neutral_buffer(
                device,
                "zircon-hzb-occlusion-compaction-metadata-neutral",
                INDIRECT_COMPACTION_METADATA_STRIDE_BYTES,
                HZB_NEUTRAL_STORAGE_USAGE,
            ),
            compacted_args: neutral_buffer(
                device,
                "zircon-hzb-occlusion-compacted-indirect-args-neutral",
                INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
                HZB_NEUTRAL_INDIRECT_STORAGE_USAGE,
            ),
            visible_index: neutral_buffer(
                device,
                "zircon-hzb-occlusion-visible-instance-index-neutral",
                INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
                HZB_NEUTRAL_STORAGE_USAGE,
            ),
            draw_count: neutral_buffer(
                device,
                "zircon-hzb-occlusion-draw-count-neutral",
                INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
                HZB_NEUTRAL_INDIRECT_STORAGE_USAGE,
            ),
            stats: neutral_buffer(
                device,
                "zircon-hzb-occlusion-stats-neutral",
                HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
                HZB_NEUTRAL_STORAGE_USAGE,
            ),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn buffer(
        &self,
        logical_name: &str,
    ) -> Option<(&wgpu::Buffer, &'static str)> {
        match logical_name {
            HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE => {
                Some((&self.indirect_args, HZB_INDIRECT_ARGS_NEUTRAL_BACKING))
            }
            HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE => {
                Some((&self.metadata, HZB_METADATA_NEUTRAL_BACKING))
            }
            HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE => {
                Some((&self.compacted_args, HZB_COMPACTED_ARGS_NEUTRAL_BACKING))
            }
            HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE => {
                Some((&self.visible_index, HZB_VISIBLE_INDEX_NEUTRAL_BACKING))
            }
            HZB_OCCLUSION_DRAW_COUNT_RESOURCE => {
                Some((&self.draw_count, HZB_DRAW_COUNT_NEUTRAL_BACKING))
            }
            HZB_OCCLUSION_STATS_RESOURCE => Some((&self.stats, HZB_STATS_NEUTRAL_BACKING)),
            _ => None,
        }
    }
}

fn neutral_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: wgpu::BufferAddress,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    zeroed_buffer(device, label, size, usage)
}

const PLUGIN_NEUTRAL_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC)
    .union(wgpu::BufferUsages::INDIRECT);

fn zeroed_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: wgpu::BufferAddress,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: true,
    });
    let mut mapped_bytes = buffer.slice(..).get_mapped_range_mut();
    let zeroes = vec![0; mapped_bytes.len()];
    mapped_bytes.copy_from_slice(&zeroes);
    drop(mapped_bytes);
    buffer.unmap();
    buffer
}

#[cfg(test)]
mod tests {
    #[test]
    fn neutral_graph_buffers_are_lazy_device_lifetime_owners() {
        let source = include_str!("neutral_graph_buffers.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert!(source.contains("light_grid: Option<LightGridNeutralBuffers>"));
        assert!(source.contains("hzb: Option<HzbNeutralBuffers>"));
        assert!(source.contains("plugin: FirstPartyPluginNeutralBuffers"));
        assert_eq!(
            source
                .matches("get_or_insert_with(|| LightGridNeutralBuffers::new(device))")
                .count(),
            1
        );
        assert!(source.contains("mapped_at_creation: true"));
        assert_eq!(
            source
                .matches("get_or_insert_with(|| HzbNeutralBuffers::new(device))")
                .count(),
            1
        );
    }

    #[test]
    fn mapped_neutral_buffers_copy_zeroes_into_the_wgpu_view_before_unmapping() {
        let source = include_str!("neutral_graph_buffers.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let zeroed_buffer_start = source
            .find("fn zeroed_buffer")
            .expect("neutral backing must have one mapped initialization helper");
        let zeroed_buffer = &source[zeroed_buffer_start..];

        assert!(zeroed_buffer
            .contains("let mut mapped_bytes = buffer.slice(..).get_mapped_range_mut();"));
        assert!(zeroed_buffer.contains("let zeroes = vec![0; mapped_bytes.len()];"));
        assert!(zeroed_buffer.contains("mapped_bytes.copy_from_slice(&zeroes);"));
        assert!(zeroed_buffer.contains("drop(mapped_bytes);"));
        assert!(!zeroed_buffer.contains("get_mapped_range_mut().as_mut()"));
        assert!(!zeroed_buffer.contains("mapped_bytes.fill(0);"));

        let zeroes = zeroed_buffer
            .find("let zeroes = vec![0; mapped_bytes.len()];")
            .expect("the mapped range must be initialized");
        let copy = zeroed_buffer
            .find("mapped_bytes.copy_from_slice(&zeroes);")
            .expect("the mapped range must receive the zero bytes");
        let release = zeroed_buffer
            .find("drop(mapped_bytes);")
            .expect("the mapped range must release before unmapping");
        let unmap = zeroed_buffer
            .find("buffer.unmap();")
            .expect("the backing buffer must be unmapped");
        assert!(zeroes < copy && copy < release && release < unmap);
    }
}
