use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::serialization::PayloadHeader;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::serialization) struct TextEnvelope {
    pub(in crate::serialization) header: PayloadHeader,
    pub(in crate::serialization) payload: Value,
}
