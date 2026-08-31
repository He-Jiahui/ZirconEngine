use serde::{Deserialize, Serialize};

use super::{
    ZrRuntimeDigestV1, ZrRuntimeModuleCompositionTargetV1, ZrRuntimeModuleProfileV1,
    ZrRuntimeSessionProfileV1,
};

pub const ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1: u32 = 1;

/// Cross-ABI receipt for the one frozen module graph owned by a runtime session.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeModuleCompositionReceiptV1 {
    pub schema_version: u32,
    pub catalog_generation: u64,
    pub source_manifest_fingerprint: u64,
    pub target_mode: ZrRuntimeModuleCompositionTargetV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_profile: Option<ZrRuntimeModuleProfileV1>,
    pub session_profile: ZrRuntimeSessionProfileV1,
    pub composition_hash: ZrRuntimeDigestV1,
}

impl ZrRuntimeModuleCompositionReceiptV1 {
    pub fn new(
        catalog_generation: u64,
        source_manifest_fingerprint: u64,
        target_mode: ZrRuntimeModuleCompositionTargetV1,
        module_profile: Option<ZrRuntimeModuleProfileV1>,
        session_profile: ZrRuntimeSessionProfileV1,
        composition_hash: ZrRuntimeDigestV1,
    ) -> Self {
        Self {
            schema_version: ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
            catalog_generation,
            source_manifest_fingerprint,
            target_mode,
            module_profile,
            session_profile,
            composition_hash,
        }
    }
}
