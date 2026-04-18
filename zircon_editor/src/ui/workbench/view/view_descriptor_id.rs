use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ViewDescriptorId(pub(crate) String);

impl ViewDescriptorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
