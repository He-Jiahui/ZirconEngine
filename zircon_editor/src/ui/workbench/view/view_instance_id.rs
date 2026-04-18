use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ViewInstanceId(pub(crate) String);

impl ViewInstanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
