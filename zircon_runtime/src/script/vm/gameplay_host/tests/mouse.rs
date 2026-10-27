use std::sync::{Arc, Mutex};

use crate::core::framework::input::{InputButton, InputEvent, InputManager, MouseWheelEvent};
use crate::core::framework::scene::WorldHandle;
use crate::core::framework::script::ScriptHostValue;
use crate::core::manager::{RegisteredManagerService, INPUT_MANAGER_NAME};
use crate::core::runtime::ServiceObject;
use crate::core::{CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, StartupMode};
use crate::input::{DefaultInputManager, INPUT_MODULE_NAME};
use crate::scene::components::NodeKind;
use crate::scene::{LevelMetadata, LevelSystem, World};
use crate::script::{
    register_gameplay_host_module, with_script_runtime_call_context, CapabilitySet,
    HostExportRegistry, HostRegistry, ScriptRuntimeCallContext, VmError,
};

use super::super::super::GAMEPLAY_MODULE;

struct MouseHost {
    core: CoreRuntime,
    input: Arc<DefaultInputManager>,
    level: LevelSystem,
    entity: u64,
    exports: HostExportRegistry,
}

impl MouseHost {
    fn new() -> Self {
        let core = CoreRuntime::new();
        let input = Arc::new(DefaultInputManager::default());
        let manager: Arc<dyn InputManager> = input.clone();
        core.register_module(
            ModuleDescriptor::new(INPUT_MODULE_NAME, "gameplay host input test").with_manager(
                ManagerDescriptor::new(
                    RegistryName::new(INPUT_MANAGER_NAME).unwrap(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(RegisteredManagerService::<dyn InputManager>::new(
                            manager.clone(),
                        )) as ServiceObject)
                    }),
                ),
            ),
        )
        .unwrap();
        core.activate_module(INPUT_MODULE_NAME).unwrap();
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        let level = LevelSystem::new(
            WorldHandle::new(64),
            Arc::new(Mutex::new(world)),
            LevelMetadata::default(),
        );
        let exports = HostExportRegistry::new(HostRegistry::default());
        register_gameplay_host_module(&exports).unwrap();
        Self {
            core,
            input,
            level,
            entity,
            exports,
        }
    }

    fn call(
        &self,
        name: &str,
        arguments: Vec<ScriptHostValue>,
    ) -> Result<ScriptHostValue, VmError> {
        with_script_runtime_call_context(
            ScriptRuntimeCallContext {
                core: self.core.weak(),
                level: self.level.clone(),
                entity: self.entity,
                delta_seconds: 0.016,
            },
            || {
                self.exports.call_with_capabilities(
                    GAMEPLAY_MODULE,
                    name,
                    arguments,
                    &CapabilitySet::default().with("gameplay.input"),
                )
            },
        )
    }

    fn float(&self, name: &str) -> f64 {
        match self.call(name, Vec::new()).unwrap() {
            ScriptHostValue::Float(value) => value,
            value => panic!("{name} returned {value:?}"),
        }
    }
}

#[test]
fn gameplay_host_mouse_pressed_uses_runtime_button_ids_and_current_held_state() {
    let host = MouseHost::new();
    for (id, button) in [
        (1, InputButton::MouseLeft),
        (2, InputButton::MouseRight),
        (3, InputButton::MouseMiddle),
    ] {
        let query = || {
            host.call("mouse_pressed", vec![ScriptHostValue::Int(id)])
                .unwrap()
        };
        assert_eq!(query(), ScriptHostValue::Bool(false));
        host.input
            .submit_event(InputEvent::ButtonPressed(button.clone()));
        assert_eq!(query(), ScriptHostValue::Bool(true));
        host.input.begin_frame();
        assert_eq!(query(), ScriptHostValue::Bool(true));
        host.input.submit_event(InputEvent::ButtonReleased(button));
        assert_eq!(query(), ScriptHostValue::Bool(false));
    }
}

#[test]
fn gameplay_host_mouse_pressed_rejects_invalid_button_ids() {
    let host = MouseHost::new();
    for id in [-1, 0, 4, i64::MAX] {
        let error = host
            .call("mouse_pressed", vec![ScriptHostValue::Int(id)])
            .unwrap_err();
        assert!(
            error.to_string().contains("invalid mouse button"),
            "{error}"
        );
    }
    assert!(host
        .call("mouse_pressed", vec![ScriptHostValue::Float(1.0)])
        .is_err());
}

#[test]
fn gameplay_host_mouse_position_and_delta_preserve_frame_snapshot_semantics() {
    let host = MouseHost::new();
    host.input
        .submit_event(InputEvent::CursorMoved { x: 125.5, y: -6.25 });
    host.input.submit_event(InputEvent::MouseMotion {
        delta_x: 1.5,
        delta_y: -2.0,
    });
    host.input.submit_event(InputEvent::MouseMotion {
        delta_x: -0.5,
        delta_y: 5.25,
    });

    for _ in 0..2 {
        assert_eq!(host.float("mouse_x"), 125.5);
        assert_eq!(host.float("mouse_y"), -6.25);
        assert_eq!(host.float("mouse_delta_x"), 1.0);
        assert_eq!(host.float("mouse_delta_y"), 3.25);
    }
    host.input.begin_frame();
    assert_eq!(host.float("mouse_x"), 125.5);
    assert_eq!(host.float("mouse_y"), -6.25);
    assert_eq!(host.float("mouse_delta_x"), 0.0);
    assert_eq!(host.float("mouse_delta_y"), 0.0);
}

#[test]
fn gameplay_host_mouse_wheel_normalizes_mixed_units_without_consuming_events() {
    let host = MouseHost::new();
    host.input
        .submit_event(InputEvent::MouseWheel(MouseWheelEvent::lines(2.0, -3.0)));
    host.input
        .submit_event(InputEvent::MouseWheel(MouseWheelEvent::pixels(10.0, 20.0)));
    host.input
        .submit_event(InputEvent::WheelScrolled { delta: 0.5 });

    for _ in 0..2 {
        assert_eq!(host.float("mouse_wheel_x"), 3.0);
        assert_eq!(host.float("mouse_wheel_y"), -0.5);
    }
    assert_eq!(host.input.frame_snapshot().mouse_wheel_events.len(), 3);
    host.input.begin_frame();
    assert_eq!(host.float("mouse_wheel_x"), 0.0);
    assert_eq!(host.float("mouse_wheel_y"), 0.0);
}
