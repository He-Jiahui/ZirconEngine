use crate::core::editor_message::SceneModeId;
use crate::scene::modes::{SceneModeActivation, SELECT_SCENE_MODE_ID, TRANSFORM_SCENE_MODE_ID};
use crate::scene::viewport::TransformHandleKind;

const CUSTOM_SCENE_MODE_PREFIX: &str = "Custom:";

pub(crate) fn symbol(mode: &SceneModeActivation) -> String {
    match mode {
        SceneModeActivation::Select => "Select".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Move) => "Transform.Move".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Rotate) => {
            "Transform.Rotate".to_string()
        }
        SceneModeActivation::Transform(TransformHandleKind::Scale) => "Transform.Scale".to_string(),
        SceneModeActivation::Custom(mode_id) => custom_symbol(mode_id.as_str()),
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<SceneModeActivation> {
    match symbol {
        "Select" => Some(SceneModeActivation::Select),
        "Transform.Move" => Some(SceneModeActivation::Transform(TransformHandleKind::Move)),
        "Transform.Rotate" => Some(SceneModeActivation::Transform(TransformHandleKind::Rotate)),
        "Transform.Scale" => Some(SceneModeActivation::Transform(TransformHandleKind::Scale)),
        custom
            if custom.starts_with(CUSTOM_SCENE_MODE_PREFIX)
                && custom.len() > CUSTOM_SCENE_MODE_PREFIX.len() =>
        {
            let mode_id = &custom[CUSTOM_SCENE_MODE_PREFIX.len()..];
            (mode_id != SELECT_SCENE_MODE_ID && mode_id != TRANSFORM_SCENE_MODE_ID)
                .then(|| SceneModeActivation::Custom(SceneModeId::new(mode_id)))
        }
        _ => None,
    }
}

fn custom_symbol(mode_id: &str) -> String {
    let mut symbol = String::with_capacity(CUSTOM_SCENE_MODE_PREFIX.len() + mode_id.len());
    symbol.push_str(CUSTOM_SCENE_MODE_PREFIX);
    symbol.push_str(mode_id);
    symbol
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::{custom_symbol, parse_symbol, symbol};
    use crate::core::editor_message::SceneModeId;
    use crate::scene::modes::{SceneModeActivation, SELECT_SCENE_MODE_ID};

    #[test]
    fn optimization_batch_fl_editor398_custom_symbol_preserves_round_trip_semantics() {
        let activation =
            SceneModeActivation::Custom(SceneModeId::new("plugin.mesh-paint.precision-brush"));
        let encoded = symbol(&activation);

        assert_eq!(encoded, "Custom:plugin.mesh-paint.precision-brush");
        assert_eq!(parse_symbol(&encoded), Some(activation));
        assert_eq!(parse_symbol("Custom:"), None);
        assert_eq!(
            parse_symbol(&format!("Custom:{SELECT_SCENE_MODE_ID}")),
            None
        );
    }

    #[test]
    #[ignore = "release-only custom scene mode symbol performance gate"]
    fn optimization_batch_fl_editor398_direct_custom_symbol_benchmark() {
        const SYMBOL_COUNT: usize = 262_144;
        const SAMPLE_COUNT: usize = 17;
        const PERFORMANCE_MARKER: &str = "EDITOR398_DIRECT_CUSTOM_SCENE_MODE_SYMBOL_BENCH_V1";
        let mode_ids = [
            "plugin.mesh-paint.precision-brush",
            "plugin.landscape.spline-edit",
            "plugin.animation.motion-warp",
            "plugin.cinematics.camera-rig",
        ];

        for _ in 0..4 {
            black_box(legacy_custom_symbols(&mode_ids, SYMBOL_COUNT));
            black_box(direct_custom_symbols(&mode_ids, SYMBOL_COUNT));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                legacy_samples.push(measure_fl(|| {
                    legacy_custom_symbols(&mode_ids, SYMBOL_COUNT)
                }));
                optimized_samples.push(measure_fl(|| {
                    direct_custom_symbols(&mode_ids, SYMBOL_COUNT)
                }));
            } else {
                optimized_samples.push(measure_fl(|| {
                    direct_custom_symbols(&mode_ids, SYMBOL_COUNT)
                }));
                legacy_samples.push(measure_fl(|| {
                    legacy_custom_symbols(&mode_ids, SYMBOL_COUNT)
                }));
            }
        }

        let legacy_p50_ns = percentile_fl_ns(&mut legacy_samples, 50);
        let legacy_p95_ns = percentile_fl_ns(&mut legacy_samples, 95);
        let optimized_p50_ns = percentile_fl_ns(&mut optimized_samples, 50);
        let optimized_p95_ns = percentile_fl_ns(&mut optimized_samples, 95);
        println!(
            "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} symbols={SYMBOL_COUNT} samples={SAMPLE_COUNT} allocations_per_symbol=1"
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
            "direct custom symbol P95 {optimized_p95_ns}ns must be at most 80% of format P95 {legacy_p95_ns}ns"
        );
    }

    fn legacy_custom_symbols(mode_ids: &[&str], symbol_count: usize) -> usize {
        let mut bytes = 0usize;
        for index in 0..symbol_count {
            let symbol = format!("Custom:{}", mode_ids[index % mode_ids.len()]);
            bytes = bytes.saturating_add(black_box(symbol).len());
        }
        bytes
    }

    fn direct_custom_symbols(mode_ids: &[&str], symbol_count: usize) -> usize {
        let mut bytes = 0usize;
        for index in 0..symbol_count {
            let symbol = custom_symbol(mode_ids[index % mode_ids.len()]);
            bytes = bytes.saturating_add(black_box(symbol).len());
        }
        bytes
    }

    fn measure_fl<T>(run: impl FnOnce() -> T) -> Duration {
        let started = Instant::now();
        black_box(run());
        started.elapsed()
    }

    fn percentile_fl_ns(samples: &mut [Duration], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * percentile).div_ceil(100);
        samples[rank.saturating_sub(1)].as_nanos()
    }
}
