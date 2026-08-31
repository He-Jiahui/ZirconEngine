use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

const COMPACT_SEARCH_MIN_WIDTH: f32 = 160.0;
const COMPACT_SEARCH_PREFERRED_RATIO: f32 = 0.38;
const COMPACT_SEARCH_PREFERRED_MIN_WIDTH: f32 = 240.0;
const COMPACT_IMPORT_BUTTON_MIN_WIDTH: f32 = 72.0;
const COMPACT_IMPORT_PATH_MIN_WIDTH: f32 = 180.0;
const COMPACT_IMPORT_PATH_MAX_WIDTH: f32 = 260.0;
const COMPACT_IMPORT_PATH_VISIBLE_WIDTH: f32 = 1040.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct AssetBrowserToolbarMetrics {
    toolbar_height: f32,
    control_height: f32,
    compact_icon_width: f32,
    control_offset_y: f32,
    side_pad: f32,
    root_gap: f32,
    group_gap: f32,
    group_frame_pad: f32,
    row_gap: f32,
    view_button_gap: f32,
}

pub(super) struct AssetBrowserToolbarLayout {
    pub(super) main_y: f32,
}

pub(super) fn apply_asset_browser_toolbar_layout(
    nodes: &mut [ViewTemplateNodeData],
    viewport_width: f32,
) -> Option<AssetBrowserToolbarLayout> {
    let metrics = asset_browser_toolbar_metrics();
    let toolbar = node_frame(nodes, "AssetBrowserToolbarPanel")?;
    let import_panel = node_frame(nodes, "AssetBrowserImportPanel")?;
    let toolbar_width = viewport_width.max(toolbar.width).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserToolbarPanel",
        toolbar.x,
        toolbar.y,
        toolbar_width,
        metrics.toolbar_height,
    );
    collapse_redundant_header_nodes(nodes, toolbar.x, toolbar.y);
    layout_single_toolbar_row(nodes, toolbar.x, toolbar.y, toolbar_width, metrics);

    set_node_frame(
        nodes,
        "AssetBrowserImportPanel",
        import_panel.x,
        toolbar.y,
        toolbar_width,
        metrics.toolbar_height,
    );

    Some(AssetBrowserToolbarLayout {
        main_y: toolbar.y + metrics.toolbar_height + metrics.root_gap,
    })
}

fn collapse_redundant_header_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for control_id in [
        "AssetBrowserToolbarTitleRow",
        "AssetBrowserTitleText",
        "AssetBrowserToolbarSubtitleRow",
        "AssetBrowserSubtitleText",
    ] {
        hide_node(nodes, control_id, x, y);
    }
}

fn layout_single_toolbar_row(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    metrics: AssetBrowserToolbarMetrics,
) {
    let row_x = x + metrics.side_pad.min(width * 0.04);
    let row_y = y + metrics.control_offset_y;
    let row_width = (width - (row_x - x) * 2.0).max(0.0);
    let mut import = compact_import_group(nodes, row_width, metrics);
    let mut view = compact_view_group(nodes, metrics);
    let locate_width = control_width(nodes, "LocateSelectedAsset", metrics.compact_icon_width);
    let preferred_filter_width =
        control_width(nodes, "AssetBrowserKindFilterDropdown", 168.0).clamp(124.0, 220.0);
    let minimum_filter_width = 124.0;
    let minimum_leading_width = COMPACT_SEARCH_MIN_WIDTH + metrics.group_gap + minimum_filter_width;
    let mut locate_visible = true;
    collapse_trailing_actions_to_fit(
        row_width,
        minimum_leading_width,
        &mut view,
        &mut locate_visible,
        &mut import,
        locate_width,
        metrics,
    );
    let trailing_width =
        trailing_actions_width(&view, locate_visible, &import, locate_width, metrics);
    let trailing_x = row_x + row_width - trailing_width;
    let leading_gap = if trailing_width > 0.0 {
        metrics.group_gap
    } else {
        0.0
    };
    let leading_span_width = (trailing_x - leading_gap - row_x).max(0.0);
    let search_width = if leading_span_width >= minimum_leading_width {
        let preferred_search_width = (row_width * COMPACT_SEARCH_PREFERRED_RATIO)
            .max(COMPACT_SEARCH_PREFERRED_MIN_WIDTH)
            .min((leading_span_width - minimum_filter_width - metrics.group_gap).max(0.0));
        preferred_search_width.max(COMPACT_SEARCH_MIN_WIDTH)
    } else {
        0.0
    };
    let search_gap = if search_width > 0.0 {
        metrics.group_gap
    } else {
        0.0
    };
    let filter_x = row_x + search_width + search_gap;
    let filter_width_limit = (trailing_x - leading_gap - filter_x).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserToolbarSearchRow",
        x,
        y,
        width,
        metrics.toolbar_height,
    );
    set_node_frame(
        nodes,
        "SearchEdited",
        row_x,
        row_y,
        search_width,
        metrics.control_height,
    );
    if has_control(nodes, "AssetBrowserKindFilterDropdown") {
        let filter_width = preferred_filter_width.min(filter_width_limit);
        set_node_frame(
            nodes,
            "AssetBrowserKindFilterDropdown",
            filter_x,
            row_y,
            filter_width,
            metrics.control_height,
        );
    } else {
        layout_kind_chips(nodes, filter_x, row_y, filter_width_limit, metrics);
    }
    let mut action_x = trailing_x;
    if view.visible {
        set_node_frame(
            nodes,
            "AssetBrowserViewModeListButton",
            action_x,
            row_y,
            view.list_width,
            metrics.control_height,
        );
        set_node_frame(
            nodes,
            "AssetBrowserViewModeThumbButton",
            action_x + view.list_width + metrics.view_button_gap,
            row_y,
            view.thumb_width,
            metrics.control_height,
        );
        action_x += view.width;
    } else {
        hide_node(nodes, "AssetBrowserViewModeListButton", action_x, row_y);
        hide_node(nodes, "AssetBrowserViewModeThumbButton", action_x, row_y);
    }
    let filter_right_edge = if view.visible {
        action_x
    } else {
        trailing_x - leading_gap
    };
    if view.visible && locate_visible {
        action_x += metrics.group_gap;
    }
    if locate_visible {
        set_node_frame(
            nodes,
            "LocateSelectedAsset",
            action_x,
            row_y,
            locate_width,
            metrics.control_height,
        );
        action_x += locate_width;
    } else {
        hide_node(nodes, "LocateSelectedAsset", action_x, row_y);
    }
    if locate_visible && import.visible {
        action_x += metrics.group_gap;
    }
    layout_filter_group_frame(nodes, filter_x, row_y, filter_right_edge, metrics);

    hide_node(nodes, "AssetBrowserImportLabel", action_x, row_y);
    layout_import_group(nodes, action_x, row_y, import, metrics);
}

fn collapse_trailing_actions_to_fit(
    row_width: f32,
    minimum_leading_width: f32,
    view: &mut CompactViewGroup,
    locate_visible: &mut bool,
    import: &mut CompactImportGroup,
    locate_width: f32,
    metrics: AssetBrowserToolbarMetrics,
) {
    while trailing_actions_width(view, *locate_visible, import, locate_width, metrics)
        + minimum_leading_width
        + metrics.group_gap
        > row_width
    {
        if view.visible {
            view.visible = false;
        } else if *locate_visible {
            *locate_visible = false;
        } else if import.visible {
            import.visible = false;
        } else {
            break;
        }
    }
}

fn trailing_actions_width(
    view: &CompactViewGroup,
    locate_visible: bool,
    import: &CompactImportGroup,
    locate_width: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> f32 {
    let mut width = 0.0;
    for action_width in [
        view.visible.then_some(view.width),
        locate_visible.then_some(locate_width),
        import.visible.then_some(import.width),
    ] {
        let Some(action_width) = action_width else {
            continue;
        };
        if width > 0.0 {
            width += metrics.group_gap;
        }
        width += action_width;
    }
    width
}

fn layout_kind_chips(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width_limit: f32,
    metrics: AssetBrowserToolbarMetrics,
) {
    let selection = select_visible_kind_chips(nodes, width_limit, metrics);

    let mut cursor_x = x;
    let mut has_visible_chip = false;
    let mut used_width = 0.0;
    for (index, &(control_id, _)) in KIND_CHIPS.iter().enumerate() {
        if !selection.visible[index] {
            hide_node(nodes, control_id, x + width_limit, y);
            continue;
        }
        let chip_width = selection.widths[index];
        if !chip_fits_in_width(
            used_width,
            has_visible_chip,
            chip_width,
            width_limit,
            metrics,
        ) {
            hide_node(nodes, control_id, x + width_limit, y);
            continue;
        }
        if has_visible_chip {
            cursor_x += metrics.row_gap;
            used_width += metrics.row_gap;
        }
        set_node_frame(
            nodes,
            control_id,
            cursor_x,
            y,
            chip_width,
            metrics.control_height,
        );
        cursor_x += chip_width;
        used_width += chip_width;
        has_visible_chip = true;
    }
}

struct KindChipSelection {
    visible: [bool; KIND_CHIP_COUNT],
    widths: [f32; KIND_CHIP_COUNT],
}

fn select_visible_kind_chips(
    nodes: &[ViewTemplateNodeData],
    width_limit: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> KindChipSelection {
    let states: [(f32, bool); KIND_CHIP_COUNT] = std::array::from_fn(|index| {
        let (control_id, fallback_width) = KIND_CHIPS[index];
        kind_chip_state(nodes, control_id, fallback_width)
    });
    let widths: [f32; KIND_CHIP_COUNT] = std::array::from_fn(|index| states[index].0);
    let selected_chip = states.iter().position(|(_, selected)| *selected);
    let mut visible = [false; KIND_CHIP_COUNT];
    let mut visible_width = 0.0;
    let mut visible_count = 0;

    for index in 0..KIND_CHIP_COUNT {
        if index == 0 || Some(index) == selected_chip {
            if visible_count > 0 {
                visible_width += metrics.row_gap;
            }
            visible[index] = true;
            visible_width += widths[index];
            visible_count += 1;
        }
    }
    for index in 0..KIND_CHIP_COUNT {
        if visible[index] {
            continue;
        }
        let leading_gap = if visible_count > 0 {
            metrics.row_gap
        } else {
            0.0
        };
        let candidate_width = visible_width + leading_gap + widths[index];
        if candidate_width <= width_limit {
            visible[index] = true;
            visible_width = candidate_width;
            visible_count += 1;
        }
    }

    KindChipSelection { visible, widths }
}

fn kind_chip_state(
    nodes: &[ViewTemplateNodeData],
    control_id: &str,
    fallback_width: f32,
) -> (f32, bool) {
    let Some(node) = nodes.iter().find(|node| node.control_id == control_id) else {
        return (fallback_width, false);
    };
    let width = (node.frame.width > 0.0)
        .then_some(node.frame.width)
        .unwrap_or(fallback_width);
    (width, node.selected)
}

fn chip_fits_in_width(
    used_width: f32,
    has_visible_chip: bool,
    chip_width: f32,
    width_limit: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> bool {
    let leading_gap = if has_visible_chip {
        metrics.row_gap
    } else {
        0.0
    };
    used_width + leading_gap + chip_width <= width_limit.max(0.0)
}

fn compact_view_group(
    nodes: &[ViewTemplateNodeData],
    metrics: AssetBrowserToolbarMetrics,
) -> CompactViewGroup {
    let list_width = control_width(
        nodes,
        "AssetBrowserViewModeListButton",
        metrics.compact_icon_width,
    );
    let thumb_width = control_width(
        nodes,
        "AssetBrowserViewModeThumbButton",
        metrics.compact_icon_width,
    );
    CompactViewGroup {
        visible: true,
        list_width,
        thumb_width,
        width: list_width + metrics.view_button_gap + thumb_width,
    }
}

fn compact_import_group(
    nodes: &[ViewTemplateNodeData],
    row_width: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> CompactImportGroup {
    let button_width = control_width(nodes, "ImportModel", 96.0)
        .min((row_width * 0.14).max(COMPACT_IMPORT_BUTTON_MIN_WIDTH))
        .min(row_width);
    let visible = button_width >= COMPACT_IMPORT_BUTTON_MIN_WIDTH;
    let show_path = row_width >= COMPACT_IMPORT_PATH_VISIBLE_WIDTH;
    let path_width = if visible && show_path {
        (row_width * 0.26)
            .max(COMPACT_IMPORT_PATH_MIN_WIDTH)
            .min(COMPACT_IMPORT_PATH_MAX_WIDTH)
            .min((row_width - button_width - metrics.group_gap).max(0.0))
    } else {
        0.0
    };
    let width = if !visible {
        0.0
    } else if path_width > 0.0 {
        path_width + metrics.group_gap + button_width
    } else {
        button_width
    };
    CompactImportGroup {
        visible,
        path_visible: path_width > 0.0,
        path_width,
        button_width,
        width,
    }
}

fn layout_import_group(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    group: CompactImportGroup,
    metrics: AssetBrowserToolbarMetrics,
) {
    if !group.visible {
        hide_node(nodes, "AssetBrowserImportPathField", x, y);
        hide_node(nodes, "ImportModel", x, y);
    } else if group.path_visible {
        set_node_frame(
            nodes,
            "AssetBrowserImportPathField",
            x,
            y,
            group.path_width,
            metrics.control_height,
        );
        set_node_frame(
            nodes,
            "ImportModel",
            x + group.path_width + metrics.group_gap,
            y,
            group.button_width,
            metrics.control_height,
        );
    } else {
        hide_node(nodes, "AssetBrowserImportPathField", x, y);
        set_node_frame(
            nodes,
            "ImportModel",
            x,
            y,
            group.button_width,
            metrics.control_height,
        );
    }
}

fn layout_filter_group_frame(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    right_edge: f32,
    metrics: AssetBrowserToolbarMetrics,
) {
    let group_x = x - metrics.group_frame_pad;
    let group_y = y - metrics.control_offset_y;
    let group_width = (right_edge - x + metrics.group_frame_pad * 2.0).max(0.0);
    set_node_frame(
        nodes,
        "AssetBrowserToolbarKindPrimaryRow",
        group_x,
        group_y,
        group_width,
        metrics.toolbar_height,
    );
}

fn asset_browser_toolbar_metrics() -> AssetBrowserToolbarMetrics {
    asset_browser_toolbar_metrics_from_tokens(
        EditorDensityTokens::workbench_dense(),
        EditorControlTokens::workbench_dense(),
    )
}

fn asset_browser_toolbar_metrics_from_tokens(
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) -> AssetBrowserToolbarMetrics {
    let toolbar_height = density.row_height + density.gap_medium;
    let control_offset_y = controls.border_width;
    AssetBrowserToolbarMetrics {
        toolbar_height,
        control_height: (toolbar_height - controls.border_width * 2.0).max(controls.border_width),
        compact_icon_width: controls.compact_height,
        control_offset_y,
        side_pad: density.gap_medium,
        root_gap: (density.gap_medium - controls.border_width * 2.0).max(0.0),
        group_gap: density.gap_medium,
        group_frame_pad: (density.gap_small - controls.border_width).max(0.0),
        row_gap: density.gap_small,
        view_button_gap: density.gap_small,
    }
}

struct CompactViewGroup {
    visible: bool,
    list_width: f32,
    thumb_width: f32,
    width: f32,
}

#[derive(Clone, Copy)]
struct CompactImportGroup {
    visible: bool,
    path_visible: bool,
    path_width: f32,
    button_width: f32,
    width: f32,
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
}

fn has_control(nodes: &[ViewTemplateNodeData], control_id: &str) -> bool {
    nodes.iter().any(|node| node.control_id == control_id)
}

fn control_width(nodes: &[ViewTemplateNodeData], control_id: &str, fallback: f32) -> f32 {
    node_frame(nodes, control_id)
        .map(|frame| frame.width)
        .filter(|width| *width > 0.0)
        .unwrap_or(fallback)
}

#[cfg(test)]
fn is_selected(nodes: &[ViewTemplateNodeData], control_id: &str) -> bool {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.selected)
        .unwrap_or(false)
}

fn hide_node(nodes: &mut [ViewTemplateNodeData], control_id: &str, x: f32, y: f32) {
    set_node_frame(nodes, control_id, x, y, 0.0, 0.0);
}

fn set_node_frame(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id == control_id)
    {
        node.frame.x = x;
        node.frame.y = y;
        node.frame.width = width.max(0.0);
        node.frame.height = height.max(0.0);
    }
}

const KIND_CHIPS: &[(&str, f32)] = &[
    ("AssetBrowserKindAllChip", 44.0),
    ("AssetBrowserKindTextureChip", 78.0),
    ("AssetBrowserKindMaterialChip", 84.0),
    ("AssetBrowserKindSceneChip", 64.0),
    ("AssetBrowserKindModelChip", 64.0),
    ("AssetBrowserKindShaderChip", 72.0),
];
const KIND_CHIP_COUNT: usize = KIND_CHIPS.len();

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::{Duration, Instant};
    use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

    #[test]
    fn asset_browser_toolbar_metrics_project_from_dense_design_tokens() {
        let mut density = EditorDensityTokens::workbench_dense();
        density.row_height = 28.0;
        density.gap_small = 5.0;
        density.gap_medium = 10.0;
        let mut controls = EditorControlTokens::workbench_dense();
        controls.border_width = 2.0;
        controls.compact_height = 31.0;

        let metrics = asset_browser_toolbar_metrics_from_tokens(density, controls);

        assert_eq!(metrics.toolbar_height, 38.0);
        assert_eq!(metrics.control_height, 34.0);
        assert_eq!(metrics.control_offset_y, 2.0);
        assert_eq!(metrics.compact_icon_width, 31.0);
        assert_eq!(metrics.side_pad, 10.0);
        assert_eq!(metrics.root_gap, 6.0);
        assert_eq!(metrics.group_gap, 10.0);
        assert_eq!(metrics.group_frame_pad, 3.0);
        assert_eq!(metrics.row_gap, 5.0);
        assert_eq!(metrics.view_button_gap, 5.0);

        assert!(!chip_fits_in_width(0.0, false, 44.0, 0.0, metrics));
        assert!(chip_fits_in_width(0.0, false, 44.0, 44.0, metrics));
        assert!(!chip_fits_in_width(44.0, true, 78.0, 126.0, metrics));
        assert!(chip_fits_in_width(44.0, true, 78.0, 127.0, metrics));
    }

    #[test]
    fn linear_kind_chip_selection_preserves_legacy_visibility() {
        let metrics = asset_browser_toolbar_metrics();
        for selected in [None, Some("AssetBrowserKindShaderChip")] {
            for missing_last_chip in [false, true] {
                let mut nodes = kind_chip_fixture(selected, 0);
                if missing_last_chip {
                    nodes.pop();
                }
                for width_limit in [0.0, 44.0, 130.0, 260.0, 1_000.0] {
                    assert_eq!(
                        select_visible_kind_chips(&nodes, width_limit, metrics).visible,
                        legacy_visible_kind_chips(&nodes, width_limit, metrics),
                        "visibility must match for selected={selected:?}, missing_last_chip={missing_last_chip}, width={width_limit}"
                    );
                }
            }
        }
    }

    #[test]
    fn linear_kind_chip_selection_avoids_candidate_clones_and_repeated_width_scans() {
        let source = include_str!("toolbar_layout.rs");
        let implementation = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(!implementation.contains("let mut candidate = visible.clone()"));
        assert!(!implementation.contains("fn chip_stack_width("));
        assert!(implementation.contains("std::array::from_fn"));
        assert!(implementation.contains("visible: [bool; KIND_CHIP_COUNT]"));
    }

    #[test]
    #[ignore = "release-only asset toolbar chip selection performance gate"]
    fn linear_kind_chip_selection_release_benchmark() {
        const SAMPLE_COUNT: usize = 11;
        const ITERATIONS_PER_SAMPLE: usize = 8_192;
        const PREFIX_NODE_COUNT: usize = 64;
        const MAX_OPTIMIZED_TO_LEGACY_PERCENT: u128 = 40;

        let nodes = kind_chip_fixture(None, PREFIX_NODE_COUNT);
        let metrics = asset_browser_toolbar_metrics();
        let width_limit = 1_000.0;
        black_box(select_visible_kind_chips(&nodes, width_limit, metrics));
        black_box(legacy_visible_kind_chips(&nodes, width_limit, metrics));

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                legacy_samples.push(measure_kind_chip_selection(
                    &nodes,
                    metrics,
                    width_limit,
                    ITERATIONS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_kind_chip_selection(
                    &nodes,
                    metrics,
                    width_limit,
                    ITERATIONS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_kind_chip_selection(
                    &nodes,
                    metrics,
                    width_limit,
                    ITERATIONS_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_kind_chip_selection(
                    &nodes,
                    metrics,
                    width_limit,
                    ITERATIONS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95_ns = duration_p95_ns(legacy_samples);
        let optimized_p95_ns = duration_p95_ns(optimized_samples);
        let reduction_basis_points = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / legacy_p95_ns.max(1);
        println!(
            "EDITOR57_LINEAR_KIND_CHIP_SELECTION_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} reduction_basis_points={reduction_basis_points} samples={SAMPLE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} prefix_nodes={PREFIX_NODE_COUNT} chips={KIND_CHIP_COUNT} node_searches_per_selection=32->6 candidate_vec_clones_per_selection=5->0"
        );
        assert!(
            optimized_p95_ns.saturating_mul(100)
                <= legacy_p95_ns.saturating_mul(MAX_OPTIMIZED_TO_LEGACY_PERCENT),
            "optimized P95 {optimized_p95_ns}ns must be at most {MAX_OPTIMIZED_TO_LEGACY_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn measure_kind_chip_selection(
        nodes: &[ViewTemplateNodeData],
        metrics: AssetBrowserToolbarMetrics,
        width_limit: f32,
        iterations: usize,
        optimized: bool,
    ) -> Duration {
        let started = Instant::now();
        for _ in 0..iterations {
            if optimized {
                black_box(select_visible_kind_chips(
                    black_box(nodes),
                    width_limit,
                    metrics,
                ));
            } else {
                black_box(legacy_visible_kind_chips(
                    black_box(nodes),
                    width_limit,
                    metrics,
                ));
            }
        }
        started.elapsed()
    }

    fn duration_p95_ns(mut samples: Vec<Duration>) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index].as_nanos()
    }

    fn legacy_visible_kind_chips(
        nodes: &[ViewTemplateNodeData],
        width_limit: f32,
        metrics: AssetBrowserToolbarMetrics,
    ) -> [bool; KIND_CHIP_COUNT] {
        let selected_chip = KIND_CHIPS
            .iter()
            .find(|(control_id, _)| is_selected(nodes, control_id))
            .map(|(control_id, _)| *control_id);
        let mut visible = Vec::new();
        for &(control_id, _) in KIND_CHIPS {
            if control_id == "AssetBrowserKindAllChip" || Some(control_id) == selected_chip {
                visible.push(control_id);
            }
        }
        for &(control_id, _) in KIND_CHIPS {
            if visible.contains(&control_id) {
                continue;
            }
            let mut candidate = visible.clone();
            candidate.push(control_id);
            if legacy_chip_stack_width(nodes, &candidate, metrics) <= width_limit {
                visible.push(control_id);
            }
        }

        std::array::from_fn(|index| visible.contains(&KIND_CHIPS[index].0))
    }

    fn legacy_chip_stack_width(
        nodes: &[ViewTemplateNodeData],
        control_ids: &[&str],
        metrics: AssetBrowserToolbarMetrics,
    ) -> f32 {
        let mut width = 0.0;
        let mut visible_count = 0;
        for &(control_id, fallback_width) in KIND_CHIPS {
            if control_ids.contains(&control_id) {
                if visible_count > 0 {
                    width += metrics.row_gap;
                }
                width += control_width(nodes, control_id, fallback_width);
                visible_count += 1;
            }
        }
        width
    }

    fn kind_chip_fixture(
        selected: Option<&str>,
        prefix_node_count: usize,
    ) -> Vec<ViewTemplateNodeData> {
        let mut nodes = (0..prefix_node_count)
            .map(|index| ViewTemplateNodeData {
                control_id: format!("FixturePrefix{index:03}").into(),
                ..ViewTemplateNodeData::default()
            })
            .collect::<Vec<_>>();
        nodes.extend(KIND_CHIPS.iter().map(|&(control_id, width)| {
            let mut node = ViewTemplateNodeData {
                control_id: control_id.into(),
                selected: selected == Some(control_id),
                ..ViewTemplateNodeData::default()
            };
            node.frame.width = width;
            node
        }));
        nodes
    }
}
