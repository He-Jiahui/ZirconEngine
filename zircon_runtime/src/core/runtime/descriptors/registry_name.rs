use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::CoreError;
use crate::core::ServiceKind;

#[derive(Clone, Debug)]
pub struct RegistryName {
    // `value` remains the equality, hash, borrow, and serde authority; the
    // cached fields only avoid re-parsing the validated registry string.
    value: Arc<str>,
    module_end: usize,
    service_start: usize,
    kind: ServiceKind,
}

impl RegistryName {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let Some((module_end, kind_end)) = registry_separator_offsets(&value) else {
            return Err(CoreError::InvalidRegistryName(value));
        };
        if !is_canonical_module_namespace(&value[..module_end]) {
            return Err(CoreError::InvalidRegistryName(value));
        }

        let kind_start = module_end + 1;
        let kind_segment = &value.as_bytes()[kind_start..kind_end];
        let Some(kind) = ServiceKind::from_registry_segment_bytes(kind_segment) else {
            return Err(CoreError::InvalidRegistryName(value));
        };

        let service_start = kind_end + 1;
        let service = &value[service_start..];
        if !is_canonical_segment(service) {
            return Err(CoreError::InvalidRegistryName(value));
        }

        Ok(Self {
            value: Arc::from(value),
            module_end,
            service_start,
            kind,
        })
    }

    pub fn from_parts(module: &str, kind: ServiceKind, service: &str) -> Self {
        assert!(
            is_canonical_module_namespace(module),
            "registry name module namespaces must contain non-empty, trim-clean segments"
        );
        assert!(
            is_canonical_dot_free_segment(service),
            "registry name service segments must be non-empty, trim-clean, and dot-free"
        );
        let kind_segment = kind.as_str();
        let service_start = module.len() + kind_segment.len() + 2;
        let mut value =
            String::with_capacity(module.len() + kind_segment.len() + service.len() + 2);
        value.push_str(module);
        value.push('.');
        value.push_str(kind_segment);
        value.push('.');
        value.push_str(service);
        Self {
            value: Arc::from(value),
            module_end: module.len(),
            service_start,
            kind,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn module_name(&self) -> &str {
        &self.value[..self.module_end]
    }

    pub fn service_kind(&self) -> ServiceKind {
        self.kind
    }

    pub fn service_name(&self) -> &str {
        &self.value[self.service_start..]
    }
}

impl PartialEq for RegistryName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for RegistryName {}

impl Hash for RegistryName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Borrow<str> for RegistryName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Serialize for RegistryName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegistryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match Self::new(value) {
            Ok(name) => Ok(name),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

impl fmt::Display for RegistryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

fn is_canonical_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_whitespace() {
        return false;
    }
    let last = match chars.next_back() {
        Some(last) => last,
        None => first,
    };
    !last.is_whitespace()
}

fn is_canonical_dot_free_segment(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first.is_whitespace() || first == '.' {
        return false;
    }

    let mut last = first;
    for ch in chars {
        if ch == '.' {
            return false;
        }
        last = ch;
    }

    !last.is_whitespace()
}

fn is_canonical_module_namespace(value: &str) -> bool {
    let mut segment_start = 0;
    for (index, ch) in value.char_indices() {
        if ch != '.' {
            continue;
        }
        if !is_canonical_dot_free_segment(&value[segment_start..index]) {
            return false;
        }
        segment_start = index + 1;
    }
    is_canonical_dot_free_segment(&value[segment_start..])
}

fn registry_separator_offsets(value: &str) -> Option<(usize, usize)> {
    let bytes = value.as_bytes();
    let mut kind_end = None;
    let mut index = bytes.len();

    while index > 0 {
        index -= 1;
        if bytes[index] == b'.' {
            if let Some(kind_end) = kind_end {
                return Some((index, kind_end));
            }
            kind_end = Some(index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use super::RegistryName;
    use crate::core::ServiceKind;

    #[test]
    fn registry_name_clones_share_value_storage() {
        let name =
            RegistryName::new("Runtime.Core.Manager.WindowManager").expect("valid registry name");
        let cloned = name.clone();

        assert!(Arc::ptr_eq(&name.value, &cloned.value));
        assert_eq!(cloned.as_str(), "Runtime.Core.Manager.WindowManager");
        assert_eq!(cloned.module_name(), "Runtime.Core");
        assert_eq!(cloned.service_kind(), ServiceKind::Manager);
        assert_eq!(cloned.service_name(), "WindowManager");
    }

    fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * percentile).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    fn legacy_clone_sample(names: &[String], clones_per_name: usize) -> (u128, usize) {
        let started = Instant::now();
        let checksum = {
            let mut clones = Vec::with_capacity(names.len() * clones_per_name);
            for name in names {
                for _ in 0..clones_per_name {
                    clones.push(name.clone());
                }
            }
            let checksum = clones.iter().map(String::len).sum();
            black_box(&clones);
            checksum
        };
        (started.elapsed().as_nanos(), checksum)
    }

    fn shared_clone_sample(names: &[RegistryName], clones_per_name: usize) -> (u128, usize) {
        let started = Instant::now();
        let checksum = {
            let mut clones = Vec::with_capacity(names.len() * clones_per_name);
            for name in names {
                for _ in 0..clones_per_name {
                    clones.push(name.clone());
                }
            }
            let checksum = clones.iter().map(|name| name.as_str().len()).sum();
            black_box(&clones);
            checksum
        };
        (started.elapsed().as_nanos(), checksum)
    }

    #[test]
    #[ignore = "release-only registry-name clone performance evidence"]
    fn registry_name_clone_release_benchmark_evidence() {
        const NAMES: usize = 65_536;
        const CLONES_PER_NAME: usize = 8;
        const SAMPLE_PAIRS: usize = 21;

        let legacy_names: Vec<String> = (0..NAMES)
            .map(|index| format!("Runtime.Feature{index}.Manager.Service{index}"))
            .collect();
        let shared_names: Vec<RegistryName> = legacy_names
            .iter()
            .cloned()
            .map(|name| RegistryName::new(name).expect("generated registry name is valid"))
            .collect();

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut shared_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let ((legacy_ns, legacy_checksum), (shared_ns, shared_checksum)) = if pair % 2 == 0 {
                (
                    legacy_clone_sample(&legacy_names, CLONES_PER_NAME),
                    shared_clone_sample(&shared_names, CLONES_PER_NAME),
                )
            } else {
                let shared = shared_clone_sample(&shared_names, CLONES_PER_NAME);
                let legacy = legacy_clone_sample(&legacy_names, CLONES_PER_NAME);
                (legacy, shared)
            };
            assert_eq!(legacy_checksum, shared_checksum);
            legacy_samples.push(legacy_ns);
            shared_samples.push(shared_ns);
        }

        let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
        let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
        let shared_p50_ns = nearest_rank(&mut shared_samples.clone(), 50);
        let shared_p95_ns = nearest_rank(&mut shared_samples, 95);
        println!(
            "RUNTIME01_REGISTRY_NAME_BENCH_V1 names={NAMES} clones_per_name={CLONES_PER_NAME} \
             sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
             shared_p50_ns={shared_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             shared_p95_ns={shared_p95_ns} legacy_payload_allocations={} \
             shared_payload_allocations=0",
            NAMES * CLONES_PER_NAME
        );

        assert!(
            shared_p50_ns.saturating_mul(2) <= legacy_p50_ns,
            "shared clone P50 must be at least 50% faster: legacy={legacy_p50_ns}ns shared={shared_p50_ns}ns"
        );
        assert!(
            shared_p95_ns.saturating_mul(2) <= legacy_p95_ns,
            "shared clone P95 must be at least 50% faster: legacy={legacy_p95_ns}ns shared={shared_p95_ns}ns"
        );
    }
}
