use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTreeError};

use super::UiSurface;

impl UiSurface {
    pub(super) fn invalidate_for_changed_text_font_generation(
        &mut self,
    ) -> Result<bool, UiTreeError> {
        if self.observed_text_font_generation == self.text_measure_cache.font_database_generation()
        {
            return Ok(false);
        }

        let roots = self.tree.roots.clone();
        let text_dirty = UiDirtyFlags {
            text: true,
            ..UiDirtyFlags::default()
        };
        for root in roots {
            self.mark_node_dirty(root, text_dirty)?;
        }
        Ok(true)
    }

    pub(super) fn record_text_font_generation_layout(&mut self, font_generation: u64) {
        self.observed_text_font_generation = font_generation;
    }
}
