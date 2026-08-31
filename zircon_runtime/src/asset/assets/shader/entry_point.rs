use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderShaderEntryPointDescriptor, RenderShaderStage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderEntryPointAsset {
    pub name: String,
    pub stage: String,
}

impl ShaderEntryPointAsset {
    pub fn descriptor(&self) -> Option<RenderShaderEntryPointDescriptor> {
        let stage = parse_stage(&self.stage)?;
        Some(RenderShaderEntryPointDescriptor {
            name: self.name.clone(),
            stage,
        })
    }
}

fn parse_stage(stage: &str) -> Option<RenderShaderStage> {
    let stage = stage.trim();
    if stage.eq_ignore_ascii_case("vertex")
        || stage.eq_ignore_ascii_case("vert")
        || stage.eq_ignore_ascii_case("vs")
    {
        Some(RenderShaderStage::Vertex)
    } else if stage.eq_ignore_ascii_case("fragment")
        || stage.eq_ignore_ascii_case("frag")
        || stage.eq_ignore_ascii_case("fs")
    {
        Some(RenderShaderStage::Fragment)
    } else if stage.eq_ignore_ascii_case("compute")
        || stage.eq_ignore_ascii_case("comp")
        || stage.eq_ignore_ascii_case("cs")
    {
        Some(RenderShaderStage::Compute)
    } else {
        None
    }
}

#[cfg(test)]
mod optimization_batch_gx_runtime579_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_gx_runtime579_shader_stage_preflight_preserves_descriptors() {
        for stage in ["vertex", "VERT", "fragment", "fs", "compute", "CS"] {
            let entry = ShaderEntryPointAsset {
                name: "main".to_string(),
                stage: stage.to_string(),
            };
            assert_eq!(entry.descriptor(), legacy_descriptor(&entry));
        }
        let invalid = ShaderEntryPointAsset {
            name: "wide-entry-name".repeat(64),
            stage: "mesh".to_string(),
        };
        assert_eq!(invalid.descriptor(), None);
        assert_eq!(invalid.descriptor(), legacy_descriptor(&invalid));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gx_runtime579_shader_stage_preflight_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 250_000;
        let entry = ShaderEntryPointAsset {
            name: "invalid-entry-point-name/".repeat(64),
            stage: "unsupported-stage".to_string(),
        };
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &entry, ITERATIONS));
                optimized.push(measure(true, &entry, ITERATIONS));
            } else {
                optimized.push(measure(true, &entry, ITERATIONS));
                legacy.push(measure(false, &entry, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME579_SHADER_STAGE_PREFLIGHT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} entry_name_bytes={} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            entry.name.len(),
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "stage preflight must improve invalid-entry P95 by at least 50%"
        );
    }

    fn legacy_descriptor(
        entry: &ShaderEntryPointAsset,
    ) -> Option<RenderShaderEntryPointDescriptor> {
        Some(RenderShaderEntryPointDescriptor {
            name: entry.name.clone(),
            stage: parse_stage(&entry.stage)?,
        })
    }

    fn measure(optimized: bool, entry: &ShaderEntryPointAsset, iterations: usize) -> u128 {
        let started = Instant::now();
        let mut rejected = 0_u64;
        for _ in 0..iterations {
            let descriptor = if optimized {
                black_box(entry).descriptor()
            } else {
                legacy_descriptor(black_box(entry))
            };
            rejected += u64::from(descriptor.is_none());
        }
        black_box(rejected);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
