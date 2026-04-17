use std::sync::Arc;

use zircon_scene::ViewportIconId;

use crate::types::GraphicsError;

use super::super::{
    icon_entry::IconEntry, icon_slot::icon_slot, viewport_icon_sprite::ViewportIconSprite,
};
use super::create_sprite::create_sprite;
use super::declaration::ViewportIconAtlas;
use super::decode_icon_rgba::decode_icon_rgba;

impl ViewportIconAtlas {
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
        let (width, height, rgba) =
            decode_icon_rgba(bytes, &format!("viewport gizmo icon {id:?}"))?;
        let sprite: Arc<ViewportIconSprite> =
            create_sprite(device, queue, texture_layout, sampler, width, height, &rgba);
        self.entries[slot] = IconEntry::Ready(sprite.clone());
        Ok(Some(sprite.bind_group.clone()))
    }
}
