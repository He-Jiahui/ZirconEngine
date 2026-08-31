use std::collections::BTreeMap;

use super::super::CommandJournalPayload;
use super::{JournalCodecDecodeError, JournalCodecError};
use crate::core::editing::engine::CommandBox;

pub trait EditCommandCodec: Send + Sync {
    fn command_type(&self) -> &str;

    fn schema_version(&self) -> u16;

    fn decode(&self, payload: &serde_json::Value) -> Result<CommandBox, JournalCodecDecodeError>;
}

#[derive(Default)]
pub struct EditCommandCodecRegistry {
    codecs: BTreeMap<(String, u16), Box<dyn EditCommandCodec>>,
}

impl EditCommandCodecRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        codec: impl EditCommandCodec + 'static,
    ) -> Result<(), JournalCodecError> {
        let command_type = codec.command_type();
        if command_type.is_empty() {
            return Err(JournalCodecError::EmptyCommandType);
        }
        let key = (command_type.to_owned(), codec.schema_version());
        if self.codecs.contains_key(&key) {
            return Err(JournalCodecError::Duplicate {
                command_type: key.0,
                schema_version: key.1,
            });
        }
        self.codecs.insert(key, Box::new(codec));
        Ok(())
    }

    pub fn decode(&self, payload: &CommandJournalPayload) -> Result<CommandBox, JournalCodecError> {
        let command_type = payload.command_type();
        let schema_version = payload.schema_version();
        let codec = self
            .codecs
            .get(&(command_type.to_owned(), schema_version))
            .ok_or_else(|| JournalCodecError::Unregistered {
                command_type: command_type.to_owned(),
                schema_version,
            })?;
        codec
            .decode(payload.payload())
            .map_err(|source| JournalCodecError::Decode {
                command_type: command_type.to_owned(),
                schema_version,
                source,
            })
    }

    pub fn decode_all(
        &self,
        payloads: &[CommandJournalPayload],
    ) -> Result<Vec<CommandBox>, JournalCodecError> {
        payloads
            .iter()
            .map(|payload| self.decode(payload))
            .collect()
    }
}
