use serde::{Deserialize, Serialize};

use crate::asset::assets::ProjectDocumentError;

pub(super) fn encode_document<T, D>(value: &T) -> Result<D, ProjectDocumentError>
where
    T: Serialize,
    D: for<'de> Deserialize<'de>,
{
    let encoded = toml::to_string(value)?;
    Ok(toml::from_str(&encoded)?)
}

pub(super) fn decode_document<T, D>(value: D) -> Result<T, ProjectDocumentError>
where
    T: for<'de> Deserialize<'de>,
    D: Serialize,
{
    let encoded = toml::to_string(&value)?;
    Ok(toml::from_str(&encoded)?)
}
