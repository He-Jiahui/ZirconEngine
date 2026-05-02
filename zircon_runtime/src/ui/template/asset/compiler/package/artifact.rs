use serde::{Deserialize, Serialize};

use crate::ui::template::{UiCompiledDocument, UiTemplateInstance};
use zircon_runtime_interface::ui::template::{
    UiAssetError, UiCompiledAssetPackageValidationReport,
    UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION,
};

const UI_COMPILED_ASSET_BINARY_MAGIC: [u8; 8] = *b"ZRUIA016";
const ENVELOPE_HEADER_LEN: usize = UI_COMPILED_ASSET_BINARY_MAGIC.len() + 4 + 8;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRuntimeCompiledAssetArtifact {
    pub report: UiCompiledAssetPackageValidationReport,
    pub compiled: UiTemplateInstance,
}

impl UiRuntimeCompiledAssetArtifact {
    pub(super) fn from_report_and_compiled(
        report: UiCompiledAssetPackageValidationReport,
        compiled: UiCompiledDocument,
    ) -> Self {
        Self {
            report,
            compiled: compiled.into_template_instance(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, UiAssetError> {
        let payload = toml::to_string(self).map_err(package_error)?.into_bytes();
        let mut bytes = Vec::with_capacity(ENVELOPE_HEADER_LEN + payload.len());
        bytes.extend_from_slice(&UI_COMPILED_ASSET_BINARY_MAGIC);
        bytes.extend_from_slice(&UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, UiAssetError> {
        if bytes.len() < ENVELOPE_HEADER_LEN {
            return Err(invalid_artifact("compiled artifact envelope is truncated"));
        }
        if bytes[..UI_COMPILED_ASSET_BINARY_MAGIC.len()] != UI_COMPILED_ASSET_BINARY_MAGIC {
            return Err(invalid_artifact("compiled artifact magic does not match"));
        }

        let version_start = UI_COMPILED_ASSET_BINARY_MAGIC.len();
        let version_end = version_start + 4;
        let schema_version = u32::from_le_bytes(
            bytes[version_start..version_end]
                .try_into()
                .expect("slice length checked"),
        );
        if schema_version != UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION {
            return Err(invalid_artifact(&format!(
                "compiled artifact schema version {schema_version} is unsupported"
            )));
        }

        let len_start = version_end;
        let len_end = len_start + 8;
        let payload_len = u64::from_le_bytes(
            bytes[len_start..len_end]
                .try_into()
                .expect("slice length checked"),
        ) as usize;
        let payload = &bytes[ENVELOPE_HEADER_LEN..];
        if payload.len() != payload_len {
            return Err(invalid_artifact(
                "compiled artifact payload length does not match envelope",
            ));
        }

        let payload = std::str::from_utf8(payload).map_err(package_error)?;
        toml::from_str(payload).map_err(package_error)
    }
}

fn package_error(error: impl std::fmt::Display) -> UiAssetError {
    UiAssetError::InvalidDocument {
        asset_id: "ui-compiled-artifact".to_string(),
        detail: error.to_string(),
    }
}

fn invalid_artifact(detail: &str) -> UiAssetError {
    UiAssetError::InvalidDocument {
        asset_id: "ui-compiled-artifact".to_string(),
        detail: detail.to_string(),
    }
}
