use std::collections::BTreeMap;

use uuid::Uuid;

use crate::project::{
    ProjectActivationOperationId, ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};
use crate::runtime_build_set::ZrRuntimeBuildSetId;

use super::{
    ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordError,
    ProjectSessionAdmissionRecordV1, ProjectSessionGenerationV1, ProjectSessionPrincipalV1,
};

const PROJECT_SESSION_ADMISSION_RECORD_WIRE_VERSION_V2: u32 = 2;

pub fn encode_project_session_admission_record(record: &ProjectSessionAdmissionRecordV1) -> String {
    format!(
        "version={PROJECT_SESSION_ADMISSION_RECORD_WIRE_VERSION_V2}\\nprocess_id={}\\ninstance_id={}\\nprincipal={}\\nbuild_set_id={}\\noperation_origin_instance={}\\noperation_sequence={}\\noperation_nonce={}\\nlifecycle={}\\nchecked_epoch={}\\nsession_generation={}\\nheartbeat_unix_millis={}\\n",
        record.process_id(),
        record.instance_id(),
        record.principal().as_str(),
        record.build_set_id().as_str(),
        record.operation_id().origin_instance().as_uuid(),
        record.operation_id().sequence().get(),
        record.operation_id().nonce(),
        record.lifecycle().as_str(),
        record.checked_epoch(),
        record
            .session_generation()
            .map(ProjectSessionGenerationV1::get)
            .unwrap_or(0),
        record.heartbeat_unix_millis()
    )
}

pub fn decode_project_session_admission_record(
    source: &str,
) -> Result<ProjectSessionAdmissionRecordV1, ProjectSessionAdmissionRecordError> {
    let mut values = BTreeMap::new();
    for line in source.lines() {
        let (key, value) = line.split_once('=').ok_or_else(|| {
            ProjectSessionAdmissionRecordError::new("every line must be key=value")
        })?;
        if values.insert(key, value).is_some() {
            return Err(ProjectSessionAdmissionRecordError::new(format!(
                "duplicate field `{key}`"
            )));
        }
    }
    let version = values
        .remove("version")
        .ok_or_else(|| invalid_field("version"))?;
    if version != PROJECT_SESSION_ADMISSION_RECORD_WIRE_VERSION_V2.to_string() {
        return Err(ProjectSessionAdmissionRecordError::new(
            "unsupported version",
        ));
    }
    let process_id = parse_number(values.remove("process_id"), "process_id")?;
    let instance_id = values
        .remove("instance_id")
        .ok_or_else(|| invalid_field("instance_id"))?;
    let principal = ProjectSessionPrincipalV1::parse(
        values
            .remove("principal")
            .ok_or_else(|| invalid_field("principal"))?,
    )?;
    let build_set_id = ZrRuntimeBuildSetId::parse(
        values
            .remove("build_set_id")
            .ok_or_else(|| invalid_field("build_set_id"))?,
    )
    .map_err(|error| ProjectSessionAdmissionRecordError::new(error.to_string()))?;
    let origin_instance = ProjectLaunchInstanceId::try_from_uuid(parse_uuid(
        values
            .remove("operation_origin_instance")
            .ok_or_else(|| invalid_field("operation_origin_instance"))?,
        "operation_origin_instance",
    )?)
    .map_err(|error| ProjectSessionAdmissionRecordError::new(error.to_string()))?;
    let sequence = ProjectActivationOperationSequence::new(parse_number(
        values.remove("operation_sequence"),
        "operation_sequence",
    )?)
    .ok_or_else(|| invalid_field("operation_sequence"))?;
    let operation_id = ProjectActivationOperationId::try_from_parts(
        origin_instance,
        sequence,
        parse_uuid(
            values
                .remove("operation_nonce")
                .ok_or_else(|| invalid_field("operation_nonce"))?,
            "operation_nonce",
        )?,
    )
    .map_err(|error| ProjectSessionAdmissionRecordError::new(error.to_string()))?;
    let lifecycle = ProjectSessionAdmissionLifecycleV1::parse(
        values
            .remove("lifecycle")
            .ok_or_else(|| invalid_field("lifecycle"))?,
    )?;
    let checked_epoch = parse_number(values.remove("checked_epoch"), "checked_epoch")?;
    let session_generation =
        match parse_number(values.remove("session_generation"), "session_generation")? {
            0 => None,
            value => ProjectSessionGenerationV1::new(value),
        };
    let heartbeat_unix_millis = parse_number(
        values.remove("heartbeat_unix_millis"),
        "heartbeat_unix_millis",
    )?;
    if !values.is_empty() {
        return Err(ProjectSessionAdmissionRecordError::new("unknown field"));
    }
    ProjectSessionAdmissionRecordV1::from_persisted(
        process_id,
        instance_id,
        principal,
        build_set_id,
        operation_id,
        lifecycle,
        checked_epoch,
        session_generation,
        heartbeat_unix_millis,
    )
}

fn parse_number<T>(
    value: Option<&str>,
    field: &'static str,
) -> Result<T, ProjectSessionAdmissionRecordError>
where
    T: std::str::FromStr,
{
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| invalid_field(field))
}

fn invalid_field(field: &'static str) -> ProjectSessionAdmissionRecordError {
    ProjectSessionAdmissionRecordError::new(format!("invalid or missing {field}"))
}

fn parse_uuid(
    value: &str,
    field: &'static str,
) -> Result<Uuid, ProjectSessionAdmissionRecordError> {
    Uuid::parse_str(value).map_err(|_| invalid_field(field))
}
