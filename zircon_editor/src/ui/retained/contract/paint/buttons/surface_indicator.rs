//! 选中页签式按钮的底部指示条投影；资源浏览器保留两侧缩进，其余页签占满宽度。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::style_selector::is_asset_browser_tab_like_button;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};

const ASSET_BROWSER_TAB_INDICATOR_MAX_INSET_RATIO: f32 = 0.24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonSurfaceIndicatorMetrics
{
    pub underline_height: f32,
    pub asset_browser_tab_inset_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonSurfaceIndicatorPalette
{
    pub underline: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_surface_indicator_metrics(
) -> WorkbenchButtonSurfaceIndicatorMetrics {
    button_surface_indicator_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_surface_indicator_palette(
) -> WorkbenchButtonSurfaceIndicatorPalette {
    button_surface_indicator_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_surface_indicator_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = button_surface_indicator_metrics();
    let height = metrics.underline_height.min(rect.height).max(0.0);
    let inset = if is_asset_browser_tab_like_button(node) {
        metrics
            .asset_browser_tab_inset_x
            .min(rect.width * ASSET_BROWSER_TAB_INDICATOR_MAX_INSET_RATIO)
            .max(0.0)
    } else {
        0.0
    };
    FrameRect {
        x: rect.x + inset,
        y: rect.y + (rect.height - height).max(0.0),
        width: (rect.width - inset * 2.0).max(0.0),
        height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_surface_indicator_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchButtonSurfaceIndicatorMetrics {
    WorkbenchButtonSurfaceIndicatorMetrics {
        underline_height: metrics.tab_underline_height,
        asset_browser_tab_inset_x: metrics.button_pad_x,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_surface_indicator_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchButtonSurfaceIndicatorPalette {
    WorkbenchButtonSurfaceIndicatorPalette {
        underline: palette.accent,
    }
}

#[cfg(test)]
#[path = "tests/surface_indicator.rs"]
mod tests;
