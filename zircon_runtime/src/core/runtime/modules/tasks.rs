use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;

pub const TASKS_MODULE_NAME: &str = "TasksModule";

#[derive(Clone, Copy, Debug, Default)]
pub struct TasksModule;

impl EngineModule for TasksModule {
    fn module_name(&self) -> &'static str {
        TASKS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Core task scheduling descriptor for the built-in job scheduler"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(TASKS_MODULE_NAME, self.module_description())
    }
}
