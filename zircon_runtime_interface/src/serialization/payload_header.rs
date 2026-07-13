use serde::{Deserialize, Serialize};

use super::SchemaId;

/// Header shared by every versioned text or binary envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadHeader {
    pub schema_id: SchemaId,
    pub schema_version: u32,
}
