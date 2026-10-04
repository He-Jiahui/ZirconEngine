//! Negotiated transport selection for versioned IME composition payloads.
//!
//! The fixed `ZrRuntimeEventV1` layout remains the transport envelope.  A composition V2 event
//! uses a new IME state only after both endpoints explicitly advertise this capability.  This
//! keeps an older Runtime API from receiving a state it cannot decode.

use super::ime_composition_v2::ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA;

pub const ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY: &str = "runtime.ime.composition.v2";
pub const ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE: u32 = 10;
pub const ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeImeCompositionWireVersion {
    LegacyV1,
    CompositionV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrRuntimeImeCompositionNegotiation {
    peer_api_version: u32,
    peer_schema: Option<u16>,
    peer_capability: bool,
}

impl ZrRuntimeImeCompositionNegotiation {
    /// Builds a negotiation from the bounded configuration ABI after its exact validation.
    /// Keeping this constructor separate from string parsing prevents a V8 table match from
    /// silently implying support for the state-10 wire format.
    pub const fn from_raw(
        peer_api_version: u32,
        peer_schema: Option<u16>,
        peer_capability: bool,
    ) -> Self {
        Self {
            peer_api_version,
            peer_schema,
            peer_capability,
        }
    }

    /// A missing capability is deliberately a V1-only session.  The producer must not infer
    /// support from a matching DLL or from the V8 table shape alone.
    pub fn from_peer<I, S>(peer_api_version: u32, peer_capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut advertised_schema = None;
        let mut peer_capability = false;
        for capability in peer_capabilities {
            let capability = capability.as_ref();
            if capability == ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY {
                peer_capability = true;
            } else if let Some(schema) =
                capability.strip_prefix("runtime.ime.composition.v2.schema=")
            {
                if let Ok(schema) = schema.parse::<u16>() {
                    advertised_schema = Some(schema);
                }
            }
        }
        // A bare capability advertises this implementation's schema. If a peer sends an
        // explicit schema, preserve it even when it appears before the capability so a wrong
        // schema cannot be overwritten by iteration order.
        let peer_schema = advertised_schema
            .or_else(|| peer_capability.then_some(ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA));
        Self {
            peer_api_version,
            peer_schema,
            peer_capability,
        }
    }

    pub const fn legacy() -> Self {
        Self {
            peer_api_version: 0,
            peer_schema: None,
            peer_capability: false,
        }
    }

    pub const fn supports_v2(&self) -> bool {
        self.peer_api_version >= ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION
            && self.peer_capability
            && matches!(self.peer_schema, Some(ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA))
    }

    pub const fn wire_version(&self) -> ZrRuntimeImeCompositionWireVersion {
        if self.supports_v2() {
            ZrRuntimeImeCompositionWireVersion::CompositionV2
        } else {
            ZrRuntimeImeCompositionWireVersion::LegacyV1
        }
    }

    pub const fn peer_api_version(&self) -> u32 {
        self.peer_api_version
    }

    pub const fn peer_schema(&self) -> Option<u16> {
        self.peer_schema
    }
}

#[cfg(test)]
#[path = "tests/ime_composition_capability_v2.rs"]
mod tests;
