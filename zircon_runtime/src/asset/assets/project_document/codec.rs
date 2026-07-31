use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::asset::assets::ProjectDocumentError;

pub(in crate::asset) struct ProjectDocumentArtifact {
    value: toml::Value,
}

impl ProjectDocumentArtifact {
    pub(in crate::asset) fn parse(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str::<toml::Value>(document).map(|value| Self { value })
    }

    pub(in crate::asset) fn value(&self) -> &toml::Value {
        &self.value
    }

    pub(in crate::asset) fn value_mut(&mut self) -> &mut toml::Value {
        &mut self.value
    }

    pub(in crate::asset) fn into_document<T>(self) -> Result<T, toml::de::Error>
    where
        T: DeserializeOwned,
    {
        self.value.try_into()
    }

    pub(in crate::asset) fn to_pretty_bytes(&self) -> Result<Vec<u8>, toml::ser::Error> {
        toml::to_string_pretty(&self.value).map(String::into_bytes)
    }
}

pub(super) fn encode_document<T, D>(value: &T) -> Result<D, ProjectDocumentError>
where
    T: Serialize,
    D: for<'de> Deserialize<'de>,
{
    let encoded = toml::Value::try_from(value)?;
    Ok(encoded.try_into()?)
}

pub(super) fn decode_document<T, D>(value: D) -> Result<T, ProjectDocumentError>
where
    T: for<'de> Deserialize<'de>,
    D: Serialize,
{
    let encoded = toml::Value::try_from(value)?;
    Ok(encoded.try_into()?)
}
