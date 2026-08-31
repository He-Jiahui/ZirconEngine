use std::sync::Arc;

use crate::core::math::Real;
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneMeshSourceIssue {
    NonFiniteLodDistance,
    NonPositiveLodDistance,
    DuplicateLodDistance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderSceneMeshBinding {
    pub(crate) mesh: ResourceHandle<MeshMarker>,
    pub(crate) material: ResourceHandle<MaterialMarker>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSceneMeshSourceLevel {
    pub(crate) model: ResourceHandle<ModelMarker>,
    pub(crate) mesh: Option<ResourceHandle<MeshMarker>>,
    pub(crate) material: ResourceHandle<MaterialMarker>,
    pub(crate) primitives: Arc<[RenderSceneMeshBinding]>,
}

impl RenderSceneMeshSourceLevel {
    pub(crate) fn new(
        model: ResourceHandle<ModelMarker>,
        mesh: Option<ResourceHandle<MeshMarker>>,
        material: ResourceHandle<MaterialMarker>,
        primitives: impl Into<Arc<[RenderSceneMeshBinding]>>,
    ) -> Self {
        Self {
            model,
            mesh,
            material,
            primitives: primitives.into(),
        }
    }

    fn geometry_eq(&self, other: &Self) -> bool {
        self.model == other.model
            && self.mesh == other.mesh
            && self.primitives.len() == other.primitives.len()
            && self
                .primitives
                .iter()
                .zip(other.primitives.iter())
                .all(|(current, previous)| current.mesh == previous.mesh)
    }

    fn materials_eq(&self, other: &Self) -> bool {
        self.material == other.material
            && self.primitives.len() == other.primitives.len()
            && self
                .primitives
                .iter()
                .zip(other.primitives.iter())
                .all(|(current, previous)| current.material == previous.material)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSceneMeshLod {
    pub(crate) min_distance: Real,
    pub(crate) source: RenderSceneMeshSourceLevel,
}

impl RenderSceneMeshLod {
    pub(crate) const fn new(min_distance: Real, source: RenderSceneMeshSourceLevel) -> Self {
        Self {
            min_distance,
            source,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSceneMeshSource {
    base: RenderSceneMeshSourceLevel,
    lods: Arc<[RenderSceneMeshLod]>,
}

impl RenderSceneMeshSource {
    pub(crate) fn new(
        base: RenderSceneMeshSourceLevel,
        lods: impl Into<Arc<[RenderSceneMeshLod]>>,
    ) -> Self {
        Self {
            base,
            lods: lods.into(),
        }
    }

    pub(crate) const fn base(&self) -> &RenderSceneMeshSourceLevel {
        &self.base
    }

    pub(crate) fn lods(&self) -> &[RenderSceneMeshLod] {
        &self.lods
    }

    pub(crate) fn select_for_distance(&self, distance: Real) -> RenderSceneMeshSelection<'_> {
        if !distance.is_finite() || distance < 0.0 {
            return RenderSceneMeshSelection::base(&self.base);
        }
        let selected_count = self
            .lods
            .partition_point(|lod| lod.min_distance <= distance);
        let Some(lod_index) = selected_count.checked_sub(1) else {
            return RenderSceneMeshSelection::base(&self.base);
        };
        let lod = &self.lods[lod_index];
        RenderSceneMeshSelection {
            source: &lod.source,
            lod_index: Some(lod_index),
            min_distance: Some(lod.min_distance),
        }
    }

    pub(super) fn canonicalize_lods(&mut self) -> Result<Vec<usize>, RenderSceneMeshSourceIssue> {
        let mut lods = self.lods.iter().cloned().enumerate().collect::<Vec<_>>();
        for (_, lod) in &lods {
            if !lod.min_distance.is_finite() {
                return Err(RenderSceneMeshSourceIssue::NonFiniteLodDistance);
            }
            if lod.min_distance <= 0.0 {
                return Err(RenderSceneMeshSourceIssue::NonPositiveLodDistance);
            }
        }
        lods.sort_by(|(_, left), (_, right)| left.min_distance.total_cmp(&right.min_distance));
        if lods
            .windows(2)
            .any(|pair| pair[0].1.min_distance == pair[1].1.min_distance)
        {
            return Err(RenderSceneMeshSourceIssue::DuplicateLodDistance);
        }
        let canonical_order = lods.iter().map(|(index, _)| *index).collect::<Vec<_>>();
        self.lods = lods
            .into_iter()
            .map(|(_, lod)| lod)
            .collect::<Vec<_>>()
            .into();
        Ok(canonical_order)
    }

    pub(super) fn geometry_eq(&self, other: &Self) -> bool {
        self.base.geometry_eq(&other.base)
            && self.lods.len() == other.lods.len()
            && self
                .lods
                .iter()
                .zip(other.lods.iter())
                .all(|(current, previous)| current.source.geometry_eq(&previous.source))
    }

    pub(super) fn lod_policy_eq(&self, other: &Self) -> bool {
        self.lods.len() == other.lods.len()
            && self
                .lods
                .iter()
                .zip(other.lods.iter())
                .all(|(current, previous)| current.min_distance == previous.min_distance)
    }

    pub(super) fn materials_eq(&self, other: &Self) -> bool {
        self.base.materials_eq(&other.base)
            && self.lods.len() == other.lods.len()
            && self
                .lods
                .iter()
                .zip(other.lods.iter())
                .all(|(current, previous)| current.source.materials_eq(&previous.source))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderSceneMeshSelection<'source> {
    source: &'source RenderSceneMeshSourceLevel,
    lod_index: Option<usize>,
    min_distance: Option<Real>,
}

impl<'source> RenderSceneMeshSelection<'source> {
    const fn base(source: &'source RenderSceneMeshSourceLevel) -> Self {
        Self {
            source,
            lod_index: None,
            min_distance: None,
        }
    }

    pub(crate) const fn source(self) -> &'source RenderSceneMeshSourceLevel {
        self.source
    }

    pub(crate) const fn lod_index(self) -> Option<usize> {
        self.lod_index
    }

    pub(crate) const fn min_distance(self) -> Option<Real> {
        self.min_distance
    }
}
