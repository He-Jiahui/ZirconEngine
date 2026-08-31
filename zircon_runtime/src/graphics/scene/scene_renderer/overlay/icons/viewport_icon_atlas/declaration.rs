use std::sync::Arc;

use crate::core::framework::render::ViewportIconId;
use zr_rhi_wgpu::WgpuTextureUploadBatch;

use super::super::super::ViewportIconSource;
use super::super::{icon_entry::IconEntry, icon_slot::icon_slot};

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

    pub(crate) fn has(&self, id: ViewportIconId) -> bool {
        matches!(
            self.entries[icon_slot(id)],
            IconEntry::Pending { .. } | IconEntry::Ready(_)
        )
    }

    pub(crate) fn append_pending_uploads(&self, texture_uploads: &mut WgpuTextureUploadBatch) {
        for entry in &self.entries {
            if let IconEntry::Pending { upload, .. } = entry {
                texture_uploads.push(upload.clone());
            }
        }
    }

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
mod tests {
    const SOURCE: &str = include_str!("declaration.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("viewport icon atlas source should retain a test-module boundary")
    }

    #[test]
    fn viewport_icon_upload_debt_is_replayed_then_committed_from_fixed_slots() {
        let source = production_source();

        assert!(source.contains("entries: vec![IconEntry::Unloaded; 2]"));
        assert!(source.contains("for entry in &self.entries"));
        assert!(source.contains("texture_uploads.push(upload.clone())"));
        assert!(source.contains("IconEntry::Pending { sprite, .. }"));
        assert!(source.contains("*entry = IconEntry::Ready(sprite)"));
    }
}
