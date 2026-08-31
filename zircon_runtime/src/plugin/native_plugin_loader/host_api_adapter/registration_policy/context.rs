use std::sync::Arc;

use crate::plugin::PluginModuleId;

use super::super::context_handles::NativeHostRegistrationScopeState;
use super::super::ecs_registration::NativeHostApiV3RegistrationContext;
use super::policy::NativeHostApiV4RegistrationPolicy;

#[derive(Clone)]
pub(in super::super) struct NativeHostApiV4RegistrationContext {
    pub(in super::super) registry: usize,
    pub(in super::super) owner: PluginModuleId,
    pub(in super::super) plugin_id: String,
    pub(in super::super) policy: NativeHostApiV4RegistrationPolicy,
    pub(in super::super) lifetime: Arc<NativeHostRegistrationScopeState>,
}

impl NativeHostApiV4RegistrationContext {
    pub(in super::super) fn v3_context(&self) -> NativeHostApiV3RegistrationContext {
        NativeHostApiV3RegistrationContext {
            registry: self.registry,
            owner: self.owner,
            lifetime: Arc::clone(&self.lifetime),
        }
    }
}
