use std::sync::Arc;

use crate::core::manager::{
    ManagerServiceHandle, RegisteredManagerService, manager_service_handle, resolve_manager_service,
};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreHandle, InitLevel, ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode,
};
use crate::engine_module::{EngineModule, factory, qualified_name};

use super::font::FontCollectionService;

pub const TEXT_MODULE_NAME: &str = "TextModule";
pub(crate) const FONT_SERVICES_MANAGER_NAME: &str = "TextModule.Manager.FontServices";

const TEXT_MODULE_DESCRIPTION: &str = "Runtime text shaping, layout, and font services";

#[derive(Debug)]
pub(crate) struct TextRuntimeServices {
    font_collection: Arc<FontCollectionService>,
}

impl TextRuntimeServices {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            font_collection: FontCollectionService::new(),
        })
    }

    pub(crate) fn font_collection(&self) -> Arc<FontCollectionService> {
        Arc::clone(&self.font_collection)
    }
}

pub(crate) fn text_runtime_services_handle(
    core: &CoreHandle,
) -> Result<ManagerServiceHandle<TextRuntimeServices>, CoreError> {
    manager_service_handle(core, FONT_SERVICES_MANAGER_NAME)
}

pub(crate) fn resolve_text_runtime_services(
    core: &CoreHandle,
) -> Result<Arc<TextRuntimeServices>, CoreError> {
    resolve_manager_service(core, text_runtime_services_handle(core)?)
}

pub(crate) fn font_collection_service_for_core(
    core: &CoreHandle,
) -> Result<Arc<FontCollectionService>, CoreError> {
    resolve_text_runtime_services(core).map(|services| services.font_collection())
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(TEXT_MODULE_NAME, TEXT_MODULE_DESCRIPTION)
        .with_init_level(InitLevel::Services)
        .with_manager(ManagerDescriptor::new(
            qualified_name(TEXT_MODULE_NAME, ServiceKind::Manager, "FontServices"),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| {
                Ok(
                    Arc::new(RegisteredManagerService::new(TextRuntimeServices::new()))
                        as ServiceObject,
                )
            }),
        ))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextModule;

impl EngineModule for TextModule {
    fn module_name(&self) -> &'static str {
        TEXT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        TEXT_MODULE_DESCRIPTION
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::CoreRuntime;

    use super::{TEXT_MODULE_NAME, font_collection_service_for_core, module_descriptor};

    fn runtime_with_text_services() -> CoreRuntime {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(module_descriptor())
            .expect("text module should register");
        runtime
            .activate_module(TEXT_MODULE_NAME)
            .expect("text module should activate");
        runtime
    }

    #[test]
    fn text_font_services_are_stable_within_one_runtime_and_isolated_across_runtimes() {
        let first_runtime = runtime_with_text_services();
        let second_runtime = runtime_with_text_services();
        let first = font_collection_service_for_core(&first_runtime.handle())
            .expect("first runtime font collection");
        let first_again = font_collection_service_for_core(&first_runtime.handle())
            .expect("first runtime font collection should remain resolvable");
        let second = font_collection_service_for_core(&second_runtime.handle())
            .expect("second runtime font collection");

        assert!(Arc::ptr_eq(&first, &first_again));
        assert!(!Arc::ptr_eq(&first, &second));
        assert_ne!(first.collection_id(), second.collection_id());
    }
}
