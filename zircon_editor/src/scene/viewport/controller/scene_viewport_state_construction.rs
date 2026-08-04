use crate::scene::modes::{
    builtin_scene_mode_registry, SceneModeActivation, SceneModeCtx, SceneModeStack,
};
use crate::scene::selection::SelectionModel;
use crate::scene::viewport::SceneViewportSettings;
use crate::scene::viewport::ViewportState;
use zircon_runtime::core::framework::camera_controller::OrbitCameraController;
use zircon_runtime_interface::math::{UVec2, Vec3};

use super::{scene_viewport_state::SceneViewportState, viewport_hover_state::ViewportHoverState};

impl SceneViewportState {
    pub(in crate::scene::viewport::controller) fn new(viewport_size: UVec2) -> Self {
        let settings = SceneViewportSettings::default();
        let mut selection = SelectionModel::default();
        let scene_mode_registry = builtin_scene_mode_registry();
        let scene_modes = {
            let mut mode_ctx = SceneModeCtx::new(&mut selection, &settings);
            SceneModeStack::new(
                scene_mode_registry
                    .create(&SceneModeActivation::Select.mode_id())
                    .expect("the default scene mode must resolve through the registry"),
                &mut mode_ctx,
            )
            .expect("the default scene mode must enter")
        };

        Self {
            settings,
            selection,
            scene_mode_registry,
            scene_modes,
            viewport: ViewportState::new(viewport_size),
            camera: None,
            orbit_target: Vec3::ZERO,
            orbit_controller: OrbitCameraController::with_target(Vec3::ZERO),
            hover: ViewportHoverState::default(),
            drag: None,
        }
    }
}
