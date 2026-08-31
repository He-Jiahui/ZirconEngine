use serde::{Deserialize, Serialize};

use crate::core::math::Real;

pub const OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 3;
pub const OIT_GPU_LAYER_SIZE_BYTES: u64 = 8;
pub const OIT_GPU_COUNT_SIZE_BYTES: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OitSettings {
    pub fragments_per_pixel_average: Real,
    pub sorted_fragment_max_count: u32,
    pub alpha_threshold: Real,
}

impl OitSettings {
    pub const DEFAULT: Self = Self {
        fragments_per_pixel_average: 4.0,
        sorted_fragment_max_count: 8,
        alpha_threshold: 0.0,
    };
}

impl Default for OitSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OitCapabilityProfile {
    pub fragment_writable_storage: bool,
    pub max_storage_buffers_per_shader_stage: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OitSupport {
    Supported,
    FallbackSorted { diagnostic: &'static str },
}

pub const fn oit_support(capabilities: OitCapabilityProfile) -> OitSupport {
    if !capabilities.fragment_writable_storage {
        return OitSupport::FallbackSorted {
            diagnostic: "OIT unavailable: fragment writable storage is not supported; using sorted transparency",
        };
    }
    if capabilities.max_storage_buffers_per_shader_stage
        < OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
    {
        return OitSupport::FallbackSorted {
            diagnostic: "OIT unavailable: max_storage_buffers_per_shader_stage is below 3; using sorted transparency",
        };
    }
    OitSupport::Supported
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OitBufferPlan {
    pub pixel_count: u64,
    pub fragments_per_pixel_capacity: u32,
    pub fragment_capacity: u64,
    pub layer_buffer_size_bytes: u64,
    pub count_buffer_size_bytes: u64,
}

impl OitBufferPlan {
    pub fn for_view(view_size: [u32; 2], settings: OitSettings) -> Self {
        let pixel_count = u64::from(view_size[0].max(1)) * u64::from(view_size[1].max(1));
        let fragments_per_pixel_capacity = if settings.fragments_per_pixel_average.is_finite() {
            settings.fragments_per_pixel_average.ceil().max(1.0) as u32
        } else {
            OitSettings::DEFAULT.fragments_per_pixel_average as u32
        };
        let fragment_capacity = pixel_count.saturating_mul(u64::from(fragments_per_pixel_capacity));
        Self {
            pixel_count,
            fragments_per_pixel_capacity,
            fragment_capacity,
            layer_buffer_size_bytes: fragment_capacity.saturating_mul(OIT_GPU_LAYER_SIZE_BYTES),
            count_buffer_size_bytes: pixel_count.saturating_mul(OIT_GPU_COUNT_SIZE_BYTES),
        }
    }

    pub(crate) const fn fits_storage_binding_size_limit(self, max_binding_size_bytes: u64) -> bool {
        max_binding_size_bytes > 0
            && self.layer_buffer_size_bytes <= max_binding_size_bytes
            && self.count_buffer_size_bytes <= max_binding_size_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OitFragment {
    pub color: [Real; 4],
    pub depth: Real,
}

impl OitFragment {
    pub const fn new(color: [Real; 4], depth: Real) -> Self {
        Self { color, depth }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OitResolveResult {
    pub color: [Real; 4],
    pub sorted_depths: Vec<Real>,
    pub merged_fragment_count: usize,
}

pub fn resolve_oit_fragments(
    fragments: &[OitFragment],
    sorted_fragment_max_count: u32,
) -> OitResolveResult {
    let ordered = ordered_oit_fragment_indices(fragments);

    let exact_count = ordered
        .len()
        .min(sorted_fragment_max_count.try_into().unwrap_or(usize::MAX));
    let mut premultiplied = [0.0; 4];
    for &(_, fragment_index) in &ordered[..exact_count] {
        let fragment = &fragments[fragment_index];
        blend_front_to_back(&mut premultiplied, fragment.color);
    }
    let mut merged_tail = [0.0; 4];
    for &(_, fragment_index) in &ordered[exact_count..] {
        let fragment = &fragments[fragment_index];
        blend_front_to_back(&mut merged_tail, fragment.color);
    }
    blend_front_to_back(&mut premultiplied, merged_tail);

    OitResolveResult {
        color: premultiplied,
        sorted_depths: ordered[..exact_count]
            .iter()
            .map(|(_, fragment_index)| fragments[*fragment_index].depth)
            .collect(),
        merged_fragment_count: ordered.len().saturating_sub(exact_count),
    }
}

fn ordered_oit_fragment_indices(fragments: &[OitFragment]) -> Vec<(u32, usize)> {
    let mut ordered = fragments
        .iter()
        .enumerate()
        .map(|(fragment_index, fragment)| (total_f32_sort_key(fragment.depth), fragment_index))
        .collect::<Vec<_>>();
    ordered.sort_unstable();
    ordered
}

fn total_f32_sort_key(value: f32) -> u32 {
    let bits = value.to_bits();
    if bits & (1 << 31) == 0 {
        bits ^ (1 << 31)
    } else {
        !bits
    }
}

fn blend_front_to_back(accumulated: &mut [Real; 4], color: [Real; 4]) {
    let alpha = color[3].clamp(0.0, 1.0);
    let remaining = 1.0 - accumulated[3];
    accumulated[0] += remaining * color[0].clamp(0.0, 1.0) * alpha;
    accumulated[1] += remaining * color[1].clamp(0.0, 1.0) * alpha;
    accumulated[2] += remaining * color[2].clamp(0.0, 1.0) * alpha;
    accumulated[3] += remaining * alpha;
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn render_oit_capability_gate_falls_back_to_sorted() {
        let supported = oit_support(OitCapabilityProfile {
            fragment_writable_storage: true,
            max_storage_buffers_per_shader_stage: OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        });
        assert_eq!(supported, OitSupport::Supported);

        let missing_fragment_storage = oit_support(OitCapabilityProfile {
            fragment_writable_storage: false,
            max_storage_buffers_per_shader_stage: OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        });
        assert!(matches!(
            missing_fragment_storage,
            OitSupport::FallbackSorted { diagnostic }
                if diagnostic.contains("fragment writable storage")
        ));

        let insufficient_bindings = oit_support(OitCapabilityProfile {
            fragment_writable_storage: true,
            max_storage_buffers_per_shader_stage: OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
        });
        assert!(matches!(
            insufficient_bindings,
            OitSupport::FallbackSorted { diagnostic }
                if diagnostic.contains("max_storage_buffers_per_shader_stage")
        ));
    }

    #[test]
    fn render_oit_buffer_plan_scales_with_viewport_and_average_fragments() {
        let plan = OitBufferPlan::for_view(
            [320, 180],
            OitSettings {
                fragments_per_pixel_average: 3.25,
                ..OitSettings::DEFAULT
            },
        );

        assert_eq!(plan.pixel_count, 57_600);
        assert_eq!(plan.fragments_per_pixel_capacity, 4);
        assert_eq!(plan.fragment_capacity, 230_400);
        assert_eq!(plan.layer_buffer_size_bytes, 230_400 * 8);
        assert_eq!(plan.count_buffer_size_bytes, 57_600 * 4);
    }

    #[test]
    fn render_oit_buffer_plan_requires_each_storage_buffer_to_fit_the_device_limit() {
        let plan = OitBufferPlan {
            pixel_count: 16,
            fragments_per_pixel_capacity: 4,
            fragment_capacity: 64,
            layer_buffer_size_bytes: 512,
            count_buffer_size_bytes: 64,
        };

        assert!(!plan.fits_storage_binding_size_limit(0));
        assert!(!plan.fits_storage_binding_size_limit(511));
        assert!(plan.fits_storage_binding_size_limit(512));

        let oversized_counts = OitBufferPlan {
            count_buffer_size_bytes: 513,
            ..plan
        };
        assert!(!oversized_counts.fits_storage_binding_size_limit(512));
    }

    #[test]
    fn render_oit_resolve_sorts_within_max_count() {
        let fragments = [
            OitFragment::new([1.0, 0.0, 0.0, 0.5], 0.2),
            OitFragment::new([0.0, 1.0, 0.0, 0.5], 0.9),
            OitFragment::new([0.0, 0.0, 1.0, 0.5], 0.5),
            OitFragment::new([1.0, 1.0, 0.0, 0.5], 0.7),
        ];

        let result = resolve_oit_fragments(&fragments, 2);

        assert_eq!(result.sorted_depths, vec![0.2, 0.5]);
        assert_eq!(result.merged_fragment_count, 2);
        assert!(result.color.iter().all(|component| component.is_finite()));
        assert!((0.0..=1.0).contains(&result.color[3]));
        assert!(
            result.color[0] > result.color[1],
            "closest red/blue layers must contribute before the merged yellow/green tail"
        );
    }

    #[test]
    fn optimization_batch_db_oit_index_order_matches_legacy_stable_sort() {
        let fragments = [
            OitFragment::new([1.0, 0.0, 0.0, 0.2], 0.5),
            OitFragment::new([0.0, 1.0, 0.0, 0.3], -0.0),
            OitFragment::new([0.0, 0.0, 1.0, 0.4], 0.5),
            OitFragment::new([1.0, 1.0, 0.0, 0.5], 0.0),
        ];

        assert_eq!(
            resolve_oit_fragments(&fragments, 3),
            legacy_resolve_oit_fragments(&fragments, 3)
        );
    }

    #[test]
    fn optimization_batch_db_oit_sort_key_matches_total_cmp() {
        let values = [
            f32::from_bits(0xffc0_0001),
            f32::NEG_INFINITY,
            -1.0,
            -0.0,
            0.0,
            1.0,
            f32::INFINITY,
            f32::from_bits(0x7fc0_0001),
        ];
        let mut expected = values;
        expected.sort_by(f32::total_cmp);
        let mut actual = values;
        actual.sort_unstable_by_key(|value| total_f32_sort_key(*value));

        assert_eq!(
            actual.map(f32::to_bits),
            expected.map(f32::to_bits),
            "integer sort key must reproduce the complete f32 total order"
        );
    }

    #[test]
    fn optimization_batch_db_oit_resolve_sorts_compact_keys_without_fragment_clone() {
        let source = include_str!("oit.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("ordered_oit_fragment_indices(fragments)"));
        assert!(production.contains("ordered.sort_unstable()"));
        assert!(!production.contains("fragments.to_vec()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_db_oit_index_sort_p95() {
        const FRAGMENT_COUNT: usize = 65_536;
        const SAMPLE_COUNT: usize = 17;
        const EXACT_COUNT: u32 = 4_096;
        let fragments = (0..FRAGMENT_COUNT)
            .map(|index| {
                let mixed = (index as u32).wrapping_mul(0x9e37_79b9);
                let depth = (mixed & 0x00ff_ffff) as f32 / 0x0100_0000 as f32;
                let channel = (index & 255) as f32 / 255.0;
                OitFragment::new([channel, 1.0 - channel, 0.5, 0.01], depth)
            })
            .collect::<Vec<_>>();

        let (legacy_samples, optimized_samples) = paired_samples::<SAMPLE_COUNT>(
            || legacy_resolve_oit_fragments(&fragments, EXACT_COUNT),
            || resolve_oit_fragments(&fragments, EXACT_COUNT),
        );
        assert_eq!(
            legacy_resolve_oit_fragments(&fragments, EXACT_COUNT),
            resolve_oit_fragments(&fragments, EXACT_COUNT)
        );

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT RUNTIME406_OIT_INDEX_SORT_BENCH_V1 fragments={FRAGMENT_COUNT} exact_fragments={EXACT_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95 * 10 <= legacy_p95 * 7,
            "optimized P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
        );
    }

    fn legacy_resolve_oit_fragments(
        fragments: &[OitFragment],
        sorted_fragment_max_count: u32,
    ) -> OitResolveResult {
        let mut ordered = fragments.to_vec();
        ordered.sort_by(|left, right| left.depth.total_cmp(&right.depth));
        let exact_count = ordered
            .len()
            .min(sorted_fragment_max_count.try_into().unwrap_or(usize::MAX));
        let mut premultiplied = [0.0; 4];
        for fragment in &ordered[..exact_count] {
            blend_front_to_back(&mut premultiplied, fragment.color);
        }
        let mut merged_tail = [0.0; 4];
        for fragment in &ordered[exact_count..] {
            blend_front_to_back(&mut merged_tail, fragment.color);
        }
        blend_front_to_back(&mut premultiplied, merged_tail);
        OitResolveResult {
            color: premultiplied,
            sorted_depths: ordered[..exact_count]
                .iter()
                .map(|fragment| fragment.depth)
                .collect(),
            merged_fragment_count: ordered.len().saturating_sub(exact_count),
        }
    }

    fn paired_samples<const SAMPLE_COUNT: usize, T>(
        mut legacy: impl FnMut() -> T,
        mut optimized: impl FnMut() -> T,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample_index in 0..SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(sample(&mut legacy));
                optimized_samples.push(sample(&mut optimized));
            } else {
                optimized_samples.push(sample(&mut optimized));
                legacy_samples.push(sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }
}
