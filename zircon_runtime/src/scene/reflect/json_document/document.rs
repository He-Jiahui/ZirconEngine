use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::ReflectedValue;

/// Current payload stored inside the reflected-JSON version envelope.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReflectedJsonDocument {
    pub(super) value: ReflectedValue,
}
