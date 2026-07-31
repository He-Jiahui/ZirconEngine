use serde::Serialize;

use crate::serialization::PayloadHeader;

/// Current-version text envelope which borrows the payload while it is written.
#[derive(Serialize)]
pub(in crate::serialization) struct TextEnvelope<'a, T: ?Sized> {
    pub(in crate::serialization) header: PayloadHeader,
    pub(in crate::serialization) payload: &'a T,
}
