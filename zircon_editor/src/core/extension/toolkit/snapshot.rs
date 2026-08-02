use std::sync::Arc;

use super::DocumentToolkitDescriptor;

#[derive(Clone, Debug)]
pub struct DocumentToolkitSnapshot {
    generation: u64,
    descriptors: Arc<[DocumentToolkitDescriptor]>,
}

impl DocumentToolkitSnapshot {
    pub(super) fn new(generation: u64, descriptors: Vec<DocumentToolkitDescriptor>) -> Self {
        Self {
            generation,
            descriptors: descriptors.into(),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn descriptors(&self) -> &[DocumentToolkitDescriptor] {
        &self.descriptors
    }
}
