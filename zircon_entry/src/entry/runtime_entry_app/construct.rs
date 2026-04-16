use std::error::Error;

use zircon_core::CoreHandle;
use zircon_graphics::{ViewportController, ViewportInput, ViewportState};
use zircon_manager::ManagerResolver;
use zircon_math::{UVec2, Vec2};
use zircon_scene::{create_default_level, NodeKind};

use super::RuntimeEntryApp;
use crate::runtime_presenter::RenderServerRuntimeBridge;

impl RuntimeEntryApp {
    pub(in crate::entry) fn new(core: CoreHandle) -> Result<Self, Box<dyn Error>> {
        let resolver = ManagerResolver::new(core.clone());
        let input_manager = resolver.input()?;
        let render_bridge = RenderServerRuntimeBridge::new(&core)?;
        let level = create_default_level(&core)?;
        let orbit_target = level.with_world_mut(|world| {
            let cube = world
                .nodes()
                .iter()
                .find(|node| matches!(&node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .unwrap_or(world.active_camera());
            world.set_selected(Some(cube));
            world
                .find_node(cube)
                .map(|node| node.transform.translation)
                .unwrap_or_default()
        });

        let mut viewport_controller =
            ViewportController::new(ViewportState::new(UVec2::new(1280, 720)));
        viewport_controller.set_orbit_target(orbit_target);

        Ok(Self {
            window: None,
            presenter: None,
            render_bridge,
            level,
            viewport_controller,
            cursor: Vec2::ZERO,
            input_manager,
            _core: core,
        })
    }

    pub(super) fn current_extract(&self) -> zircon_scene::RenderFrameExtract {
        self.level.with_world(|world| {
            world
                .to_render_frame_extract()
                .with_viewport_size(self.viewport_controller.viewport().size)
        })
    }

    pub(super) fn handle_viewport_input(&mut self, input: ViewportInput) {
        let _feedback = self
            .level
            .with_world_mut(|world| self.viewport_controller.handle_input(world, input));
        let orbit_target = self.level.with_world(|world| {
            world
                .selected_node()
                .and_then(|selected| world.find_node(selected))
                .map(|node| node.transform.translation)
        });
        if let Some(target) = orbit_target {
            self.viewport_controller.set_orbit_target(target);
        }
    }
}
