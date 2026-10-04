use std::collections::{BTreeSet, HashMap};

use crate::core::framework::scene::EntityId;
use crate::core::resource::ResourceId;

use super::{CameraRenderDescriptor, CameraRenderType, RenderCameraTarget};

#[cfg(test)]
#[path = "camera_ordering/tests/hash_target_count_tests.rs"]
mod hash_target_count_tests;

/// 场景世界提供的相机排序输入；实体 ID 只用于同序同目标时的稳定破同。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderCameraOrderInput {
    pub entity: EntityId,
    pub camera: CameraRenderDescriptor,
}

impl RenderCameraOrderInput {
    pub fn from_descriptor(entity: EntityId, camera: CameraRenderDescriptor) -> Self {
        Self { entity, camera }
    }
}

/// 帧抽取和渲染提交共用的相机顺序，歧义项供诊断使用。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderCameraOrderReport {
    pub cameras: Vec<SortedRenderCamera>,
    pub ambiguities: Vec<RenderCameraOrderAmbiguity>,
}

impl RenderCameraOrderReport {
    pub fn has_ambiguities(&self) -> bool {
        !self.ambiguities.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SortedRenderCamera {
    pub entity: EntityId,
    pub camera: CameraRenderDescriptor,
    pub render_type: CameraRenderType,
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

/// 先排除停用相机，再按绘制顺序与目标分组；调用方应保留此报告，
/// 避免可见性与实际渲染各自重新排序后产生不同的目标内索引。
pub fn sort_render_cameras(
    cameras: impl IntoIterator<Item = RenderCameraOrderInput>,
) -> RenderCameraOrderReport {
    let mut sorted = cameras
        .into_iter()
        .filter(|input| input.camera.is_active())
        .map(|input| {
            let descriptor = input.camera;
            let order = descriptor.render_order;
            let target = RenderCameraTargetOrderKey::from_target(&descriptor.target);
            let hdr = descriptor.hdr();
            let render_type = descriptor.render_type;
            SortedRenderCamera {
                entity: input.entity,
                order,
                target,
                hdr,
                render_type,
                camera: descriptor,
                sorted_camera_index_for_target: 0,
            }
        })
        .collect::<Vec<_>>();

    // Match Bevy's render-app ordering contract: order first, then target grouping.
    // Entity id is only a deterministic tiebreaker inside otherwise ambiguous groups.
    sorted.sort_by(|left, right| {
        (left.order, &left.target, left.entity).cmp(&(right.order, &right.target, right.entity))
    });

    let mut previous_order_target = None;
    let mut ambiguities = BTreeSet::new();
    let mut target_counts = HashMap::new();

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

#[cfg(test)]
#[path = "tests/camera_ordering.rs"]
mod tests;
