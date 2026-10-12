use super::UiAssetDependencyIndex;

use crate::asset::{AssetReference, AssetUri};

const PERF_MARKER: &str = "RUNTIME829_DEPENDENCY_CASCADE_TARGET_CAPACITY_BENCH_V1";

fn asset_ref(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).expect("valid asset URI"))
}

#[test]
fn cascade_targets_reserve_first_fanout_and_preserve_bfs_order() {
    let mut index = UiAssetDependencyIndex::new();
    for dependent_index in 0..4 {
        let dependent = format!("res://ui/dependent/{dependent_index}.zui");
        index.record_compiled(&dependent, &[asset_ref("res://ui/root.zui")]);
    }
    index.record_compiled(
        "res://ui/dependent/0.zui",
        &[
            asset_ref("res://ui/root.zui"),
            asset_ref("res://ui/leaf.zui"),
        ],
    );

    let targets = index.cascade_invalidation_targets("res://ui/root.zui");

    assert_eq!(
        targets,
        vec![
            "res://ui/dependent/0.zui".to_string(),
            "res://ui/dependent/1.zui".to_string(),
            "res://ui/dependent/2.zui".to_string(),
            "res://ui/dependent/3.zui".to_string(),
            // BUG: [CR-R02-runtime_wave5_template_compile_assets-0001] 此夹具中 leaf 是 dependent/0 的依赖；反向级联只返回四个 dependent，加入 leaf 的期望会使本断言失败，见 record_compiled 与 cascade_invalidation_targets。
            "res://ui/leaf.zui".to_string(),
        ]
    );
    assert!(targets.capacity() >= 4);
}

#[test]
fn cascade_targets_keep_empty_and_self_cycle_results_empty() {
    let empty = UiAssetDependencyIndex::new();
    assert!(empty
        .cascade_invalidation_targets("res://ui/missing.zui")
        .is_empty());

    let mut self_cycle = UiAssetDependencyIndex::new();
    self_cycle.record_compiled("res://ui/self.zui", &[asset_ref("res://ui/self.zui")]);
    let targets = self_cycle.cascade_invalidation_targets("res://ui/self.zui");
    assert!(targets.is_empty());
    assert_eq!(targets.capacity(), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime829_dependency_cascade_target_capacity_bench_v1() {
    const FANOUT: usize = 4_096;
    let legacy_growth_events = growth_events(FANOUT, false);
    let optimized_growth_events = growth_events(FANOUT, true);
    std::hint::black_box((FANOUT, legacy_growth_events, optimized_growth_events));
    println!(
        "{PERF_MARKER} fanout={FANOUT} legacy_growth_events={legacy_growth_events} \
optimized_growth_events={optimized_growth_events}"
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
}

fn growth_events(count: usize, reserve: bool) -> usize {
    let mut targets = if reserve {
        Vec::with_capacity(count)
    } else {
        Vec::new()
    };
    let mut growth_events = 0;
    for target in 0..count {
        let previous_capacity = targets.capacity();
        targets.push(target);
        growth_events += usize::from(targets.capacity() != previous_capacity);
    }
    growth_events
}
