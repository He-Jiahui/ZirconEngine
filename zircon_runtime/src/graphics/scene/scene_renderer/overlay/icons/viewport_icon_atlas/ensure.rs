use std::sync::Arc;

use crate::core::framework::render::ViewportIconId;

use crate::graphics::types::GraphicsError;

use super::super::{icon_entry::IconEntry, icon_slot::icon_slot};
use super::create_sprite::prepare_sprite;
use super::declaration::ViewportIconAtlas;
use super::decode_icon_rgba::decode_icon_rgba;

impl ViewportIconAtlas {
    /// 按需取得本帧绘制候选；首次创建会保留上传债务，解码失败则保持可重试状态。
    /// 返回绑定后，调用方仍须把缓存债务加入帧上传批次，不能单独提交绘制。
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
#[path = "tests/ensure.rs"]
mod tests;
