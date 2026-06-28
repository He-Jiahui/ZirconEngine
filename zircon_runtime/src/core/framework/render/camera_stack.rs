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
        .cloned()
        .collect::<Vec<_>>();
    resolve_active_camera_sequence(active)
}

fn resolve_active_camera_sequence(mut active: Vec<CameraRenderDescriptor>) -> CameraSequenceReport {
    active.sort_by(|left, right| {
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

    let mut violations = Vec::new();
    let mut sequence = Vec::new();

    for camera in &active {
        if camera.render_type == CameraRenderType::Overlay && !camera.stack.is_empty() {
            violations.push(CameraSequenceViolation {
                entity: camera.entity,
                reason: CameraSequenceViolationReason::OverlayCameraHasStack,
            });
        }
    }

    for base in active
        .iter()
        .filter(|camera| camera.render_type == CameraRenderType::Base)
    {
        let mut overlays = Vec::new();
        for referenced in &base.stack {
            match active
                .iter()
                .find(|camera| camera.entity == Some(*referenced))
            {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{RenderCameraTarget, RenderViewportRect};
    use crate::core::math::UVec2;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

    #[test]
    fn render_camera_sequence_sorts_by_render_order() {
        let sequence = resolve_camera_sequence([
            descriptor(
                30,
                3,
                CameraRenderType::Base,
                RenderCameraTarget::PrimarySurface,
            ),
            descriptor(
                10,
                1,
                CameraRenderType::Base,
                RenderCameraTarget::PrimarySurface,
            ),
            descriptor(
                20,
                2,
                CameraRenderType::Base,
                RenderCameraTarget::PrimarySurface,
            ),
        ]);

        assert_eq!(
            sequence
                .sequence
                .iter()
                .map(|entry| entry.base.entity)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(3)]
        );
        assert!(!sequence.has_violations());
    }

    #[test]
    fn render_camera_stack_overlay_follows_base_and_inherits_target_viewport() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "res://camera/base-target.png",
        ));
        let viewport = RenderViewportRect::new(UVec2::new(16, 8), UVec2::new(320, 180));
        let base = descriptor(
            0,
            1,
            CameraRenderType::Base,
            RenderCameraTarget::Texture(texture),
        )
        .with_stack([2, 3])
        .with_viewport(viewport);
        let overlay_a = descriptor(10, 2, CameraRenderType::Overlay, base.target.clone());
        let overlay_b = descriptor(5, 3, CameraRenderType::Overlay, base.target.clone());

        let report = resolve_camera_sequence([overlay_a, base, overlay_b]);

        assert_eq!(report.sequence.len(), 1);
        assert_eq!(
            report.sequence[0]
                .overlays
                .iter()
                .map(|camera| camera.entity)
                .collect::<Vec<_>>(),
            vec![Some(2), Some(3)]
        );
        assert_eq!(
            report.sequence[0].overlays[0].target,
            report.sequence[0].base.target
        );
        assert_eq!(report.sequence[0].overlays[0].viewport_rect, Some(viewport));
        assert!(!report.has_violations());
    }

    #[test]
    fn render_camera_stack_rejects_invalid_members() {
        let base = descriptor(
            0,
            1,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        )
        .with_stack([2, 3, 4]);
        let referenced_base = descriptor(
            0,
            2,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        );
        let mismatched_overlay = descriptor(
            0,
            3,
            CameraRenderType::Overlay,
            RenderCameraTarget::Headless {
                size: UVec2::new(64, 64),
            },
        );
        let stacked_overlay = descriptor(
            0,
            5,
            CameraRenderType::Overlay,
            RenderCameraTarget::PrimarySurface,
        )
        .with_stack([2]);

        let report =
            resolve_camera_sequence([base, referenced_base, mismatched_overlay, stacked_overlay]);

        assert_eq!(report.sequence.len(), 2);
        assert_eq!(
            report.violations,
            vec![
                CameraSequenceViolation {
                    entity: Some(5),
                    reason: CameraSequenceViolationReason::OverlayCameraHasStack,
                },
                CameraSequenceViolation {
                    entity: Some(1),
                    reason: CameraSequenceViolationReason::BaseStackReferencesNonOverlay {
                        referenced: 2,
                    },
                },
                CameraSequenceViolation {
                    entity: Some(1),
                    reason: CameraSequenceViolationReason::OverlayTargetDoesNotMatchBase {
                        referenced: 3,
                    },
                },
                CameraSequenceViolation {
                    entity: Some(1),
                    reason: CameraSequenceViolationReason::BaseStackReferencesMissingCamera {
                        referenced: 4,
                    },
                },
            ]
        );
        assert!(report.sequence[0].overlays.is_empty());
    }

    #[test]
    fn render_camera_sequence_resolves_borrowed_descriptors_without_consuming_source() {
        let base = descriptor(
            0,
            1,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        );
        let overlay = descriptor(
            0,
            2,
            CameraRenderType::Overlay,
            RenderCameraTarget::PrimarySurface,
        );
        let cameras = vec![base.with_stack([2]), overlay];

        let report = resolve_camera_sequence_borrowed(&cameras);

        assert_eq!(report.sequence.len(), 1);
        assert_eq!(report.sequence[0].base.entity, Some(1));
        assert_eq!(report.sequence[0].overlays[0].entity, Some(2));
        assert_eq!(cameras.len(), 2);
        assert!(!report.has_violations());
    }

    fn descriptor(
        order: i32,
        entity: EntityId,
        render_type: CameraRenderType,
        target: RenderCameraTarget,
    ) -> CameraRenderDescriptor {
        CameraRenderDescriptor {
            entity: Some(entity),
            render_order: order,
            render_type,
            target,
            ..CameraRenderDescriptor::from_camera_payload(
                Some(entity),
                ViewportCameraSnapshot::default(),
            )
        }
    }

    trait DescriptorTestExt {
        fn with_stack(self, stack: impl IntoIterator<Item = EntityId>) -> Self;
        fn with_viewport(self, viewport: RenderViewportRect) -> Self;
    }

    impl DescriptorTestExt for CameraRenderDescriptor {
        fn with_stack(mut self, stack: impl IntoIterator<Item = EntityId>) -> Self {
            self.stack = stack.into_iter().collect();
            self
        }

        fn with_viewport(mut self, viewport: RenderViewportRect) -> Self {
            self.viewport_rect = Some(viewport);
            self
        }
    }
}
