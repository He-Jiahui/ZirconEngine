use super::AxisLabelKind;

const SCALE_LINK_CONTROL_ID: &str = "WorkbenchTransformScaleLink";
const TRANSFORM_CONTROL_ID_PREFIX: &str = "WorkbenchTransform";
const TRANSFORM_SCALE_AXIS_CONTROL_ID_PREFIX: &str = "WorkbenchTransformScaleAxis";

const AXIS_X_LABEL: &str = "X";
const AXIS_Y_LABEL: &str = "Y";
const AXIS_Z_LABEL: &str = "Z";

pub(super) fn axis_label_kind_from_control_id(control_id: &str) -> Option<AxisLabelKind> {
    if control_id == SCALE_LINK_CONTROL_ID {
        return Some(AxisLabelKind::ScaleLink);
    }
    transform_axis_label(control_id).map(AxisLabelKind::Axis)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_transform_scale_axis_control_id(
    control_id: &str,
) -> bool {
    control_id.starts_with(TRANSFORM_SCALE_AXIS_CONTROL_ID_PREFIX)
}

fn transform_axis_label(control_id: &str) -> Option<&'static str> {
    let field = control_id.strip_prefix(TRANSFORM_CONTROL_ID_PREFIX)?;
    let (axis_offset, axis) = field.char_indices().next_back()?;
    if !field[..axis_offset].ends_with("Axis") {
        return None;
    }
    match axis {
        'X' => Some(AXIS_X_LABEL),
        'Y' => Some(AXIS_Y_LABEL),
        'Z' => Some(AXIS_Z_LABEL),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn axis_label_kind_matches_transform_axis_and_scale_link_ids() {
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformPositionAxisX"),
            Some(AxisLabelKind::Axis("X"))
        );
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformRotationAxisY"),
            Some(AxisLabelKind::Axis("Y"))
        );
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformScaleAxisZ"),
            Some(AxisLabelKind::Axis("Z"))
        );
        assert_eq!(
            axis_label_kind_from_control_id("WorkbenchTransformScaleLink"),
            Some(AxisLabelKind::ScaleLink)
        );
    }

    #[test]
    fn transform_scale_axis_prefix_excludes_scale_link() {
        assert!(is_transform_scale_axis_control_id(
            "WorkbenchTransformScaleAxisX"
        ));
        assert!(!is_transform_scale_axis_control_id(
            "WorkbenchTransformScaleLink"
        ));
    }

    #[test]
    fn optimization_batch_gb_editor414_axis_label_suffix_dispatch_rejects_partial_suffixes() {
        assert_eq!(
            transform_axis_label("WorkbenchTransformPositionAxisX"),
            Some("X")
        );
        assert_eq!(
            transform_axis_label("WorkbenchTransformRotationAxisY"),
            Some("Y")
        );
        assert_eq!(
            transform_axis_label("WorkbenchTransformScaleAxisZ"),
            Some("Z")
        );
        assert_eq!(
            transform_axis_label("WorkbenchTransformPositionAxisQ"),
            None
        );
        assert_eq!(transform_axis_label("WorkbenchTransformAxis"), None);
        assert_eq!(transform_axis_label("WorkbenchTransform"), None);
    }

    const CHECKS_PER_SAMPLE: usize = 1_048_576;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_gb_editor414_axis_label_suffix_dispatch_benchmark() {
        const INPUT: &str = "WorkbenchTransformScaleAxisZ";
        for _ in 0..4 {
            black_box(measure_checks(INPUT, false));
            black_box(measure_checks(INPUT, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_checks(INPUT, false));
                optimized_samples.push(measure_checks(INPUT, true));
            } else {
                optimized_samples.push(measure_checks(INPUT, true));
                legacy_samples.push(measure_checks(INPUT, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR414_AXIS_LABEL_SUFFIX_DISPATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_suffix_checks_per_lookup=3 optimized_suffix_checks_per_lookup=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            INPUT.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let label = if optimized {
                transform_axis_label(black_box(input))
            } else {
                legacy_transform_axis_label(black_box(input))
            };
            black_box(label);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_transform_axis_label(control_id: &str) -> Option<&'static str> {
        let field = control_id.strip_prefix(TRANSFORM_CONTROL_ID_PREFIX)?;
        if field.ends_with("AxisX") {
            Some(AXIS_X_LABEL)
        } else if field.ends_with("AxisY") {
            Some(AXIS_Y_LABEL)
        } else if field.ends_with("AxisZ") {
            Some(AXIS_Z_LABEL)
        } else {
            None
        }
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
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
