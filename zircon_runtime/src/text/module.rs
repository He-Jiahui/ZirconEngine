use std::sync::Arc;
use std::time::Instant;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::manager::{
    manager_service_handle, resolve_manager_service, ManagerServiceHandle, RegisteredManagerService,
};
use crate::core::runtime::{ServiceCallGuard, ServiceObject};
use crate::core::{
    CoreError, CoreHandle, CoreResult, InitLevel, ManagerDescriptor, ModuleCleanupContext,
    ModuleDescriptor, ModuleLifecycle, ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name, EngineModule};

use super::context::{TextRuntimeContext, TextRuntimeContextLifecycleState, TextSystemFontPolicy};

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
) -> Result<ServiceCallGuard<TextRuntimeContext>, CoreError> {
    resolve_manager_service(core, text_runtime_context_handle(core)?)
}

#[derive(Debug, Default)]
struct TextModuleLifecycle;

impl ModuleLifecycle for TextModuleLifecycle {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        close_runtime_context(context, None)
    }

    fn cleanup_until(
        &self,
        context: &ModuleCleanupContext<'_>,
        deadline: Instant,
    ) -> CoreResult<()> {
        close_runtime_context(context, Some(deadline))
    }
}

fn close_runtime_context(
    context: &ModuleCleanupContext<'_>,
    deadline: Option<Instant>,
) -> CoreResult<()> {
    let budget = deadline
        .map(|deadline| deadline.saturating_duration_since(Instant::now()))
        .unwrap_or(std::time::Duration::ZERO);
    context.with_existing_registered_manager::<TextRuntimeContext>(
        TEXT_RUNTIME_CONTEXT_MANAGER_NAME,
        |runtime_context| {
            runtime_context.begin_draining();
            loop {
                let state = runtime_context.close();
                if state == TextRuntimeContextLifecycleState::Closed {
                    return Ok(());
                }
                let remaining = deadline
                    .map(|deadline| deadline.saturating_duration_since(Instant::now()))
                    .unwrap_or(std::time::Duration::ZERO);
                if state == TextRuntimeContextLifecycleState::Faulted || remaining.is_zero() {
                    let active = runtime_context
                        .health_snapshot()
                        .active_layout_session_family_count();
                    return Err(CoreError::ModuleCleanupTimeout {
                        module: context.module_name().to_owned(),
                        operation: "text_runtime_context".to_owned(),
                        budget,
                        incomplete_entries: usize::try_from(active).unwrap_or(usize::MAX).max(1),
                        failed: u64::from(state == TextRuntimeContextLifecycleState::Faulted),
                        cancelled: 0,
                    });
                }
                std::thread::sleep(remaining.min(std::time::Duration::from_millis(1)));
            }
        },
    )?;
    Ok(())
}

pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_system_font_policy(TextSystemFontPolicy::PackagedOnly)
}

fn module_descriptor_with_system_font_policy(
    system_font_policy: TextSystemFontPolicy,
) -> ModuleDescriptor {
    let lifecycle = Arc::new(TextModuleLifecycle);
    ModuleDescriptor::new(TEXT_MODULE_NAME, TEXT_MODULE_DESCRIPTION)
        .with_init_level(InitLevel::Services)
        .with_lifecycle(lifecycle)
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
