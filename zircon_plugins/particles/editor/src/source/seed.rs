use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};

use super::ParticleSourceError;

/// Field input stays decimal text until this boundary; it must never pass through a float.
pub fn parse_particle_seed(value: &str) -> Result<u64, ParticleSourceError> {
    if value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
        return Err(ParticleSourceError::invalid(
            "system.seed",
            "seed must contain only unsigned decimal digits",
        ));
    }
    value.parse().map_err(|error: std::num::ParseIntError| {
        ParticleSourceError::invalid("system.seed", error.to_string())
    })
}

pub(super) fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    struct SeedVisitor;

    impl<'de> Visitor<'de> for SeedVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an unsigned decimal u64 string or a legacy nonnegative integer")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<u64, E> {
            parse_particle_seed(value).map_err(E::custom)
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<u64, E> {
            Ok(value)
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<u64, E> {
            u64::try_from(value).map_err(|_| E::custom("seed cannot be negative"))
        }
    }

    // TOML's integer range is signed. Strings preserve the full u64 range across every bridge.
    deserializer.deserialize_any(SeedVisitor)
}
