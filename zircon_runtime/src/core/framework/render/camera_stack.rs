use std::collections::HashMap;

use crate::core::framework::scene::EntityId;
use crate::core::math::{UVec2, Vec4};

use super::{
    aspect_ratio_from_viewport_size, RenderCameraTarget, RenderCameraTargetOrderKey,
    RenderLayerSet, ViewportCameraSnapshot,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CameraRenderType {
    #[default]
    Base,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderCameraClear {
    Skybox,
    Color(Vec4),
    DepthOnly,
    None,
}

impl Default for RenderCameraClear {
    fn default() -> Self {
        Self::Skybox
    }
}

/// 相机描述是提取阶段的中立 DTO：基础相机声明目标与清除规则，并通过 stack 按实体 ID 引用叠加相机。
/// `resolve_camera_sequence` 过滤未激活项、排序并校验引用，输出供图形视图构建使用的确定顺序。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraRenderDescriptor {
    pub entity: Option<EntityId>,
    pub render_order: i32,
    pub render_type: CameraRenderType,
    pub stack: Vec<EntityId>,
    pub target: RenderCameraTarget,
    pub viewport_rect: Option<super::RenderViewportRect>,
    pub clear: RenderCameraClear,
    pub clear_depth: bool,
    pub culling_mask: RenderLayerSet,
    pub volume_mask: RenderLayerSet,
    pub camera: ViewportCameraSnapshot,
}

impl CameraRenderDescriptor {
    pub fn from_camera_payload(entity: Option<EntityId>, camera: ViewportCameraSnapshot) -> Self {
        Self {
            entity,
            render_order: 0,
            render_type: CameraRenderType::Base,
            stack: Vec::new(),
            target: RenderCameraTarget::default(),
            viewport_rect: None,
            clear: RenderCameraClear::default(),
            clear_depth: true,
            culling_mask: RenderLayerSet::default(),
            volume_mask: RenderLayerSet::default(),
            camera,
        }
    }

    pub fn target_key(&self) -> RenderCameraTargetOrderKey {
        RenderCameraTargetOrderKey::from_target(&self.target)
    }

    pub fn hdr(&self) -> bool {
        self.camera.hdr
    }

    pub fn is_active(&self) -> bool {
        self.camera.is_active
    }

    pub fn as_effective_camera(&self) -> ViewportCameraSnapshot {
        self.camera.clone()
    }

    pub fn effective_viewport_size(&self, target_size: UVec2) -> UVec2 {
        self.viewport_rect
            .map(|viewport| viewport.clamped_to_size(target_size).physical_size)
            .unwrap_or(target_size)
    }

    pub fn effective_render_size(&self, target_size: UVec2) -> UVec2 {
        self.camera
            .dynamic_resolution
            .apply_to_size(self.effective_viewport_size(target_size))
    }

    pub fn apply_target_size(&mut self, target_size: UVec2) {
        self.camera.aspect_ratio =
            aspect_ratio_from_viewport_size(self.effective_viewport_size(target_size));
    }
}

impl From<ViewportCameraSnapshot> for CameraRenderDescriptor {
    fn from(value: ViewportCameraSnapshot) -> Self {
        Self::from_camera_payload(None, value)
    }
}

impl From<super::RenderCameraClearColor> for RenderCameraClear {
    fn from(value: super::RenderCameraClearColor) -> Self {
        match value {
            super::RenderCameraClearColor::Default => Self::Skybox,
            super::RenderCameraClearColor::None => Self::None,
            super::RenderCameraClearColor::Color(color) => Self::Color(color),
        }
    }
}

impl From<RenderCameraClear> for super::RenderCameraClearColor {
    fn from(value: RenderCameraClear) -> Self {
        match value {
            RenderCameraClear::Skybox => Self::Default,
            RenderCameraClear::Color(color) => Self::Color(color),
            RenderCameraClear::DepthOnly | RenderCameraClear::None => Self::None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraSequenceReport {
    pub sequence: Vec<CameraSequenceEntry>,
    pub violations: Vec<CameraSequenceViolation>,
}

impl CameraSequenceReport {
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CameraSequenceEntry {
    pub base: CameraRenderDescriptor,
    pub overlays: Vec<CameraRenderDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CameraSequenceViolation {
    pub entity: Option<EntityId>,
    pub reason: CameraSequenceViolationReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CameraSequenceViolationReason {
    OverlayCameraHasStack,
    BaseStackReferencesMissingCamera { referenced: EntityId },
    BaseStackReferencesNonOverlay { referenced: EntityId },
    OverlayTargetDoesNotMatchBase { referenced: EntityId },
}

pub fn resolve_camera_sequence(
    cameras: impl IntoIterator<Item = CameraRenderDescriptor>,
) -> CameraSequenceReport {
    let active = cameras
        .into_iter()
        .filter(CameraRenderDescriptor::is_active)
        .collect::<Vec<_>>();
    resolve_active_camera_sequence(active)
}

pub fn resolve_camera_sequence_borrowed<'a>(
    cameras: impl IntoIterator<Item = &'a CameraRenderDescriptor>,
) -> CameraSequenceReport {
    let active = cameras
        .into_iter()
        .filter(|camera| camera.is_active())
        .collect::<Vec<_>>();
    resolve_active_camera_sequence(active)
}

trait CameraDescriptorRef {
    fn camera_descriptor(&self) -> &CameraRenderDescriptor;
}

impl CameraDescriptorRef for CameraRenderDescriptor {
    fn camera_descriptor(&self) -> &CameraRenderDescriptor {
        self
    }
}

impl CameraDescriptorRef for &CameraRenderDescriptor {
    fn camera_descriptor(&self) -> &CameraRenderDescriptor {
        self
    }
}

fn resolve_active_camera_sequence(
    mut active: Vec<impl CameraDescriptorRef>,
) -> CameraSequenceReport {
    active.sort_by(|left, right| {
        let left = left.camera_descriptor();
        let right = right.camera_descriptor();
        (
            left.render_order,
            left.target_key(),
            left.entity.unwrap_or(EntityId::MAX),
        )
            .cmp(&(
                right.render_order,
                right.target_key(),
                right.entity.unwrap_or(EntityId::MAX),
            ))
    });

    let violation_capacity = active
        .iter()
        .map(|camera| camera.camera_descriptor().stack.len())
        .fold(active.len(), usize::saturating_add);
    let mut violations = Vec::with_capacity(violation_capacity);
    let mut sequence = Vec::with_capacity(active.len());

    for camera in &active {
        let camera = camera.camera_descriptor();
        if camera.render_type == CameraRenderType::Overlay && !camera.stack.is_empty() {
            violations.push(CameraSequenceViolation {
                entity: camera.entity,
                reason: CameraSequenceViolationReason::OverlayCameraHasStack,
            });
        }
    }
    let active_by_entity = index_active_cameras(&active);

    for base in active
        .iter()
        .map(|camera| camera.camera_descriptor())
        .filter(|camera| camera.render_type == CameraRenderType::Base)
    {
        let mut overlays = Vec::with_capacity(base.stack.len());
        for referenced in &base.stack {
            match active_by_entity.get(referenced).copied() {
                None => violations.push(CameraSequenceViolation {
                    entity: base.entity,
                    reason: CameraSequenceViolationReason::BaseStackReferencesMissingCamera {
                        referenced: *referenced,
                    },
                }),
                Some(overlay) if overlay.render_type != CameraRenderType::Overlay => {
                    violations.push(CameraSequenceViolation {
                        entity: base.entity,
                        reason: CameraSequenceViolationReason::BaseStackReferencesNonOverlay {
                            referenced: *referenced,
                        },
                    });
                }
                Some(overlay) if overlay.target_key() != base.target_key() => {
                    violations.push(CameraSequenceViolation {
                        entity: base.entity,
                        reason: CameraSequenceViolationReason::OverlayTargetDoesNotMatchBase {
                            referenced: *referenced,
                        },
                    });
                }
                Some(overlay) => {
                    let mut overlay = overlay.clone();
                    overlay.target = base.target.clone();
                    overlay.viewport_rect = base.viewport_rect;
                    overlays.push(overlay);
                }
            }
        }

        sequence.push(CameraSequenceEntry {
            base: base.clone(),
            overlays,
        });
    }

    CameraSequenceReport {
        sequence,
        violations,
    }
}

fn index_active_cameras<'a, T: CameraDescriptorRef>(
    active: &'a [T],
) -> HashMap<EntityId, &'a CameraRenderDescriptor> {
    let mut index = HashMap::with_capacity(active.len());
    for camera in active {
        let camera = camera.camera_descriptor();
        if let Some(entity) = camera.entity {
            index.entry(entity).or_insert(camera);
        }
    }
    index
}

#[cfg(test)]
#[path = "tests/camera_stack.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/optimization_batch_iz_runtime639_tests.rs"]
mod optimization_batch_iz_runtime639_tests;
