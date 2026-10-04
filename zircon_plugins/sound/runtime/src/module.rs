//! 模块注册先建立驱动，再延迟创建依赖项目资产服务的声音经理；中立 SoundManager 句柄由注册表包装具体实现。
use std::sync::Arc;

use zircon_runtime::asset::ASSET_MODULE_NAME;
use zircon_runtime::core::framework::sound::SoundManager;
use zircon_runtime::core::manager::RegisteredManagerService;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name, EngineModule};

use super::{DefaultSoundManager, SoundDriver};

pub const SOUND_MODULE_NAME: &str = "sound.runtime";
pub const SOUND_DRIVER_NAME: &str = "sound.runtime.Driver.SoundDriver";
pub(crate) const DEFAULT_SOUND_MANAGER_NAME: &str = "sound.runtime.Manager.DefaultSoundManager";
pub const SOUND_MANAGER_NAME: &str = zircon_runtime::core::manager::SOUND_MANAGER_NAME;

/// Sound 基础服务的模块注册入口；具体经理延迟创建，并通过框架的中立服务句柄供调用方解析。
#[derive(Clone, Copy, Debug, Default)]
pub struct SoundModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(SOUND_MODULE_NAME, "Audio mixing, buses, and playback")
        .with_driver(DriverDescriptor::new(
            qualified_name(SOUND_MODULE_NAME, ServiceKind::Driver, "SoundDriver"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(SoundDriver) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(
                SOUND_MODULE_NAME,
                ServiceKind::Manager,
                "DefaultSoundManager",
            ),
            StartupMode::Lazy,
            vec![
                dependency_on(SOUND_MODULE_NAME, ServiceKind::Driver, "SoundDriver"),
                dependency_on(
                    ASSET_MODULE_NAME,
                    ServiceKind::Manager,
                    "ProjectAssetManager",
                ),
            ],
            factory(
                // TODO: [CR-SOUND-AUDIT-0005] 核实项目 Sound 选项何时注入延迟经理；当前工厂走 from_weak_core 的默认配置，非默认项目选项转换仅见测试调用。
                |core| Ok(Arc::new(DefaultSoundManager::from_weak_core(core)) as ServiceObject),
            ),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(SOUND_MODULE_NAME, ServiceKind::Manager, "SoundManager"),
            StartupMode::Lazy,
            vec![dependency_on(
                SOUND_MODULE_NAME,
                ServiceKind::Manager,
                "DefaultSoundManager",
            )],
            factory(|core| {
                let manager =
                    core.resolve_manager::<DefaultSoundManager>(DEFAULT_SOUND_MANAGER_NAME)?;
                Ok(
                    Arc::new(RegisteredManagerService::<dyn SoundManager>::new(manager))
                        as ServiceObject,
                )
            }),
        ))
}

impl EngineModule for SoundModule {
    fn module_name(&self) -> &'static str {
        SOUND_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Audio mixing, buses, and playback"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
