use serde::{Deserialize, Serialize};

use crate::ui::{
    layout::DesiredSize, layout::UiFrame, layout::UiSize, layout::UiVirtualListWindow,
};

/// 节点测量与排列阶段共享的缓存；调用方在输入变化时失效，在测量完成后标记有效。
/// 写入几何值本身不会完成测量；文本修订用于保留式绘制缓存，与测量有效位独立。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutCache {
    /// 测量所得的期望尺寸，供父容器分配空间。
    pub desired_size: DesiredSize,
    /// 当前排列几何；手动预置该值不等于重新测量。
    pub frame: UiFrame,
    /// 排列阶段给出的裁剪框；无值时由消费方采用节点框等回退规则。
    pub clip_frame: Option<UiFrame>,
    /// 内容范围，供滚动等消费者使用；测量或排列阶段可更新。
    pub content_size: UiSize,
    /// 虚拟列表当前窗口信息；与内容总范围分别保存。
    pub virtual_window: Option<UiVirtualListWindow>,
    /// True when desired/content size was measured for the node's current layout inputs.
    /// This is deliberately independent from `frame`: a valid layout may be zero-sized.
    #[serde(default)]
    pub measure_valid: bool,
    /// Advances whenever the retained node's text-layout inputs change.
    /// `u64::MAX` is a serialized exhaustion sentinel and is never a publishable cache revision.
    #[serde(default)]
    pub text_layout_revision: u64,
}

impl UiLayoutCache {
    /// 仅清除测量有效位，保留旧尺寸和几何供调用方决定如何使用。
    pub fn invalidate_measure(&mut self) {
        self.measure_valid = false;
    }

    /// 仅设置有效位；调用方须先写入本轮测量结果，本方法不计算或校验尺寸。
    pub fn complete_measure(&mut self) {
        self.measure_valid = true;
    }

    /// 文本布局输入改变后由调用方推进修订；达到极值后保持哨兵值，不回绕复用旧键。
    pub fn advance_text_layout_revision(&mut self) {
        self.text_layout_revision = self.text_layout_revision.checked_add(1).unwrap_or(u64::MAX);
    }

    /// Returns a revision only while the retained identity cannot alias an earlier cache key.
    /// Layout remains available after exhaustion, but retained reuse must stay disabled.
    pub fn retained_text_layout_revision(&self) -> Option<u64> {
        (self.text_layout_revision != u64::MAX).then_some(self.text_layout_revision)
    }
}

#[cfg(test)]
#[path = "tests/layout_cache.rs"]
mod tests;
