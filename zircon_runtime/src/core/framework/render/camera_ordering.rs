use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::scene::EntityId;
use crate::core::resource::ResourceId;

use super::{RenderCameraTarget, ViewportCameraSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderCameraOrderInput {
    pub entity: EntityId,
    pub camera: ViewportCameraSnapshot,
}

impl RenderCameraOrderInput {
    pub fn new(entity: EntityId, camera: ViewportCameraSnapshot) -> Self {
        Self { entity, camera }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraOrderReport {
    pub cameras: Vec<SortedRenderCamera>,
    pub ambiguities: Vec<RenderCameraOrderAmbiguity>,
}

impl RenderCameraOrderReport {
    pub fn has_ambiguities(&self) -> bool {
        !self.ambiguities.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortedRenderCamera {
    pub entity: EntityId,
    pub order: i32,
    pub target: RenderCameraTargetOrderKey,
    pub hdr: bool,
    pub sorted_camera_index_for_target: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderCameraOrderAmbiguity {
    pub order: i32,
    pub target: RenderCameraTargetOrderKey,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderCameraTargetOrderKey {
    PrimarySurface,
    Headless { width: u32, height: u32 },
    Texture(ResourceId),
}

impl RenderCameraTargetOrderKey {
    pub fn from_target(target: &RenderCameraTarget) -> Self {
        match target {
            RenderCameraTarget::PrimarySurface => Self::PrimarySurface,
            RenderCameraTarget::Texture(handle) => Self::Texture(handle.id()),
            RenderCameraTarget::Headless { size } => Self::Headless {
                width: size.x,
                height: size.y,
            },
        }
    }
}

impl From<&RenderCameraTarget> for RenderCameraTargetOrderKey {
    fn from(value: &RenderCameraTarget) -> Self {
        Self::from_target(value)
    }
}

pub fn sort_render_cameras(
    cameras: impl IntoIterator<Item = RenderCameraOrderInput>,
) -> RenderCameraOrderReport {
    let mut sorted = cameras
        .into_iter()
        .filter(|input| input.camera.is_active)
        .map(|input| SortedRenderCamera {
            entity: input.entity,
            order: input.camera.order,
            target: RenderCameraTargetOrderKey::from_target(&input.camera.target),
            hdr: input.camera.hdr,
            sorted_camera_index_for_target: 0,
        })
        .collect::<Vec<_>>();

    // Match Bevy's render-app ordering contract: order first, then target grouping.
    // Entity id is only a deterministic tiebreaker inside otherwise ambiguous groups.
    sorted.sort_by(|left, right| {
        (left.order, &left.target, left.entity).cmp(&(right.order, &right.target, right.entity))
    });

    let mut previous_order_target = None;
    let mut ambiguities = BTreeSet::new();
    let mut target_counts = BTreeMap::new();

    for camera in &mut sorted {
        let order_target = (camera.order, camera.target.clone());
        if previous_order_target.as_ref() == Some(&order_target) {
            ambiguities.insert(RenderCameraOrderAmbiguity {
                order: camera.order,
                target: camera.target.clone(),
            });
        }

        let count = target_counts
            .entry((camera.target.clone(), camera.hdr))
            .or_insert(0usize);
        camera.sorted_camera_index_for_target = *count;
        *count += 1;

        previous_order_target = Some(order_target);
    }

    RenderCameraOrderReport {
        cameras: sorted,
        ambiguities: ambiguities.into_iter().collect(),
    }
}
