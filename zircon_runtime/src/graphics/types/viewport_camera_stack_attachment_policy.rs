//! 清屏意图与 WGPU attachment load 分离：分屏相机要局部清屏，不能误清整张目标纹理。
use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, PostProcessGraphResourceNames, RenderCameraClear,
};
use crate::core::math::Vec4;
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ViewportCameraStackAttachmentPolicy {
    scene_clear_plan: ViewportSceneClearPlan,
}

/// Camera clear intent is kept separate from graph attachment load ops because WGPU load clears
/// affect the whole texture view; split-view cameras need a later region-scoped draw clear.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ViewportSceneClearPlan {
    scene_color: Option<ViewportSceneColorClear>,
    scene_depth: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ViewportSceneColorClear {
    Preview,
    Color(Vec4),
    Transparent,
}

impl ViewportSceneClearPlan {
    pub(crate) const fn new(
        scene_color: Option<ViewportSceneColorClear>,
        scene_depth: bool,
    ) -> Self {
        Self {
            scene_color,
            scene_depth,
        }
    }

    pub(crate) fn scene_color(self) -> Option<ViewportSceneColorClear> {
        self.scene_color
    }

    pub(crate) fn scene_depth(self) -> bool {
        self.scene_depth
    }

    pub(crate) fn has_clear(self) -> bool {
        self.scene_color.is_some() || self.scene_depth
    }
}

impl ViewportSceneColorClear {
    pub(crate) fn resolve(self, preview_clear_color: Vec4) -> Vec4 {
        match self {
            Self::Preview => preview_clear_color,
            Self::Color(color) => color,
            Self::Transparent => Vec4::ZERO,
        }
    }
}

impl ViewportCameraStackAttachmentPolicy {
    pub(crate) fn from_camera(camera: &CameraRenderDescriptor) -> Self {
        match camera.render_type {
            CameraRenderType::Base => Self::from_base_camera(camera),
            CameraRenderType::Overlay => Self::from_overlay_camera(camera),
        }
    }

    #[cfg(test)]
    fn scene_color_ops(self) -> RenderGraphAttachmentOps {
        self.apply_to_first_attachment_write(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphAttachmentOps::clear_store(),
        )
    }

    #[cfg(test)]
    fn scene_depth_ops(self) -> RenderGraphAttachmentOps {
        self.apply_to_first_attachment_write(
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphAttachmentOps::clear_store(),
        )
    }

    pub(crate) fn scene_clear_plan(self) -> ViewportSceneClearPlan {
        self.scene_clear_plan
    }

    pub(crate) fn apply_to_first_attachment_write(
        self,
        resource_name: &str,
        graph_ops: RenderGraphAttachmentOps,
    ) -> RenderGraphAttachmentOps {
        if graph_ops.load != RenderGraphAttachmentLoadOp::Clear {
            return graph_ops;
        }
        match resource_name {
            PostProcessGraphResourceNames::SCENE_COLOR => RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Load,
                store: graph_ops.store,
            },
            PostProcessGraphResourceNames::SCENE_DEPTH => RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Load,
                store: graph_ops.store,
            },
            _ => graph_ops,
        }
    }

    const fn from_base_camera(camera: &CameraRenderDescriptor) -> Self {
        let scene_color = match camera.clear {
            RenderCameraClear::Skybox => Some(ViewportSceneColorClear::Preview),
            RenderCameraClear::Color(color) => Some(ViewportSceneColorClear::Color(color)),
            RenderCameraClear::DepthOnly => None,
            RenderCameraClear::None if camera.camera.msaa_samples > 1 => {
                Some(ViewportSceneColorClear::Transparent)
            }
            RenderCameraClear::None => None,
        };
        let scene_depth = match camera.clear {
            RenderCameraClear::Skybox
            | RenderCameraClear::Color(_)
            | RenderCameraClear::DepthOnly => true,
            RenderCameraClear::None => false,
        };
        Self {
            scene_clear_plan: ViewportSceneClearPlan::new(scene_color, scene_depth),
        }
    }

    const fn from_overlay_camera(camera: &CameraRenderDescriptor) -> Self {
        Self {
            scene_clear_plan: ViewportSceneClearPlan::new(None, camera.clear_depth),
        }
    }
}

impl Default for ViewportCameraStackAttachmentPolicy {
    fn default() -> Self {
        Self::from_base_camera(&CameraRenderDescriptor::from_camera_payload(
            None,
            Default::default(),
        ))
    }
}

#[cfg(test)]
#[path = "tests/viewport_camera_stack_attachment_policy.rs"]
mod tests;
