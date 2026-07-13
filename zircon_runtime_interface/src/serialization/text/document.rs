use serde::{Deserialize, Serialize};

use super::envelope::TextEnvelope;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::serialization) struct TextDocument {
    #[serde(rename = "$zircon")]
    pub(in crate::serialization) envelope: TextEnvelope,
}
