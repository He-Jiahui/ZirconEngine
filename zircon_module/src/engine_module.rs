use std::fmt;

use zircon_core::ModuleDescriptor;

pub trait EngineModule: Send + Sync + fmt::Debug {
    fn descriptor(&self) -> ModuleDescriptor;
}
