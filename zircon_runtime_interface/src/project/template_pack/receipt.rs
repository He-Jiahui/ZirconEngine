use serde::{Deserialize, Deserializer, Serialize};

use crate::project::{
    ProjectActivationOperationId, ProjectEngineCompatibilityError, ProjectEngineVersion,
    ProjectGuid,
};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::ProjectTemplateDescriptor;

/// The only accepted wire revision for a materialized template provenance receipt.
pub const PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1: u32 = 1;

/// Authenticated creation identities that the project authority binds to a rendered template.
///
/// This input is deliberately separate from the static template descriptor because operation and
/// BuildSet identities exist only after a product surface has crossed startup admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectCreationProvenance {
    operation_id: ProjectActivationOperationId,
    creator_engine_version: ProjectEngineVersion,
    build_set_id: ZrRuntimeBuildSetId,
}

impl ProjectCreationProvenance {
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

    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    pub fn creator_engine_version(&self) -> &ProjectEngineVersion {
        &self.creator_engine_version
    }

    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }

    /// 将启动准入来源绑定到模板描述符和新项目 GUID，并检查创建引擎兼容范围。
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectTemplateReceipt {
    schema_version: u32,
    descriptor: ProjectTemplateDescriptor,
    project_guid: ProjectGuid,
    operation_id: ProjectActivationOperationId,
    creator_engine_version: ProjectEngineVersion,
    build_set_id: ZrRuntimeBuildSetId,
}

impl ProjectTemplateReceipt {
    /// 构造可持久化来源记录；不兼容的创建引擎不能签发该模板的 receipt。
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

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn descriptor(&self) -> &ProjectTemplateDescriptor {
        &self.descriptor
    }

    pub const fn project_guid(&self) -> ProjectGuid {
        self.project_guid
    }

    pub const fn operation_id(&self) -> ProjectActivationOperationId {
        self.operation_id
    }

    pub fn creator_engine_version(&self) -> &ProjectEngineVersion {
        &self.creator_engine_version
    }

    pub fn build_set_id(&self) -> &ZrRuntimeBuildSetId {
        &self.build_set_id
    }
}

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
    #[error("unsupported project template receipt schema version {actual}; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    #[error(
        "project template {template} requires engine {requirement:?}, but creator engine is {creator_engine_version}"
    )]
    IncompatibleCreatorEngine {
        template: String,
        requirement: Option<String>,
        creator_engine_version: ProjectEngineVersion,
    },
    #[error(transparent)]
    InvalidEngineRequirement(#[from] ProjectEngineCompatibilityError),
}

#[cfg(test)]
#[path = "tests/receipt.rs"]
mod tests;
