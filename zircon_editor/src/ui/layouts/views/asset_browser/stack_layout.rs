use crate::ui::layouts::views::ViewTemplateNodeData;
use zircon_runtime_interface::ui::layout::UiSize;

use super::toolbar_layout::AssetBrowserToolbarLayout;

const STANDARD_PANEL_GAP: f32 = 6.0;

pub(super) fn apply_asset_browser_standard_stack_layout(
    nodes: &mut [ViewTemplateNodeData],
    size: UiSize,
    toolbar_layout: &AssetBrowserToolbarLayout,
) {
    let Some((main, utility)) = stack_frames(nodes) else {
        return;
    };

    let viewport_height = size.height.max(0.0);
    let utility_y = utility
        .y
        .min((viewport_height - utility.height).max(utility.y));
    let main_y = toolbar_layout.main_y;
    let main_height = (utility_y - STANDARD_PANEL_GAP - main_y).max(main.height);
    let delta_y = main_y - main.y;
    let delta_height = main_height - main.height;

    for node in nodes {
        let control_id = node.control_id.as_str();
        if control_id == "AssetBrowserMainPanel" {
            node.frame.y = main_y;
            node.frame.height = main_height;
            continue;
        }
        if !is_asset_browser_main_stack_control(control_id) {
            continue;
        }
        node.frame.y += delta_y;
        if is_stretchable_main_stack_surface(control_id) {
            node.frame.height = (node.frame.height + delta_height).max(0.0);
        }
    }
}

fn is_asset_browser_main_stack_control(control_id: &str) -> bool {
    control_id.starts_with("AssetBrowserSources")
        || control_id.starts_with("AssetBrowserContent")
        || control_id.starts_with("AssetBrowserDetails")
        || control_id.starts_with("WorkbenchAssetBrowser")
}

fn is_stretchable_main_stack_surface(control_id: &str) -> bool {
    matches!(
        control_id,
        "AssetBrowserSourcesPanel"
            | "AssetBrowserSourcesScrollBody"
            | "AssetBrowserContentPanel"
            | "AssetBrowserAssetTablePanel"
            | "AssetBrowserDetailsPanel"
            | "AssetBrowserDetailsScrollBody"
            | "AssetBrowserDetailsContentPanel"
    )
}

fn stack_frames(
    nodes: &[ViewTemplateNodeData],
) -> Option<(
    crate::ui::layouts::views::ViewTemplateFrameData,
    crate::ui::layouts::views::ViewTemplateFrameData,
)> {
    let mut main = None;
    let mut utility = None;
    for node in nodes {
        match node.control_id.as_str() {
            "AssetBrowserMainPanel" if main.is_none() => main = Some(node.frame.clone()),
            "AssetBrowserUtilityPanel" if utility.is_none() => utility = Some(node.frame.clone()),
            _ => {}
        }
        if main.is_some() && utility.is_some() {
            break;
        }
    }
    Some((main?, utility?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::views::ViewTemplateFrameData;

    #[test]
    fn single_pass_stack_frame_discovery_preserves_retired_frames() {
        let mut retired = stack_fixture(128, true);
        let mut optimized = retired.clone();
        let size = UiSize {
            width: 1280.0,
            height: 720.0,
        };
        let toolbar = AssetBrowserToolbarLayout { main_y: 96.0 };

        retired_apply_stack_layout(&mut retired, size, &toolbar);
        apply_asset_browser_standard_stack_layout(&mut optimized, size, &toolbar);

        for (retired_node, optimized_node) in retired.iter().zip(&optimized) {
            assert_eq!(
                optimized_node.frame, retired_node.frame,
                "frame changed for {}",
                retired_node.control_id
            );
        }
    }

    #[test]
    fn single_pass_stack_frame_discovery_keeps_nodes_when_anchor_is_missing() {
        let mut nodes = stack_fixture(16, false);
        let before = nodes.clone();
        let size = UiSize {
            width: 1280.0,
            height: 720.0,
        };

        apply_asset_browser_standard_stack_layout(
            &mut nodes,
            size,
            &AssetBrowserToolbarLayout { main_y: 96.0 },
        );

        for (before_node, after_node) in before.iter().zip(&nodes) {
            assert_eq!(after_node.frame, before_node.frame);
        }
    }

    #[test]
    fn single_pass_stack_frame_discovery_uses_one_anchor_scan() {
        let source = include_str!("stack_layout.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        let layout = implementation
            .split("pub(super) fn apply_asset_browser_standard_stack_layout")
            .nth(1)
            .expect("stack layout")
            .split("fn is_asset_browser_main_stack_control")
            .next()
            .expect("stack layout body");

        assert!(layout.contains("stack_frames(nodes)"));
        assert!(!layout.contains("node_frame(nodes"));
        assert!(implementation.contains("fn stack_frames"));
        assert!(implementation.contains("main.is_some() && utility.is_some()"));
        assert!(!implementation.contains("fn node_frame"));
    }

    #[test]
    #[ignore = "release performance benchmark"]
    fn single_pass_stack_frame_discovery_release_benchmark() {
        const SAMPLES: usize = 11;
        const ITERATIONS: usize = 256;
        const FILLER_NODE_COUNT: usize = 512;
        const RETIRED_NODE_SCANS: usize = 3;
        const OPTIMIZED_NODE_SCANS: usize = 2;

        let base = stack_fixture(FILLER_NODE_COUNT, true);
        let size = UiSize {
            width: 1280.0,
            height: 720.0,
        };
        let toolbar = AssetBrowserToolbarLayout { main_y: 96.0 };
        let mut retired_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let benchmark =
                |layout: fn(&mut [ViewTemplateNodeData], UiSize, &AssetBrowserToolbarLayout)| {
                    let mut nodes = base.clone();
                    let started = std::time::Instant::now();
                    for _ in 0..ITERATIONS {
                        layout(&mut nodes, size, &toolbar);
                        std::hint::black_box(&nodes);
                    }
                    started.elapsed().as_nanos()
                };

            if sample % 2 == 0 {
                retired_samples.push(benchmark(retired_apply_stack_layout));
                optimized_samples.push(benchmark(apply_asset_browser_standard_stack_layout));
            } else {
                optimized_samples.push(benchmark(apply_asset_browser_standard_stack_layout));
                retired_samples.push(benchmark(retired_apply_stack_layout));
            }
        }

        let retired_p95_ns = percentile_95(&mut retired_samples);
        let optimized_p95_ns = percentile_95(&mut optimized_samples);
        let reduction_bps = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "EDITOR57_SINGLE_PASS_STACK_FRAME_DISCOVERY_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             nodes={} node_scans={RETIRED_NODE_SCANS}->{OPTIMIZED_NODE_SCANS}",
            base.len()
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(85),
            "optimized P95 must be at least 15% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn retired_apply_stack_layout(
        nodes: &mut [ViewTemplateNodeData],
        size: UiSize,
        toolbar_layout: &AssetBrowserToolbarLayout,
    ) {
        let Some(main) = retired_node_frame(nodes, "AssetBrowserMainPanel") else {
            return;
        };
        let Some(utility) = retired_node_frame(nodes, "AssetBrowserUtilityPanel") else {
            return;
        };

        apply_stack_frames(nodes, size, toolbar_layout, main, utility);
    }

    fn apply_stack_frames(
        nodes: &mut [ViewTemplateNodeData],
        size: UiSize,
        toolbar_layout: &AssetBrowserToolbarLayout,
        main: ViewTemplateFrameData,
        utility: ViewTemplateFrameData,
    ) {
        let viewport_height = size.height.max(0.0);
        let utility_y = utility
            .y
            .min((viewport_height - utility.height).max(utility.y));
        let main_y = toolbar_layout.main_y;
        let main_height = (utility_y - STANDARD_PANEL_GAP - main_y).max(main.height);
        let delta_y = main_y - main.y;
        let delta_height = main_height - main.height;

        for node in nodes {
            let control_id = node.control_id.as_str();
            if control_id == "AssetBrowserMainPanel" {
                node.frame.y = main_y;
                node.frame.height = main_height;
                continue;
            }
            if !is_asset_browser_main_stack_control(control_id) {
                continue;
            }
            node.frame.y += delta_y;
            if is_stretchable_main_stack_surface(control_id) {
                node.frame.height = (node.frame.height + delta_height).max(0.0);
            }
        }
    }

    fn retired_node_frame(
        nodes: &[ViewTemplateNodeData],
        control_id: &str,
    ) -> Option<ViewTemplateFrameData> {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
    }

    fn stack_fixture(filler_count: usize, include_utility: bool) -> Vec<ViewTemplateNodeData> {
        let mut nodes = (0..filler_count)
            .map(|index| stack_node(&format!("WorkbenchUnrelatedNode{index:04}"), 0.0, 0.0))
            .collect::<Vec<_>>();
        nodes.extend([
            stack_node("AssetBrowserSourcesPanel", 120.0, 360.0),
            stack_node("AssetBrowserContentPanel", 120.0, 360.0),
            stack_node("AssetBrowserDetailsScrollBody", 120.0, 320.0),
            stack_node("AssetBrowserMainPanel", 120.0, 360.0),
        ]);
        if include_utility {
            nodes.push(stack_node("AssetBrowserUtilityPanel", 560.0, 120.0));
        }
        nodes
    }

    fn stack_node(control_id: &str, y: f32, height: f32) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            frame: ViewTemplateFrameData {
                x: 0.0,
                y,
                width: 800.0,
                height,
            },
            ..ViewTemplateNodeData::default()
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index]
    }
}
