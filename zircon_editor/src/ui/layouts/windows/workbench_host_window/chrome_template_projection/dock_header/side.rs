use std::cell::RefCell;

use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

use super::*;
use crate::ui::retained_host::{
    current_host_metrics, measure_runtime_text_width_with_style, runtime_text_metrics_generation,
    HostControlMetrics,
};
#[path = "side/dimensions.rs"]
mod chrome_dimensions;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SIDE_DOCK_HEADER_CACHE_CAPACITY: usize = 12;

struct SideDockHeaderProjectionCacheEntry {
    tabs: ModelRc<TabData>,
    width_bits: u32,
    height_bits: u32,
    scale_bits: u32,
    paint_metrics: HostControlMetrics,
    text_metrics_generation: [u64; 3],
    nodes: ModelRc<ViewTemplateNodeData>,
}

#[derive(Default)]
struct SideDockHeaderProjectionCache {
    entries: Vec<SideDockHeaderProjectionCacheEntry>,
    #[cfg(test)]
    builds: usize,
}

thread_local! {
    static SIDE_DOCK_HEADER_PROJECTION_CACHE: RefCell<SideDockHeaderProjectionCache> =
        RefCell::new(SideDockHeaderProjectionCache::default());
}

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    side_dock_header_nodes_with_metrics(
        tabs,
        width,
        height,
        runtime_text_metrics_generation(),
        chrome_dimensions::effective_scale(),
        current_host_metrics(),
    )
}

#[cfg(test)]
fn side_dock_header_nodes_with_text_generation(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
    text_metrics_generation: [u64; 3],
) -> ModelRc<ViewTemplateNodeData> {
    side_dock_header_nodes_with_metrics(
        tabs,
        width,
        height,
        text_metrics_generation,
        chrome_dimensions::effective_scale(),
        current_host_metrics(),
    )
}

fn side_dock_header_nodes_with_metrics(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
    text_metrics_generation: [u64; 3],
    scale: f32,
    paint_metrics: HostControlMetrics,
) -> ModelRc<ViewTemplateNodeData> {
    let width_bits = width.to_bits();
    let height_bits = height.to_bits();
    SIDE_DOCK_HEADER_PROJECTION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.entries.iter().find(|entry| {
            entry.width_bits == width_bits
                && entry.height_bits == height_bits
                && entry.scale_bits == scale.to_bits()
                && entry.paint_metrics == paint_metrics
                && entry.text_metrics_generation == text_metrics_generation
                && entry.tabs.shares_values_with(tabs)
        }) {
            return entry.nodes.clone();
        }

        let logical =
            build_side_dock_header_nodes(tabs, width / scale, height / scale, scale, paint_metrics);
        let nodes = chrome_dimensions::physical_nodes(&logical, scale);
        if cache.entries.len() == SIDE_DOCK_HEADER_CACHE_CAPACITY {
            cache.entries.remove(0);
        }
        cache.entries.push(SideDockHeaderProjectionCacheEntry {
            tabs: tabs.clone(),
            width_bits,
            height_bits,
            scale_bits: scale.to_bits(),
            paint_metrics,
            text_metrics_generation,
            nodes: nodes.clone(),
        });
        #[cfg(test)]
        {
            cache.builds += 1;
        }
        nodes
    })
}

fn build_side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
    scale: f32,
    paint_metrics: HostControlMetrics,
) -> ModelRc<ViewTemplateNodeData> {
    let header_height = if height.is_finite() && height > 0.0 {
        height
    } else {
        DOCK_HEADER_HEIGHT_PX
    };
    let tab_height = (header_height - DOCUMENT_TAB_STRIP_Y)
        .max(0.0)
        .min(DOCUMENT_TAB_HEIGHT);
    let close_extent = DOCUMENT_TAB_CLOSE_EXTENT.min(header_height);
    let mut layout = side_dock_tab_layout(tabs, width, header_height, scale, paint_metrics);
    if let Some(frame) = &mut layout.overflow_frame {
        frame.height = tab_height;
    }
    let slots = &layout.slots;
    let close_count = (0..tabs.row_count())
        .filter(|row| {
            tabs.get(*row).is_some_and(|tab| {
                tab.closeable && slots.get(*row).is_some_and(|slot| slot.shows_label)
            })
        })
        .count();
    let mut nodes = Vec::with_capacity(
        tabs.row_count() + close_count + usize::from(layout.overflow_frame.is_some()) + 1,
    );
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackSideDockHeaderBar".into(),
        control_id: DOCK_HEADER_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: header_height,
        },
        ..ViewTemplateNodeData::default()
    });

    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.get(row) else {
            continue;
        };
        let slot = slots.get(row).copied().unwrap_or_default();
        if slot.width <= f32::EPSILON {
            continue;
        }
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackSideDockTab{row}").into(),
            control_id: format!("{DOCK_TAB_PREFIX}{row}").into(),
            role: "Button".into(),
            text: if slot.shows_label {
                tab.title.clone()
            } else {
                SharedString::default()
            },
            text_tone: text_tone.into(),
            font_size: DOCUMENT_TAB_TITLE_FONT_SIZE
                .min(tab_height / paint_metrics.line_height_ratio.max(1.0)),
            font_weight,
            surface_variant: if tab.active { "inset" } else { "transparent" }.into(),
            button_variant: "ghost".into(),
            corner_radius: fallback_chrome_control_radius(),
            selected: tab.active,
            focused: false,
            frame: ViewTemplateFrameData {
                x: slot.x,
                y: DOCUMENT_TAB_STRIP_Y,
                width: slot.width,
                height: tab_height,
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut tab_node, &chrome_tab_icon_name(&tab));
        nodes.push(tab_node);
        if tab.closeable && slot.shows_label {
            let mut close_node = ViewTemplateNodeData {
                node_id: format!("FallbackSideDockTabClose{row}").into(),
                control_id: format!("{DOCK_TAB_CLOSE_PREFIX}{row}").into(),
                role: "IconButton".into(),
                text_tone: "muted".into(),
                font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
                surface_variant: "transparent".into(),
                button_variant: "ghost".into(),
                corner_radius: fallback_chrome_control_radius(),
                value_number: 14.0,
                frame: ViewTemplateFrameData {
                    x: slot.x + slot.width
                        - crate::ui::workbench::document_tabs::DOCUMENT_TAB_CLOSE_RIGHT_INSET
                        - close_extent,
                    y: (header_height - close_extent).max(0.0) * 0.5,
                    width: close_extent,
                    height: close_extent,
                },
                ..ViewTemplateNodeData::default()
            };
            apply_template_icon(&mut close_node, DOCK_TAB_CLOSE_ICON);
            nodes.push(close_node);
        }
    }
    if let Some(overflow_frame) = layout.overflow_frame {
        let mut overflow_node = ViewTemplateNodeData {
            node_id: "FallbackSideDockTabOverflow".into(),
            control_id: DOCK_TAB_OVERFLOW_CONTROL_ID.into(),
            role: "IconButton".into(),
            text_tone: "subtle".into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            surface_variant: "transparent".into(),
            button_variant: "ghost".into(),
            corner_radius: fallback_chrome_control_radius(),
            frame: overflow_frame,
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut overflow_node, "ellipsis-horizontal-outline");
        nodes.push(overflow_node);
    }
    model_rc(nodes)
}

fn side_dock_tab_layout(
    tabs: &ModelRc<TabData>,
    width: f32,
    header_height: f32,
    scale: f32,
    paint_metrics: HostControlMetrics,
) -> DockTabLayout {
    let controls = EditorControlTokens::workbench_dense();
    let density = EditorDensityTokens::workbench_dense();
    let compact_width = paint_metrics.control_default_height.max(
        (paint_metrics.row_height - paint_metrics.gap_l).max(1.0)
            + paint_metrics.button_pad_x * 2.0,
    ) / scale;
    let preferred_widths = (0..tabs.row_count())
        .map(|index| {
            tabs.get(index)
                .map(|tab| {
                    let title_width = measure_runtime_text_width_with_style(
                        tab.title.as_str(),
                        DOCUMENT_TAB_TITLE_FONT_SIZE * scale,
                        UiTextRunPaintStyle {
                            strong: tab.active,
                            ..Default::default()
                        },
                    ) / scale;
                    side_dock_tab_preferred_width_from_paint(
                        &tab,
                        title_width,
                        scale,
                        paint_metrics,
                        controls,
                        density,
                    )
                })
                .unwrap_or(compact_width)
        })
        .collect::<Vec<_>>();
    adaptive_dock_tab_layout(tabs, width, compact_width, header_height, &preferred_widths)
}

#[cfg(test)]
fn side_dock_tab_preferred_width_from_measured(
    title: &str,
    title_width: f32,
    controls: EditorControlTokens,
    density: EditorDensityTokens,
) -> f32 {
    side_dock_tab_preferred_width_from_paint(
        &TabData {
            title: title.into(),
            ..Default::default()
        },
        title_width,
        chrome_dimensions::effective_scale(),
        current_host_metrics(),
        controls,
        density,
    )
}

fn side_dock_tab_preferred_width_from_paint(
    tab: &TabData,
    title_width: f32,
    scale: f32,
    metrics: HostControlMetrics,
    controls: EditorControlTokens,
    density: EditorDensityTokens,
) -> f32 {
    let title_width = if title_width.is_finite() && title_width > 0.0 {
        title_width
    } else {
        // The first frame can precede font admission and report zero intrinsic width.
        tab.title
            .chars()
            .map(|c| if c.is_ascii() { 0.8 } else { 1.0 })
            .sum::<f32>()
            * DOCUMENT_TAB_TITLE_FONT_SIZE
    };
    // Match the generic Button painter: Icon16, icon gap, two button insets,
    // full text guard, and the existing close affordance for closeable tabs.
    let icon_width = (metrics.row_height - metrics.gap_l).max(1.0) / scale;
    let close_reserve = if tab.closeable {
        crate::ui::workbench::document_tabs::DOCUMENT_TAB_CLOSE_EXTENT
            + crate::ui::workbench::document_tabs::DOCUMENT_TAB_CLOSE_RIGHT_INSET
    } else {
        0.0
    };
    (title_width
        + icon_width
        + (metrics.button_icon_gap + metrics.button_pad_x * 2.0 + metrics.text_clip_guard) / scale
        + close_reserve)
        .max(controls.default_height * 3.0 + density.gap_medium * 2.0)
}

#[cfg(test)]
#[path = "tests/side_font_generation_tests.rs"]
mod font_generation_tests;

#[cfg(test)]
pub(super) fn clear_side_dock_header_projection_cache_for_tests() {
    SIDE_DOCK_HEADER_PROJECTION_CACHE.with(|cache| *cache.borrow_mut() = Default::default());
}

#[cfg(test)]
pub(super) fn side_dock_header_projection_builds_for_tests() -> usize {
    SIDE_DOCK_HEADER_PROJECTION_CACHE.with(|cache| cache.borrow().builds)
}
