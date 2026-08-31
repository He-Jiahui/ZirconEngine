use serde::de::DeserializeOwned;

use crate::core::editing::command::{
    BatchTransformJournalPayload, CreateNodeJournalPayload, DeleteNodeJournalPayload,
    EditorCommand, SetReflectedSceneFieldJournalPayload, UpdateNodeJournalPayload,
};
use crate::core::editing::engine::{
    CommandBox, EditCommandCodec, EditCommandCodecRegistry, JournalCodecDecodeError,
    JournalCodecError,
};

const SCHEMA_VERSION: u16 = 1;
const CREATE_NODE_COMMAND_TYPE: &str = "zircon.editor.scene.create_node";
const DELETE_NODE_COMMAND_TYPE: &str = "zircon.editor.scene.delete_node";
const UPDATE_NODE_COMMAND_TYPE: &str = "zircon.editor.scene.update_node";
const BATCH_TRANSFORM_COMMAND_TYPE: &str = "zircon.editor.scene.batch_transform";
const SET_REFLECTED_FIELD_COMMAND_TYPE: &str = "zircon.editor.scene.set_reflected_field";

/// Registers the concrete scene commands that may appear in a durable editor journal.
///
/// Startup recovery owns when this registry is assembled; the transaction engine remains
/// command-domain agnostic and never imports concrete scene command types.
pub(crate) fn register_scene_command_codecs(
    codecs: &mut EditCommandCodecRegistry,
) -> Result<(), JournalCodecError> {
    codecs.register(CreateNodeCodec)?;
    codecs.register(DeleteNodeCodec)?;
    codecs.register(UpdateNodeCodec)?;
    codecs.register(BatchTransformCodec)?;
    codecs.register(SetReflectedSceneFieldCodec)?;
    Ok(())
}

struct CreateNodeCodec;

impl EditCommandCodec for CreateNodeCodec {
    fn command_type(&self) -> &str {
        CREATE_NODE_COMMAND_TYPE
    }

    fn schema_version(&self) -> u16 {
        SCHEMA_VERSION
    }

    fn decode(&self, payload: &serde_json::Value) -> Result<CommandBox, JournalCodecDecodeError> {
        let payload = decode_payload(payload, CREATE_NODE_COMMAND_TYPE)?;
        EditorCommand::from_journal_create(payload)
            .map(|command| Box::new(command) as CommandBox)
            .map_err(command_decode_error)
    }
}

struct DeleteNodeCodec;

impl EditCommandCodec for DeleteNodeCodec {
    fn command_type(&self) -> &str {
        DELETE_NODE_COMMAND_TYPE
    }

    fn schema_version(&self) -> u16 {
        SCHEMA_VERSION
    }

    fn decode(&self, payload: &serde_json::Value) -> Result<CommandBox, JournalCodecDecodeError> {
        let payload = decode_payload(payload, DELETE_NODE_COMMAND_TYPE)?;
        Ok(Box::new(EditorCommand::from_journal_delete(payload)))
    }
}

struct UpdateNodeCodec;

impl EditCommandCodec for UpdateNodeCodec {
    fn command_type(&self) -> &str {
        UPDATE_NODE_COMMAND_TYPE
    }

    fn schema_version(&self) -> u16 {
        SCHEMA_VERSION
    }

    fn decode(&self, payload: &serde_json::Value) -> Result<CommandBox, JournalCodecDecodeError> {
        let payload = decode_payload(payload, UPDATE_NODE_COMMAND_TYPE)?;
        EditorCommand::from_journal_update(payload)
            .map(|command| Box::new(command) as CommandBox)
            .map_err(command_decode_error)
    }
}

struct BatchTransformCodec;

impl EditCommandCodec for BatchTransformCodec {
    fn command_type(&self) -> &str {
        BATCH_TRANSFORM_COMMAND_TYPE
    }

    fn schema_version(&self) -> u16 {
        SCHEMA_VERSION
    }

    fn decode(&self, payload: &serde_json::Value) -> Result<CommandBox, JournalCodecDecodeError> {
        let payload: BatchTransformJournalPayload =
            decode_payload(payload, BATCH_TRANSFORM_COMMAND_TYPE)?;
        EditorCommand::from_journal_batch_transform(payload)
            .map(|command| Box::new(command) as CommandBox)
            .map_err(command_decode_error)
    }
}

struct SetReflectedSceneFieldCodec;

impl EditCommandCodec for SetReflectedSceneFieldCodec {
    fn command_type(&self) -> &str {
        SET_REFLECTED_FIELD_COMMAND_TYPE
    }

    fn schema_version(&self) -> u16 {
        SCHEMA_VERSION
    }

    fn decode(&self, payload: &serde_json::Value) -> Result<CommandBox, JournalCodecDecodeError> {
        let payload = decode_payload(payload, SET_REFLECTED_FIELD_COMMAND_TYPE)?;
        EditorCommand::from_journal_reflected_field(payload)
            .map(|command| Box::new(command) as CommandBox)
            .map_err(command_decode_error)
    }
}

fn decode_payload<T>(
    payload: &serde_json::Value,
    command_type: &str,
) -> Result<T, JournalCodecDecodeError>
where
    T: DeserializeOwned,
{
    serde_json::from_value(payload.clone()).map_err(|error| {
        JournalCodecDecodeError::invalid_payload(format!(
            "{command_type} payload does not match schema {SCHEMA_VERSION}: {error}"
        ))
    })
}

fn command_decode_error(
    error: crate::core::editing::engine::EditCommandError,
) -> JournalCodecDecodeError {
    JournalCodecDecodeError::invalid_payload(error.to_string())
}
