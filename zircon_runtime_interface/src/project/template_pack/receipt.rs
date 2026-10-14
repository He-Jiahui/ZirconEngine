use serde::{Deserialize, Deserializer, Serialize};

use crate::project::{
    ProjectActivationOperationId, ProjectEngineCompatibilityError, ProjectEngineVersion,
    ProjectGuid,
};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::ProjectTemplateDescriptor;

/// The only accepted wire revision for a materialized template provenance receipt.
///
/// 当前序列化协议的固定版本号；解码时只接受这一版本并重新检查模板与创建引擎兼容性。
pub const PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1: u32 = 1;

/// Authenticated creation identities that the project authority binds to a rendered template.
///
/// This input is deliberately separate from the static template descriptor because operation and
/// BuildSet identities exist only after a product surface has crossed startup admission.
///
/// 封装调用方提供的操作 ID、创建引擎版本与 BuildSet ID，作为模板来源记录的字段。
/// `new` 只保存这些值，不验证它们来自启动准入；调用方须传入已核实的创建上下文。
/// 此值本身不执行准入，也不授予激活能力。
///
/// # 生命周期
/// 由 Editor 创建事务在落盘前用于签发 [`ProjectTemplateReceipt`]；生成的 receipt 可写入项目清单。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectCreationProvenance {
    /// 调用方提供的创建操作身份；此字段不证明该操作已通过启动准入。
    operation_id: ProjectActivationOperationId,
    /// 发起创建的引擎版本，用于检查模板声明的 semver 范围。
    creator_engine_version: ProjectEngineVersion,
    /// 发起创建的运行时构建集身份。
    build_set_id: ZrRuntimeBuildSetId,
}

impl ProjectCreationProvenance {
    /// 组合一份创建来源输入。
    ///
    /// # 参数
    /// - `operation_id`：此次准入关联的操作身份。
    /// - `creator_engine_version`：执行创建的引擎版本。
    /// - `build_set_id`：此次准入使用的运行时 BuildSet 身份。
    ///
    /// # 返回值
    /// 返回不可变来源值；具体模板兼容性在 [`Self::issue_receipt`] 时检查。
    pub fn new(
        operation_id: ProjectActivationOperationId,
        creator_engine_version: ProjectEngineVersion,
        build_set_id: ZrRuntimeBuildSetId,
    ) -> Self {
        Self {
            operation_id,
            creator_engine_version,
            build_set_id,
        }
    }

    /// 返回此次创建关联的操作身份。
    ///
    /// # 返回值
    /// 返回按值复制的操作 ID。
    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    /// 借用发起创建时的引擎版本。
    ///
    /// # 返回值
    /// 返回在此来源对象存活期间有效的版本引用。
    pub fn creator_engine_version(&self) -> &ProjectEngineVersion {
        &self.creator_engine_version
    }

    /// 借用发起创建时的运行时 BuildSet 身份。
    ///
    /// # 返回值
    /// 返回在此来源对象存活期间有效的 BuildSet 引用。
    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }

    /// 将启动准入来源绑定到模板描述符和新项目 GUID，并检查创建引擎兼容范围。
    /// 此处只绑定调用方给定的来源字段；来源是否已经核实由上层负责。
    ///
    /// # 参数
    /// - `descriptor`：要实例化的规范模板描述符。
    /// - `project_guid`：已渲染清单中的新项目身份。
    ///
    /// # 返回值
    /// 返回包含模板、项目、操作、引擎版本及 BuildSet 身份的 receipt；模板要求不兼容或无效时返回错误。
    ///
    /// # 使用时机与生命周期
    /// Editor 创建事务在写入 staging 目录前调用；结果是来源记录，不构成后续激活授权。
    pub fn issue_receipt(
        &self,
        descriptor: ProjectTemplateDescriptor,
        project_guid: ProjectGuid,
    ) -> Result<ProjectTemplateReceipt, ProjectTemplateReceiptError> {
        ProjectTemplateReceipt::try_new(
            descriptor,
            project_guid,
            self.operation_id,
            self.creator_engine_version.clone(),
            self.build_set_id.clone(),
        )
    }
}

/// Immutable provenance recording which template, project, operation, and BuildSet produced a
/// materialized project. The receipt contains identity only; it grants no activation capability.
///
/// 记录调用方提供的模板、项目、操作、引擎版本和 BuildSet 身份，随清单持久化。
/// 字段通过 [`ProjectTemplateReceipt::try_new`] 或来源值组合；解码时重验 schema、模板描述符与引擎版本范围。
///
/// # 生命周期
/// receipt 不验证所记录身份的来源真实性，不持有模板内容，也不代替项目预检或运行时准入。
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectTemplateReceipt {
    /// 当前 receipt wire schema 版本。
    schema_version: u32,
    /// 由内置注册表规范化的模板描述符。
    descriptor: ProjectTemplateDescriptor,
    /// 新项目的稳定身份。
    project_guid: ProjectGuid,
    /// 调用方记录的创建操作身份；receipt 不验证操作的准入来源。
    operation_id: ProjectActivationOperationId,
    /// 产生此项目的引擎版本。
    creator_engine_version: ProjectEngineVersion,
    /// 创建时使用的运行时构建集身份。
    build_set_id: ZrRuntimeBuildSetId,
}

impl ProjectTemplateReceipt {
    /// 构造可持久化来源记录；不兼容的创建引擎不能签发该模板的 receipt。
    ///
    /// # 参数
    /// - `descriptor`：规范模板身份及其引擎版本要求。
    /// - `project_guid`：已渲染项目清单中的 GUID。
    /// - `operation_id`：创建事务所绑定的启动操作身份。
    /// - `creator_engine_version`：实际发起创建的引擎版本。
    /// - `build_set_id`：实际发起创建的运行时构建集身份。
    ///
    /// # 返回值
    /// 返回固定 V1 schema 的 receipt；不兼容或无法解析模板版本要求时返回错误。
    ///
    /// # 约束
    /// 此检查仅验证创建引擎版本范围，不检查其它产品能力，也不签发激活权限。
    pub fn try_new(
        descriptor: ProjectTemplateDescriptor,
        project_guid: ProjectGuid,
        operation_id: ProjectActivationOperationId,
        creator_engine_version: ProjectEngineVersion,
        build_set_id: ZrRuntimeBuildSetId,
    ) -> Result<Self, ProjectTemplateReceiptError> {
        let compatibility = descriptor.assess_engine_compatibility(&creator_engine_version)?;
        if !compatibility.is_compatible() {
            return Err(ProjectTemplateReceiptError::IncompatibleCreatorEngine {
                template: descriptor.qualified_id(),
                requirement: descriptor.engine_version_req().map(str::to_string),
                creator_engine_version,
            });
        }
        Ok(Self {
            schema_version: PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1,
            descriptor,
            project_guid,
            operation_id,
            creator_engine_version,
            build_set_id,
        })
    }

    /// 返回此记录的 wire schema 版本。
    ///
    /// # 返回值
    /// 返回当前固定为 V1 的版本号。
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// 借用此记录绑定的规范模板描述符。
    ///
    /// # 返回值
    /// 返回在 receipt 存活期间有效的描述符引用。
    pub fn descriptor(&self) -> &ProjectTemplateDescriptor {
        &self.descriptor
    }

    /// 返回此 receipt 所属项目的 GUID。
    ///
    /// # 返回值
    /// 返回按值复制的项目身份。
    pub const fn project_guid(&self) -> ProjectGuid {
        self.project_guid
    }

    /// 返回此记录保存的创建操作身份。
    ///
    /// # 返回值
    /// 返回按值复制的操作 ID。
    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    /// 借用签发时使用的引擎版本。
    ///
    /// # 返回值
    /// 返回在 receipt 存活期间有效的版本引用。
    pub fn creator_engine_version(&self) -> &ProjectEngineVersion {
        &self.creator_engine_version
    }

    /// 借用签发时使用的运行时 BuildSet 身份。
    ///
    /// # 返回值
    /// 返回在 receipt 存活期间有效的 BuildSet 引用。
    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }
}

// 私有 wire DTO 拒绝未知字段；公开类型通过手工 Deserialize 再执行 schema 和语义校验。
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
struct ProjectTemplateReceiptWire {
    schema_version: u32,
    descriptor: ProjectTemplateDescriptor,
    project_guid: ProjectGuid,
    operation_id: ProjectActivationOperationId,
    creator_engine_version: ProjectEngineVersion,
    build_set_id: ZrRuntimeBuildSetId,
}

impl<'de> Deserialize<'de> for ProjectTemplateReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 载入时重验 schema、规范描述符及引擎兼容性；receipt 本身仍不授予准入能力。
        let wire = ProjectTemplateReceiptWire::deserialize(deserializer)?;
        if wire.schema_version != PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1 {
            return Err(serde::de::Error::custom(
                ProjectTemplateReceiptError::UnsupportedSchemaVersion {
                    expected: PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1,
                    actual: wire.schema_version,
                },
            ));
        }
        Self::try_new(
            wire.descriptor,
            wire.project_guid,
            wire.operation_id,
            wire.creator_engine_version,
            wire.build_set_id,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectTemplateReceiptError {
    /// 解码时遇到当前实现不支持的 schema 版本。
    #[error("unsupported project template receipt schema version {actual}; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    /// 创建引擎版本不在模板声明的兼容范围内。
    #[error(
        "project template {template} requires engine {requirement:?}, but creator engine is {creator_engine_version}"
    )]
    IncompatibleCreatorEngine {
        template: String,
        requirement: Option<String>,
        creator_engine_version: ProjectEngineVersion,
    },
    /// 模板声明的版本要求无法解析或评估。
    #[error(transparent)]
    InvalidEngineRequirement(#[from] ProjectEngineCompatibilityError),
}

#[cfg(test)]
#[path = "tests/receipt.rs"]
mod tests;
