use crate::core::math::UVec2;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn size(&self) -> UVec2 {
        self.descriptor.size
    }

    pub(in crate::graphics::runtime::render_framework) fn requires_hit_proxies(&self) -> bool {
        self.descriptor.requires_hit_proxies
    }
}
