use serde::{Deserialize, Serialize};

use crate::ui::template::{UiCompiledDocument, UiTemplateInstance};
use zircon_runtime_interface::ui::template::{
    UiAssetError, UiCompiledAssetPackageValidationReport,
    UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
};

const UI_COMPILED_ASSET_TOML_ENVELOPE_MAGIC: [u8; 8] = *b"ZRUIA018";
const ENVELOPE_HEADER_LEN: usize = UI_COMPILED_ASSET_TOML_ENVELOPE_MAGIC.len() + 4 + 8;
pub const UI_COMPILED_ASSET_ARTIFACT_GENERATED_POLICY: &str =
    "runtime_09_m3_1_toml_envelope_leaf_dto_not_generated_source";
pub const UI_COMPILED_ASSET_ARTIFACT_GENERATED_SOURCE_MARKER_REQUIRED: bool = false;

/// 包产物保留编译验证报告和可实例化模板；宿主需另行检查动作许可并安装 surface，反序列化本身不执行绑定。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRuntimeCompiledAssetArtifact {
    pub report: UiCompiledAssetPackageValidationReport,
    pub compiled: UiTemplateInstance,
}

impl UiRuntimeCompiledAssetArtifact {
    pub const fn generated_policy() -> &'static str {
        UI_COMPILED_ASSET_ARTIFACT_GENERATED_POLICY
    }

    pub const fn requires_generated_source_marker() -> bool {
        UI_COMPILED_ASSET_ARTIFACT_GENERATED_SOURCE_MARKER_REQUIRED
    }

    pub(super) fn from_report_and_compiled(
        report: UiCompiledAssetPackageValidationReport,
        compiled: UiCompiledDocument,
    ) -> Self {
        Self {
            report,
            compiled: compiled.into_template_instance(),
        }
    }

    /// 输出带固定 magic、版本和长度的 TOML envelope，供磁盘缓存与包清单指纹共同使用；不是生成 Rust 源码。
    pub fn to_bytes(&self) -> Result<Vec<u8>, UiAssetError> {
        let payload = toml::to_string(self).map_err(package_error)?.into_bytes();
        let mut bytes = Vec::with_capacity(ENVELOPE_HEADER_LEN + payload.len());
        bytes.extend_from_slice(&UI_COMPILED_ASSET_TOML_ENVELOPE_MAGIC);
        bytes.extend_from_slice(&UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    /// 拒绝截断、未知格式和内部不合法的绑定程序；只恢复数据，不加载报告列出的资源依赖。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, UiAssetError> {
        if bytes.len() < ENVELOPE_HEADER_LEN {
            return Err(invalid_artifact("compiled artifact envelope is truncated"));
        }
        if bytes[..UI_COMPILED_ASSET_TOML_ENVELOPE_MAGIC.len()]
            != UI_COMPILED_ASSET_TOML_ENVELOPE_MAGIC
        {
            return Err(invalid_artifact("compiled artifact magic does not match"));
        }

        let version_start = UI_COMPILED_ASSET_TOML_ENVELOPE_MAGIC.len();
        let version_end = version_start + 4;
        let schema_version = u32::from_le_bytes(
            bytes[version_start..version_end]
                .try_into()
                .expect("slice length checked"),
        );
        if schema_version != UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION {
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
        let artifact: Self = toml::from_str(payload).map_err(package_error)?;
        // TODO: [CR-UI-TEMPLATE-COMP-0003] 确认是否还需校验模板内容与绑定程序一致；目前只比较节点数量和程序内部结构；缺少节点数量不变但改绑的载荷回归，下一步核对树构建与程序安装约束。
        let binding_program = artifact.compiled.binding_program();
        if binding_program.generation().is_invalid()
            || binding_program.node_count() != template_node_count(&artifact.compiled.root)
            || !binding_program.is_well_formed()
        {
            return Err(invalid_artifact(
                "compiled artifact binding program is malformed",
            ));
        }
        Ok(artifact)
    }
}

// 与绑定编译和树构建采用同一孩子遍历域；解码至少要求模板节点和程序节点数量一致。
fn template_node_count(root: &zircon_runtime_interface::ui::template::UiTemplateNode) -> usize {
    let mut pending = vec![root];
    let mut count = 0usize;
    while let Some(node) = pending.pop() {
        count = count.saturating_add(1);
        pending.extend(node.children.iter().rev());
    }
    count
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
