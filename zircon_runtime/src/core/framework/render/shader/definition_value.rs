use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderShaderDefinitionValue {
    #[serde(rename = "bool")]
    Bool { name: String, value: bool },
    #[serde(rename = "int")]
    Int { name: String, value: i32 },
    #[serde(rename = "uint")]
    UInt { name: String, value: u32 },
}

impl RenderShaderDefinitionValue {
    pub fn bool(name: impl Into<String>, value: bool) -> Self {
        Self::Bool {
            name: name.into(),
            value,
        }
    }

    pub fn int(name: impl Into<String>, value: i32) -> Self {
        Self::Int {
            name: name.into(),
            value,
        }
    }

    pub fn uint(name: impl Into<String>, value: u32) -> Self {
        Self::UInt {
            name: name.into(),
            value,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Bool { name, .. } | Self::Int { name, .. } | Self::UInt { name, .. } => name,
        }
    }

    /// Returns the normalized view without copying the owned definition name.
    pub fn normalized_name(&self) -> &str {
        self.name().trim()
    }

    pub fn value_as_string(&self) -> String {
        match self {
            Self::Bool { value, .. } => value.to_string(),
            Self::Int { value, .. } => value.to_string(),
            Self::UInt { value, .. } => value.to_string(),
        }
    }
}

impl From<&str> for RenderShaderDefinitionValue {
    fn from(name: &str) -> Self {
        Self::bool(name, true)
    }
}

impl From<String> for RenderShaderDefinitionValue {
    fn from(name: String) -> Self {
        Self::bool(name, true)
    }
}

impl<'de> Deserialize<'de> for RenderShaderDefinitionValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum TaggedDefinitionValue {
            #[serde(rename = "bool")]
            Bool { name: String, value: bool },
            #[serde(rename = "int")]
            Int { name: String, value: i32 },
            #[serde(rename = "uint")]
            UInt { name: String, value: u32 },
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DefinitionValueRepr {
            BareFlag(String),
            Tagged(TaggedDefinitionValue),
        }

        Ok(match DefinitionValueRepr::deserialize(deserializer)? {
            DefinitionValueRepr::BareFlag(name) => Self::from(name),
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::Bool { name, value }) => {
                Self::bool(name, value)
            }
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::Int { name, value }) => {
                Self::int(name, value)
            }
            DefinitionValueRepr::Tagged(TaggedDefinitionValue::UInt { name, value }) => {
                Self::uint(name, value)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const NAMES_PER_SAMPLE: usize = 1_048_576;

    #[test]
    fn optimization_batch_fc_runtime461_normalized_name_trims_without_copying() {
        let definition = RenderShaderDefinitionValue::bool("  USE_CLUSTERED_LIGHTING  ", true);
        let normalized = definition.normalized_name();

        assert_eq!(normalized, "USE_CLUSTERED_LIGHTING");
        assert_eq!(
            normalized.as_ptr(),
            definition.name().as_ptr().wrapping_add(2)
        );
        assert_eq!(definition.name(), "  USE_CLUSTERED_LIGHTING  ");
    }

    #[test]
    fn optimization_batch_fc_runtime461_normalized_name_preserves_edge_cases() {
        for (name, expected) in [("", ""), ("   ", ""), ("USE_TAA", "USE_TAA")] {
            let definition = RenderShaderDefinitionValue::from(name);
            assert_eq!(definition.normalized_name(), expected);
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fc_runtime461_borrowed_shader_definition_name_benchmark() {
        let definition =
            RenderShaderDefinitionValue::bool("  ZR_GEOMETRY_SOURCE_SKINNED_MORPHED_MESH  ", true);
        for _ in 0..4 {
            black_box(measure_legacy(&definition));
            black_box(measure_optimized(&definition));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&definition));
                optimized_samples.push(measure_optimized(&definition));
            } else {
                optimized_samples.push(measure_optimized(&definition));
                legacy_samples.push(measure_legacy(&definition));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy(definition: &RenderShaderDefinitionValue) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..NAMES_PER_SAMPLE {
            let normalized = black_box(definition.name()).trim().to_string();
            checksum = checksum.wrapping_add(black_box(normalized.len()));
            black_box(normalized);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(definition: &RenderShaderDefinitionValue) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..NAMES_PER_SAMPLE {
            let normalized = black_box(definition).normalized_name();
            checksum = checksum.wrapping_add(black_box(normalized.len()));
            black_box(normalized);
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
            "RUNTIME461_BORROWED_SHADER_DEFINITION_NAME_BENCH_V1 sample_pairs={SAMPLE_PAIRS} names_per_sample={NAMES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=70",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(30) / 100,
            "borrowed shader definition names must reduce P95 by at least 70%"
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
