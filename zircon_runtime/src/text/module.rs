use std::sync::Arc;
use std::time::Instant;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::manager::{
    manager_service_handle, resolve_manager_service, ManagerServiceHandle, RegisteredManagerService,
};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreHandle, CoreResult, InitLevel, ManagerDescriptor, ModuleContext,
    ModuleDescriptor, ModuleLifecycle, ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};

use super::context::{TextRuntimeContext, TextSystemFontPolicy};

pub const TEXT_MODULE_NAME: &str = "TextModule";
// Keep the registry name stable until Graphics' declared dependency migrates with its owner.
pub(crate) const TEXT_RUNTIME_CONTEXT_MANAGER_NAME: &str = "TextModule.Manager.FontServices";

const TEXT_MODULE_DESCRIPTION: &str = "Runtime text shaping, layout, and font services";

pub(crate) fn text_runtime_context_handle(
    core: &CoreHandle,
) -> Result<ManagerServiceHandle<TextRuntimeContext>, CoreError> {
    manager_service_handle(core, TEXT_RUNTIME_CONTEXT_MANAGER_NAME)
}

pub fn text_runtime_context_for_core(
    core: &CoreHandle,
) -> Result<Arc<TextRuntimeContext>, CoreError> {
    resolve_manager_service(core, text_runtime_context_handle(core)?)
}

#[derive(Debug, Default)]
struct TextModuleLifecycle;

impl ModuleLifecycle for TextModuleLifecycle {
    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?;
        let runtime_context = text_runtime_context_for_core(&core)?;
        runtime_context.begin_draining();
        runtime_context.close();
        Ok(())
    }

    fn cleanup_until(&self, context: &ModuleContext, deadline: Instant) -> CoreResult<()> {
        if Instant::now() >= deadline {
            return Err(CoreError::ModuleCleanupTimeout {
                module: context.module_name.clone(),
                operation: "text_runtime_context".to_owned(),
                budget: std::time::Duration::ZERO,
                incomplete_entries: 1,
                failed: 0,
                cancelled: 0,
            });
        }
        self.cleanup(context)
    }
}

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_system_font_policy(TextSystemFontPolicy::PackagedOnly)
}

fn module_descriptor_with_system_font_policy(
    system_font_policy: TextSystemFontPolicy,
) -> ModuleDescriptor {
    ModuleDescriptor::new(TEXT_MODULE_NAME, TEXT_MODULE_DESCRIPTION)
        .with_init_level(InitLevel::Services)
        .with_lifecycle(Arc::new(TextModuleLifecycle))
        .with_manager(ManagerDescriptor::new(
            qualified_name(TEXT_MODULE_NAME, ServiceKind::Manager, "FontServices"),
            StartupMode::Immediate,
            Vec::new(),
            factory(move |_| {
                let context = TextRuntimeContext::new_with_system_font_policy(system_font_policy)
                    .map_err(|error| {
                    CoreError::Initialization(TEXT_MODULE_NAME.to_owned(), error.to_string())
                })?;
                Ok(Arc::new(RegisteredManagerService::new(context)) as ServiceObject)
            }),
        ))
}

#[derive(Clone, Copy, Debug)]
pub struct TextModule {
    system_font_policy: TextSystemFontPolicy,
}

impl TextModule {
    pub const fn for_target(target: RuntimeTargetMode) -> Self {
        let system_font_policy = match target {
            RuntimeTargetMode::ClientRuntime | RuntimeTargetMode::EditorHost => {
                TextSystemFontPolicy::DiscoverPlatform
            }
            RuntimeTargetMode::ServerRuntime => TextSystemFontPolicy::PackagedOnly,
        };
        Self { system_font_policy }
    }

    pub const fn with_system_font_policy(system_font_policy: TextSystemFontPolicy) -> Self {
        Self { system_font_policy }
    }
}

impl Default for TextModule {
    fn default() -> Self {
        Self::with_system_font_policy(TextSystemFontPolicy::PackagedOnly)
    }
}

impl EngineModule for TextModule {
    fn module_name(&self) -> &'static str {
        TEXT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        TEXT_MODULE_DESCRIPTION
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor_with_system_font_policy(self.system_font_policy)
    }
}

#[cfg(test)]
#[path = "tests/module.rs"]
mod tests;
