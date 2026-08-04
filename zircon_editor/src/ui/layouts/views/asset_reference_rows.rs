use std::collections::BTreeMap;

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::snapshot::{AssetReferenceSnapshot, AssetWorkspaceSnapshot};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};

#[derive(Clone, Copy)]
pub(super) struct AssetReferenceListControls {
    pub(super) title_control_id: &'static str,
    pub(super) empty_control_id: &'static str,
    pub(super) panel_control_id: &'static str,
    pub(super) scroll_body_control_id: &'static str,
    pub(super) row_panel_control_id: &'static str,
    pub(super) row_name_control_id: &'static str,
    pub(super) row_locator_control_id: &'static str,
    pub(super) row_kind_control_id: &'static str,
    pub(super) node_id_scope: &'static str,
    pub(super) title: &'static str,
    pub(super) empty_text: &'static str,
}

#[derive(Clone, Copy)]
struct AssetReferenceListMetrics {
    header_height: f32,
    panel_gap: f32,
    min_column_width: f32,
    row_height: f32,
    row_gap: f32,
    text_inset: f32,
    text_top_inset: f32,
    text_line_height: f32,
    caption_font_size: f32,
    kind_width: f32,
    kind_max_width_fraction: f32,
}

fn asset_reference_list_metrics() -> AssetReferenceListMetrics {
    let density = EditorDensityTokens::workbench_dense();
    let controls = EditorControlTokens::workbench_dense();
    let typography = EditorTypographyTokens::workbench_default();
    AssetReferenceListMetrics {
        header_height: density.gap_medium * 2.0 + density.gap_small,
        panel_gap: density.gap_medium,
        min_column_width: controls.default_height * 5.0,
        row_height: density.row_height + density.gap_small + controls.border_width * 2.0,
        row_gap: density.gap_small,
        text_inset: density.gap_medium,
        text_top_inset: density.gap_small,
        text_line_height: typography.caption_size * typography.line_height,
        caption_font_size: typography.caption_size,
        kind_width: controls.dense_height * 2.0,
        kind_max_width_fraction: 0.4,
    }
}

pub(super) fn sync_asset_reference_lists(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
    left: AssetReferenceListControls,
    right: AssetReferenceListControls,
) {
    sync_asset_reference_list(nodes, left, &snapshot.selection.references);
    sync_asset_reference_list(nodes, right, &snapshot.selection.used_by);
}

pub(super) fn apply_asset_reference_lists_layout(
    nodes: &mut [ViewTemplateNodeData],
    content_control_id: &str,
    left: AssetReferenceListControls,
    right: AssetReferenceListControls,
) {
    let Some(content) = node_frame(nodes, content_control_id) else {
        return;
    };
    let metrics = asset_reference_list_metrics();
    let content = frame(content.x, content.y, content.width, content.height);
    let split_columns = content.width >= metrics.min_column_width * 2.0 + metrics.panel_gap;
    if split_columns {
        let column_width = finite_non_negative((content.width - metrics.panel_gap) / 2.0);
        apply_asset_reference_list_layout(
            nodes,
            left,
            frame(content.x, content.y, column_width, content.height),
            metrics,
        );
        apply_asset_reference_list_layout(
            nodes,
            right,
            frame(
                content.x + column_width + metrics.panel_gap,
                content.y,
                column_width,
                content.height,
            ),
            metrics,
        );
        return;
    }

    let stacked_height = finite_non_negative((content.height - metrics.panel_gap) / 2.0);
    apply_asset_reference_list_layout(
        nodes,
        left,
        frame(content.x, content.y, content.width, stacked_height),
        metrics,
    );
    apply_asset_reference_list_layout(
        nodes,
        right,
        frame(
            content.x,
            content.y + stacked_height + metrics.panel_gap,
            content.width,
            stacked_height,
        ),
        metrics,
    );
}

fn sync_asset_reference_list(
    nodes: &mut Vec<ViewTemplateNodeData>,
    controls: AssetReferenceListControls,
    references: &[AssetReferenceSnapshot],
) {
    let Some(prototypes) = AssetReferenceNodePrototypes::from_nodes(nodes, controls) else {
        return;
    };

    nodes.retain(|node| !is_dynamic_reference_row_component(controls, node.control_id.as_str()));
    hide_reference_row_prototypes(nodes, controls);
    if let Some(title) = find_node_mut(nodes, controls.title_control_id) {
        title.text = if references.is_empty() {
            controls.title.into()
        } else {
            format!("{} ({})", controls.title, references.len()).into()
        };
    }

    if references.is_empty() {
        if let Some(empty) = find_node_mut(nodes, controls.empty_control_id) {
            empty.text = controls.empty_text.into();
        }
        return;
    }

    if let Some(empty) = find_node_mut(nodes, controls.empty_control_id) {
        empty.text = "".into();
    }
    for (index, reference) in references.iter().enumerate() {
        nodes.extend(prototypes.row_nodes(controls, index + 1, reference));
    }
}

fn apply_asset_reference_list_layout(
    nodes: &mut [ViewTemplateNodeData],
    controls: AssetReferenceListControls,
    panel: ViewTemplateFrameData,
    metrics: AssetReferenceListMetrics,
) {
    let panel = frame(panel.x, panel.y, panel.width, panel.height);
    let header_height = metrics.header_height.min(panel.height);
    let scroll = frame(
        panel.x,
        panel.y + header_height,
        panel.width,
        finite_non_negative(panel.height - header_height),
    );
    set_node_frame(nodes, controls.panel_control_id, panel.clone());
    set_node_frame(
        nodes,
        controls.title_control_id,
        frame(panel.x, panel.y, panel.width, header_height),
    );
    set_node_frame(nodes, controls.scroll_body_control_id, scroll.clone());
    set_node_frame(
        nodes,
        controls.empty_control_id,
        frame(
            panel.x + metrics.text_inset.min(panel.width),
            scroll.y + metrics.text_top_inset.min(scroll.height),
            finite_non_negative(panel.width - metrics.text_inset * 2.0),
            metrics
                .text_line_height
                .min(finite_non_negative(scroll.height - metrics.text_top_inset)),
        ),
    );

    if scroll.height <= f32::EPSILON {
        for node in nodes.iter_mut() {
            if reference_row_index(controls, node.control_id.as_str()).is_some() {
                node.frame = frame(panel.x, scroll.y, panel.width, 0.0);
            }
        }
        return;
    }

    let row_width = finite_non_negative(panel.width - metrics.row_gap);
    let kind_widths = reference_kind_slot_widths(nodes, controls, row_width, metrics);
    for node in nodes.iter_mut() {
        let Some(index) = reference_row_index(controls, node.control_id.as_str()) else {
            continue;
        };
        let row_y = scroll.y + index as f32 * (metrics.row_height + metrics.row_gap);
        let control_id = node.control_id.as_str();
        let kind_width = kind_widths
            .get(&index)
            .copied()
            .unwrap_or_else(|| reference_kind_slot_width("Unknown", row_width, metrics));
        let text_width = finite_non_negative(row_width - metrics.text_inset * 2.0 - kind_width);
        node.frame = if control_id.starts_with(controls.row_panel_control_id) {
            frame(panel.x, row_y, row_width, metrics.row_height)
        } else if control_id.starts_with(controls.row_name_control_id) {
            frame(
                panel.x + metrics.text_inset.min(row_width),
                row_y + metrics.text_top_inset,
                text_width,
                metrics.text_line_height,
            )
        } else if control_id.starts_with(controls.row_locator_control_id) {
            frame(
                panel.x + metrics.text_inset.min(row_width),
                row_y + metrics.text_top_inset + metrics.text_line_height + metrics.row_gap / 2.0,
                text_width,
                metrics.text_line_height,
            )
        } else {
            frame(
                panel.x + finite_non_negative(row_width - metrics.text_inset - kind_width),
                row_y + finite_non_negative((metrics.row_height - metrics.text_line_height) / 2.0),
                kind_width.min(row_width),
                metrics.text_line_height,
            )
        };
    }
}

struct AssetReferenceNodePrototypes {
    panel: ViewTemplateNodeData,
    name: ViewTemplateNodeData,
    locator: ViewTemplateNodeData,
    kind: ViewTemplateNodeData,
}

impl AssetReferenceNodePrototypes {
    fn from_nodes(
        nodes: &[ViewTemplateNodeData],
        controls: AssetReferenceListControls,
    ) -> Option<Self> {
        Some(Self {
            panel: find_node(nodes, controls.row_panel_control_id)?.clone(),
            name: find_node(nodes, controls.row_name_control_id)?.clone(),
            locator: find_node(nodes, controls.row_locator_control_id)?.clone(),
            kind: find_node(nodes, controls.row_kind_control_id)?.clone(),
        })
    }

    fn row_nodes(
        &self,
        controls: AssetReferenceListControls,
        index: usize,
        reference: &AssetReferenceSnapshot,
    ) -> [ViewTemplateNodeData; 4] {
        let mut panel = self.panel.clone();
        panel.node_id = format!("{}.row_{index:02}", controls.node_id_scope).into();
        panel.control_id = indexed_control_id(controls.row_panel_control_id, index).into();
        panel.value_text = reference.uuid.clone().into();
        panel.selected = false;
        panel.focused = false;
        panel.hovered = false;
        panel.pressed = false;

        let mut name = self.name.clone();
        name.node_id = format!("{}.row_{index:02}.name", controls.node_id_scope).into();
        name.control_id = indexed_control_id(controls.row_name_control_id, index).into();
        name.text = reference_display_name(reference).into();

        let mut locator = self.locator.clone();
        locator.node_id = format!("{}.row_{index:02}.locator", controls.node_id_scope).into();
        locator.control_id = indexed_control_id(controls.row_locator_control_id, index).into();
        locator.text = reference_locator(reference).into();

        let mut kind = self.kind.clone();
        kind.node_id = format!("{}.row_{index:02}.kind", controls.node_id_scope).into();
        kind.control_id = indexed_control_id(controls.row_kind_control_id, index).into();
        kind.text = reference_kind_label(reference).into();

        [panel, name, locator, kind]
    }
}

fn is_dynamic_reference_row_component(
    controls: AssetReferenceListControls,
    control_id: &str,
) -> bool {
    row_control_ids(controls)
        .into_iter()
        .any(|prefix| indexed_control_id_suffix(control_id, prefix).is_some())
}

fn hide_reference_row_prototypes(
    nodes: &mut [ViewTemplateNodeData],
    controls: AssetReferenceListControls,
) {
    let prototype_control_ids = row_control_ids(controls);
    for node in nodes
        .iter_mut()
        .filter(|node| prototype_control_ids.contains(&node.control_id.as_str()))
    {
        node.frame = ViewTemplateFrameData::default();
        node.text = "".into();
        node.value_text = "".into();
        node.selected = false;
        node.focused = false;
        node.hovered = false;
        node.pressed = false;
    }
}

fn reference_row_index(controls: AssetReferenceListControls, control_id: &str) -> Option<usize> {
    row_control_ids(controls)
        .into_iter()
        .find_map(|prefix| indexed_control_id_suffix(control_id, prefix))
}

fn row_control_ids(controls: AssetReferenceListControls) -> [&'static str; 4] {
    [
        controls.row_panel_control_id,
        controls.row_name_control_id,
        controls.row_locator_control_id,
        controls.row_kind_control_id,
    ]
}

fn indexed_control_id(prefix: &str, index: usize) -> String {
    format!("{prefix}{index:02}")
}

fn indexed_control_id_suffix(control_id: &str, prefix: &str) -> Option<usize> {
    let index = control_id.strip_prefix(prefix)?.parse::<usize>().ok()?;
    index.checked_sub(1)
}

fn reference_kind_slot_widths(
    nodes: &[ViewTemplateNodeData],
    controls: AssetReferenceListControls,
    row_width: f32,
    metrics: AssetReferenceListMetrics,
) -> BTreeMap<usize, f32> {
    let mut widths = BTreeMap::new();
    for node in nodes {
        if !node.control_id.starts_with(controls.row_kind_control_id) {
            continue;
        }
        let Some(index) = reference_row_index(controls, node.control_id.as_str()) else {
            continue;
        };
        widths.insert(
            index,
            reference_kind_slot_width(node.text.as_str(), row_width, metrics),
        );
    }
    widths
}

fn reference_kind_slot_width(
    label: &str,
    row_width: f32,
    metrics: AssetReferenceListMetrics,
) -> f32 {
    let text_budget = finite_non_negative(row_width - metrics.text_inset * 2.0);
    let max_kind_width = text_budget * metrics.kind_max_width_fraction;
    let measured_width =
        measure_runtime_text_width(label, metrics.caption_font_size) + metrics.text_inset;
    metrics.kind_width.max(measured_width).min(max_kind_width)
}

fn reference_display_name(reference: &AssetReferenceSnapshot) -> &str {
    if reference.display_name.trim().is_empty() {
        reference.locator.as_str()
    } else {
        reference.display_name.as_str()
    }
}

fn reference_locator(reference: &AssetReferenceSnapshot) -> &str {
    if reference.locator.trim().is_empty() {
        reference.uuid.as_str()
    } else {
        reference.locator.as_str()
    }
}

fn reference_kind_label(reference: &AssetReferenceSnapshot) -> &str {
    reference
        .asset_type
        .as_ref()
        .map(|asset_type| asset_type.display_name.as_str())
        .filter(|display_name| !display_name.is_empty())
        .unwrap_or("Unknown")
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    find_node(nodes, control_id).map(|node| node.frame.clone())
}

fn find_node<'a>(
    nodes: &'a [ViewTemplateNodeData],
    control_id: &str,
) -> Option<&'a ViewTemplateNodeData> {
    nodes.iter().find(|node| node.control_id == control_id)
}

fn find_node_mut<'a>(
    nodes: &'a mut [ViewTemplateNodeData],
    control_id: &str,
) -> Option<&'a mut ViewTemplateNodeData> {
    nodes.iter_mut().find(|node| node.control_id == control_id)
}

fn set_node_frame(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    frame: ViewTemplateFrameData,
) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id == control_id)
    {
        node.frame = frame.clone();
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> ViewTemplateFrameData {
    ViewTemplateFrameData {
        x: finite_coordinate(x),
        y: finite_coordinate(y),
        width: finite_non_negative(width),
        height: finite_non_negative(height),
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::{
        AssetReferenceListControls, apply_asset_reference_lists_layout,
        asset_reference_list_metrics, reference_kind_slot_width, sync_asset_reference_lists,
    };
    use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
    use crate::ui::workbench::snapshot::{
        AssetReferenceSnapshot, AssetSelectionSnapshot, AssetWorkspaceSnapshot,
    };
    use zircon_runtime_interface::ui::design_tokens::{
        EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
    };

    const LEFT: AssetReferenceListControls = AssetReferenceListControls {
        title_control_id: "LeftTitleText",
        empty_control_id: "LeftEmptyText",
        panel_control_id: "LeftPanel",
        scroll_body_control_id: "LeftScrollBody",
        row_panel_control_id: "LeftRowPanel",
        row_name_control_id: "LeftRowNameText",
        row_locator_control_id: "LeftRowLocatorText",
        row_kind_control_id: "LeftRowKindText",
        node_id_scope: "test.references.left",
        title: "References",
        empty_text: "No direct references",
    };
    const RIGHT: AssetReferenceListControls = AssetReferenceListControls {
        title_control_id: "RightTitleText",
        empty_control_id: "RightEmptyText",
        panel_control_id: "RightPanel",
        scroll_body_control_id: "RightScrollBody",
        row_panel_control_id: "RightRowPanel",
        row_name_control_id: "RightRowNameText",
        row_locator_control_id: "RightRowLocatorText",
        row_kind_control_id: "RightRowKindText",
        node_id_scope: "test.references.right",
        title: "Used By",
        empty_text: "No usages",
    };

    #[test]
    fn reference_list_metrics_follow_shared_component_tokens() {
        let metrics = asset_reference_list_metrics();
        let density = EditorDensityTokens::workbench_dense();
        let controls = EditorControlTokens::workbench_dense();
        let typography = EditorTypographyTokens::workbench_default();

        assert_eq!(metrics.panel_gap, density.gap_medium);
        assert_eq!(metrics.min_column_width, controls.default_height * 5.0);
        assert_eq!(
            metrics.row_height,
            density.row_height + density.gap_small + controls.border_width * 2.0
        );
        assert_eq!(
            metrics.text_line_height,
            typography.caption_size * typography.line_height
        );
    }

    #[test]
    fn dynamic_reference_rows_resync_from_retained_prototypes() {
        let mut nodes = prototypes();
        let initial = snapshot(
            vec![reference("first", "First", "Content/First")],
            vec![reference("used", "Used", "Content/Used")],
        );
        let refreshed = snapshot(
            vec![
                reference("second", "Second", "Content/Second"),
                reference("third", "Third", "Content/Third"),
            ],
            Vec::new(),
        );

        sync_asset_reference_lists(&mut nodes, &initial, LEFT, RIGHT);
        sync_asset_reference_lists(&mut nodes, &refreshed, LEFT, RIGHT);

        assert_eq!(text(&nodes, "LeftTitleText"), "References (2)");
        assert_eq!(text(&nodes, "LeftRowNameText02"), "Third");
        assert!(node(&nodes, "RightRowPanel01").is_none());
        assert_eq!(text(&nodes, "RightEmptyText"), "No usages");
        let left_prototype = node(&nodes, "LeftRowPanel").expect("retained prototype");
        assert_eq!(left_prototype.frame.width, 0.0);
        assert_eq!(left_prototype.frame.height, 0.0);
        assert!(left_prototype.text.is_empty());
    }

    #[test]
    fn reference_lists_use_columns_then_stack_for_narrow_content() {
        let mut nodes = prototypes();
        nodes.push(frame_node("ReferenceContent", 20.0, 40.0, 520.0, 132.0));
        sync_asset_reference_lists(
            &mut nodes,
            &snapshot(
                vec![reference("left", "Left", "Content/Left")],
                vec![reference("right", "Right", "Content/Right")],
            ),
            LEFT,
            RIGHT,
        );

        apply_asset_reference_lists_layout(&mut nodes, "ReferenceContent", LEFT, RIGHT);
        assert_eq!(node(&nodes, "LeftPanel").expect("left").frame.width, 256.0);
        assert_eq!(node(&nodes, "RightPanel").expect("right").frame.x, 284.0);

        node_mut(&mut nodes, "ReferenceContent").frame.width = 300.0;
        apply_asset_reference_lists_layout(&mut nodes, "ReferenceContent", LEFT, RIGHT);
        assert!(
            node(&nodes, "RightPanel").expect("stacked right").frame.y
                > node(&nodes, "LeftPanel").expect("stacked left").frame.y
        );
    }

    #[test]
    fn kind_slot_uses_runtime_text_width_with_a_relative_budget_cap() {
        let metrics = asset_reference_list_metrics();
        let label = "W".repeat(64);
        let narrow_width = 80.0;
        let wide_width = 320.0;
        let narrow = reference_kind_slot_width(&label, narrow_width, metrics);
        let wide = reference_kind_slot_width(&label, wide_width, metrics);
        assert_eq!(
            narrow,
            (narrow_width - metrics.text_inset * 2.0) * metrics.kind_max_width_fraction
        );
        assert_eq!(
            wide,
            (wide_width - metrics.text_inset * 2.0) * metrics.kind_max_width_fraction
        );
        assert!(wide > narrow);
    }

    fn prototypes() -> Vec<ViewTemplateNodeData> {
        [LEFT, RIGHT]
            .into_iter()
            .flat_map(|controls| {
                [
                    controls.title_control_id,
                    controls.empty_control_id,
                    controls.panel_control_id,
                    controls.scroll_body_control_id,
                    controls.row_panel_control_id,
                    controls.row_name_control_id,
                    controls.row_locator_control_id,
                    controls.row_kind_control_id,
                ]
            })
            .map(|control_id| ViewTemplateNodeData {
                node_id: control_id.into(),
                control_id: control_id.into(),
                ..ViewTemplateNodeData::default()
            })
            .collect()
    }

    fn snapshot(
        references: Vec<AssetReferenceSnapshot>,
        used_by: Vec<AssetReferenceSnapshot>,
    ) -> AssetWorkspaceSnapshot {
        AssetWorkspaceSnapshot {
            selection: AssetSelectionSnapshot {
                references,
                used_by,
                ..AssetSelectionSnapshot::default()
            },
            ..AssetWorkspaceSnapshot::default()
        }
    }

    fn reference(uuid: &str, display_name: &str, locator: &str) -> AssetReferenceSnapshot {
        AssetReferenceSnapshot {
            uuid: uuid.to_string(),
            display_name: display_name.to_string(),
            locator: locator.to_string(),
            ..AssetReferenceSnapshot::default()
        }
    }

    fn frame_node(
        control_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            node_id: control_id.into(),
            control_id: control_id.into(),
            frame: ViewTemplateFrameData {
                x,
                y,
                width,
                height,
            },
            ..ViewTemplateNodeData::default()
        }
    }

    fn node<'a>(
        nodes: &'a [ViewTemplateNodeData],
        control_id: &str,
    ) -> Option<&'a ViewTemplateNodeData> {
        nodes.iter().find(|node| node.control_id == control_id)
    }

    fn node_mut<'a>(
        nodes: &'a mut [ViewTemplateNodeData],
        control_id: &str,
    ) -> &'a mut ViewTemplateNodeData {
        nodes
            .iter_mut()
            .find(|node| node.control_id == control_id)
            .expect("test node")
    }

    fn text(nodes: &[ViewTemplateNodeData], control_id: &str) -> &str {
        node(nodes, control_id).expect("text node").text.as_str()
    }
}
