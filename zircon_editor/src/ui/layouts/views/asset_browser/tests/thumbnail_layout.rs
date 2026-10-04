use super::super::thumbnail_nodes::thumbnail_control_id;
use super::*;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn thumbnail_type_badge_width_uses_runtime_text_measurement() {
    let nodes = vec![node(&thumbnail_control_id("Type", 0), "Label", "iiiiiiii")];
    let measured =
        measure_runtime_text_width(nodes[0].text.as_str(), THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE);
    let card = AssetContentRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 160.0,
    };
    let narrow = asset_thumbnail_card_geometry(card, false, 0.0)
        .for_role(BrowserThumbnailNodeRole::TypeBadge);
    let measured = asset_thumbnail_card_geometry(card, false, measured)
        .for_role(BrowserThumbnailNodeRole::TypeBadge);

    assert!(measured.width >= narrow.width);
    assert!(measured.width <= 48.0);
}

#[test]
fn thumbnail_badge_measurement_uses_the_workbench_caption_size() {
    assert_eq!(
        THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE,
        zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    );
}

#[test]
fn thumbnail_text_line_geometry_follows_workbench_typography_without_overlap() {
    let geometry = asset_thumbnail_card_geometry(
        AssetContentRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 160.0,
        },
        true,
        0.0,
    );
    let name = geometry.for_role(BrowserThumbnailNodeRole::Name);
    let continuation = geometry.for_role(BrowserThumbnailNodeRole::NameContinuation);

    assert!(continuation.height > 0.0);
    assert!(continuation.y >= name.y + name.height);
}

#[test]
fn thumbnail_file_name_compacts_to_its_actual_card_text_frame() {
    let source_name = "workbench_extension_accessibility_workspace.zui";
    let mut name = node(&thumbnail_control_id("Name", 0), "Label", source_name);
    name.value_text = source_name.into();
    let mut nodes = vec![
        node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""),
        node(&thumbnail_control_id("Card", 0), "Panel", ""),
        name,
    ];

    apply_compact_thumbnail_grid_layout(&mut nodes, 0.0, 0.0, 120.0, 160.0);

    let name_frame = node_frame(&nodes, &thumbnail_control_id("Name", 0))
        .expect("thumbnail name should receive a frame");
    let compact_name = node_text(&nodes, &thumbnail_control_id("Name", 0))
        .expect("thumbnail name should retain text");
    assert!(compact_name.ends_with(".zui"));
    assert!(
        measure_runtime_text_width(compact_name, EditorTypographyTokens::WORKBENCH_BODY_SIZE)
            <= name_frame.width + 0.01,
        "thumbnail title must fit its real frame: text={compact_name}, frame={name_frame:?}"
    );
}

#[test]
fn collapsed_grid_clears_each_thumbnail_card_descendant() {
    let mut nodes = [
        node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""),
        node(&thumbnail_control_id("Card", 0), "Panel", ""),
        node(&thumbnail_control_id("Visual", 0), "Panel", ""),
        node(&thumbnail_control_id("InfoBand", 0), "Panel", ""),
        node(&thumbnail_control_id("SelectionMarker", 0), "Panel", ""),
        node(&thumbnail_control_id("Name", 0), "Label", "asset"),
        node(
            &thumbnail_control_id("NameContinuation", 0),
            "Label",
            "asset",
        ),
        node(&thumbnail_control_id("TypeBadge", 0), "Panel", ""),
        node(&thumbnail_control_id("Type", 0), "Label", "UI"),
        node(&thumbnail_control_id("Meta", 0), "Label", "Ready"),
    ]
    .to_vec();

    apply_compact_thumbnail_grid_layout(&mut nodes, 10.0, 20.0, 0.0, 120.0);

    for part in THUMBNAIL_CARD_LAYOUT_PARTS {
        let frame = node_frame(&nodes, &thumbnail_control_id(part, 0))
            .expect("thumbnail part should remain in the node model");
        assert_eq!(frame.width, 0.0, "{part} width should collapse");
        assert_eq!(frame.height, 0.0, "{part} height should collapse");
    }
}

#[test]
fn ten_thousand_thumbnail_cards_use_the_same_indexed_layout_path() {
    let mut nodes = Vec::with_capacity(10_001);
    nodes.push(node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""));
    nodes.extend((0..10_000).map(|index| node(&thumbnail_control_id("Card", index), "Panel", "")));

    apply_compact_thumbnail_grid_layout(&mut nodes, 0.0, 0.0, 900.0, 620.0);

    let grid = node_frame(&nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID)
        .expect("thumbnail grid should remain materialized");
    let first = node_frame(&nodes, &thumbnail_control_id("Card", 0))
        .expect("first thumbnail card should receive a frame");
    let last = node_frame(&nodes, &thumbnail_control_id("Card", 9_999))
        .expect("last thumbnail card should receive a frame");
    assert!(grid.height > 0.0);
    assert!(last.y > first.y);
    assert!(nodes[0].value_number > last.y + last.height);
}

#[test]
fn compact_thumbnail_layout_matches_per_node_geometry_for_interleaved_sparse_cards() {
    let mut source = vec![
        node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""),
        node("UnrelatedControl", "Panel", ""),
    ];
    for (index, continuation) in [(2, "second line"), (0, "")] {
        for part in THUMBNAIL_CARD_LAYOUT_PARTS.into_iter().rev() {
            let text = match part {
                "Name" => "long_file_name_for_thumbnail_layout.asset",
                "NameContinuation" => continuation,
                "Type" if index == 2 => "Material",
                "Type" => "UI",
                _ => "",
            };
            let mut item = node(&thumbnail_control_id(part, index), "Label", text);
            if part == "Name" {
                item.value_text = text.into();
            }
            source.push(item);
        }
    }
    // The orphan is within the grid count; the out-of-range node is cleared.
    source.push(node(&thumbnail_control_id("Visual", 1), "Panel", ""));
    source.push(node(&thumbnail_control_id("Visual", 4), "Panel", ""));

    for width in [520.0, 0.0] {
        let mut expected = source.clone();
        retired_apply_compact_thumbnail_grid_layout(&mut expected, 13.0, 29.0, width, 620.0);
        let mut actual = source.clone();
        apply_compact_thumbnail_grid_layout(&mut actual, 13.0, 29.0, width, 620.0);

        for (old, new) in expected.iter().zip(&actual) {
            assert_eq!(new.frame, old.frame, "{} frame", new.control_id);
            assert_eq!(new.text, old.text, "{} text", new.control_id);
            assert_eq!(
                new.value_number, old.value_number,
                "{} extent",
                new.control_id
            );
        }
    }
}

#[test]
#[ignore = "run the old/new compact thumbnail layout comparison in Windows Release"]
fn editor57_compact_thumbnail_frame_cache_release_benchmark() {
    const MARKER: &str = "EDITOR57_COMPACT_THUMBNAIL_FRAME_CACHE_BENCH_V1";
    const WARMUPS: usize = 5;
    const SAMPLES: usize = 31;
    let mut gates = Vec::new();
    for (cards, iterations, max_p95_percent) in [(48, 128, 105), (240, 32, 85)] {
        let source = complete_thumbnail_nodes(cards);
        for _ in 0..WARMUPS {
            black_box(timed_thumbnail_layout_batch(&source, iterations, true));
            black_box(timed_thumbnail_layout_batch(&source, iterations, false));
        }
        let mut old_ns = Vec::with_capacity(SAMPLES);
        let mut new_ns = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                old_ns.push(timed_thumbnail_layout_batch(&source, iterations, true));
                new_ns.push(timed_thumbnail_layout_batch(&source, iterations, false));
            } else {
                new_ns.push(timed_thumbnail_layout_batch(&source, iterations, false));
                old_ns.push(timed_thumbnail_layout_batch(&source, iterations, true));
            }
        }
        let (old_p50, old_p95, old_p99) = percentiles_ns(&mut old_ns.clone());
        let (new_p50, new_p95, new_p99) = percentiles_ns(&mut new_ns.clone());
        println!(
            "PERF_RESULT {MARKER} cards={cards} iterations={iterations} warmups={WARMUPS} samples={SAMPLES} old_p50_ns={old_p50} old_p95_ns={old_p95} old_p99_ns={old_p99} new_p50_ns={new_p50} new_p95_ns={new_p95} new_p99_ns={new_p99} max_p95_percent={max_p95_percent} raw_old_ns={old_ns:?} raw_new_ns={new_ns:?} os={} arch={} package_version={}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            env!("CARGO_PKG_VERSION")
        );
        gates.push((cards, old_p95, new_p95, max_p95_percent));
    }
    for (cards, old_p95, new_p95, max_p95_percent) in gates {
        assert!(
            new_p95.saturating_mul(100) <= old_p95.saturating_mul(max_p95_percent),
            "{MARKER} cards={cards}: new p95 {new_p95} ns exceeds {max_p95_percent}% of old p95 {old_p95} ns"
        );
    }
}

fn complete_thumbnail_nodes(cards: usize) -> Vec<ViewTemplateNodeData> {
    let mut nodes = Vec::with_capacity(1 + cards * THUMBNAIL_CARD_LAYOUT_PARTS.len());
    nodes.push(node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""));
    for index in 0..cards {
        for part in THUMBNAIL_CARD_LAYOUT_PARTS {
            let text = match part {
                "Name" => "sample_material.asset",
                "Type" => "Material",
                "Meta" => "Ready",
                _ => "",
            };
            let mut item = node(&thumbnail_control_id(part, index), "Label", text);
            if part == "Name" {
                item.value_text = text.into();
            }
            nodes.push(item);
        }
    }
    nodes
}

fn timed_thumbnail_layout_batch(
    source: &[ViewTemplateNodeData],
    iterations: usize,
    retired: bool,
) -> u128 {
    let mut batches = (0..iterations).map(|_| source.to_vec()).collect::<Vec<_>>();
    let started = Instant::now();
    for nodes in &mut batches {
        if retired {
            retired_apply_compact_thumbnail_grid_layout(nodes, 0.0, 0.0, 900.0, 620.0);
        } else {
            apply_compact_thumbnail_grid_layout(nodes, 0.0, 0.0, 900.0, 620.0);
        }
        black_box(nodes);
    }
    started.elapsed().as_nanos()
}

fn percentiles_ns(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    let percentile = |percent: usize| samples[(samples.len() * percent).div_ceil(100) - 1];
    (percentile(50), percentile(95), percentile(99))
}

// The retired per-node geometry path is kept only as a behavior and Release baseline.
fn retired_apply_compact_thumbnail_grid_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let count = thumbnail_card_count(nodes);
    let layout_inputs = thumbnail_layout_inputs(nodes, count);
    let metrics = AssetThumbnailGridMetrics::new(width, count);
    let grid_height = if count == 0 { 0.0 } else { height };
    let content_extent = metrics.content_extent();

    for node in nodes.iter_mut() {
        if node.control_id == BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID {
            node.frame = ViewTemplateFrameData {
                x,
                y,
                width,
                height: grid_height,
            };
            node.value_number = content_extent;
            continue;
        }
        let Some((kind, index)) = thumbnail_node_identity(node.control_id.as_str()) else {
            continue;
        };
        let Some(input) = layout_inputs.get(index).copied() else {
            node.frame = ViewTemplateFrameData::default();
            continue;
        };
        let Some(frames) = thumbnail_card_frames(metrics, index, x, y, input) else {
            node.frame = ViewTemplateFrameData::default();
            continue;
        };
        node.frame = frames.for_kind(kind);
        if kind == ThumbnailNodeKind::Name && !node.value_text.is_empty() {
            node.text =
                compact_thumbnail_file_name_to_width(node.value_text.as_str(), frames.name.width)
                    .into();
        }
    }
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

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .map(|node| node.frame.clone())
}

fn node_text<'a>(nodes: &'a [ViewTemplateNodeData], control_id: &str) -> Option<&'a str> {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .map(|node| node.text.as_str())
}
