use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserializer, Serializer};

use crate::core::math::Real;

use super::PhysicsJointDrive;

const AXIS_LIMIT_KEYS: [&str; 3] = ["x", "y", "z"];

pub(super) fn axis_limits_are_empty(limits: &[Option<[Real; 2]>; 3]) -> bool {
    limits.iter().all(Option::is_none)
}

pub(super) fn joint_drives_are_default(drives: &[PhysicsJointDrive; 3]) -> bool {
    drives
        .iter()
        .all(|drive| *drive == PhysicsJointDrive::default())
}

pub(super) fn serialize_axis_limits<S>(
    limits: &[Option<[Real; 2]>; 3],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let entry_count = limits.iter().filter(|limit| limit.is_some()).count();
    let mut map = serializer.serialize_map(Some(entry_count))?;
    for (index, limit) in limits.iter().enumerate() {
        if let Some(limit) = limit {
            map.serialize_entry(AXIS_LIMIT_KEYS[index], limit)?;
        }
    }
    map.end()
}

pub(super) fn deserialize_axis_limits<'de, D>(
    deserializer: D,
) -> Result<[Option<[Real; 2]>; 3], D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(AxisLimitsVisitor)
}

struct AxisLimitsVisitor;

impl<'de> Visitor<'de> for AxisLimitsVisitor {
    type Value = [Option<[Real; 2]>; 3];

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("three optional axis limits or an x/y/z limit map")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut limits = [None, None, None];
        for slot in &mut limits {
            if let Some(limit) = sequence.next_element::<Option<[Real; 2]>>()? {
                *slot = limit;
            } else {
                return Ok(limits);
            }
        }
        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(4, &"at most three axis limits"));
        }
        Ok(limits)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut limits = [None, None, None];
        while let Some(key) = map.next_key::<String>()? {
            let index = axis_limit_index(&key)
                .ok_or_else(|| de::Error::unknown_field(&key, &AXIS_LIMIT_KEYS))?;
            if limits[index].is_some() {
                return Err(de::Error::duplicate_field(AXIS_LIMIT_KEYS[index]));
            }
            limits[index] = Some(map.next_value()?);
        }
        Ok(limits)
    }
}

fn axis_limit_index(key: &str) -> Option<usize> {
    match key {
        "x" | "0" => Some(0),
        "y" | "1" => Some(1),
        "z" | "2" => Some(2),
        _ => None,
    }
}
