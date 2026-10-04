use crate::text::layout_geometry::{finite_f32_or_geometry, finite_sum};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextWritingMode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TableAxes {
    HorizontalTb,
    VerticalRl,
}

impl TableAxes {
    pub(super) fn from_style(style: &UiResolvedStyle) -> Self {
        if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
            Self::VerticalRl
        } else {
            Self::HorizontalTb
        }
    }

    pub(super) fn inline_extent(self, frame: UiFrame) -> f32 {
        match self {
            Self::HorizontalTb => frame.width,
            Self::VerticalRl => frame.height,
        }
    }

    pub(super) fn block_extent(self, frame: UiFrame) -> f32 {
        match self {
            Self::HorizontalTb => frame.height,
            Self::VerticalRl => frame.width,
        }
    }

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

    pub(super) fn physical_extents(self, inline_extent: f32, block_extent: f32) -> (f32, f32) {
        match self {
            Self::HorizontalTb => (inline_extent, block_extent),
            Self::VerticalRl => (block_extent, inline_extent),
        }
    }

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

    pub(super) fn layout_inline_extent(
        self,
        layout: &zircon_runtime_interface::ui::surface::UiResolvedTextLayout,
    ) -> f32 {
        match self {
            Self::HorizontalTb => layout.measured_width,
            Self::VerticalRl => layout.measured_height,
        }
    }

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
