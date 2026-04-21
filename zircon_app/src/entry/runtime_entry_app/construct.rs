use std::error::Error;

use zircon_runtime::core::framework::render::RenderFrameExtract;
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::core::math::{UVec2, Vec2};
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::create_default_level;

use super::{camera_controller::RuntimeCameraController, RuntimeEntryApp};
use crate::runtime_presenter::RenderFrameworkRuntimeBridge;

impl RuntimeEntryApp {
    pub(in crate::entry) fn new(core: CoreHandle) -> Result<Self, Box<dyn Error>> {
        let resolver = ManagerResolver::new(core.clone());
        let input_manager = resolver.input()?;
        let render_bridge = RenderFrameworkRuntimeBridge::new(&core)?;
        let level = create_default_level(&core)?;
        let (selected_node, orbit_target) = level.with_world(|world| {
            let cube = world
                .nodes()
                .iter()
                .find(|node| matches!(&node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .unwrap_or(world.active_camera());
            let orbit_target = world
                .find_node(cube)
                .map(|node| node.transform.translation)
                .unwrap_or_default();
            (Some(cube), orbit_target)
        });

        let mut camera_controller = RuntimeCameraController::new(UVec2::new(1280, 720));
        camera_controller.set_orbit_target(orbit_target);

        Ok(Self {
            window: None,
            presenter: None,
            render_bridge,
            level,
            selected_node,
            camera_controller,
            cursor: Vec2::ZERO,
            input_manager,
            _core: core,
        })
    }

    pub(super) fn current_extract(&self) -> RenderFrameExtract {
        self.level.with_world(|world| {
            world
                .to_render_frame_extract()
                .with_viewport_size(self.camera_controller.viewport_size())
        })
    }

    pub(super) fn resize_viewport(&mut self, size: UVec2) {
        self.camera_controller.resize(size);
    }

    pub(super) fn handle_cursor_moved(&mut self, position: Vec2) {
        self.cursor = position;
        self.level
            .with_world_mut(|world| self.camera_controller.pointer_moved(world, position));
        self.sync_orbit_target_from_selection();
    }

    pub(super) fn handle_left_pressed(&mut self) {
        self.camera_controller.left_pressed(self.cursor);
    }

    pub(super) fn handle_left_released(&mut self) {
        self.camera_controller.left_released();
    }

    pub(super) fn handle_right_pressed(&mut self) {
        self.camera_controller.right_pressed(self.cursor);
    }

    pub(super) fn handle_right_released(&mut self) {
        self.camera_controller.right_released();
    }

    pub(super) fn handle_middle_pressed(&mut self) {
        self.camera_controller.middle_pressed(self.cursor);
    }

    pub(super) fn handle_middle_released(&mut self) {
        self.camera_controller.middle_released();
    }

    pub(super) fn handle_scroll(&mut self, delta: f32) {
        self.level
            .with_world_mut(|world| self.camera_controller.scrolled(world, delta));
        self.sync_orbit_target_from_selection();
    }

    fn sync_orbit_target_from_selection(&mut self) {
        let selected = self.selected_node;
        let orbit_target = self.level.with_world(|world| {
            selected
                .and_then(|selected| world.find_node(selected))
                .map(|node| node.transform.translation)
        });
        if let Some(target) = orbit_target {
            self.camera_controller.set_orbit_target(target);
        }
    }
}
