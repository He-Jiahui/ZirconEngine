mod artifact_identity;
mod artifact_manifest;
mod build_mode;
mod build_set_expectation;
mod build_set_id;
mod composition_receipt;
mod composition_target;
mod digest;
mod identity_encoding_error;
mod identity_format_error;
mod interface_spec;
mod module_profile;
mod payload_schema;
mod session_profile;
mod slot_catalog;
mod target_model;
mod validation_error;

pub use artifact_identity::ZrRuntimeArtifactIdentityV1;
pub use artifact_manifest::{ZrRuntimeArtifactManifestV1, ZR_RUNTIME_ARTIFACT_MANIFEST_SCHEMA_V1};
pub use build_mode::ZrRuntimeBuildModeV1;
pub use build_set_expectation::ZrRuntimeBuildSetExpectationV1;
pub use build_set_id::ZrRuntimeBuildSetId;
pub use composition_receipt::{
    ZrRuntimeModuleCompositionReceiptV1, ZR_RUNTIME_MODULE_COMPOSITION_RECEIPT_SCHEMA_V1,
};
pub use composition_target::ZrRuntimeModuleCompositionTargetV1;
pub use digest::ZrRuntimeDigestV1;
pub use identity_encoding_error::ZrRuntimeIdentityEncodingError;
pub use identity_format_error::ZrRuntimeIdentityFormatError;
pub use interface_spec::ZrRuntimeInterfaceSpecV1;
pub use module_profile::ZrRuntimeModuleProfileV1;
pub use payload_schema::current_runtime_payload_schema_set_digest;
pub use session_profile::ZrRuntimeSessionProfileV1;
// Frozen generated InterfaceSpec metadata and V8 ABI slot inventories.
pub use slot_catalog::{
    ZIRCON_RUNTIME_API_VERSION_V8, ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES,
    ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES, ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES,
    ZR_RUNTIME_GET_API_SYMBOL_V8, ZR_RUNTIME_INTERFACE_FAMILY_V1,
    ZR_RUNTIME_INTERFACE_SPEC_VERSION_V1,
};
pub use target_model::{ZrRuntimeEndianV1, ZrRuntimeTargetModelV1};
pub use validation_error::ZrRuntimeArtifactManifestValidationError;

#[cfg(test)]
mod slot_catalog_build_tests;
#[cfg(test)]
mod tests;
