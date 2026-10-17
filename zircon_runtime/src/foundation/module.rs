#[cfg(feature = "native-host")]
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::framework::foundation::{ConfigManager, FOUNDATION_MODULE_NAME};
use crate::core::manager::{RegisteredManagerService, CONFIG_MANAGER_NAME};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, InitLevel, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};

use super::DefaultConfigManager;

/// 描述 Kernel 阶段立即启动的 Foundation 配置服务，供运行时组装和模块注册使用。
///
/// # 返回与使用时机
///
/// 返回包含规范 ConfigManager 工厂的模块描述符，此调用本身不创建 manager 或执行配置 I/O。
/// 注册和激活时，工厂升级所属 Core 的弱句柄，再创建默认持久化服务。
///
/// # 参考
///
/// 宿主文件绑定在注册前使用 [bind_config_file_path]；类型适配入口见 [FoundationModule]。
pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        FOUNDATION_MODULE_NAME,
        "Built-in runtime foundation services",
    )
    .with_init_level(InitLevel::Kernel)
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            FOUNDATION_MODULE_NAME,
            ServiceKind::Manager,
            "ConfigManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|core| {
            let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
            let manager = Arc::new(DefaultConfigManager::new(&core)?);
            Ok(
                Arc::new(RegisteredManagerService::<dyn ConfigManager>::new(manager))
                    as ServiceObject,
            )
        }),
    ))
}

/// Binds one host-owned persistence file before the normal Foundation descriptor is registered.
/// Only the canonical ConfigManager factory changes; module lifecycle and dependencies remain intact.
///
/// # 参数
///
/// - `descriptor`：尚未注册且名称为 Foundation 的描述符，必须含且仅含一个规范 ConfigManager。
/// - `path`：非空绝对配置文件路径，移动到替换后的工厂中，供之后创建服务时使用。
///
/// # 返回与生命周期
///
/// 检查失败返回错误且不替换工厂；成功仅完成描述符绑定，不执行文件 I/O。
/// 路径由工厂捕获，之后的服务使用独立 clone，不随环境变量变化。
/// 调用应先于模块注册；已有服务不受此描述符修改影响。
///
/// # 参考
///
/// 默认描述符由 [module_descriptor] 创建，实例构造见 [DefaultConfigManager::new_with_file_path]。
#[cfg(feature = "native-host")]
pub fn bind_config_file_path(
    descriptor: &mut ModuleDescriptor,
    path: PathBuf,
) -> Result<(), CoreError> {
    DefaultConfigManager::validate_file_path(&path)?;
    if descriptor.name != FOUNDATION_MODULE_NAME
        || descriptor
            .managers
            .iter()
            .filter(|manager| manager.name.as_str() == CONFIG_MANAGER_NAME)
            .count()
            != 1
    {
        return Err(CoreError::Initialization(
            "foundation config persistence".to_owned(),
            "descriptor must own exactly one canonical Foundation ConfigManager".to_owned(),
        ));
    }
    let manager = descriptor
        .managers
        .iter_mut()
        .find(|manager| manager.name.as_str() == CONFIG_MANAGER_NAME)
        .expect("canonical ConfigManager was checked above");
    manager.factory = factory(move |core| {
        let core = core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
        let manager = Arc::new(DefaultConfigManager::new_with_file_path(
            &core,
            path.clone(),
        )?);
        Ok(Arc::new(RegisteredManagerService::<dyn ConfigManager>::new(manager)) as ServiceObject)
    });
    Ok(())
}

/// 把内建 Foundation 描述符接入 [EngineModule] 组装入口的无状态适配器。
///
/// # 生命周期与参考
///
/// 复制此值不会创建或复制配置服务；[EngineModule::descriptor] 委托 [module_descriptor] 创建描述符。
/// 实际服务实例由描述符工厂在所属 Core 激活阶段建立。
#[derive(Clone, Copy, Debug, Default)]
pub struct FoundationModule;

impl EngineModule for FoundationModule {
    fn module_name(&self) -> &'static str {
        FOUNDATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Built-in runtime foundation services"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
