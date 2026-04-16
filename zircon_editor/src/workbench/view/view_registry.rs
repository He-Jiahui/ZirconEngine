use std::collections::HashMap;

use super::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, Default)]
pub struct ViewRegistry {
    pub(super) descriptors: HashMap<ViewDescriptorId, ViewDescriptor>,
    pub(super) instances: HashMap<ViewInstanceId, ViewInstance>,
    pub(super) single_instance_index: HashMap<ViewDescriptorId, ViewInstanceId>,
    pub(super) counters: HashMap<ViewDescriptorId, usize>,
}
