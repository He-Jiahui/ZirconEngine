use serde::{Deserialize, Serialize};

use super::{NetEndpoint, NetRequestId, NetSecurityPolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetHttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetHttpRequestDescriptor {
    pub request: NetRequestId,
    pub method: NetHttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub timeout_ms: u64,
    pub security: NetSecurityPolicy,
    pub max_retry_attempts: u8,
}

impl NetHttpRequestDescriptor {
    pub fn new(request: NetRequestId, method: NetHttpMethod, url: impl Into<String>) -> Self {
        Self {
            request,
            method,
            url: url.into(),
            headers: Vec::new(),
            body: Vec::new(),
            timeout_ms: 30_000,
            security: NetSecurityPolicy::default(),
            max_retry_attempts: 0,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn with_max_retry_attempts(mut self, attempts: u8) -> Self {
        self.max_retry_attempts = attempts;
        self
    }

    pub fn with_byte_range(mut self, start: u64, end_inclusive: u64) -> Self {
        self.headers
            .retain(|(name, _)| !name.eq_ignore_ascii_case("range"));
        self.headers.push((
            "range".to_string(),
            byte_range_header_value(start, end_inclusive),
        ));
        self
    }
}

fn byte_range_header_value(start: u64, end_inclusive: u64) -> String {
    let mut value = String::with_capacity("bytes=".len() + 20 + 1 + 20);
    value.push_str("bytes=");
    push_u64_decimal(&mut value, start);
    value.push('-');
    push_u64_decimal(&mut value, end_inclusive);
    value
}

fn push_u64_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[start..] {
        output.push(char::from(*digit));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetHttpResponseDescriptor {
    pub request: NetRequestId,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub body_bytes: usize,
}

impl NetHttpResponseDescriptor {
    pub fn new(request: NetRequestId, status_code: u16, body: impl Into<Vec<u8>>) -> Self {
        let body = body.into();
        Self {
            request,
            status_code,
            headers: Vec::new(),
            body_bytes: body.len(),
            body,
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn for_request(mut self, request: NetRequestId) -> Self {
        self.request = request;
        self.body_bytes = self.body.len();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetHttpRouteDescriptor {
    pub path: String,
    pub methods: Vec<NetHttpMethod>,
    pub endpoint: Option<NetEndpoint>,
}

impl NetHttpRouteDescriptor {
    pub fn new(path: impl Into<String>, methods: impl IntoIterator<Item = NetHttpMethod>) -> Self {
        Self {
            path: path.into(),
            methods: methods.into_iter().collect(),
            endpoint: None,
        }
    }
}

#[cfg(test)]
mod optimization_batch_fa_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const VALUES_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fa_runtime459_preserves_http_byte_range_values() {
        for (start, end_inclusive) in [
            (0, 0),
            (1, 99),
            (4_096, 8_191),
            (u32::MAX as u64, u64::MAX),
            (u64::MAX, u64::MAX),
        ] {
            assert_eq!(
                byte_range_header_value(start, end_inclusive),
                format!("bytes={start}-{end_inclusive}")
            );
        }

        let request = NetHttpRequestDescriptor::new(
            NetRequestId::new(7),
            NetHttpMethod::Get,
            "https://example.invalid/chunk",
        )
        .with_header("accept", "application/octet-stream")
        .with_header("Range", "bytes=1-2")
        .with_byte_range(4_096, 8_191);
        assert_eq!(
            request.headers,
            vec![
                ("accept".to_string(), "application/octet-stream".to_string()),
                ("range".to_string(), "bytes=4096-8191".to_string()),
            ]
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fa_runtime459_direct_http_byte_range_benchmark() {
        for _ in 0..4 {
            black_box(measure_legacy());
            black_box(measure_optimized());
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy() -> u128 {
        measure(|start, end_inclusive| format!("bytes={start}-{end_inclusive}"))
    }

    fn measure_optimized() -> u128 {
        measure(byte_range_header_value)
    }

    fn measure(mut encode: impl FnMut(u64, u64) -> String) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for index in 0..VALUES_PER_SAMPLE {
            let start = black_box((index as u64).wrapping_mul(1_048_583));
            let end_inclusive = black_box(start.saturating_add(65_535));
            let value = encode(start, end_inclusive);
            checksum = checksum.wrapping_add(black_box(value.len()));
            black_box(value);
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
            "RUNTIME459_DIRECT_HTTP_BYTE_RANGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
            "direct HTTP byte-range construction must reduce P95 by at least 30%"
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
