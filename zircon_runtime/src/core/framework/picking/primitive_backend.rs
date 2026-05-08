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
        rays.iter()
            .filter_map(|(ray_id, ray)| {
                let hits = self
                    .primitives
                    .iter()
                    .filter_map(|primitive| primitive.hit(ray_id.camera, ray))
                    .collect::<Vec<_>>();
                (!hits.is_empty()).then(|| PointerHits::new(ray_id.pointer, hits, self.info.order))
            })
            .collect()
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
