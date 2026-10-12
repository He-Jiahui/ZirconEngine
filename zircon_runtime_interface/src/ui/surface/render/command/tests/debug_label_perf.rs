use super::*;

#[test]
fn runtime_interface03_batch74_75_static_debug_labels_preserve_debug_formatting() {
    for kind in [
        UiRenderCommandKind::Group,
        UiRenderCommandKind::Quad,
        UiRenderCommandKind::Text,
        UiRenderCommandKind::Image,
    ] {
        assert_eq!(
            render_command_debug_label(kind),
            render_command_debug_label_formatting(kind),
        );
    }
}

#[test]
#[ignore = "release-only static render-command debug label benchmark"]
fn runtime_interface03_batch74_75_static_render_command_debug_label_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 1_000_000;
    const SAMPLE_COUNT: usize = 11;
    let kinds = [
        UiRenderCommandKind::Group,
        UiRenderCommandKind::Quad,
        UiRenderCommandKind::Text,
        UiRenderCommandKind::Image,
    ];
    let mut formatting_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut static_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_formatting = || {
            let started = Instant::now();
            for index in 0..BUILD_COUNT {
                black_box(render_command_debug_label_formatting(black_box(
                    kinds[index % kinds.len()],
                )));
            }
            started.elapsed().as_nanos()
        };
        let measure_static = || {
            let started = Instant::now();
            for index in 0..BUILD_COUNT {
                black_box(render_command_debug_label(black_box(
                    kinds[index % kinds.len()],
                )));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            formatting_samples.push(measure_formatting());
            static_samples.push(measure_static());
        } else {
            static_samples.push(measure_static());
            formatting_samples.push(measure_formatting());
        }
    }

    formatting_samples.sort_unstable();
    static_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STATIC_RENDER_COMMAND_DEBUG_LABEL_BENCH_V1 builds={BUILD_COUNT} samples={SAMPLE_COUNT} formatting_p95_ns={} static_p95_ns={}",
        formatting_samples[p95], static_samples[p95],
    );
    assert!(
        static_samples[p95].saturating_mul(5) <= formatting_samples[p95].saturating_mul(4),
        "static render-command debug labels must improve P95 by at least 20%: formatting={}ns static={}ns",
        formatting_samples[p95],
        static_samples[p95],
    );
}
