use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ZrRuntimeBuildSetId;

include!(concat!(
    env!("OUT_DIR"),
    "/runtime_build_set_trusted_host.rs"
));

pub const ZR_RUNTIME_TRUSTED_HOST_BUILD_SET_METADATA_SCHEMA_V1: u32 = 1;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeTrustedHostBuildSetError {
    #[error("trusted Host BuildSet identity is absent from the compiled product")]
    Missing,
    #[error("trusted Host BuildSet identity is invalid: {message}")]
    Invalid { message: String },
}

/// Source BuildSet metadata packaged beside the runtime artifact. The source ID is
/// distinct from the content-derived runtime artifact `build_set_id`; the optional
/// product ID preserves the `build_set_id_for` product-profile domain.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZrRuntimeTrustedHostBuildSetMetadataV1 {
    pub schema_version: u32,
    pub source_build_set_id: ZrRuntimeBuildSetId,
    #[serde(default)]
    pub product_profile: Option<String>,
    #[serde(default)]
    pub product_build_set_id: Option<ZrRuntimeBuildSetId>,
}

/// 读取构建时嵌入的源码 BuildSet 身份；构建环境未提供时返回 None，由宿主选择准入策略。
pub fn trusted_host_build_set_id() -> Option<&'static str> {
    ZR_TRUSTED_HOST_BUILD_SET_ID_V1
}

/// 要求身份存在且为 64 个小写十六进制字符，成功后借用原输入。
/// 身份的可信来源由调用方保证；此处只验证文本格式。
pub fn require_trusted_host_build_set_id(
    value: Option<&str>,
) -> Result<&str, ZrRuntimeTrustedHostBuildSetError> {
    let value = value.ok_or(ZrRuntimeTrustedHostBuildSetError::Missing)?;
    ZrRuntimeBuildSetId::parse(value.to_owned()).map_err(|error| {
        ZrRuntimeTrustedHostBuildSetError::Invalid {
            message: error.to_string(),
        }
    })?;
    Ok(value)
}

#[cfg(test)]
#[path = "tests/trusted_host.rs"]
mod tests;
