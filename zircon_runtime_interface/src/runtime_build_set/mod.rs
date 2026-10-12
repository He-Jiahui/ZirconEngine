//! 动态库加载前的 BuildSet 身份与会话建立后的模块组合回执共用的中立契约。
//! 冻结的 InterfaceSpec 同时驱动构建时 ABI 槽表生成与装载前一致性校验。

#[path = "artifact_identity.rs"]
mod artifact_identity;
#[path = "artifact_manifest.rs"]
mod artifact_manifest;
#[path = "build_mode.rs"]
mod build_mode;
#[path = "build_set_expectation.rs"]
mod build_set_expectation;
#[path = "build_set_id.rs"]
mod build_set_id;
#[path = "composition_receipt.rs"]
mod composition_receipt;
#[path = "composition_target.rs"]
mod composition_target;
#[path = "digest.rs"]
mod digest;
#[path = "identity_encoding_error.rs"]
mod identity_encoding_error;
#[path = "identity_format_error.rs"]
mod identity_format_error;
#[path = "interface_spec.rs"]
mod interface_spec;
#[path = "module_profile.rs"]
mod module_profile;
#[path = "payload_schema.rs"]
mod payload_schema;
#[path = "session_profile.rs"]
mod session_profile;
#[path = "slot_catalog.rs"]
mod slot_catalog;
#[path = "target_model.rs"]
mod target_model;
#[path = "trusted_host.rs"]
mod trusted_host;
#[path = "validation_error.rs"]
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
pub use trusted_host::{
    require_trusted_host_build_set_id, trusted_host_build_set_id,
    ZrRuntimeTrustedHostBuildSetError, ZrRuntimeTrustedHostBuildSetMetadataV1,
    ZR_RUNTIME_TRUSTED_HOST_BUILD_SET_METADATA_SCHEMA_V1,
};
pub use validation_error::ZrRuntimeArtifactManifestValidationError;

#[cfg(test)]
#[path = "tests/slot_catalog_build_tests.rs"]
mod slot_catalog_build_tests;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
