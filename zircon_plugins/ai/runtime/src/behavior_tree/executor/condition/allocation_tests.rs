use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{AiPerceptionSense, AiPerceptionStimulus};
use zircon_runtime::core::math::Vec3;

use super::PerceptionCondition;

const BENCHMARK_STIMULUS_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn perception_condition_matches_all_compiled_filter_fields() {
    let condition = PerceptionCondition {
        configured: true,
        sense: Some(AiPerceptionSense::Sight),
        source: Some(17),
        minimum_strength: Some(0.5),
        maximum_age_seconds: Some(0.25),
        expected_exists: true,
    };
    let matching = stimulus(AiPerceptionSense::Sight, 17, 0.75, 0.1);

    assert!(condition.matches(&matching));
    assert!(!condition.matches(&stimulus(AiPerceptionSense::Hearing, 17, 0.75, 0.1)));
    assert!(!condition.matches(&stimulus(AiPerceptionSense::Sight, 18, 0.75, 0.1)));
    assert!(!condition.matches(&stimulus(AiPerceptionSense::Sight, 17, 0.25, 0.1)));
    assert!(!condition.matches(&stimulus(AiPerceptionSense::Sight, 17, 0.75, 0.5)));
}

#[test]
fn perception_evaluation_compiles_filters_before_scanning_stimuli() {
    let source = include_str!("../condition.rs");
    let raw = source
        .split("fn raw_perception_condition_passes(")
        .nth(1)
        .and_then(|body| body.split("struct PerceptionCondition").next())
        .expect("raw perception condition body");

    assert!(raw.contains("PerceptionCondition::from_node(node)"));
    assert!(raw.contains("condition.matches(stimulus)"));
    assert!(!raw.contains("perception_stimulus_matches(node, stimulus)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn compiled_perception_condition_release_benchmark_evidence() {
    let parameters = synthetic_parameters();
    let stimuli = synthetic_stimuli();
    assert_eq!(
        legacy_match_count(&parameters, &stimuli),
        compiled_match_count(&parameters, &stimuli)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_match_count(black_box(&parameters), black_box(&stimuli)),
        || compiled_match_count(black_box(&parameters), black_box(&stimuli)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_compiled_perception_condition stimuli={BENCHMARK_STIMULUS_COUNT} parameters=4 samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_parameter_linear_scans_per_sample={} optimized_parameter_linear_scans_per_sample=1 legacy_sense_normalization_allocations_per_sample={BENCHMARK_STIMULUS_COUNT} optimized_sense_normalization_allocations_per_sample=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        BENCHMARK_STIMULUS_COUNT * 4,
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 2,
        "optimized P95 {optimized_p95}ns must be no more than 20% of legacy P95 {legacy_p95}ns"
    );
}

#[derive(Clone, Copy)]
struct SyntheticStimulus {
    sense: u8,
    source: u64,
    strength: f32,
    age_seconds: f32,
}

enum SyntheticValue {
    Text(String),
    Entity(u64),
    Scalar(f32),
}

struct SyntheticParameter {
    key: &'static str,
    value: SyntheticValue,
}

fn synthetic_parameters() -> Vec<SyntheticParameter> {
    vec![
        SyntheticParameter {
            key: "perception_sense",
            value: SyntheticValue::Text(" Sight ".to_string()),
        },
        SyntheticParameter {
            key: "perception_source",
            value: SyntheticValue::Entity(17),
        },
        SyntheticParameter {
            key: "perception_min_strength",
            value: SyntheticValue::Scalar(0.5),
        },
        SyntheticParameter {
            key: "perception_max_age_seconds",
            value: SyntheticValue::Scalar(0.25),
        },
    ]
}

fn synthetic_stimuli() -> Vec<SyntheticStimulus> {
    (0..BENCHMARK_STIMULUS_COUNT)
        .map(|index| SyntheticStimulus {
            sense: (index % 2) as u8,
            source: if index % 3 == 0 { 17 } else { 18 },
            strength: if index % 5 == 0 { 0.75 } else { 0.25 },
            age_seconds: if index % 7 == 0 { 0.1 } else { 0.5 },
        })
        .collect()
}

fn legacy_match_count(parameters: &[SyntheticParameter], stimuli: &[SyntheticStimulus]) -> usize {
    stimuli
        .iter()
        .filter(|stimulus| {
            let sense = text_parameter(parameters, "perception_sense")
                .map(|value| value.trim().to_ascii_lowercase())
                .map(|value| u8::from(value != "sight"))
                .unwrap_or_default();
            let source = entity_parameter(parameters, "perception_source");
            let minimum = scalar_parameter(parameters, "perception_min_strength");
            let maximum = scalar_parameter(parameters, "perception_max_age_seconds");
            stimulus.sense == sense
                && source.is_none_or(|value| stimulus.source == value)
                && minimum.is_none_or(|value| stimulus.strength >= value)
                && maximum.is_none_or(|value| stimulus.age_seconds <= value)
        })
        .count()
}

fn compiled_match_count(parameters: &[SyntheticParameter], stimuli: &[SyntheticStimulus]) -> usize {
    let filter = SyntheticFilter::from_parameters(parameters);
    stimuli
        .iter()
        .filter(|stimulus| {
            stimulus.sense == filter.sense
                && filter.source.is_none_or(|value| stimulus.source == value)
                && filter
                    .minimum_strength
                    .is_none_or(|value| stimulus.strength >= value)
                && filter
                    .maximum_age_seconds
                    .is_none_or(|value| stimulus.age_seconds <= value)
        })
        .count()
}

struct SyntheticFilter {
    sense: u8,
    source: Option<u64>,
    minimum_strength: Option<f32>,
    maximum_age_seconds: Option<f32>,
}

impl SyntheticFilter {
    fn from_parameters(parameters: &[SyntheticParameter]) -> Self {
        let mut filter = Self {
            sense: 0,
            source: None,
            minimum_strength: None,
            maximum_age_seconds: None,
        };
        for parameter in parameters {
            match (parameter.key, &parameter.value) {
                ("perception_sense", SyntheticValue::Text(value)) => {
                    filter.sense = u8::from(value.trim().to_ascii_lowercase() != "sight");
                }
                ("perception_source", SyntheticValue::Entity(value)) => {
                    filter.source = Some(*value);
                }
                ("perception_min_strength", SyntheticValue::Scalar(value)) => {
                    filter.minimum_strength = Some(*value);
                }
                ("perception_max_age_seconds", SyntheticValue::Scalar(value)) => {
                    filter.maximum_age_seconds = Some(*value);
                }
                _ => {}
            }
        }
        filter
    }
}

fn text_parameter<'a>(parameters: &'a [SyntheticParameter], key: &str) -> Option<&'a str> {
    parameters
        .iter()
        .find(|parameter| parameter.key == key)
        .and_then(|parameter| match &parameter.value {
            SyntheticValue::Text(value) => Some(value.as_str()),
            _ => None,
        })
}

fn entity_parameter(parameters: &[SyntheticParameter], key: &str) -> Option<u64> {
    parameters
        .iter()
        .find(|parameter| parameter.key == key)
        .and_then(|parameter| match &parameter.value {
            SyntheticValue::Entity(value) => Some(*value),
            _ => None,
        })
}

fn scalar_parameter(parameters: &[SyntheticParameter], key: &str) -> Option<f32> {
    parameters
        .iter()
        .find(|parameter| parameter.key == key)
        .and_then(|parameter| match &parameter.value {
            SyntheticValue::Scalar(value) => Some(*value),
            _ => None,
        })
}

fn stimulus(
    sense: AiPerceptionSense,
    source: u64,
    strength: f32,
    age_seconds: f32,
) -> AiPerceptionStimulus {
    AiPerceptionStimulus {
        source,
        sense,
        position: Vec3::new(0.0, 0.0, 0.0),
        strength,
        age_seconds,
    }
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
