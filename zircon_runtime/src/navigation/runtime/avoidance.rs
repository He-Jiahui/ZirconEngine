use crate::core::framework::navigation::{NavAvoidanceQuality, NavMeshAgentDescriptor};
use crate::core::math::{Real, Vec3};

use super::world_scan::RuntimeObstacle;

const MIN_AVOIDANCE_DISTANCE: Real = 0.001;

pub(super) fn avoidance_adjusted_target(
    entity: u64,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    obstacles: &[RuntimeObstacle],
    agents: &[(u64, Vec3, Real)],
) -> Vec3 {
    if matches!(agent.avoidance_quality, NavAvoidanceQuality::None) {
        return target;
    }
    let desired_delta = Vec3::new(target.x - current.x, 0.0, target.z - current.z);
    let desired_distance = desired_delta.length();
    if desired_distance <= Real::EPSILON {
        return target;
    }
    let mut avoidance = Vec3::ZERO;
    for obstacle in obstacles
        .iter()
        .filter(|obstacle| obstacle.avoidance_enabled && obstacle.entity != entity)
    {
        let limit = obstacle.radius + agent.radius.max(0.05) + 0.5;
        if let Some(contribution) = avoidance_contribution(current, obstacle.center, limit) {
            avoidance += contribution;
        }
    }
    for (other_entity, other_position, other_radius) in agents {
        if *other_entity == entity {
            continue;
        }
        let limit = agent.radius.max(0.05) + *other_radius + 0.25;
        if let Some(contribution) = avoidance_contribution(current, *other_position, limit) {
            avoidance += contribution;
        }
    }
    if avoidance.length_squared() <= Real::EPSILON {
        return target;
    }
    let direction = avoidance.normalize_or_zero();
    if direction.length_squared() <= Real::EPSILON {
        current
    } else {
        current + direction * desired_distance
    }
}

#[inline]
fn avoidance_contribution(current: Vec3, other: Vec3, limit: Real) -> Option<Vec3> {
    if !limit.is_finite() || limit <= MIN_AVOIDANCE_DISTANCE {
        return None;
    }
    let delta = Vec3::new(current.x - other.x, 0.0, current.z - other.z);
    let distance_squared = delta.x * delta.x + delta.z * delta.z;
    if !distance_squared.is_finite()
        || distance_squared <= MIN_AVOIDANCE_DISTANCE * MIN_AVOIDANCE_DISTANCE
        || distance_squared >= limit * limit
    {
        return None;
    }
    let distance = distance_squared.sqrt();
    Some((delta / distance) * (limit - distance))
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::avoidance_contribution;
    use crate::core::math::{Real, Vec3};

    const SAMPLE_PAIRS: usize = 17;
    const FAR_NEIGHBOR_COUNT: usize = 4_096;
    const PASSES_PER_SAMPLE: usize = 128;

    #[test]
    fn optimization_batch_hi_runtime590_squared_rejection_preserves_avoidance_boundaries() {
        let current = Vec3::ZERO;

        assert_eq!(
            avoidance_contribution(current, Vec3::new(4.0, 0.0, 0.0), 1.0),
            None
        );
        assert_eq!(
            avoidance_contribution(current, Vec3::new(0.001, 0.0, 0.0), 1.0),
            None
        );
        assert_eq!(
            avoidance_contribution(current, Vec3::new(0.5, 9.0, 0.0), 1.0),
            Some(Vec3::new(-0.5, 0.0, 0.0))
        );
    }

    #[test]
    #[ignore = "release-only navigation avoidance squared-rejection benchmark"]
    fn optimization_batch_hi_runtime590_squared_rejection_performance_evidence() {
        fn legacy_contribution(current: Vec3, other: Vec3, limit: Real) -> Option<Vec3> {
            let delta = Vec3::new(current.x - other.x, 0.0, current.z - other.z);
            let distance = (delta.x * delta.x + delta.z * delta.z).sqrt();
            if distance > 0.001 && distance < limit {
                Some(delta.normalize_or_zero() * (limit - distance))
            } else {
                None
            }
        }

        fn measure(
            neighbors: &[Vec3],
            contribution: impl Fn(Vec3, Vec3, Real) -> Option<Vec3>,
        ) -> u128 {
            let started = Instant::now();
            for _ in 0..PASSES_PER_SAMPLE {
                for neighbor in neighbors {
                    black_box(contribution(Vec3::ZERO, black_box(*neighbor), 1.0));
                }
            }
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let neighbors = (0..FAR_NEIGHBOR_COUNT)
            .map(|index| Vec3::new(8.0 + index as Real * 0.01, 0.0, 3.0))
            .collect::<Vec<_>>();

        for _ in 0..4 {
            black_box(measure(&neighbors, legacy_contribution));
            black_box(measure(&neighbors, avoidance_contribution));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure(&neighbors, legacy_contribution));
                optimized_samples.push(measure(&neighbors, avoidance_contribution));
            } else {
                optimized_samples.push(measure(&neighbors, avoidance_contribution));
                legacy_samples.push(measure(&neighbors, legacy_contribution));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        let checks_per_sample = FAR_NEIGHBOR_COUNT * PASSES_PER_SAMPLE;

        println!(
            "RUNTIME590_NAV_AVOIDANCE_SQUARED_REJECTION_BENCH_V1 \
sample_pairs={SAMPLE_PAIRS} far_neighbors={FAR_NEIGHBOR_COUNT} \
passes_per_sample={PASSES_PER_SAMPLE} checks_per_sample={checks_per_sample} \
legacy_sqrt_evaluations_per_sample={checks_per_sample} \
optimized_sqrt_evaluations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
            "squared-distance rejection must reduce far-neighbor P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
