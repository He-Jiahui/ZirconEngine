use std::hint::black_box;
use std::time::Instant;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_runtime_interface::ui::binding::{UiBindingCall, UiBindingValue};

use super::menu_binding_projection;

const SAMPLE_PAIRS: usize = 17;
const PROJECTIONS_PER_SAMPLE: usize = 8_192;

#[test]
fn optimization_batch_df_editor343_single_binding_projection_matches_legacy() {
    let binding = benchmark_binding();

    assert_eq!(
        menu_binding_projection(&binding),
        legacy_projection(&binding)
    );
}

#[test]
fn optimization_batch_df_editor343_menu_projection_converts_binding_once() {
    let source = include_str!("../adapter.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("const MENU_ITEM_PROPERTY_CAPACITY: usize = 4"));
    assert!(production.contains("Vec::with_capacity(MENU_ITEM_PROPERTY_CAPACITY)"));
    assert!(production.contains("fn menu_binding_projection"));
    assert!(!production.contains("item.binding.native_binding()"));
    assert!(!production.contains("call.symbol.clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_df_editor343_single_binding_projection_p95() {
    let binding = benchmark_binding();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&binding, false));
            optimized.push(measure(&binding, true));
        } else {
            optimized.push(measure(&binding, true));
            legacy.push(measure(&binding, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR343_MENU_BINDING_SINGLE_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} projections_per_sample={PROJECTIONS_PER_SAMPLE} legacy_binding_conversions_per_projection=2 optimized_binding_conversions_per_projection=1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single menu binding projection must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn benchmark_binding() -> EditorUiBinding {
    let call = (0..16).fold(
        UiBindingCall::new(format!("PluginMenuAction{}", "x".repeat(96))),
        |call, index| {
            call.with_argument(UiBindingValue::string(format!(
                "plugin.menu.argument.{index:02}.{}",
                "y".repeat(96)
            )))
        },
    );
    EditorUiBinding::new(
        "WorkbenchMenuBarWithStableReflectionIdentity",
        "OpenDeepPluginConfigurationWithStableControlIdentity",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::Custom(call),
    )
}

fn legacy_projection(
    binding: &EditorUiBinding,
) -> (
    zircon_runtime_interface::ui::binding::UiEventKind,
    String,
    String,
) {
    let action = binding.as_ui_binding();
    let symbol = action
        .action
        .as_ref()
        .map(|call| call.symbol.clone())
        .unwrap_or_else(|| "Action".to_string());
    (action.path.event_kind, symbol, binding.native_binding())
}

fn measure(binding: &EditorUiBinding, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..PROJECTIONS_PER_SAMPLE {
        let projection = if optimized {
            menu_binding_projection(black_box(binding))
        } else {
            legacy_projection(black_box(binding))
        };
        black_box(projection);
    }
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() - 1) * percentile / 100]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
