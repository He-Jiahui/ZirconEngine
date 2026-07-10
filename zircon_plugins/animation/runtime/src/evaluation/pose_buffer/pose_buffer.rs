use zircon_runtime::core::math::{Quat, Transform, Vec3};

/// Structure-of-arrays local pose storage indexed by a dense skeleton row.
#[derive(Debug)]
pub struct PoseBuffer {
    pub(super) translations: Vec<Vec3>,
    pub(super) rotations: Vec<Quat>,
    pub(super) scales: Vec<Vec3>,
    pub(super) weights: Vec<f32>,
}

impl PoseBuffer {
    pub fn new(joint_count: usize) -> Self {
        let mut buffer = Self::with_capacity(joint_count);
        buffer.reset(joint_count);
        buffer
    }

    pub(in crate::evaluation) fn with_capacity(joint_capacity: usize) -> Self {
        Self {
            translations: Vec::with_capacity(joint_capacity),
            rotations: Vec::with_capacity(joint_capacity),
            scales: Vec::with_capacity(joint_capacity),
            weights: Vec::with_capacity(joint_capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.translations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.translations.is_empty()
    }

    pub fn transform(&self, index: usize) -> Option<Transform> {
        Some(Transform {
            translation: *self.translations.get(index)?,
            rotation: *self.rotations.get(index)?,
            scale: *self.scales.get(index)?,
        })
    }

    pub fn weight(&self, index: usize) -> Option<f32> {
        self.weights.get(index).copied()
    }

    pub(in crate::evaluation) fn joint_capacity(&self) -> usize {
        self.translations
            .capacity()
            .min(self.rotations.capacity())
            .min(self.scales.capacity())
            .min(self.weights.capacity())
    }
}
