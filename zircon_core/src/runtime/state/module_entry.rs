use crate::lifecycle::LifecycleState;

use super::super::descriptors::ModuleDescriptor;

pub(crate) struct ModuleEntry {
    #[allow(dead_code)]
    pub(crate) descriptor: ModuleDescriptor,
    pub(crate) lifecycle: LifecycleState,
}
