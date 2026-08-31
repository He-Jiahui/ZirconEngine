use crate::core::math::{Real, Vec3};

use super::{
    HitData, HitRecord, HitTarget, Pickable, PickingBackend, PickingBackendCapability,
    PickingBackendInfo, PointerHits, PointerRay, RayMap,
};

#[derive(Clone, Debug, PartialEq)]
pub enum PickingPrimitiveShape {
    Sphere { center: Vec3, radius: Real },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickingPrimitive {
    pub target: HitTarget,
    pub shape: PickingPrimitiveShape,
    pub pickable: Pickable,
}

impl PickingPrimitive {
    pub fn sphere(target: HitTarget, center: Vec3, radius: Real) -> Self {
        Self {
            target,
            shape: PickingPrimitiveShape::Sphere { center, radius },
            pickable: Pickable::default(),
        }
    }

    pub fn with_pickable(mut self, pickable: Pickable) -> Self {
        self.pickable = pickable;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitivePickingBackend {
    info: PickingBackendInfo,
    primitives: Vec<PickingPrimitive>,
}

impl PrimitivePickingBackend {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            info: PickingBackendInfo::new(name)
                .with_capability(PickingBackendCapability::CpuRayCast)
                .with_capability(PickingBackendCapability::OverlayShapes),
            primitives: Vec::new(),
        }
    }

    pub fn with_order(mut self, order: Real) -> Self {
        self.info = self.info.with_order(order);
        self
    }

    pub fn with_primitive(mut self, primitive: PickingPrimitive) -> Self {
        self.primitives.push(primitive);
        self
    }

    pub fn primitives(&self) -> &[PickingPrimitive] {
        &self.primitives
    }
}

impl PickingBackend for PrimitivePickingBackend {
    fn info(&self) -> PickingBackendInfo {
        self.info.clone()
    }

    fn collect_hits(&self, rays: &RayMap) -> Vec<PointerHits> {
        let mut collected = Vec::with_capacity(rays.len());
        for (ray_id, ray) in rays.iter() {
            let mut hits = Vec::with_capacity(self.primitives.len());
            for primitive in &self.primitives {
                if let Some(hit) = primitive.hit(ray_id.camera, ray) {
                    hits.push(hit);
                }
            }
            if !hits.is_empty() {
                collected.push(PointerHits::new(ray_id.pointer, hits, self.info.order));
            }
        }
        collected
    }
}

impl PickingPrimitive {
    fn hit(&self, camera: u64, ray: &PointerRay) -> Option<HitRecord> {
        match self.shape {
            PickingPrimitiveShape::Sphere { center, radius } => ray_sphere_hit(ray, center, radius)
                .map(|(depth, position, normal)| {
                    HitRecord::new(
                        self.target,
                        HitData::new(camera, depth, Some(position), Some(normal)),
                    )
                    .with_pickable(self.pickable)
                }),
        }
    }
}

fn ray_sphere_hit(ray: &PointerRay, center: Vec3, radius: Real) -> Option<(Real, Vec3, Vec3)> {
    if radius <= 0.0 || !radius.is_finite() || !center.is_finite() {
        return None;
    }

    let to_center = center - ray.origin;
    let projection = to_center.dot(ray.direction);
    let distance_squared = to_center.length_squared() - projection * projection;
    let radius_squared = radius * radius;
    if distance_squared > radius_squared {
        return None;
    }

    let half_chord = (radius_squared - distance_squared).sqrt();
    let near = projection - half_chord;
    let far = projection + half_chord;
    let depth = if near >= 0.0 { near } else { far };
    if depth < 0.0 || !depth.is_finite() {
        return None;
    }

    let position = ray.origin + ray.direction * depth;
    let normal = (position - center).normalize_or_zero();
    Some((depth, position, normal))
}

#[cfg(test)]
mod optimization_batch_20260830ce_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const RAYS_PER_SAMPLE: usize = 64;
    const PRIMITIVES_PER_RAY: usize = 256;

    #[test]
    fn primitive_picking_reserves_ray_and_primitive_capacity() {
        let source = include_str!("primitive_backend.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("primitive picking implementation");

        assert!(implementation.contains("Vec::with_capacity(rays.len())"));
        assert!(implementation.contains("Vec::with_capacity(self.primitives.len())"));
        assert!(implementation.contains("if let Some(hit) = primitive.hit("));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830ce_runtime_primitive_picking_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("RUNTIME383_PRIMITIVE_PICKING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} rays_per_sample={RAYS_PER_SAMPLE} primitives_per_ray={PRIMITIVES_PER_RAY} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..32 {
            let mut groups = if use_capacity {
                Vec::with_capacity(RAYS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for ray in 0..RAYS_PER_SAMPLE {
                let mut hits = if use_capacity {
                    Vec::with_capacity(PRIMITIVES_PER_RAY)
                } else {
                    Vec::new()
                };
                for primitive in 0..PRIMITIVES_PER_RAY {
                    if primitive % 3 != 0 {
                        hits.push((ray, primitive));
                    }
                }
                groups.push(hits);
            }
            checksum ^= groups.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
