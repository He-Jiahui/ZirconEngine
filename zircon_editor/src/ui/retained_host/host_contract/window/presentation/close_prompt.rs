use super::super::super::data::HostClosePromptData;
use super::super::super::redraw::HostRedrawRequest;
use super::super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_close_prompt(&self, prompt: HostClosePromptData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let close_prompt = &state.host_presentation.close_prompt;
            let damage = if close_prompt.visible {
                close_prompt.overlay_frame.clone()
            } else {
                prompt.overlay_frame.clone()
            };
            state.update_host_presentation(|presentation| {
                presentation.close_prompt = prompt;
            });
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_close_prompt(&self) {
        self.set_close_prompt(HostClosePromptData::default());
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::super::super::data::{FrameRect, HostClosePromptData};

    #[test]
    fn optimization_batch_ei_close_prompt_damage_clones_only_the_frame() {
        let source = include_str!("close_prompt.rs");
        let implementation = source
            .split("fn set_close_prompt")
            .nth(1)
            .expect("close prompt setter implementation")
            .split("pub(crate) fn clear_close_prompt")
            .next()
            .expect("bounded close prompt setter");

        assert!(implementation.contains("close_prompt.overlay_frame.clone()"));
        assert!(!implementation.contains("close_prompt.clone()"));
        assert!(!implementation.contains("let current ="));
    }

    #[test]
    #[ignore = "release-only close prompt frame-only clone benchmark"]
    fn optimization_batch_ei_close_prompt_frame_only_clone_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const UPDATES_PER_SAMPLE: usize = 16_384;

        fn measure_legacy(fixture: &HostClosePromptData) -> u128 {
            let started = Instant::now();
            let mut checksum = 0f32;
            for _ in 0..UPDATES_PER_SAMPLE {
                let current = black_box(fixture).clone();
                let damage = if current.visible {
                    current.overlay_frame
                } else {
                    FrameRect::default()
                };
                checksum += damage.width;
                black_box(damage);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(fixture: &HostClosePromptData) -> u128 {
            let started = Instant::now();
            let mut checksum = 0f32;
            for _ in 0..UPDATES_PER_SAMPLE {
                let current = black_box(fixture);
                let damage = if current.visible {
                    current.overlay_frame.clone()
                } else {
                    FrameRect::default()
                };
                checksum += damage.width;
                black_box(damage);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let fixture = HostClosePromptData {
            visible: true,
            target_window_id: format!("window.{}", "nested.".repeat(32)),
            title: "Unsaved project changes ".repeat(16),
            message: "The current editor document contains pending changes. ".repeat(16),
            details: "Save, discard, or cancel before closing this window. ".repeat(16),
            overlay_frame: FrameRect {
                x: 10.0,
                y: 20.0,
                width: 1_920.0,
                height: 1_080.0,
            },
            ..HostClosePromptData::default()
        };
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&fixture));
                optimized_samples.push(measure_optimized(&fixture));
            } else {
                optimized_samples.push(measure_optimized(&fixture));
                legacy_samples.push(measure_legacy(&fixture));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR371_CLOSE_PROMPT_FRAME_ONLY_CLONE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             updates_per_sample={UPDATES_PER_SAMPLE} prompt_string_bytes={} pair_order=alternating_legacy_even \
             legacy_full_prompt_clones_per_sample={UPDATES_PER_SAMPLE} optimized_full_prompt_clones_per_sample=0 \
             legacy_frame_clones_per_sample=0 optimized_frame_clones_per_sample={UPDATES_PER_SAMPLE} \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            fixture.target_window_id.len()
                + fixture.title.len()
                + fixture.message.len()
                + fixture.details.len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(20),
            "frame-only close prompt cloning must reduce P95 by at least 80%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
