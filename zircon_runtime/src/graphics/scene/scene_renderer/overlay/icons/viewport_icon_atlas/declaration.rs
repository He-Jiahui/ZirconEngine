use std::sync::Arc;

use crate::core::framework::render::ViewportIconId;
use zr_rhi_wgpu::WgpuTextureUploadBatch;

use super::super::super::ViewportIconSource;
use super::super::{icon_entry::IconEntry, icon_slot::icon_slot};

/// 固定图标集合的延迟资源缓存；不存在的素材也保留结果，避免每帧重复解码或查找。
pub(crate) struct ViewportIconAtlas {
    pub(super) source: Arc<dyn ViewportIconSource>,
    pub(super) entries: Vec<IconEntry>,
}

impl ViewportIconAtlas {
    pub(crate) fn new(source: Arc<dyn ViewportIconSource>) -> Self {
        Self {
            source,
            entries: vec![IconEntry::Unloaded; 2],
        }
    }

    /// 判断本帧是否可使用图标绑定，从而关闭几何回退；准备态仍需同帧上传完成后再绘制。
    pub(crate) fn has(&self, id: ViewportIconId) -> bool {
        matches!(
            self.entries[icon_slot(id)],
            IconEntry::Pending { .. } | IconEntry::Ready(_)
        )
    }

    /// 每次帧准备都重放未确认的上传债务；失败帧不消耗债务，后续帧可以重试。
    pub(crate) fn append_pending_uploads(&self, texture_uploads: &mut WgpuTextureUploadBatch) {
        for entry in &self.entries {
            if let IconEntry::Pending { upload, .. } = entry {
                texture_uploads.push(upload.clone());
            }
        }
    }

    /// 仅在包含图标上传的帧提交成功后调用；此确认会停止后续帧重放，不能在准备阶段调用。
    pub(crate) fn commit_pending_uploads(&mut self) -> u32 {
        let mut committed = 0_u32;
        for entry in &mut self.entries {
            let pending_sprite = match entry {
                IconEntry::Pending { sprite, .. } => Some(sprite.clone()),
                IconEntry::Unloaded | IconEntry::Missing | IconEntry::Ready(_) => None,
            };
            if let Some(sprite) = pending_sprite {
                *entry = IconEntry::Ready(sprite);
                committed = committed.saturating_add(1);
            }
        }
        committed
    }
}

#[cfg(test)]
#[path = "tests/declaration.rs"]
mod tests;
