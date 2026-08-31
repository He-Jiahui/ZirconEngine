use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerIdentity {
    pub tool: String,
    pub tool_version: String,
    pub worker_id: String,
    pub operation_id: String,
}
