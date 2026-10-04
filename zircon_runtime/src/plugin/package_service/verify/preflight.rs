use super::{PackageError, Result, MAX_FILES};
use serde::{
    de::{IgnoredAny, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

const MAX_PACK_MANIFEST_BYTES: usize = 1024 * 1024;

struct BoundedEntries;
impl<'de> Deserialize<'de> for BoundedEntries {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct Entries;
        impl<'de> Visitor<'de> for Entries {
            type Value = BoundedEntries;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("bounded package member list")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                for _ in 0..MAX_FILES {
                    if sequence.next_element::<IgnoredAny>()?.is_none() {
                        return Ok(BoundedEntries);
                    }
                }
                if sequence.next_element::<IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom("package member capacity"));
                }
                Ok(BoundedEntries)
            }
        }
        deserializer.deserialize_seq(Entries)
    }
}

#[derive(Deserialize)]
struct Pack {
    chunks: BoundedEntries,
}
#[derive(Deserialize)]
struct Manifest {
    pack: Pack,
    assets: BoundedEntries,
}

pub(super) fn admit(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 24 || &bytes[..4] != b"ZRPK" {
        return Err(PackageError::Invalid);
    }
    let offset = usize::try_from(u64::from_le_bytes(
        bytes[8..16].try_into().map_err(|_| PackageError::Invalid)?,
    ))
    .map_err(|_| PackageError::Capacity)?;
    let size = usize::try_from(u64::from_le_bytes(
        bytes[16..24]
            .try_into()
            .map_err(|_| PackageError::Invalid)?,
    ))
    .map_err(|_| PackageError::Capacity)?;
    if size > MAX_PACK_MANIFEST_BYTES {
        return Err(PackageError::Capacity);
    }
    if offset < 24 || offset.checked_add(size) != Some(bytes.len()) {
        return Err(PackageError::Invalid);
    }
    let Manifest {
        pack: Pack { chunks: _chunks },
        assets: _assets,
    } = serde_json::from_slice::<Manifest>(&bytes[offset..]).map_err(|_| PackageError::Capacity)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/preflight.rs"]
mod tests;
