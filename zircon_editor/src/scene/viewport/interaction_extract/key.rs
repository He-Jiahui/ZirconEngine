//! 缓存身份绑定世界代际、主项、设置、相机与尺寸；更换世界所有者或改变未入键的叠加依赖须显式失效。

use crate::scene::viewport::{SceneViewportSettings, ViewportCameraSnapshot};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

#[derive(Clone, Debug, PartialEq)]
// TODO: [CR-EDITOR-SP-0008] 核对同主项下多选变化的失效责任；中心枢轴依赖整个选中集，本键仅记录主项，调用端须补齐选择代际或显式失效。
pub(super) struct ViewportInteractionExtractKey {
    world_generation: u64,
    selected: Option<u64>,
    settings: SceneViewportSettings,
    camera: ViewportCameraSnapshot,
    viewport: UVec2,
}

impl ViewportInteractionExtractKey {
    pub(super) fn new(
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Self {
        Self {
            world_generation: scene.world_generation(),
            selected,
            settings: settings.clone(),
            camera: camera.clone(),
            viewport,
        }
    }
}
