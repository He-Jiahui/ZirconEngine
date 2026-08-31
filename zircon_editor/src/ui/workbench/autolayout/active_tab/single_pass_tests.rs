use std::hint::black_box;
use std::time::Instant;

use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::{PaneTabModel, ToolWindowStackModel};
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::preferred_tool_stack;

const MARKER: &str = "EDITOR192_ACTIVE_TOOL_TAB_SINGLE_PASS_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const STACK_COUNT: usize = 4_096;
const REPEATS: usize = 512;

#[test]
fn optimization_batch_20260826gz_editor192_active_tool_stack_preserves_priority_and_fallback() {
    let stacks = vec![
        stack(
            ActivityDrawerSlot::LeftTop,
            ActivityDrawerMode::Collapsed,
            true,
        ),
        stack(
            ActivityDrawerSlot::LeftBottom,
            ActivityDrawerMode::Pinned,
            false,
        ),
        stack(
            ActivityDrawerSlot::RightTop,
            ActivityDrawerMode::Pinned,
            true,
        ),
    ];

    assert_eq!(
        preferred_tool_stack(stacks.iter()).map(|stack| stack.slot),
        Some(ActivityDrawerSlot::RightTop)
    );
    assert_eq!(
        preferred_tool_stack(stacks[..2].iter()).map(|stack| stack.slot),
        Some(ActivityDrawerSlot::LeftTop)
    );
}

#[test]
fn optimization_batch_20260826gz_editor192_active_tool_stack_scans_once() {
    let source = include_str!("../active_tab.rs");
    let implementation = source
        .split("fn preferred_tool_stack")
        .nth(1)
        .and_then(|tail| tail.split("pub(super) fn active_document_tab").next())
        .expect("active tool stack selection implementation");
    assert!(implementation.contains("let mut fallback = None"));
    assert!(implementation.contains("return Some(stack)"));
    assert!(implementation.contains("fallback.get_or_insert(stack)"));
    assert!(!implementation.contains(".or_else"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gz_editor192_active_tool_tab_single_pass_bench() {
    let stacks = (0..STACK_COUNT)
        .map(|_| {
            stack(
                ActivityDrawerSlot::LeftTop,
                ActivityDrawerMode::Collapsed,
                true,
            )
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&stacks, legacy_preferred_tool_stack));
            optimized_samples.push(measure(&stacks, optimized_preferred_tool_stack));
        } else {
            optimized_samples.push(measure(&stacks, optimized_preferred_tool_stack));
            legacy_samples.push(measure(&stacks, legacy_preferred_tool_stack));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-pass stack selection must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn stack(
    slot: ActivityDrawerSlot,
    mode: ActivityDrawerMode,
    visible: bool,
) -> ToolWindowStackModel {
    ToolWindowStackModel {
        slot,
        mode,
        visible,
        tabs: vec![PaneTabModel {
            instance_id: ViewInstanceId::new("test.instance"),
            descriptor_id: ViewDescriptorId::new("test.descriptor"),
            title: "Test".to_string(),
            icon_key: "tool".to_string(),
            content_kind: ViewContentKind::Project,
            active: true,
            closeable: false,
            empty_state: None,
        }],
        active_tab: Some(ViewInstanceId::new("test.instance")),
    }
}

fn legacy_preferred_tool_stack(stacks: &[ToolWindowStackModel]) -> Option<&ToolWindowStackModel> {
    stacks
        .iter()
        .find(|stack| {
            stack.visible && stack.mode != ActivityDrawerMode::Collapsed && !stack.tabs.is_empty()
        })
        .or_else(|| {
            stacks
                .iter()
                .find(|stack| stack.visible && !stack.tabs.is_empty())
        })
}

fn optimized_preferred_tool_stack(
    stacks: &[ToolWindowStackModel],
) -> Option<&ToolWindowStackModel> {
    preferred_tool_stack(stacks.iter())
}

fn measure(
    stacks: &[ToolWindowStackModel],
    implementation: fn(&[ToolWindowStackModel]) -> Option<&ToolWindowStackModel>,
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let selected = implementation(black_box(stacks)).expect("fallback stack");
        checksum = checksum.wrapping_add(selected.tabs.len());
        black_box(selected);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
