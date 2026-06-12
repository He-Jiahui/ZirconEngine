//! Event DTOs shared by framework contracts and runtime delivery.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineEvent {
    pub topic: String,
    pub payload: Value,
}
