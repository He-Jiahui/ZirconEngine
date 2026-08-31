use zircon_runtime_interface::ui::surface::UiPointerEventKind;

pub(super) fn world_space_ui_pointer_status(
    kind: UiPointerEventKind,
    control_id: &str,
) -> Option<String> {
    match kind {
        UiPointerEventKind::Down => Some(world_space_status(
            "World-space UI target selected: ",
            control_id,
        )),
        UiPointerEventKind::Scroll => Some(world_space_status(
            "World-space UI scroll routed: ",
            control_id,
        )),
        UiPointerEventKind::Up => Some(world_space_status(
            "World-space UI target released: ",
            control_id,
        )),
        UiPointerEventKind::Move => None,
        UiPointerEventKind::Cancel => Some(world_space_status(
            "World-space UI target canceled: ",
            control_id,
        )),
    }
}

fn world_space_status(prefix: &str, control_id: &str) -> String {
    let mut status = String::with_capacity(prefix.len() + control_id.len());
    status.push_str(prefix);
    status.push_str(control_id);
    status
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::ui::surface::UiPointerEventKind;

    use super::{world_space_status, world_space_ui_pointer_status};

    const SAMPLE_PAIRS: usize = 17;
    const STATUSES_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fg_editor393_world_space_status_preserves_bytes() {
        for (kind, expected) in [
            (
                UiPointerEventKind::Down,
                "World-space UI target selected: button.save",
            ),
            (
                UiPointerEventKind::Scroll,
                "World-space UI scroll routed: list.body",
            ),
            (
                UiPointerEventKind::Up,
                "World-space UI target released: button.save",
            ),
            (
                UiPointerEventKind::Cancel,
                "World-space UI target canceled: list.body",
            ),
        ] {
            assert_eq!(
                world_space_ui_pointer_status(kind, expected.split_once(": ").unwrap().1),
                Some(expected.to_string())
            );
        }
        assert_eq!(
            world_space_ui_pointer_status(UiPointerEventKind::Move, "button.save"),
            None
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fg_editor393_direct_world_space_status_benchmark() {
        const CONTROL_ID: &str = "viewport.world_partition.content_surface";
        const PREFIX: &str = "World-space UI target selected: ";
        for _ in 0..4 {
            black_box(measure_statuses(|id| format!("{PREFIX}{id}"), CONTROL_ID));
            black_box(measure_statuses(
                |id| world_space_status(PREFIX, id),
                CONTROL_ID,
            ));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_statuses(|id| format!("{PREFIX}{id}"), CONTROL_ID));
                optimized_samples.push(measure_statuses(
                    |id| world_space_status(PREFIX, id),
                    CONTROL_ID,
                ));
            } else {
                optimized_samples.push(measure_statuses(
                    |id| world_space_status(PREFIX, id),
                    CONTROL_ID,
                ));
                legacy_samples.push(measure_statuses(|id| format!("{PREFIX}{id}"), CONTROL_ID));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_statuses(mut build: impl FnMut(&str) -> String, control_id: &str) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..STATUSES_PER_SAMPLE {
            let status = black_box(build(black_box(control_id)));
            checksum = checksum.wrapping_add(status.len());
            black_box(status);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR393_DIRECT_WORLD_SPACE_STATUS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} statuses_per_sample={STATUSES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
            "direct world-space status construction must reduce P95 by at least 25%"
        );
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
