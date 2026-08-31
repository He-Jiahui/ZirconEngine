use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::UiNodePath;

const ACTIVITY_WINDOW_ROOT_PREFIX: &str = "editor/windows/";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityWindowDescriptor {
    pub window_id: String,
    pub title: String,
    pub icon_key: String,
    pub multi_instance: bool,
    pub supports_document_tab: bool,
    pub supports_exclusive_page: bool,
    pub supports_floating_window: bool,
    pub reflection_root: UiNodePath,
}

impl ActivityWindowDescriptor {
    pub fn new(
        window_id: impl Into<String>,
        title: impl Into<String>,
        icon_key: impl Into<String>,
    ) -> Self {
        let window_id = window_id.into();
        Self {
            reflection_root: activity_window_reflection_root(&window_id),
            window_id,
            title: title.into(),
            icon_key: icon_key.into(),
            multi_instance: false,
            supports_document_tab: true,
            supports_exclusive_page: true,
            supports_floating_window: true,
        }
    }

    pub fn with_multi_instance(mut self, multi_instance: bool) -> Self {
        self.multi_instance = multi_instance;
        self
    }

    pub fn with_supports_document_tab(mut self, supports: bool) -> Self {
        self.supports_document_tab = supports;
        self
    }

    pub fn with_supports_exclusive_page(mut self, supports: bool) -> Self {
        self.supports_exclusive_page = supports;
        self
    }

    pub fn with_supports_floating_window(mut self, supports: bool) -> Self {
        self.supports_floating_window = supports;
        self
    }

    pub fn with_reflection_root(mut self, root: UiNodePath) -> Self {
        self.reflection_root = root;
        self
    }
}

fn activity_window_reflection_root(window_id: &str) -> UiNodePath {
    let mut path = String::with_capacity(ACTIVITY_WINDOW_ROOT_PREFIX.len() + window_id.len());
    path.push_str(ACTIVITY_WINDOW_ROOT_PREFIX);
    path.push_str(window_id);
    UiNodePath::new(path)
}

#[cfg(test)]
mod optimization_batch_fn_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const PATHS_PER_SAMPLE: usize = 262_144;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fn_editor400_direct_reflection_root_preserves_path() {
        let descriptor = ActivityWindowDescriptor::new(
            "editor.material-instance",
            "Material Instance",
            "material-instance",
        );

        assert_eq!(
            descriptor.reflection_root.0,
            "editor/windows/editor.material-instance"
        );
        assert_eq!(descriptor.window_id, "editor.material-instance");
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fn_editor400_direct_reflection_root_benchmark() {
        let window_id = "editor.material-instance.long-lived-inspector-window";
        for _ in 0..4 {
            black_box(measure(window_id, legacy_reflection_root));
            black_box(measure(window_id, activity_window_reflection_root));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure(window_id, legacy_reflection_root));
                optimized_samples.push(measure(window_id, activity_window_reflection_root));
            } else {
                optimized_samples.push(measure(window_id, activity_window_reflection_root));
                legacy_samples.push(measure(window_id, legacy_reflection_root));
            }
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR400_DIRECT_ACTIVITY_WINDOW_ROOT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(80) / 100,
            "direct activity window reflection root must reduce P95 by at least 20%"
        );
    }

    fn legacy_reflection_root(window_id: &str) -> UiNodePath {
        UiNodePath::new(format!("editor/windows/{window_id}"))
    }

    fn measure(window_id: &str, build: fn(&str) -> UiNodePath) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..PATHS_PER_SAMPLE {
            let path = black_box(build(black_box(window_id)));
            checksum = checksum.wrapping_add(path.0.len());
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
