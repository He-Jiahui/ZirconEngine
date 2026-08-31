use std::sync::Arc;

use super::{EditorLogError, LogJump, LogSeverity, LogSource};

const MAX_LOG_MESSAGE_BYTES: usize = 8 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    source: LogSource,
    severity: LogSeverity,
    message: Arc<str>,
    timestamp_frame: u64,
    jump: Option<LogJump>,
}

impl LogEntry {
    pub fn new(
        source: LogSource,
        severity: LogSeverity,
        message: impl Into<String>,
        timestamp_frame: u64,
        jump: Option<LogJump>,
    ) -> Result<Self, EditorLogError> {
        let message = validated_message(message.into())?;
        Ok(Self {
            source,
            severity,
            message,
            timestamp_frame,
            jump,
        })
    }

    pub(crate) fn new_with_fallback(
        source: LogSource,
        severity: LogSeverity,
        message: String,
        fallback: &'static str,
        timestamp_frame: u64,
        jump: Option<LogJump>,
    ) -> Result<Self, EditorLogError> {
        let message =
            validated_message(message).or_else(|_| validated_message(fallback.to_owned()))?;
        Ok(Self {
            source,
            severity,
            message,
            timestamp_frame,
            jump,
        })
    }

    pub fn source(&self) -> &LogSource {
        &self.source
    }

    pub const fn severity(&self) -> LogSeverity {
        self.severity
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn timestamp_frame(&self) -> u64 {
        self.timestamp_frame
    }

    pub fn jump(&self) -> Option<&LogJump> {
        self.jump.as_ref()
    }

    pub fn estimated_bytes(&self) -> usize {
        self.source.estimated_bytes()
            + self.message.len()
            + self.jump.as_ref().map_or(0, LogJump::estimated_bytes)
            + std::mem::size_of::<u64>()
    }
}

fn validated_message(message: String) -> Result<Arc<str>, EditorLogError> {
    if message.trim().is_empty() {
        return Err(EditorLogError::EmptyMessage);
    }
    if message.len() > MAX_LOG_MESSAGE_BYTES {
        return Err(EditorLogError::MessageTooLong {
            maximum: MAX_LOG_MESSAGE_BYTES,
            actual: message.len(),
        });
    }
    Ok(Arc::from(message))
}

#[cfg(test)]
mod optimization_batch_hc_editor584_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_hc_editor584_log_entry_fallback_preserves_message_and_jump() {
        let jump = LogJump::asset("res://models/hero.glb").unwrap();
        let valid = LogEntry::new_with_fallback(
            LogSource::import(),
            LogSeverity::Info,
            "imported".to_owned(),
            "fallback",
            4,
            Some(jump.clone()),
        )
        .unwrap();
        let fallback = LogEntry::new_with_fallback(
            LogSource::import(),
            LogSeverity::Warning,
            String::new(),
            "fallback",
            5,
            Some(jump.clone()),
        )
        .unwrap();

        assert_eq!(valid.message(), "imported");
        assert_eq!(valid.jump(), Some(&jump));
        assert_eq!(fallback.message(), "fallback");
        assert_eq!(fallback.jump(), Some(&jump));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_hc_editor584_log_entry_fallback_move_p95() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 262_144;
        let jump = LogJump::asset("res://asset/".repeat(128)).unwrap();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &jump, ITERATIONS));
                optimized.push(measure(true, &jump, ITERATIONS));
            } else {
                optimized.push(measure(true, &jump, ITERATIONS));
                legacy.push(measure(false, &jump, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR584_LOG_ENTRY_FALLBACK_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(95),
            "single-move log fallback construction must improve P95 by at least 5%"
        );
    }

    fn measure(optimized: bool, jump: &LogJump, iterations: usize) -> u128 {
        let started = Instant::now();
        let mut bytes = 0_usize;
        for _ in 0..iterations {
            let entry = if optimized {
                LogEntry::new_with_fallback(
                    LogSource::import(),
                    LogSeverity::Info,
                    "imported".to_owned(),
                    "fallback",
                    0,
                    Some(jump.clone()),
                )
            } else {
                let owned_jump = jump.clone();
                LogEntry::new(
                    LogSource::import(),
                    LogSeverity::Info,
                    "imported",
                    0,
                    Some(owned_jump.clone()),
                )
                .or_else(|_| {
                    LogEntry::new(
                        LogSource::import(),
                        LogSeverity::Info,
                        "fallback",
                        0,
                        Some(owned_jump),
                    )
                })
            }
            .expect("fixture log entry should be valid");
            bytes ^= black_box(entry.estimated_bytes());
        }
        black_box(bytes);
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
