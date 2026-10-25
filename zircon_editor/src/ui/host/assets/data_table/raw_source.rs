use std::{collections::HashSet, fmt, ops::Range};

use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

use super::DataTableSourceError;

pub(super) struct RawRow<'a>(pub Vec<(String, &'a RawValue)>);

pub(super) fn rows(text: &str) -> Result<Vec<RawRow<'_>>, DataTableSourceError> {
    serde_json::from_str(text).map_err(|error| DataTableSourceError::Encoding(error.to_string()))
}

/// RawValue is borrowed from the same original text. Validate its byte range before replacing
/// anything; never search by repeated value text or use a display/filter index as identity.
pub(super) fn span(text: &str, raw: &RawValue) -> Option<Range<usize>> {
    let start = (raw.get().as_ptr() as usize).checked_sub(text.as_ptr() as usize)?;
    let end = start.checked_add(raw.get().len())?;
    (text.get(start..end) == Some(raw.get())).then_some(start..end)
}

impl<'de> Deserialize<'de> for RawRow<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RowVisitor;
        impl<'de> Visitor<'de> for RowVisitor {
            type Value = RawRow<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a data table row object")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut names = HashSet::new();
                let mut values = Vec::new();
                while let Some((name, value)) = map.next_entry::<String, &'de RawValue>()? {
                    if !names.insert(name.clone()) {
                        return Err(M::Error::custom(format!("duplicate row property {name}")));
                    }
                    values.push((name, value));
                }
                Ok(RawRow(values))
            }
        }
        deserializer.deserialize_map(RowVisitor)
    }
}
