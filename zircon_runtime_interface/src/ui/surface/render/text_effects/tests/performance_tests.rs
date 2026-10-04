use super::*;

fn legacy_normalized(effects: &UiTextDistanceFieldEffects) -> UiTextDistanceFieldEffects {
    UiTextDistanceFieldEffects {
        outline: effects
            .outline
            .as_ref()
            .map(UiTextOutlineEffect::normalized)
            .filter(UiTextOutlineEffect::is_active),
        shadow: effects
            .shadow
            .as_ref()
            .map(UiTextShadowEffect::normalized)
            .filter(UiTextShadowEffect::is_active),
        glow: effects
            .glow
            .as_ref()
            .map(UiTextGlowEffect::normalized)
            .filter(UiTextGlowEffect::is_active),
    }
}

#[test]
fn runtime_interface03_batch55_61_prefiltered_text_effects_preserve_legacy_normalization() {
    let cases = [
        UiTextDistanceFieldEffects::default(),
        UiTextDistanceFieldEffects {
            outline: Some(UiTextOutlineEffect {
                width_px: f32::INFINITY,
                color: String::new(),
            }),
            shadow: Some(UiTextShadowEffect {
                offset_x_px: 0.0,
                offset_y_px: 0.0,
                color: "#00000000".to_string(),
            }),
            glow: Some(UiTextGlowEffect {
                radius_px: f32::NAN,
                color: "#fff".to_string(),
            }),
        },
        UiTextDistanceFieldEffects {
            outline: Some(UiTextOutlineEffect {
                width_px: 96.0,
                color: "  #11223344  ".to_string(),
            }),
            shadow: Some(UiTextShadowEffect {
                offset_x_px: -128.0,
                offset_y_px: 3.0,
                color: String::new(),
            }),
            glow: Some(UiTextGlowEffect {
                radius_px: 2.0,
                color: "#ffffff".to_string(),
            }),
        },
        UiTextDistanceFieldEffects {
            outline: Some(UiTextOutlineEffect {
                width_px: 1.0,
                color: "#00000000".to_string(),
            }),
            shadow: Some(UiTextShadowEffect {
                offset_x_px: f32::EPSILON,
                offset_y_px: -f32::EPSILON,
                color: "#123456".to_string(),
            }),
            glow: Some(UiTextGlowEffect {
                radius_px: 3.0,
                color: "#ffffff80".to_string(),
            }),
        },
    ];

    for effects in cases {
        assert_eq!(effects.normalized(), legacy_normalized(&effects));
    }
}

#[test]
#[ignore = "release-only inactive text effect normalization benchmark"]
fn runtime_interface03_batch55_61_prefiltered_text_effects_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 250_000;
    const SAMPLE_COUNT: usize = 11;
    let effects = UiTextDistanceFieldEffects {
        outline: Some(UiTextOutlineEffect {
            width_px: f32::INFINITY,
            color: "#00000000".repeat(8),
        }),
        shadow: Some(UiTextShadowEffect {
            offset_x_px: 0.0,
            offset_y_px: 0.0,
            color: "#00000000".repeat(8),
        }),
        glow: Some(UiTextGlowEffect {
            radius_px: f32::NAN,
            color: "#00000000".repeat(8),
        }),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut prefiltered_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(legacy_normalized(black_box(&effects)));
            }
            started.elapsed().as_nanos()
        };
        let measure_prefiltered = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(black_box(&effects).normalized());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            prefiltered_samples.push(measure_prefiltered());
        } else {
            prefiltered_samples.push(measure_prefiltered());
            legacy_samples.push(measure_legacy());
        }
    }

    legacy_samples.sort_unstable();
    prefiltered_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_PREFILTER_TEXT_EFFECTS_BENCH_V1 builds={BUILD_COUNT} samples={SAMPLE_COUNT} legacy_p95_ns={} prefiltered_p95_ns={}",
        legacy_samples[p95], prefiltered_samples[p95],
    );
    assert!(
        prefiltered_samples[p95].saturating_mul(5) <= legacy_samples[p95].saturating_mul(4),
        "prefiltered inactive effects must improve P95 by at least 20%: legacy={}ns prefiltered={}ns",
        legacy_samples[p95],
        prefiltered_samples[p95],
    );
}

#[test]
fn shadow_prefilter_preserves_independently_normalized_axes() {
    let offsets = [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -128.0,
        -f32::EPSILON,
        0.0,
        f32::EPSILON,
        128.0,
    ];
    for offset_x_px in offsets {
        for offset_y_px in offsets {
            for color in ["", "#00000000", "  #11223344  "] {
                let effects = UiTextDistanceFieldEffects {
                    shadow: Some(UiTextShadowEffect {
                        offset_x_px,
                        offset_y_px,
                        color: color.to_string(),
                    }),
                    ..Default::default()
                };
                assert_eq!(effects.normalized(), legacy_normalized(&effects));
            }
        }
    }
}
