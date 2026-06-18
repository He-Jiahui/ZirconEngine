use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, PostProcessGraphResourceNames, RenderCameraClear,
};
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ViewportCameraStackAttachmentPolicy {
    scene_color_load: RenderGraphAttachmentLoadOp,
    scene_depth_load: RenderGraphAttachmentLoadOp,
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
        Self::ops_with_load(self.scene_color_load)
    }

    #[cfg(test)]
    fn scene_depth_ops(self) -> RenderGraphAttachmentOps {
        Self::ops_with_load(self.scene_depth_load)
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
                load: self.scene_color_load,
                store: graph_ops.store,
            },
            PostProcessGraphResourceNames::SCENE_DEPTH => RenderGraphAttachmentOps {
                load: self.scene_depth_load,
                store: graph_ops.store,
            },
            _ => graph_ops,
        }
    }

    const fn from_base_camera(camera: &CameraRenderDescriptor) -> Self {
        let scene_color_load = match camera.clear {
            RenderCameraClear::Skybox | RenderCameraClear::Color(_) => {
                RenderGraphAttachmentLoadOp::Clear
            }
            RenderCameraClear::DepthOnly => RenderGraphAttachmentLoadOp::Load,
            RenderCameraClear::None if camera.camera.msaa_samples > 1 => {
                RenderGraphAttachmentLoadOp::Clear
            }
            RenderCameraClear::None => RenderGraphAttachmentLoadOp::Load,
        };
        let scene_depth_load = match camera.clear {
            RenderCameraClear::Skybox
            | RenderCameraClear::Color(_)
            | RenderCameraClear::DepthOnly => RenderGraphAttachmentLoadOp::Clear,
            RenderCameraClear::None => RenderGraphAttachmentLoadOp::Load,
        };
        Self {
            scene_color_load,
            scene_depth_load,
        }
    }

    const fn from_overlay_camera(camera: &CameraRenderDescriptor) -> Self {
        let scene_depth_load = if camera.clear_depth {
            RenderGraphAttachmentLoadOp::Clear
        } else {
            RenderGraphAttachmentLoadOp::Load
        };
        Self {
            scene_color_load: RenderGraphAttachmentLoadOp::Load,
            scene_depth_load,
        }
    }

    #[cfg(test)]
    const fn ops_with_load(load: RenderGraphAttachmentLoadOp) -> RenderGraphAttachmentOps {
        RenderGraphAttachmentOps {
            load,
            store: crate::render_graph::RenderGraphAttachmentStoreOp::Store,
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
mod tests {
    use super::*;
    use crate::core::framework::render::ViewportCameraSnapshot;
    use crate::core::math::Vec4;
    use crate::render_graph::RenderGraphAttachmentStoreOp;

    fn camera(render_type: CameraRenderType, clear: RenderCameraClear) -> CameraRenderDescriptor {
        CameraRenderDescriptor {
            render_type,
            clear,
            ..CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default())
        }
    }

    #[test]
    fn base_camera_clear_modes_translate_to_scene_load_ops() {
        let color = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
            CameraRenderType::Base,
            RenderCameraClear::Color(Vec4::ONE),
        ));
        assert_eq!(
            color.scene_color_ops(),
            RenderGraphAttachmentOps::clear_store()
        );
        assert_eq!(
            color.scene_depth_ops(),
            RenderGraphAttachmentOps::clear_store()
        );

        let depth_only = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
            CameraRenderType::Base,
            RenderCameraClear::DepthOnly,
        ));
        assert_eq!(
            depth_only.scene_color_ops(),
            RenderGraphAttachmentOps::load_store()
        );
        assert_eq!(
            depth_only.scene_depth_ops(),
            RenderGraphAttachmentOps::clear_store()
        );

        let no_clear = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
            CameraRenderType::Base,
            RenderCameraClear::None,
        ));
        assert_eq!(
            no_clear.scene_color_ops(),
            RenderGraphAttachmentOps::load_store()
        );
        assert_eq!(
            no_clear.scene_depth_ops(),
            RenderGraphAttachmentOps::load_store()
        );
    }

    #[test]
    fn base_camera_none_clear_with_msaa_clears_scene_color_only() {
        let mut descriptor = camera(CameraRenderType::Base, RenderCameraClear::None);
        descriptor.camera.msaa_samples = 4;

        let policy = ViewportCameraStackAttachmentPolicy::from_camera(&descriptor);

        assert_eq!(
            policy.scene_color_ops(),
            RenderGraphAttachmentOps::clear_store()
        );
        assert_eq!(
            policy.scene_depth_ops(),
            RenderGraphAttachmentOps::load_store()
        );
    }

    #[test]
    fn overlay_camera_never_clears_scene_color_and_uses_clear_depth_for_depth() {
        let mut descriptor = camera(
            CameraRenderType::Overlay,
            RenderCameraClear::Color(Vec4::ONE),
        );
        descriptor.clear_depth = true;

        let clear_depth = ViewportCameraStackAttachmentPolicy::from_camera(&descriptor);
        assert_eq!(
            clear_depth.scene_color_ops(),
            RenderGraphAttachmentOps::load_store()
        );
        assert_eq!(
            clear_depth.scene_depth_ops(),
            RenderGraphAttachmentOps::clear_store()
        );

        descriptor.clear_depth = false;
        let load_depth = ViewportCameraStackAttachmentPolicy::from_camera(&descriptor);
        assert_eq!(
            load_depth.scene_color_ops(),
            RenderGraphAttachmentOps::load_store()
        );
        assert_eq!(
            load_depth.scene_depth_ops(),
            RenderGraphAttachmentOps::load_store()
        );
    }

    #[test]
    fn policy_only_rewrites_scene_first_clear_writes_and_preserves_store() {
        let policy = ViewportCameraStackAttachmentPolicy::from_camera(&camera(
            CameraRenderType::Overlay,
            RenderCameraClear::Skybox,
        ));
        let clear_discard = RenderGraphAttachmentOps::clear_discard();

        assert_eq!(
            policy.apply_to_first_attachment_write(
                PostProcessGraphResourceNames::SCENE_COLOR,
                clear_discard,
            ),
            RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Load,
                store: RenderGraphAttachmentStoreOp::Discard,
            }
        );
        assert_eq!(
            policy.apply_to_first_attachment_write(
                PostProcessGraphResourceNames::SCENE_DEPTH,
                clear_discard,
            ),
            RenderGraphAttachmentOps {
                load: RenderGraphAttachmentLoadOp::Clear,
                store: RenderGraphAttachmentStoreOp::Discard,
            }
        );
        assert_eq!(
            policy.apply_to_first_attachment_write(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
            RenderGraphAttachmentOps::load_store()
        );
        assert_eq!(
            policy.apply_to_first_attachment_write("taa-output", clear_discard),
            clear_discard
        );
    }
}
