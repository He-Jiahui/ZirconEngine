use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use zircon_runtime_interface::ProfileSnapshot;

use super::MEASURED_FRAMES;

pub(super) fn managed_output_root() -> PathBuf {
    let target_dir = PathBuf::from(
        std::env::var_os("CARGO_TARGET_DIR")
            .expect("managed Windows baseline requires coordinator CARGO_TARGET_DIR"),
    );
    let target_dir = std::fs::canonicalize(&target_dir).unwrap_or(target_dir);
    let normalized = target_dir
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_uppercase();
    assert!(
        [
            "D:\\",
            "E:\\",
            "F:\\",
            "\\\\?\\D:\\",
            "\\\\?\\E:\\",
            "\\\\?\\F:\\"
        ]
        .iter()
        .any(|prefix| normalized.starts_with(prefix)),
        "text profiling baseline output must use a coordinator-managed D/E/F root: {target_dir:?}"
    );
    target_dir
        .join("runtime-text-profile-baseline")
        .join(format!("run-{}", std::process::id()))
}

pub(super) fn assert_span_frame_count(snapshot: &ProfileSnapshot, category: &str, name: &str) {
    assert_exact_frame_index_coverage(
        snapshot
            .spans
            .iter()
            .filter(|span| span.category == category && span.name == name)
            .map(|span| span.frame_index),
        &format!("profile span {category}:{name}"),
    );
}

pub(super) fn assert_span_is_absent(snapshot: &ProfileSnapshot, category: &str, name: &str) {
    assert!(
        snapshot
            .spans
            .iter()
            .all(|span| span.category != category || span.name != name),
        "retained static-label frames must not record {category}:{name}"
    );
}

pub(super) fn assert_counter_frame_count(snapshot: &ProfileSnapshot, name: &str) {
    assert_exact_frame_index_coverage(
        snapshot
            .counters
            .iter()
            .filter(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.frame_index),
        &format!("profile counter {name}"),
    );
}

pub(super) fn assert_counter_is_absent(snapshot: &ProfileSnapshot, name: &str) {
    assert!(
        snapshot
            .counters
            .iter()
            .all(|counter| counter.stream != "runtime" || counter.name != name),
        "retained static-label frames must not record `{name}`"
    );
}

pub(super) fn assert_counter_is_zero(snapshot: &ProfileSnapshot, name: &str) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_exact_frame_index_coverage(
        samples.iter().map(|counter| counter.frame_index),
        &format!("profile counter {name}"),
    );
    assert!(
        samples.iter().all(|counter| counter.value == 0.0),
        "stable static-label baseline requires zero `{name}` samples"
    );
}

pub(super) fn assert_counter_is_positive(snapshot: &ProfileSnapshot, name: &str) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_exact_frame_index_coverage(
        samples.iter().map(|counter| counter.frame_index),
        &format!("profile counter {name}"),
    );
    assert!(
        samples.iter().all(|counter| counter.value > 0.0),
        "static-label baseline requires positive `{name}` samples"
    );
}

pub(super) fn assert_counter_peak_at_least(snapshot: &ProfileSnapshot, name: &str, minimum: f64) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_exact_frame_index_coverage(
        samples.iter().map(|counter| counter.frame_index),
        &format!("profile counter {name}"),
    );
    assert!(
        samples.iter().any(|counter| counter.value >= minimum),
        "queue-pressure baseline requires `{name}` to reach {minimum} on a measured frame"
    );
}

pub(super) fn assert_any_counter_is_positive(snapshot: &ProfileSnapshot, names: &[&str]) {
    let observed_positive = names.iter().any(|name| {
        let samples = counter_samples_by_frame(snapshot, name);
        assert_eq!(
            samples.len(),
            MEASURED_FRAMES,
            "profile counter {name} must publish exactly one sample per measured frame"
        );
        samples.values().any(|value| *value > 0.0)
    });
    assert!(
        observed_positive,
        "queue pressure must trigger either the frame request cap or bounded worker backpressure"
    );
}

pub(super) fn assert_counter_equals(snapshot: &ProfileSnapshot, name: &str, expected: f64) {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .collect::<Vec<_>>();
    assert_exact_frame_index_coverage(
        samples.iter().map(|counter| counter.frame_index),
        &format!("profile counter {name}"),
    );
    assert!(
        samples.iter().all(|counter| counter.value == expected),
        "stable static-label baseline requires `{name}` to equal {expected} on every frame"
    );
}

pub(super) fn assert_counter_does_not_exceed(
    snapshot: &ProfileSnapshot,
    observed_name: &str,
    limit_name: &str,
) {
    let observed = counter_samples_by_frame(snapshot, observed_name);
    let limits = counter_samples_by_frame(snapshot, limit_name);
    assert_eq!(
        observed.len(),
        MEASURED_FRAMES,
        "profile counter {observed_name} must publish exactly one sample per measured frame"
    );
    assert_eq!(
        limits.len(),
        MEASURED_FRAMES,
        "profile counter {limit_name} must publish exactly one sample per measured frame"
    );
    for (frame_index, observed_value) in observed {
        let limit_value = limits
            .get(&frame_index)
            .unwrap_or_else(|| panic!("profile counter {limit_name} omitted frame {frame_index}"));
        assert!(
            observed_value <= *limit_value,
            "profile counter {observed_name} exceeded {limit_name} at frame {frame_index}: {observed_value} > {limit_value}"
        );
    }
}

fn counter_samples_by_frame(snapshot: &ProfileSnapshot, name: &str) -> BTreeMap<u64, f64> {
    let samples = snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| {
            (
                counter.frame_index.unwrap_or_else(|| {
                    panic!("profile counter {name} omitted a profiler frame index")
                }),
                counter.value,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        samples.len(),
        snapshot
            .counters
            .iter()
            .filter(|counter| counter.stream == "runtime" && counter.name == name)
            .count(),
        "profile counter {name} must not publish duplicate frame samples"
    );
    samples
}

fn assert_exact_frame_index_coverage(
    frame_indices: impl IntoIterator<Item = Option<u64>>,
    sample_kind: &str,
) {
    let frame_indices = frame_indices.into_iter().collect::<Vec<_>>();
    assert_eq!(
        frame_indices.len(),
        MEASURED_FRAMES,
        "{sample_kind} must publish exactly one sample per measured frame"
    );
    let observed_frames = frame_indices
        .into_iter()
        .collect::<Option<BTreeSet<_>>>()
        .unwrap_or_else(|| panic!("{sample_kind} omitted a profiler frame index"));
    let expected_frames = (0..MEASURED_FRAMES as u64).collect::<BTreeSet<_>>();
    assert_eq!(
        observed_frames, expected_frames,
        "{sample_kind} must cover every measured frame exactly once"
    );
}
