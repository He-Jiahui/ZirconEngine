use std::sync::Arc;

use image::GenericImageView;
use zircon_scene::ViewportIconId;

use crate::types::GraphicsError;

use super::super::ViewportIconSource;
use super::{
    icon_entry::IconEntry, icon_slot::icon_slot, viewport_icon_sprite::ViewportIconSprite,
};

pub(crate) struct ViewportIconAtlas {
    source: Arc<dyn ViewportIconSource>,
    entries: Vec<IconEntry>,
}

impl ViewportIconAtlas {
    pub(crate) fn new(source: Arc<dyn ViewportIconSource>) -> Self {
        Self {
            source,
            entries: vec![IconEntry::Unloaded; 2],
        }
    }

    pub(crate) fn has(&self, id: ViewportIconId) -> bool {
        matches!(self.entries[icon_slot(id)], IconEntry::Ready(_))
    }

    pub(crate) fn ensure(
        &mut self,
        id: ViewportIconId,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Result<Option<Arc<wgpu::BindGroup>>, GraphicsError> {
        let slot = icon_slot(id);
        match &self.entries[slot] {
            IconEntry::Ready(sprite) => return Ok(Some(sprite.bind_group.clone())),
            IconEntry::Missing => return Ok(None),
            IconEntry::Unloaded => {}
        }

        let Some(bytes) = self.source.bytes(id) else {
            self.entries[slot] = IconEntry::Missing;
            return Ok(None);
        };
        let image = image::load_from_memory(bytes).map_err(|error| {
            GraphicsError::Asset(format!("viewport gizmo icon {id:?}: {error}"))
        })?;
        let luma = image.to_luma8();
        let (width, height) = image.dimensions();
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for alpha in luma.into_raw() {
            rgba.extend_from_slice(&[255, 255, 255, alpha]);
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-viewport-icon-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-viewport-icon-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        }));
        let sprite = Arc::new(ViewportIconSprite {
            _texture: texture,
            _view: view,
            bind_group,
        });
        self.entries[slot] = IconEntry::Ready(sprite.clone());
        Ok(Some(sprite.bind_group.clone()))
    }
}
