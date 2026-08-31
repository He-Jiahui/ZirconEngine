use serde::{Deserialize, Serialize};

use super::{ZrRuntimeDigestV1, ZrRuntimeIdentityEncodingError};

const CURRENT_INTERFACE_SPEC_SOURCE: &str = include_str!("interface_spec_v1.json");

/// Machine-readable definition of the frozen internal V8 runtime ABI family.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeInterfaceSpecV1 {
    pub family: String,
    pub spec_version: u32,
    pub runtime_api_version: u32,
    pub entry_symbol: String,
    pub runtime_api_required_slots: Vec<String>,
    pub runtime_api_optional_slots: Vec<String>,
    pub host_api_optional_slots: Vec<String>,
}

impl ZrRuntimeInterfaceSpecV1 {
    pub fn current() -> Result<Self, ZrRuntimeIdentityEncodingError> {
        serde_json::from_str(CURRENT_INTERFACE_SPEC_SOURCE).map_err(|error| {
            ZrRuntimeIdentityEncodingError::InterfaceSpecDecode {
                message: error.to_string(),
            }
        })
    }

    pub fn digest(&self) -> Result<ZrRuntimeDigestV1, ZrRuntimeIdentityEncodingError> {
        serde_json::to_vec(self)
            .map(ZrRuntimeDigestV1::sha256)
            .map_err(
                |error| ZrRuntimeIdentityEncodingError::InterfaceSpecEncode {
                    message: error.to_string(),
                },
            )
    }
}
