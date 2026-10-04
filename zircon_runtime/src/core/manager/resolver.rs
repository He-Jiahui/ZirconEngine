use crate::core::runtime::ServiceCallGuard;

#[cfg(feature = "ai-contracts")]
use crate::core::framework::ai::AiManager;
#[cfg(feature = "net-contracts")]
use crate::core::framework::net::NetManager;
#[cfg(feature = "physics-contracts")]
use crate::core::framework::physics::PhysicsManager;
#[cfg(feature = "sound-contracts")]
use crate::core::framework::sound::SoundManager;
use crate::core::framework::{
    animation::AnimationManager,
    asset::ResourceManager,
    foundation::ConfigManager,
    input::{InputActionManager, InputManager},
    navigation::NavigationManager,
    platform::PreferenceStorage,
    render::{RenderFramework, RenderingManager},
    scene::LevelManager,
};
use crate::core::{CoreError, CoreHandle, CoreWeak};

#[cfg(feature = "ai-contracts")]
use super::AI_MANAGER_NAME;
#[cfg(feature = "net-contracts")]
use super::NET_MANAGER_NAME;
#[cfg(feature = "physics-contracts")]
use super::PHYSICS_MANAGER_NAME;
#[cfg(feature = "sound-contracts")]
use super::SOUND_MANAGER_NAME;
use super::{
    manager_service_handle, ManagerServiceHandle, ManagerServiceResolver, ANIMATION_MANAGER_NAME,
    CONFIG_MANAGER_NAME, INPUT_ACTION_MANAGER_NAME, INPUT_MANAGER_NAME, LEVEL_MANAGER_NAME,
    NAVIGATION_MANAGER_NAME, PLATFORM_MANAGER_NAME, RENDERING_MANAGER_NAME, RENDER_FRAMEWORK_NAME,
    RESOURCE_MANAGER_NAME,
};

// 每次展开将框架 trait 与注册名配对，同时生成显式 CoreHandle 函数和弱引用 resolver 方法。
macro_rules! define_manager_handle_access {
    ($trait_name:ident, $handle_fn:ident, $service_name:ident, $method:ident) => {
        /// 按注册名从给定 Core 取得管理器身份句柄；具体 trait 类型在后续解析时由服务包装校验。
        pub fn $handle_fn(
            core: &CoreHandle,
        ) -> Result<ManagerServiceHandle<dyn $trait_name>, CoreError> {
            manager_service_handle(core, $service_name)
        }

        impl ManagerResolver {
            /// 使用 resolver 绑定的 Core 查询同一服务的身份句柄；Core 已释放或服务不可用时返回错误。
            pub fn $method(&self) -> Result<ManagerServiceHandle<dyn $trait_name>, CoreError> {
                let core = self.upgrade_core()?;
                $handle_fn(&core)
            }
        }
    };
}

/// 只弱引用创建时的 CoreRuntime；每次访问先升级它，再由该运行时查询句柄或解析服务。
#[derive(Clone, Debug)]
pub struct ManagerResolver {
    core: CoreWeak,
}

impl ManagerResolver {
    /// 从现有 CoreHandle 创建不会延长运行时寿命的 resolver。
    pub fn new(core: CoreHandle) -> Self {
        Self {
            core: core.downgrade(),
        }
    }

    fn upgrade_core(&self) -> Result<CoreHandle, CoreError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable("CoreRuntime".to_owned()))
    }

    /// 消费句柄并由绑定的 Core 重新核验来源、槽位代数与服务包装类型后取回共享管理器。
    pub fn resolve<T: ?Sized + Send + Sync + 'static>(
        &self,
        handle: ManagerServiceHandle<T>,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        let core = self.upgrade_core()?;
        ManagerServiceResolver::resolve(&core, handle)
    }
}

impl ManagerServiceResolver for ManagerResolver {
    fn resolve<T: ?Sized + Send + Sync + 'static>(
        &self,
        handle: ManagerServiceHandle<T>,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        let core = self.upgrade_core()?;
        ManagerServiceResolver::resolve(&core, handle)
    }
}

define_manager_handle_access!(
    RenderingManager,
    rendering_manager_handle,
    RENDERING_MANAGER_NAME,
    rendering_handle
);
define_manager_handle_access!(
    RenderFramework,
    render_framework_handle,
    RENDER_FRAMEWORK_NAME,
    render_framework_handle
);
define_manager_handle_access!(
    LevelManager,
    level_manager_handle,
    LEVEL_MANAGER_NAME,
    level_handle
);
define_manager_handle_access!(
    ResourceManager,
    resource_manager_handle,
    RESOURCE_MANAGER_NAME,
    resource_handle
);
define_manager_handle_access!(
    InputManager,
    input_manager_handle,
    INPUT_MANAGER_NAME,
    input_handle
);
define_manager_handle_access!(
    InputActionManager,
    input_action_manager_handle,
    INPUT_ACTION_MANAGER_NAME,
    input_actions_handle
);
define_manager_handle_access!(
    ConfigManager,
    config_manager_handle,
    CONFIG_MANAGER_NAME,
    config_handle
);
#[cfg(feature = "ai-contracts")]
define_manager_handle_access!(AiManager, ai_manager_handle, AI_MANAGER_NAME, ai_handle);
#[cfg(feature = "net-contracts")]
define_manager_handle_access!(NetManager, net_manager_handle, NET_MANAGER_NAME, net_handle);
#[cfg(feature = "physics-contracts")]
define_manager_handle_access!(
    PhysicsManager,
    physics_manager_handle,
    PHYSICS_MANAGER_NAME,
    physics_handle
);
define_manager_handle_access!(
    AnimationManager,
    animation_manager_handle,
    ANIMATION_MANAGER_NAME,
    animation_handle
);
#[cfg(feature = "sound-contracts")]
define_manager_handle_access!(
    SoundManager,
    sound_manager_handle,
    SOUND_MANAGER_NAME,
    sound_handle
);
define_manager_handle_access!(
    NavigationManager,
    navigation_manager_handle,
    NAVIGATION_MANAGER_NAME,
    navigation_handle
);
define_manager_handle_access!(
    PreferenceStorage,
    platform_preference_storage_handle,
    PLATFORM_MANAGER_NAME,
    platform_preferences_handle
);
