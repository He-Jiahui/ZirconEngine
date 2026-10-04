use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTreeError};

use super::UiSurface;

// 布局入口在开启文字会话前比较字体代次，令字体热加载即使没有作者属性变更也能使整树文字重新布局。
// 记录值只应在本轮安排/提取成功发布后推进；保留操作开始的代次，避免把途中变化误认为已消费。
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
