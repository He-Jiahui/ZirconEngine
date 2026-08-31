use std::sync::Arc;

use crate::core::framework::render::ViewportIconId;

use crate::graphics::types::GraphicsError;

use super::super::{icon_entry::IconEntry, icon_slot::icon_slot};
use super::create_sprite::prepare_sprite;
use super::declaration::ViewportIconAtlas;
use super::decode_icon_rgba::decode_icon_rgba;

impl ViewportIconAtlas {
    pub(crate) fn ensure(
        &mut self,
        id: ViewportIconId,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Result<Option<Arc<wgpu::BindGroup>>, GraphicsError> {
        let slot = icon_slot(id);
        match &self.entries[slot] {
            IconEntry::Pending { sprite, .. } | IconEntry::Ready(sprite) => {
                return Ok(Some(sprite.bind_group.clone()));
            }
            IconEntry::Missing => return Ok(None),
            IconEntry::Unloaded => {}
        }

        let Some(bytes) = self.source.bytes(id) else {
            self.entries[slot] = IconEntry::Missing;
            return Ok(None);
        };
        let (width, height, rgba) =
            decode_icon_rgba(bytes, &format!("viewport gizmo icon {id:?}"))?;
        let prepared = prepare_sprite(device, texture_layout, sampler, width, height, rgba)?;
        let bind_group = prepared.sprite.bind_group.clone();
        self.entries[slot] = IconEntry::Pending {
            sprite: prepared.sprite,
            upload: prepared.upload,
        };
        Ok(Some(bind_group))
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("ensure.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("viewport icon ensure source should retain a test-module boundary")
    }

    #[test]
    fn viewport_icon_cache_publishes_pending_before_returning_the_candidate_binding() {
        let source = production_source();
        let prepare = source
            .find("let prepared = prepare_sprite(")
            .expect("viewport icon prepare stage");
        let pending = source
            .find("self.entries[slot] = IconEntry::Pending")
            .expect("viewport icon pending publication");
        let return_binding = source
            .find("Ok(Some(bind_group))")
            .expect("viewport icon candidate binding return");

        assert!(prepare < pending);
        assert!(pending < return_binding);
        assert!(!source.contains("wgpu::Queue"));
        assert!(!source.contains("IconEntry::Ready(sprite.clone())"));
    }
}
