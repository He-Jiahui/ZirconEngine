use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::HUB_PROTOCOL_VERSION_V1;

/// Exact protocol marker that prevents a v1 mailbox reader from accepting another revision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HubProtocolVersionV1;

impl Serialize for HubProtocolVersionV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(HUB_PROTOCOL_VERSION_V1)
    }
}

impl<'de> Deserialize<'de> for HubProtocolVersionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = u32::deserialize(deserializer)?;
        if version != HUB_PROTOCOL_VERSION_V1 {
            return Err(serde::de::Error::custom(format!(
                "unsupported Hub protocol version {version}; expected {HUB_PROTOCOL_VERSION_V1}"
            )));
        }
        Ok(Self)
    }
}
