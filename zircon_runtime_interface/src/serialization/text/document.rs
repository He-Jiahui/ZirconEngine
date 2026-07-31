use serde::Serialize;

use super::envelope::TextEnvelope;

/// Top-level text document which keeps the current payload borrowed through encoding.
#[derive(Serialize)]
pub(in crate::serialization) struct TextDocument<'a, T: ?Sized> {
    #[serde(rename = "$zircon")]
    pub(in crate::serialization) envelope: TextEnvelope<'a, T>,
}
