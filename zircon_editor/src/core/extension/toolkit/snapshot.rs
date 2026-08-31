use std::collections::BTreeMap;
use std::sync::Arc;

use super::{DocumentToolkitDescriptor, ToolkitInstanceId};

#[derive(Clone, Debug)]
pub struct DocumentToolkitSnapshot {
    generation: u64,
    descriptors: Arc<[DocumentToolkitDescriptor]>,
    descriptors_by_instance: Arc<BTreeMap<ToolkitInstanceId, usize>>,
}

impl DocumentToolkitSnapshot {
    pub(super) fn new(generation: u64, descriptors: Vec<DocumentToolkitDescriptor>) -> Self {
        let descriptors_by_instance = descriptors
            .iter()
            .enumerate()
            .map(|(index, descriptor)| (descriptor.instance_id().clone(), index))
            .collect();
        Self {
            generation,
            descriptors: descriptors.into(),
            descriptors_by_instance: Arc::new(descriptors_by_instance),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn descriptors(&self) -> &[DocumentToolkitDescriptor] {
        &self.descriptors
    }

    pub fn descriptor_for_instance(
        &self,
        instance: &ToolkitInstanceId,
    ) -> Option<&DocumentToolkitDescriptor> {
        self.descriptors_by_instance
            .get(instance)
            .and_then(|index| self.descriptors.get(*index))
    }
}
