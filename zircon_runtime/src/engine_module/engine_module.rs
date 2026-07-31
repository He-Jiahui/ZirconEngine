use std::fmt;

use crate::core::ModuleDescriptor;

pub trait EngineModule: Send + Sync + fmt::Debug {
    fn module_name(&self) -> &str;
    fn module_description(&self) -> &str;
    fn descriptor(&self) -> ModuleDescriptor;
}
