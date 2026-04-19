use std::fmt;

use crate::core::ModuleDescriptor;

pub trait EngineModule: Send + Sync + fmt::Debug {
    fn module_name(&self) -> &'static str;
    fn module_description(&self) -> &'static str;
    fn descriptor(&self) -> ModuleDescriptor;
}
