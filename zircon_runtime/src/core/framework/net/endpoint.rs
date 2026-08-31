use std::fmt;
use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::NetError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetEndpoint {
    pub host: String,
    pub port: u16,
}

impl NetEndpoint {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn to_socket_addr(&self) -> Result<SocketAddr, NetError> {
        let literal_host = self
            .host
            .strip_prefix('[')
            .and_then(|host| host.strip_suffix(']'))
            .unwrap_or(self.host.as_str());
        let address = literal_host
            .parse::<IpAddr>()
            .map_err(|_| NetError::InvalidEndpoint {
                endpoint: self.to_string(),
            })?;
        Ok(SocketAddr::new(address, self.port))
    }
}

impl From<SocketAddr> for NetEndpoint {
    fn from(value: SocketAddr) -> Self {
        Self {
            host: value.ip().to_string(),
            port: value.port(),
        }
    }
}

impl fmt::Display for NetEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') && !(self.host.starts_with('[') && self.host.ends_with(']')) {
            return write!(f, "[{}]:{}", self.host, self.port);
        }
        write!(f, "{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod optimization_batch_ez_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const PARSES_PER_SAMPLE: usize = 131_072;

    #[test]
    fn optimization_batch_ez_runtime458_parses_literal_endpoint_families() {
        for (host, port, expected) in [
            ("127.0.0.1", 8080, "127.0.0.1:8080"),
            ("0.0.0.0", 0, "0.0.0.0:0"),
            ("::1", 443, "[::1]:443"),
            ("[2001:db8::7]", 9000, "[2001:db8::7]:9000"),
        ] {
            let endpoint = NetEndpoint::new(host, port);
            assert_eq!(endpoint.to_socket_addr().unwrap().to_string(), expected);
            assert_eq!(endpoint.to_string(), expected);
        }

        let invalid = NetEndpoint::new("not-an-ip-address", 80);
        assert!(matches!(
            invalid.to_socket_addr(),
            Err(NetError::InvalidEndpoint { endpoint }) if endpoint == "not-an-ip-address:80"
        ));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ez_runtime458_literal_endpoint_parse_benchmark() {
        let endpoint = NetEndpoint::new("192.0.2.17", 32_768);
        for _ in 0..4 {
            black_box(measure_legacy(&endpoint));
            black_box(measure_optimized(&endpoint));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&endpoint));
                optimized_samples.push(measure_optimized(&endpoint));
            } else {
                optimized_samples.push(measure_optimized(&endpoint));
                legacy_samples.push(measure_legacy(&endpoint));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy(endpoint: &NetEndpoint) -> u128 {
        measure(|| {
            format!("{}:{}", endpoint.host, endpoint.port)
                .parse::<SocketAddr>()
                .unwrap()
        })
    }

    fn measure_optimized(endpoint: &NetEndpoint) -> u128 {
        measure(|| endpoint.to_socket_addr().unwrap())
    }

    fn measure(mut parse: impl FnMut() -> SocketAddr) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..PARSES_PER_SAMPLE {
            let address = black_box(parse());
            checksum = checksum.wrapping_add(u64::from(address.port()));
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
            "RUNTIME458_LITERAL_ENDPOINT_PARSE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} parses_per_sample={PARSES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
            "literal endpoint parsing must reduce P95 by at least 30%"
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
