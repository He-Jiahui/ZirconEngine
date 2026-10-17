//! 将富文本表格的行内轴、块轴转换为物理坐标。
//! 竖排的行内方向向下、块方向从右向左；坐标的有限值恢复与几何预算准入分属不同步骤。

use crate::text::layout_geometry::{finite_f32_or_geometry, finite_sum};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextWritingMode},
};

/// 表格布局使用的轴映射；不持有样式或布局结果的借用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TableAxes {
    HorizontalTb,
    VerticalRl,
}

impl TableAxes {
    /// 从样式选择轴映射；只有显式竖排模式进入竖排分支，其余使用横排。
    pub(super) fn from_style(style: &UiResolvedStyle) -> Self {
        if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
            Self::VerticalRl
        } else {
            Self::HorizontalTb
        }
    }

    /// 返回框在行内方向上的长度：横排取宽，竖排取高，不在此验证数值。
    pub(super) fn inline_extent(self, frame: UiFrame) -> f32 {
        match self {
            Self::HorizontalTb => frame.width,
            Self::VerticalRl => frame.height,
        }
    }

    /// 返回框在块方向上的长度：横排取高，竖排取宽，不在此验证数值。
    pub(super) fn block_extent(self, frame: UiFrame) -> f32 {
        match self {
            Self::HorizontalTb => frame.height,
            Self::VerticalRl => frame.width,
        }
    }

    /// 按当前书写方向生成单元或轨道的物理框。
    ///
    /// # 参数与返回
    /// `container` 提供物理原点和竖排右边界；两个起点与两个长度均为逻辑轴上的量。
    /// 返回值交换竖排宽高，并将块起点解释为距容器右侧的距离。
    ///
    /// # 约束
    /// 原点运算在普通结果非有限时使用较宽中间值恢复；长度仍按输入映射。
    /// 此步骤不保证预算、容器包含关系或精确实数运算，调用层仍须做几何准入。
    /// Maps one logical table rect into the final physical writing-mode frame.
    pub(super) fn physical_frame(
        self,
        container: UiFrame,
        inline_start: f32,
        block_start: f32,
        inline_extent: f32,
        block_extent: f32,
    ) -> UiFrame {
        match self {
            Self::HorizontalTb => {
                let x = finite_f32_or_geometry(
                    container.x + inline_start,
                    f64::from(container.x) + f64::from(inline_start),
                );
                let y = finite_f32_or_geometry(
                    container.y + block_start,
                    f64::from(container.y) + f64::from(block_start),
                );
                UiFrame::new(x, y, inline_extent, block_extent)
            }
            Self::VerticalRl => {
                let right_exact = f64::from(container.x) + f64::from(container.width);
                let x_candidate = container.right() - block_start - block_extent;
                let x = finite_f32_or_geometry(
                    x_candidate,
                    right_exact - f64::from(block_start) - f64::from(block_extent),
                );
                let y = finite_f32_or_geometry(
                    container.y + inline_start,
                    f64::from(container.y) + f64::from(inline_start),
                );
                UiFrame::new(x, y, block_extent, inline_extent)
            }
        }
    }

    /// 将逻辑行内/块长度转换为物理宽高二元组，不平移坐标。
    pub(super) fn physical_extents(self, inline_extent: f32, block_extent: f32) -> (f32, f32) {
        match self {
            Self::HorizontalTb => (inline_extent, block_extent),
            Self::VerticalRl => (block_extent, inline_extent),
        }
    }

    /// 为后续块返回尚未消费的框；消耗量和剩余块长度均取零下限。
    /// 横排下移原点，竖排保持左边界并缩短宽度，使新的右边界向左推进。
    /// 只计算剩余框，不执行裁剪或预算验证。
    pub(super) fn remaining_frame(self, container: UiFrame, consumed_block: f32) -> UiFrame {
        let consumed_block = consumed_block.max(0.0);
        match self {
            Self::HorizontalTb => UiFrame::new(
                container.x,
                finite_sum([container.y, consumed_block]),
                container.width,
                (container.height - consumed_block).max(0.0),
            ),
            Self::VerticalRl => UiFrame::new(
                container.x,
                container.y,
                (container.width - consumed_block).max(0.0),
                container.height,
            ),
        }
    }

    /// 借用布局并读取其行内测量长度，不修改测量结果。
    pub(super) fn layout_inline_extent(
        self,
        layout: &zircon_runtime_interface::ui::surface::UiResolvedTextLayout,
    ) -> f32 {
        match self {
            Self::HorizontalTb => layout.measured_width,
            Self::VerticalRl => layout.measured_height,
        }
    }

    /// 借用布局并读取其块向测量长度，供表格块累计使用。
    pub(super) fn layout_block_extent(
        self,
        layout: &zircon_runtime_interface::ui::surface::UiResolvedTextLayout,
    ) -> f32 {
        match self {
            Self::HorizontalTb => layout.measured_height,
            Self::VerticalRl => layout.measured_width,
        }
    }
}

#[cfg(test)]
#[path = "tests/axes.rs"]
mod tests;
