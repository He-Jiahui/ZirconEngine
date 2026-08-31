use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityPath;

const ANIMATION_TARGET_NAMESPACE: &[u8] = b"zircon.animation.target.v1";
const ANIMATION_TARGET_HASH_BLOCK_BYTES: usize = 64;
const ANIMATION_TARGET_SEGMENT_LENGTH_BYTES: usize = std::mem::size_of::<u64>();
const LOWER_HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Stable identity for an animation target derived from its import path.
///
/// The identity contains no scene entity handle. Importers and runtime target
/// tables may therefore independently derive the same value from the same
/// ordered path segments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AnimationTargetId([u8; 16]);

impl AnimationTargetId {
    pub fn from_path(path: &EntityPath) -> Self {
        Self::from_segments(path.segments())
    }

    pub fn from_segments<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut hasher = blake3::Hasher::new();
        hasher.update(ANIMATION_TARGET_NAMESPACE);
        for segment in segments {
            update_segment_hash(&mut hasher, segment.as_ref());
        }

        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
        Self(bytes)
    }

    pub fn as_bytes(self) -> [u8; 16] {
        self.0
    }
}

fn update_segment_hash(hasher: &mut blake3::Hasher, segment: &str) {
    let bytes = segment.as_bytes();
    let segment_length = (bytes.len() as u64).to_le_bytes();
    if bytes.len() <= ANIMATION_TARGET_HASH_BLOCK_BYTES - ANIMATION_TARGET_SEGMENT_LENGTH_BYTES {
        let mut framed = [0_u8; ANIMATION_TARGET_HASH_BLOCK_BYTES];
        framed[..ANIMATION_TARGET_SEGMENT_LENGTH_BYTES].copy_from_slice(&segment_length);
        let framed_len = ANIMATION_TARGET_SEGMENT_LENGTH_BYTES + bytes.len();
        framed[ANIMATION_TARGET_SEGMENT_LENGTH_BYTES..framed_len].copy_from_slice(bytes);
        hasher.update(&framed[..framed_len]);
    } else {
        hasher.update(&segment_length);
        hasher.update(bytes);
    }
}

impl From<&EntityPath> for AnimationTargetId {
    fn from(path: &EntityPath) -> Self {
        Self::from_path(path)
    }
}

impl fmt::Display for AnimationTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut encoded = [0_u8; 32];
        for (index, byte) in self.0.iter().copied().enumerate() {
            encoded[index * 2] = LOWER_HEX_DIGITS[usize::from(byte >> 4)];
            encoded[index * 2 + 1] = LOWER_HEX_DIGITS[usize::from(byte & 0x0f)];
        }
        let encoded = std::str::from_utf8(&encoded).expect("lower hexadecimal digits are ASCII");
        formatter.write_str(encoded)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;
    use std::hint::black_box;
    use std::time::Instant;

    use super::AnimationTargetId;

    const SAMPLE_PAIRS: usize = 13;
    const IDS_PER_SAMPLE: usize = 131_072;
    const PATHS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_20260830ev_runtime558_formats_with_one_fixed_buffer_write() {
        let production = include_str!("target_id.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("formatter.write_str"));
        assert!(!production.contains("for byte in self.0"));

        let id = AnimationTargetId([
            0x00, 0x01, 0x0f, 0x10, 0x2a, 0x7f, 0x80, 0xab, 0xcd, 0xef, 0x55, 0xaa, 0x12, 0x34,
            0x56, 0xff,
        ]);
        assert_eq!(id.to_string(), "00010f102a7f80abcdef55aa123456ff");
        assert_eq!(id.to_string(), legacy_hex(id.0));
    }

    #[test]
    fn optimization_batch_20260830ev_runtime559_frames_short_segments_in_one_hash_update() {
        let production = include_str!("target_id.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("update_segment_hash"));
        assert!(production.contains("ANIMATION_TARGET_HASH_BLOCK_BYTES"));
        assert!(production.contains("hasher.update(&framed[..framed_len])"));

        for segments in [
            vec![],
            vec![""],
            vec!["Root", "Spine", "Chest", "Hand.R"],
            vec!["01234567890123456789012345678901234567890123456789012345"],
            vec!["012345678901234567890123456789012345678901234567890123456"],
            vec!["long segment that exceeds one complete BLAKE3 input block and uses the fallback"],
        ] {
            assert_eq!(
                AnimationTargetId::from_segments(segments.iter().copied()),
                legacy_target_id(&segments)
            );
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_20260830ev_runtime558_fixed_buffer_display_benchmark() {
        for _ in 0..4 {
            black_box(measure_display(false));
            black_box(measure_display(true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_display(false));
                optimized_samples.push(measure_display(true));
            } else {
                optimized_samples.push(measure_display(true));
                legacy_samples.push(measure_display(false));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_20260830ev_runtime559_short_segment_hash_benchmark() {
        for _ in 0..4 {
            black_box(measure_segment_hash(false));
            black_box(measure_segment_hash(true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_segment_hash(false));
                optimized_samples.push(measure_segment_hash(true));
            } else {
                optimized_samples.push(measure_segment_hash(true));
                legacy_samples.push(measure_segment_hash(false));
            }
        }

        report_segment_hash_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_display(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut total_len = 0_usize;
        for index in 0..IDS_PER_SAMPLE {
            let seed = (index as u64).wrapping_mul(0xd6e8_feb8_6659_fd93);
            let mut bytes = [0_u8; 16];
            bytes[..8].copy_from_slice(&seed.to_le_bytes());
            bytes[8..].copy_from_slice(&seed.rotate_left(29).to_le_bytes());
            let encoded = if optimized {
                AnimationTargetId(black_box(bytes)).to_string()
            } else {
                legacy_hex(black_box(bytes))
            };
            total_len += black_box(encoded.len());
            black_box(encoded);
        }
        assert_eq!(black_box(total_len), IDS_PER_SAMPLE * 32);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_hex(bytes: [u8; 16]) -> String {
        let mut encoded = String::with_capacity(32);
        for byte in bytes {
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
        }
        encoded
    }

    fn measure_segment_hash(optimized: bool) -> u128 {
        let paths = [
            ["Root", "Spine", "Chest", "Hand.R"],
            ["Character", "Skeleton", "UpperArm.L", "WeaponSocket"],
            ["Scene", "AnimatedMesh", "Face", "Jaw"],
        ];
        let started = Instant::now();
        let mut checksum = 0_u8;
        for index in 0..PATHS_PER_SAMPLE {
            let path = black_box(&paths[index % paths.len()]);
            let target = if optimized {
                AnimationTargetId::from_segments(path.iter().copied())
            } else {
                legacy_target_id(path)
            };
            checksum ^= target.0[index & 15];
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_target_id(segments: &[&str]) -> AnimationTargetId {
        let mut hasher = blake3::Hasher::new();
        hasher.update(super::ANIMATION_TARGET_NAMESPACE);
        for segment in segments {
            hasher.update(&(segment.len() as u64).to_le_bytes());
            hasher.update(segment.as_bytes());
        }
        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
        AnimationTargetId(bytes)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME558_FIXED_BUFFER_DISPLAY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} ids_per_sample={IDS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40"
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(60) / 100,
            "fixed-buffer display must reduce P95 by at least 40%"
        );
    }

    fn report_segment_hash_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME559_SHORT_SEGMENT_HASH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=15"
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(85) / 100,
            "single-update short segment hashing must reduce P95 by at least 15%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }
}
