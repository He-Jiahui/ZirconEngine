use std::time::Duration;

use serde_json::Value;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundMixerGraphManager, SoundSourceDescriptor, AUDIO_SOURCE_COMPONENT_TYPE,
};
use zircon_runtime::core::runtime::modules::{TasksModule, TimeModule};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::{
    RuntimeExtensionRegistry, RuntimePluginCatalog, RuntimePluginRegistrationReport,
};
use zircon_runtime::scene::{self, LevelSystem, LevelTickError, NodeKind, World};

use crate::module::DEFAULT_SOUND_MANAGER_NAME;
use crate::{runtime_plugin, DefaultSoundManager, SOUND_MODULE_NAME};

pub(super) fn extensions(target: RuntimeTargetMode) -> RuntimeExtensionRegistry {
    let mut report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());
    assert!(report.is_success(), "{:?}", report.diagnostics);
    report.project_selection.enabled = true;
    let manifest = ProjectPluginManifest {
        selections: vec![report.project_selection.clone()],
    };
    let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    let report = catalog.runtime_extensions_for_project(&manifest, target);
    assert!(!report.has_fatal_diagnostics(), "{:?}", report.diagnostics);
    report.registry.clone()
}

pub(super) struct Fixture {
    pub(super) runtime: CoreRuntime,
    pub(super) level: LevelSystem,
    pub(super) manager: DefaultSoundManager,
    pub(super) clip: SoundClipId,
}

impl Fixture {
    pub(super) fn new(target: RuntimeTargetMode) -> Self {
        let registry = extensions(target);
        let runtime = CoreRuntime::new();
        for descriptor in [
            TasksModule.descriptor(),
            TimeModule.descriptor(),
            zircon_runtime::asset::module_descriptor(),
            scene::module_descriptor(),
        ] {
            runtime.register_module(descriptor).unwrap();
        }
        for descriptor in registry.modules() {
            runtime.register_module(descriptor.clone()).unwrap();
        }
        runtime.activate_registered_modules().unwrap();
        scene::install_world_runtime_extension_plan(
            &runtime.handle(),
            registry.world_runtime_extension_plan().unwrap(),
        )
        .unwrap();
        let manager = (*runtime
            .handle()
            .resolve_manager::<DefaultSoundManager>(DEFAULT_SOUND_MANAGER_NAME)
            .unwrap())
        .clone();
        let clip = manager.insert_clip_for_test(super::test_clip_with_rate(
            "res://sound/scene.wav",
            8,
            &[0.25; 8],
        ));
        let level =
            scene::create_level(&runtime.handle(), World::empty(), Default::default()).unwrap();
        Self {
            runtime,
            level,
            manager,
            clip,
        }
    }

    pub(super) fn client() -> Self {
        Self::new(RuntimeTargetMode::ClientRuntime)
    }

    pub(super) fn component(&self) -> Value {
        serde_json::json!({"input": {"Clip": self.clip}})
    }

    pub(super) fn sources(&self) -> Vec<SoundSourceDescriptor> {
        self.manager.mixer_snapshot().unwrap().graph.sources
    }

    pub(super) fn try_tick(&self) -> Result<(), LevelTickError> {
        let frame = self.runtime.advance_time_by(Duration::from_millis(100), 1);
        self.level.tick(&self.runtime.handle(), frame)
    }

    pub(super) fn tick(&self) {
        self.try_tick().unwrap();
    }

    pub(super) fn end(&self) {
        self.runtime.deactivate_module(SOUND_MODULE_NAME).unwrap();
    }
}

pub(super) fn add(level: &LevelSystem, value: Value) -> u64 {
    level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .set_dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE, value)
            .unwrap();
        entity
    })
}

pub(super) fn set(level: &LevelSystem, entity: u64, value: Value) {
    level.with_world_mut(|world| {
        world
            .set_dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE, value)
            .unwrap();
    });
}
