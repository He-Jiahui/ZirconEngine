use std::sync::Arc;

use crate::plugin::PluginModuleId;

use super::super::context_handles::NativeHostRegistrationScopeState;
use super::super::ecs_registration::NativeHostApiV3RegistrationContext;
use super::policy::NativeHostApiV4RegistrationPolicy;

#[derive(Clone)]
/// 注册回调共享宿主策略和关闭状态；借用的注册表地址仅应在所属 scope 的存活期内使用。
pub(in super::super) struct NativeHostApiV4RegistrationContext {
    pub(in super::super) registry: usize,
    pub(in super::super) owner: PluginModuleId,
    pub(in super::super) plugin_id: String,
    pub(in super::super) policy: NativeHostApiV4RegistrationPolicy,
    pub(in super::super) lifetime: Arc<NativeHostRegistrationScopeState>,
}

impl NativeHostApiV4RegistrationContext {
    /// 把 V4 上下文投影为旧注册入口需要的 owner 与生命周期，不重新借用注册表。
    pub(in super::super) fn v3_context(&self) -> NativeHostApiV3RegistrationContext {
        NativeHostApiV3RegistrationContext {
            registry: self.registry,
            owner: self.owner,
            lifetime: Arc::clone(&self.lifetime),
        }
    }
}
