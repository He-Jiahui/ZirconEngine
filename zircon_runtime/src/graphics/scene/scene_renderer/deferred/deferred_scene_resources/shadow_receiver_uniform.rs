use bytemuck::{Pod, Zeroable};

use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::scene::scene_renderer::shadow::{
    DEFERRED_SHADOW_RECEIVER_DEPTH_BIAS, DEFERRED_SHADOW_RECEIVER_MIN_VISIBILITY,
};

const SHADOW_RECEIVER_DISABLED: f32 = 0.0;
const SHADOW_RECEIVER_ENABLED: f32 = 1.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredShadowReceiverUniform {
    light_view_proj: [[f32; 4]; 4],
    params: [f32; 4],
}

impl DeferredShadowReceiverUniform {
    pub(in crate::graphics::scene::scene_renderer::deferred) fn disabled() -> Self {
        Self {
            light_view_proj: crate::core::math::RenderMat4::IDENTITY.to_cols_array_2d(),
            params: [SHADOW_RECEIVER_DISABLED, 0.0, 1.0, 0.0],
        }
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn from_shadow_scene_uniform(
        shadow_scene_uniform: Option<SceneUniform>,
    ) -> Self {
        let Some(shadow_scene_uniform) = shadow_scene_uniform else {
            return Self::disabled();
        };

        Self {
            light_view_proj: shadow_scene_uniform.view_proj,
            params: [
                SHADOW_RECEIVER_ENABLED,
                DEFERRED_SHADOW_RECEIVER_DEPTH_BIAS,
                DEFERRED_SHADOW_RECEIVER_MIN_VISIBILITY,
                0.0,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeferredShadowReceiverUniform;
    use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
    use crate::graphics::scene::scene_renderer::shadow::{
        DEFERRED_SHADOW_RECEIVER_DEPTH_BIAS, DEFERRED_SHADOW_RECEIVER_MIN_VISIBILITY,
    };

    #[test]
    fn disabled_receiver_keeps_shadow_sampling_neutral() {
        let uniform = DeferredShadowReceiverUniform::from_shadow_scene_uniform(None);

        assert_eq!(uniform.params, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn enabled_receiver_forwards_light_view_projection_and_bias_policy() {
        let light_view_proj = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 3.0, 0.0],
            [4.0, 5.0, 6.0, 1.0],
        ];
        let uniform =
            DeferredShadowReceiverUniform::from_shadow_scene_uniform(Some(SceneUniform {
                view_proj: light_view_proj,
                inverse_view_proj: crate::core::math::RenderMat4::IDENTITY.to_cols_array_2d(),
                light_dir: [0.0, -1.0, 0.0, 0.0],
                light_color: [1.0, 1.0, 1.0, 1.0],
                ambient_color: [0.0, 0.0, 0.0, 1.0],
                previous_view_proj: light_view_proj,
                motion_params: [0.0, 0.0, 0.0, 0.0],
            }));

        assert_eq!(uniform.light_view_proj, light_view_proj);
        assert_eq!(
            uniform.params,
            [
                1.0,
                DEFERRED_SHADOW_RECEIVER_DEPTH_BIAS,
                DEFERRED_SHADOW_RECEIVER_MIN_VISIBILITY,
                0.0,
            ]
        );
        assert!(uniform.params[1] > 0.0 && uniform.params[1] < 0.02);
        assert!(uniform.params[2] > 0.0 && uniform.params[2] <= 1.0);
    }
}
