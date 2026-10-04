use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_20260826g_editor50_borrowed_tab_dedup_preserves_order_and_errors() {
    let area = ToolkitArea::new(
        ToolkitAreaSlot::Center,
        ["inspector", "viewport", "console"],
        "viewport",
    )
    .expect("valid area");
    assert_eq!(area.tabs(), &["inspector", "viewport", "console"]);
    assert_eq!(area.active_tab(), "viewport");

    let error = ToolkitArea::new(
        ToolkitAreaSlot::Bottom,
        ["output", "search", "output"],
        "search",
    )
    .expect_err("duplicate tab must be rejected");
    assert!(matches!(
        error,
        ToolkitLayoutError::DuplicateTabId {
            slot: ToolkitAreaSlot::Bottom,
            tab,
        } if tab == "output"
    ));

    let error = ToolkitArea::new(
        ToolkitAreaSlot::Right,
        ["zeta", "zeta", "alpha", "alpha"],
        "alpha",
    )
    .expect_err("multiple duplicate IDs must preserve legacy error ordering");
    assert!(matches!(
        error,
        ToolkitLayoutError::DuplicateTabId { tab, .. } if tab == "alpha"
    ));
}

#[test]
fn optimization_batch_20260826g_editor50_toolkit_area_uses_borrowed_hash_dedup() {
    let source = include_str!("../area.rs");
    let constructor = source
        .split("pub fn new")
        .nth(1)
        .expect("ToolkitArea constructor")
        .split("pub const fn slot")
        .next()
        .expect("bounded ToolkitArea constructor");

    assert!(source.contains("use std::collections::HashSet;"));
    assert!(constructor.contains("HashSet::with_capacity"));
    assert!(!constructor.contains("tabs.clone()"));
    assert!(!constructor.contains("sort"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826g_editor50_borrowed_tab_dedup_performance_evidence() {
    fn legacy_new(
        slot: ToolkitAreaSlot,
        tabs: Vec<String>,
        active_tab: String,
    ) -> Result<ToolkitArea, ToolkitLayoutError> {
        if tabs.is_empty() {
            return Err(ToolkitLayoutError::EmptyTabs { slot });
        }
        if tabs.iter().any(|tab| tab.trim().is_empty()) {
            return Err(ToolkitLayoutError::EmptyTabId);
        }
        let mut sorted = tabs.clone();
        sorted.sort();
        for pair in sorted.windows(2) {
            if pair[0] == pair[1] {
                return Err(ToolkitLayoutError::DuplicateTabId {
                    slot,
                    tab: pair[0].clone(),
                });
            }
        }
        if !tabs.contains(&active_tab) {
            return Err(ToolkitLayoutError::ActiveTabNotFound { slot, active_tab });
        }
        Ok(ToolkitArea {
            slot,
            tabs: tabs.into(),
            active_tab,
        })
    }

    let tabs = (0..32_768)
        .map(|index| format!("plugin.example.toolkit.tab.{index:05}"))
        .collect::<Vec<_>>();
    let active_tab = tabs[tabs.len() / 2].clone();
    let copied_tab_bytes = tabs.iter().map(String::len).sum::<usize>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut borrowed_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let input = tabs.clone();
        let active = active_tab.clone();
        let started = Instant::now();
        black_box(legacy_new(ToolkitAreaSlot::Center, input, active).expect("legacy valid area"));
        legacy_samples.push(started.elapsed().as_nanos());

        let input = tabs.clone();
        let active = active_tab.clone();
        let started = Instant::now();
        black_box(
            ToolkitArea::new(ToolkitAreaSlot::Center, input, active).expect("borrowed valid area"),
        );
        borrowed_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let borrowed_p95 = borrowed_samples[16];
    println!(
        "EDITOR50_TOOLKIT_AREA_BORROWED_TAB_DEDUP_BENCH_V1 tabs={} legacy_p95_ns={} borrowed_p95_ns={} legacy_tab_clones={} borrowed_tab_clones=0 copied_tab_bytes={} target_ratio_bp=6000",
        tabs.len(),
        legacy_p95,
        borrowed_p95,
        tabs.len(),
        copied_tab_bytes,
    );
    assert!(
        borrowed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed tab dedup P95 {borrowed_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}
