use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

const SUMMARY_CARD_INSET_X: f32 = 8.0;
const SUMMARY_CARD_INSET_Y: f32 = 6.0;
const SUMMARY_VISUAL_MAX_EDGE: f32 = 44.0;
const SUMMARY_TEXT_GAP: f32 = 10.0;
const SUMMARY_TEXT_RIGHT_INSET: f32 = 10.0;
const SUMMARY_NAME_OFFSET_Y: f32 = 6.0;
const SUMMARY_NAME_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const SUMMARY_NAME_CONTINUATION_OFFSET_Y: f32 = SUMMARY_NAME_OFFSET_Y + SUMMARY_NAME_HEIGHT;
const SUMMARY_NAME_CONTINUATION_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const SUMMARY_META_ROW_OFFSET_Y: f32 = SUMMARY_NAME_CONTINUATION_OFFSET_Y + 2.0;
const SUMMARY_META_ROW_STACKED_OFFSET_Y: f32 =
    SUMMARY_NAME_CONTINUATION_OFFSET_Y + SUMMARY_NAME_CONTINUATION_HEIGHT + 1.0;
const SUMMARY_META_ROW_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const SUMMARY_TYPE_BADGE_MIN_WIDTH: f32 = 40.0;
const SUMMARY_TYPE_BADGE_MAX_WIDTH: f32 = 76.0;
const SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO: f32 = 0.50;
const SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const SUMMARY_TYPE_BADGE_PADDING_X: f32 = 6.0;
const SUMMARY_TYPE_BADGE_TEXT_INSET_X: f32 = 4.0;
const SUMMARY_META_ROW_GAP: f32 = 6.0;
const SUMMARY_REVISION_MIN_WIDTH: f32 = 34.0;
const SUMMARY_REVISION_MAX_WIDTH: f32 = 62.0;
const SUMMARY_REVISION_MAX_WIDTH_RATIO: f32 = 0.28;
const SUMMARY_REVISION_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const SUMMARY_REVISION_PADDING_X: f32 = 4.0;
const SUMMARY_CONTROL_PREFIX: &str = "AssetBrowserContentPreview";

pub(super) fn apply_compact_content_preview_summary_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    if width <= f32::EPSILON || height <= f32::EPSILON {
        collapse_summary_nodes(nodes, finite_coordinate(x), finite_coordinate(y));
        return;
    }

    let visual_edge = summary_visual_slot_edge(width, height);
    let card_right = x + width;
    let text_x = (x + SUMMARY_CARD_INSET_X + visual_edge + SUMMARY_TEXT_GAP).min(card_right);
    let text_width = finite_non_negative(card_right - text_x - SUMMARY_TEXT_RIGHT_INSET);
    let labels = summary_text_labels(nodes);
    let continuation_height = summary_name_continuation_height(labels.continuation.unwrap_or(""))
        .min(summary_line_height(
            height,
            SUMMARY_NAME_CONTINUATION_OFFSET_Y,
            SUMMARY_NAME_CONTINUATION_HEIGHT,
        ));
    let meta_y = y + summary_meta_row_offset_y(continuation_height);
    let type_badge_width = summary_type_badge_width(labels.kind.unwrap_or(""), text_width);
    let revision_width = summary_revision_width(labels.revision.unwrap_or(""), text_width);
    let revision_x = if revision_width > 0.0 {
        (card_right - SUMMARY_TEXT_RIGHT_INSET - revision_width).max(text_x)
    } else {
        card_right - SUMMARY_TEXT_RIGHT_INSET
    };
    let state_x = text_x + type_badge_width + SUMMARY_META_ROW_GAP;
    let state_width = finite_non_negative(revision_x - state_x - SUMMARY_META_ROW_GAP);

    let frames = summary_frames(
        x,
        y,
        width,
        height,
        visual_edge,
        text_x,
        text_width,
        continuation_height,
        meta_y,
        type_badge_width,
        state_x,
        state_width,
        revision_x,
        revision_width,
    );
    apply_summary_node_frames(nodes, &frames);
}

fn summary_visual_slot_edge(width: f32, height: f32) -> f32 {
    let available_width = finite_non_negative(width - SUMMARY_CARD_INSET_X * 2.0);
    let available_height = finite_non_negative(height - SUMMARY_CARD_INSET_Y * 2.0);
    available_width
        .min(available_height)
        .min(SUMMARY_VISUAL_MAX_EDGE)
}

fn summary_name_continuation_height(text: &str) -> f32 {
    if text.is_empty() {
        0.0
    } else {
        SUMMARY_NAME_CONTINUATION_HEIGHT
    }
}

fn summary_meta_row_offset_y(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        SUMMARY_META_ROW_STACKED_OFFSET_Y
    } else {
        SUMMARY_META_ROW_OFFSET_Y
    }
}

fn summary_type_badge_width(label: &str, text_width: f32) -> f32 {
    let text_width = finite_non_negative(text_width);
    if label.is_empty() || text_width <= f32::EPSILON {
        return 0.0;
    }
    let content_width = measure_runtime_text_width(label, SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE)
        + SUMMARY_TYPE_BADGE_PADDING_X * 2.0;
    let badge_max_width = SUMMARY_TYPE_BADGE_MAX_WIDTH
        .min(text_width * SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO)
        .min(text_width);
    content_width
        .max(SUMMARY_TYPE_BADGE_MIN_WIDTH.min(badge_max_width))
        .min(badge_max_width)
}

fn summary_revision_width(label: &str, text_width: f32) -> f32 {
    let text_width = finite_non_negative(text_width);
    if label.is_empty() || text_width <= f32::EPSILON {
        return 0.0;
    }
    let content_width = measure_runtime_text_width(label, SUMMARY_REVISION_FONT_SIZE)
        + SUMMARY_REVISION_PADDING_X * 2.0;
    let max_width = SUMMARY_REVISION_MAX_WIDTH
        .min(text_width * SUMMARY_REVISION_MAX_WIDTH_RATIO)
        .min(text_width);
    content_width
        .max(SUMMARY_REVISION_MIN_WIDTH.min(max_width))
        .min(max_width)
}

fn summary_line_height(card_height: f32, offset_y: f32, preferred_height: f32) -> f32 {
    finite_non_negative(preferred_height).min(finite_non_negative(
        finite_non_negative(card_height) - finite_non_negative(offset_y),
    ))
}

fn collapse_summary_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        if node.control_id.starts_with("AssetBrowserContentPreview") {
            node.frame = ViewTemplateFrameData {
                x,
                y,
                width: 0.0,
                height: 0.0,
            };
        }
    }
}

#[derive(Clone, Copy, Default)]
struct SummaryTextLabels<'a> {
    continuation: Option<&'a str>,
    kind: Option<&'a str>,
    revision: Option<&'a str>,
}

fn summary_text_labels(nodes: &[ViewTemplateNodeData]) -> SummaryTextLabels<'_> {
    let mut labels = SummaryTextLabels::default();
    for node in nodes {
        let Some(suffix) = node.control_id.strip_prefix(SUMMARY_CONTROL_PREFIX) else {
            continue;
        };
        match suffix {
            "NameContinuation" if labels.continuation.is_none() => {
                labels.continuation = Some(node.text.as_str());
            }
            "Type" if labels.kind.is_none() => labels.kind = Some(node.text.as_str()),
            "Revision" if labels.revision.is_none() => {
                labels.revision = Some(node.text.as_str());
            }
            _ => {}
        }
    }
    labels
}

#[allow(clippy::too_many_arguments)]
fn summary_frames(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    visual_edge: f32,
    text_x: f32,
    text_width: f32,
    continuation_height: f32,
    meta_y: f32,
    type_badge_width: f32,
    state_x: f32,
    state_width: f32,
    revision_x: f32,
    revision_width: f32,
) -> [ViewTemplateFrameData; 9] {
    let meta_height = summary_line_height(height, meta_y - y, SUMMARY_META_ROW_HEIGHT);
    [
        summary_frame(x, y, width, height),
        summary_frame(
            x + SUMMARY_CARD_INSET_X,
            y + SUMMARY_CARD_INSET_Y,
            visual_edge,
            visual_edge,
        ),
        summary_frame(
            text_x,
            y + SUMMARY_NAME_OFFSET_Y,
            text_width,
            summary_line_height(height, SUMMARY_NAME_OFFSET_Y, SUMMARY_NAME_HEIGHT),
        ),
        summary_frame(
            text_x,
            y + SUMMARY_NAME_CONTINUATION_OFFSET_Y,
            text_width,
            continuation_height,
        ),
        summary_frame(text_x, meta_y, 0.0, 0.0),
        summary_frame(text_x, meta_y, type_badge_width, meta_height),
        summary_frame(
            text_x + SUMMARY_TYPE_BADGE_TEXT_INSET_X,
            meta_y,
            finite_non_negative(type_badge_width - SUMMARY_TYPE_BADGE_TEXT_INSET_X * 2.0),
            meta_height,
        ),
        summary_frame(state_x, meta_y, state_width, meta_height),
        summary_frame(revision_x, meta_y, revision_width, meta_height),
    ]
}

fn apply_summary_node_frames(
    nodes: &mut [ViewTemplateNodeData],
    frames: &[ViewTemplateFrameData; 9],
) {
    for node in nodes {
        let Some(frame_index) = summary_frame_index(&node.control_id) else {
            continue;
        };
        node.frame = frames[frame_index].clone();
    }
}

fn summary_frame_index(control_id: &str) -> Option<usize> {
    match control_id.strip_prefix(SUMMARY_CONTROL_PREFIX)? {
        "Card" => Some(0),
        "Visual" => Some(1),
        "Name" => Some(2),
        "NameContinuation" => Some(3),
        "Meta" => Some(4),
        "TypeBadge" => Some(5),
        "Type" => Some(6),
        "State" => Some(7),
        "Revision" => Some(8),
        _ => None,
    }
}

fn summary_frame(x: f32, y: f32, width: f32, height: f32) -> ViewTemplateFrameData {
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
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETIRED_SUMMARY_FRAME_CONTROL_IDS: [&str; 9] = [
        "AssetBrowserContentPreviewCard",
        "AssetBrowserContentPreviewVisual",
        "AssetBrowserContentPreviewName",
        "AssetBrowserContentPreviewNameContinuation",
        "AssetBrowserContentPreviewMeta",
        "AssetBrowserContentPreviewTypeBadge",
        "AssetBrowserContentPreviewType",
        "AssetBrowserContentPreviewState",
        "AssetBrowserContentPreviewRevision",
    ];

    #[test]
    fn summary_layout_splits_meta_row_and_collapses_legacy_label() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node(
                "AssetBrowserContentPreviewNameContinuation",
                "Label",
                "authoring.zui",
            ),
            node(
                "AssetBrowserContentPreviewMeta",
                "Label",
                "Mesh | Ready | rev 12",
            ),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "UI Layout"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);

        let card = frame(&nodes, "AssetBrowserContentPreviewCard");
        let visual = frame(&nodes, "AssetBrowserContentPreviewVisual");
        let name = frame(&nodes, "AssetBrowserContentPreviewName");
        let continuation = frame(&nodes, "AssetBrowserContentPreviewNameContinuation");
        let legacy_meta = frame(&nodes, "AssetBrowserContentPreviewMeta");
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");
        let type_label = frame(&nodes, "AssetBrowserContentPreviewType");
        let state = frame(&nodes, "AssetBrowserContentPreviewState");
        let revision = frame(&nodes, "AssetBrowserContentPreviewRevision");

        assert_eq!(card.width, 420.0);
        assert!(visual.x > card.x);
        assert_eq!(visual.width, visual.height);
        assert!(
            visual.width <= SUMMARY_VISUAL_MAX_EDGE,
            "summary icon slot should stay square and compact: {:?}",
            visual
        );
        assert!(name.x > visual.x + visual.width);
        assert!(
            name.height >= 14.0,
            "summary primary title should keep a baseline-safe slot for compact file names: {:?}",
            name
        );
        assert!(
            name.x - (visual.x + visual.width) <= SUMMARY_TEXT_GAP,
            "summary text should sit near the square preview slot: visual={:?}, name={:?}",
            visual,
            name
        );
        assert_eq!(continuation.x, name.x);
        assert!(continuation.y >= name.y + name.height);
        assert!(continuation.height > 0.0);
        assert_eq!(legacy_meta.height, 0.0);
        assert!(type_badge.y >= continuation.y + continuation.height);
        assert_close(
            type_badge.width,
            expected_summary_type_width("UI Layout", name.width),
        );
        assert!(type_badge.width <= SUMMARY_TYPE_BADGE_MAX_WIDTH);
        assert_eq!(type_label.x, type_badge.x + SUMMARY_TYPE_BADGE_TEXT_INSET_X);
        assert!(state.x > type_badge.x + type_badge.width);
        assert!(revision.x > state.x);
        assert!(revision.x + revision.width <= card.x + card.width);
    }

    #[test]
    fn summary_layout_collapses_empty_name_continuation_and_keeps_meta_compact() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node("AssetBrowserContentPreviewNameContinuation", "Label", ""),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "MESH"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);

        let name = frame(&nodes, "AssetBrowserContentPreviewName");
        let visual = frame(&nodes, "AssetBrowserContentPreviewVisual");
        let continuation = frame(&nodes, "AssetBrowserContentPreviewNameContinuation");
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");

        assert_eq!(visual.width, visual.height);
        assert!(
            visual.width <= SUMMARY_VISUAL_MAX_EDGE,
            "single-line summary preview should use a compact square slot: {:?}",
            visual
        );
        assert_eq!(continuation.height, 0.0);
        assert!(type_badge.y > name.y);
        assert!(
            type_badge.y - name.y < 24.0,
            "single-line summary should not reserve second-line spacing: name={:?}, badge={:?}",
            name,
            type_badge
        );
    }

    #[test]
    fn summary_badge_and_revision_widths_use_runtime_text_measurement() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node(
                "AssetBrowserContentPreviewName",
                "Label",
                "MaterialVariant.zmat",
            ),
            node("AssetBrowserContentPreviewNameContinuation", "Label", ""),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "iiiiiiiiii"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev iiiiii"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);

        let text_width = frame(&nodes, "AssetBrowserContentPreviewName").width;
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");
        let revision = frame(&nodes, "AssetBrowserContentPreviewRevision");
        let expected_type_width = expected_summary_type_width("iiiiiiiiii", text_width);
        let expected_revision_width = expected_summary_revision_width("rev iiiiii", text_width);

        assert_close(type_badge.width, expected_type_width);
        assert_close(revision.width, expected_revision_width);
    }

    #[test]
    fn summary_layout_clips_tiny_cards_without_expanding_their_children() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node("AssetBrowserContentPreviewNameContinuation", "Label", ""),
            node("AssetBrowserContentPreviewMeta", "Label", ""),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", "MESH"),
            node("AssetBrowserContentPreviewType", "Label", "MESH"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 40.0, 80.0, 24.0, 18.0);

        let card = frame(&nodes, "AssetBrowserContentPreviewCard");
        for node in &nodes {
            if node.frame.width <= f32::EPSILON || node.frame.height <= f32::EPSILON {
                continue;
            }
            assert!(node.frame.x >= card.x);
            assert!(node.frame.y >= card.y);
            assert!(node.frame.x + node.frame.width <= card.x + card.width);
            assert!(node.frame.y + node.frame.height <= card.y + card.height);
        }

        apply_compact_content_preview_summary_layout(&mut nodes, 40.0, 80.0, 0.0, 0.0);
        assert!(nodes
            .iter()
            .all(|node| node.frame.width == 0.0 && node.frame.height == 0.0));
    }

    #[test]
    fn single_pass_summary_layout_preserves_retired_frames() {
        for (width, height, continuation) in [
            (420.0, 50.0, "authoring.zui"),
            (420.0, 50.0, ""),
            (24.0, 18.0, "tiny"),
        ] {
            let mut retired = summary_layout_fixture(128, continuation);
            let mut optimized = retired.clone();

            retired_apply_summary_layout(&mut retired, 80.0, 320.0, width, height);
            apply_compact_content_preview_summary_layout(
                &mut optimized,
                80.0,
                320.0,
                width,
                height,
            );

            for (retired_node, optimized_node) in retired.iter().zip(&optimized) {
                assert_eq!(
                    optimized_node.frame, retired_node.frame,
                    "frame changed for {}",
                    retired_node.control_id
                );
            }
        }
    }

    #[test]
    fn single_pass_summary_layout_uses_two_node_scans() {
        let source = include_str!("summary_layout.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        let layout = implementation
            .split("pub(super) fn apply_compact_content_preview_summary_layout")
            .nth(1)
            .expect("summary layout")
            .split("fn summary_visual_slot_edge")
            .next()
            .expect("summary layout body");

        assert!(layout.contains("summary_text_labels(nodes)"));
        assert!(layout.contains("apply_summary_node_frames(nodes"));
        assert!(!layout.contains("set_node_frame("));
        assert!(implementation.contains("strip_prefix(SUMMARY_CONTROL_PREFIX)"));
        assert!(!implementation.contains("fn node_text"));
        assert!(!implementation.contains("fn set_node_frame"));
    }

    #[test]
    #[ignore = "release performance benchmark"]
    fn single_pass_summary_layout_release_benchmark() {
        const SAMPLES: usize = 11;
        const ITERATIONS: usize = 128;
        const FILLER_NODE_COUNT: usize = 512;
        const RETIRED_NODE_SCANS: usize = 12;
        const OPTIMIZED_NODE_SCANS: usize = 2;

        let base = summary_layout_fixture(FILLER_NODE_COUNT, "authoring.zui");
        let mut retired_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let benchmark = |layout: fn(&mut [ViewTemplateNodeData], f32, f32, f32, f32)| {
                let mut nodes = base.clone();
                let started = std::time::Instant::now();
                for _ in 0..ITERATIONS {
                    layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);
                    std::hint::black_box(&nodes);
                }
                started.elapsed().as_nanos()
            };

            if sample % 2 == 0 {
                retired_samples.push(benchmark(retired_apply_summary_layout));
                optimized_samples.push(benchmark(apply_compact_content_preview_summary_layout));
            } else {
                optimized_samples.push(benchmark(apply_compact_content_preview_summary_layout));
                retired_samples.push(benchmark(retired_apply_summary_layout));
            }
        }

        let retired_p95_ns = percentile_95(&mut retired_samples);
        let optimized_p95_ns = percentile_95(&mut optimized_samples);
        let reduction_bps = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "EDITOR57_SINGLE_PASS_SUMMARY_LAYOUT_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             nodes={} node_scans={RETIRED_NODE_SCANS}->{OPTIMIZED_NODE_SCANS}",
            base.len()
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(35),
            "optimized P95 must be at least 65% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn summary_layout_fixture(
        filler_node_count: usize,
        continuation: &str,
    ) -> Vec<ViewTemplateNodeData> {
        let mut nodes = (0..filler_node_count)
            .map(|index| node(&format!("WorkbenchUnrelatedNode{index:04}"), "Panel", ""))
            .collect::<Vec<_>>();
        nodes.extend([
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node(
                "AssetBrowserContentPreviewNameContinuation",
                "Label",
                continuation,
            ),
            node("AssetBrowserContentPreviewMeta", "Label", "legacy"),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "UI Layout"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ]);
        nodes
    }

    fn retired_apply_summary_layout(
        nodes: &mut [ViewTemplateNodeData],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let width = finite_non_negative(width);
        let height = finite_non_negative(height);
        if width <= f32::EPSILON || height <= f32::EPSILON {
            collapse_summary_nodes(nodes, finite_coordinate(x), finite_coordinate(y));
            return;
        }

        let visual_edge = summary_visual_slot_edge(width, height);
        let card_right = x + width;
        let text_x = (x + SUMMARY_CARD_INSET_X + visual_edge + SUMMARY_TEXT_GAP).min(card_right);
        let text_width = finite_non_negative(card_right - text_x - SUMMARY_TEXT_RIGHT_INSET);
        let continuation_text =
            retired_node_text(nodes, "AssetBrowserContentPreviewNameContinuation").unwrap_or("");
        let continuation_height =
            summary_name_continuation_height(continuation_text).min(summary_line_height(
                height,
                SUMMARY_NAME_CONTINUATION_OFFSET_Y,
                SUMMARY_NAME_CONTINUATION_HEIGHT,
            ));
        let meta_y = y + summary_meta_row_offset_y(continuation_height);
        let type_label = retired_node_text(nodes, "AssetBrowserContentPreviewType").unwrap_or("");
        let revision_label =
            retired_node_text(nodes, "AssetBrowserContentPreviewRevision").unwrap_or("");
        let type_badge_width = summary_type_badge_width(type_label, text_width);
        let revision_width = summary_revision_width(revision_label, text_width);
        let revision_x = if revision_width > 0.0 {
            (card_right - SUMMARY_TEXT_RIGHT_INSET - revision_width).max(text_x)
        } else {
            card_right - SUMMARY_TEXT_RIGHT_INSET
        };
        let state_x = text_x + type_badge_width + SUMMARY_META_ROW_GAP;
        let state_width = finite_non_negative(revision_x - state_x - SUMMARY_META_ROW_GAP);

        let frames = summary_frames(
            x,
            y,
            width,
            height,
            visual_edge,
            text_x,
            text_width,
            continuation_height,
            meta_y,
            type_badge_width,
            state_x,
            state_width,
            revision_x,
            revision_width,
        );
        for (index, control_id) in RETIRED_SUMMARY_FRAME_CONTROL_IDS.iter().enumerate() {
            retired_set_node_frame(nodes, control_id, frames[index].clone());
        }
    }

    fn retired_node_text<'a>(
        nodes: &'a [ViewTemplateNodeData],
        control_id: &str,
    ) -> Option<&'a str> {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .map(|node| node.text.as_str())
    }

    fn retired_set_node_frame(
        nodes: &mut [ViewTemplateNodeData],
        control_id: &str,
        frame: ViewTemplateFrameData,
    ) {
        for node in nodes
            .iter_mut()
            .filter(|node| node.control_id.as_str() == control_id)
        {
            node.frame = frame.clone();
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index]
    }

    fn node(control_id: &str, role: &str, text: &str) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            role: role.into(),
            text: text.into(),
            frame: ViewTemplateFrameData::default(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> ViewTemplateFrameData {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing {control_id}"))
    }

    fn expected_summary_type_width(label: &str, text_width: f32) -> f32 {
        let content_width = measure_runtime_text_width(label, SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE)
            + SUMMARY_TYPE_BADGE_PADDING_X * 2.0;
        let max_width = SUMMARY_TYPE_BADGE_MAX_WIDTH
            .min(text_width * SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO)
            .min(text_width);
        content_width
            .max(SUMMARY_TYPE_BADGE_MIN_WIDTH.min(max_width))
            .min(max_width)
    }

    fn expected_summary_revision_width(label: &str, text_width: f32) -> f32 {
        let content_width = measure_runtime_text_width(label, SUMMARY_REVISION_FONT_SIZE)
            + SUMMARY_REVISION_PADDING_X * 2.0;
        let max_width = SUMMARY_REVISION_MAX_WIDTH
            .min(text_width * SUMMARY_REVISION_MAX_WIDTH_RATIO)
            .min(text_width);
        content_width
            .max(SUMMARY_REVISION_MIN_WIDTH.min(max_width))
            .min(max_width)
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.01,
            "expected {expected:.3}, got {actual:.3}",
        );
    }
}
