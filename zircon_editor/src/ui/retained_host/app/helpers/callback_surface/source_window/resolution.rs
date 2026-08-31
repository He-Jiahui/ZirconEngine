use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::layout::MainPageId;

pub(crate) fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId> {
    let generation = ui.get_host_presentation_generation();
    let host_shell = &generation.structure().host_shell;
    if !host_shell.native_floating_window_mode {
        return None;
    }

    owned_non_blank_window_id(&host_shell.native_floating_window_id).map(MainPageId::new)
}

fn owned_non_blank_window_id(window_id: &str) -> Option<String> {
    if window_id.trim().is_empty() {
        None
    } else {
        Some(window_id.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{owned_non_blank_window_id, resolve_callback_source_window_id};
    use crate::ui::retained_host::UiHostWindow;
    use crate::ui::workbench::layout::MainPageId;

    #[test]
    fn resolve_callback_source_window_id_returns_none_for_root_shell() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        assert_eq!(resolve_callback_source_window_id(&ui), None);
    }

    #[test]
    fn resolve_callback_source_window_id_reads_native_child_window_identity() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        let mut host_presentation = ui.get_host_presentation();
        host_presentation.host_shell.native_floating_window_mode = true;
        host_presentation.host_shell.native_floating_window_id = "window:native-preview".into();
        ui.set_host_presentation(host_presentation);

        assert_eq!(
            resolve_callback_source_window_id(&ui),
            Some(MainPageId::new("window:native-preview"))
        );
    }

    #[test]
    fn optimization_batch_fz_editor412_blank_native_child_window_id_stays_unresolved() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        let mut host_presentation = ui.get_host_presentation();
        host_presentation.host_shell.native_floating_window_mode = true;
        host_presentation.host_shell.native_floating_window_id = " \t ".into();
        ui.set_host_presentation(host_presentation);

        assert_eq!(resolve_callback_source_window_id(&ui), None);
        assert_eq!(
            owned_non_blank_window_id("window:native-preview"),
            Some("window:native-preview".to_owned())
        );
    }

    const BLANK_ID_BYTES: usize = 256;
    const CHECKS_PER_SAMPLE: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fz_editor412_borrowed_empty_source_window_id_benchmark() {
        let input = " ".repeat(BLANK_ID_BYTES);
        for _ in 0..4 {
            black_box(measure_resolutions(&input, false));
            black_box(measure_resolutions(&input, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_resolutions(&input, false));
                optimized_samples.push(measure_resolutions(&input, true));
            } else {
                optimized_samples.push(measure_resolutions(&input, true));
                legacy_samples.push(measure_resolutions(&input, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR412_BORROWED_EMPTY_SOURCE_WINDOW_ID_BENCH_V1 sample_pairs={SAMPLE_PAIRS} blank_id_bytes={BLANK_ID_BYTES} checks_per_sample={CHECKS_PER_SAMPLE} legacy_allocations_per_check=1 optimized_allocations_per_check=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 65 / 100);
    }

    fn measure_resolutions(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let result = if optimized {
                owned_non_blank_window_id(black_box(input))
            } else {
                legacy_owned_non_blank_window_id(black_box(input))
            };
            black_box(result);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_owned_non_blank_window_id(window_id: &str) -> Option<String> {
        let window_id = window_id.to_owned();
        if window_id.trim().is_empty() {
            None
        } else {
            Some(window_id)
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
