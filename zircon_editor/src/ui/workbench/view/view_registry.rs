use std::collections::{HashMap, HashSet};

use super::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, Default)]
pub struct ViewRegistry {
    pub(super) descriptors: HashMap<ViewDescriptorId, ViewDescriptor>,
    pub(super) instances: HashMap<ViewInstanceId, ViewInstance>,
    pub(super) single_instance_index: HashMap<ViewDescriptorId, ViewInstanceId>,
    pub(super) counters: HashMap<ViewDescriptorId, usize>,
    pub(super) available_capabilities: HashSet<String>,
}

impl ViewRegistry {
    pub fn set_available_capabilities<I, S>(&mut self, capabilities: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.available_capabilities = capabilities.into_iter().map(Into::into).collect();
    }

    pub fn descriptor_capability_error(&self, descriptor: &ViewDescriptor) -> Option<String> {
        let missing = descriptor
            .required_capabilities
            .iter()
            .filter(|capability| !self.available_capabilities.contains(*capability))
            .cloned()
            .collect::<Vec<_>>();
        (!missing.is_empty()).then(|| {
            format!(
                "view descriptor {} requires disabled capabilities: {}",
                descriptor.descriptor_id.0,
                missing.join(", ")
            )
        })
    }
}
