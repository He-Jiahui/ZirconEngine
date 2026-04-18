use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ViewDescriptorId, ViewHost, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewInstance {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub serializable_payload: Value,
    pub dirty: bool,
    pub host: ViewHost,
}
