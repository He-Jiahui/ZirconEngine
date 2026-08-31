use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::deadline::{DeadlineReader, ProcessingDeadline};
use super::BoundedJsonError;

pub(super) fn preflight_json_graph(
    bytes: &[u8],
    max_items: usize,
    deadline: ProcessingDeadline,
) -> Result<(), BoundedJsonError> {
    let mut counter = JsonItemCounter::new(max_items);
    let reader = DeadlineReader::new(bytes, deadline);
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let result = JsonItemSeed {
        counter: &mut counter,
    }
    .deserialize(&mut deserializer)
    .and_then(|()| deserializer.end());
    if let Some(observed) = counter.overflow_observed {
        return Err(BoundedJsonError::Items {
            observed,
            limit: max_items,
        });
    }
    result.map_err(|error| {
        if deadline.exceeded() {
            BoundedJsonError::ProcessingTime {
                limit_micros: deadline.limit.as_micros().try_into().unwrap_or(u64::MAX),
            }
        } else {
            BoundedJsonError::Json(error.to_string())
        }
    })?;
    deadline.check()
}

struct JsonItemCounter {
    observed: usize,
    limit: usize,
    overflow_observed: Option<usize>,
}

impl JsonItemCounter {
    fn new(limit: usize) -> Self {
        Self {
            observed: 0,
            limit,
            overflow_observed: None,
        }
    }

    fn observe<E: serde::de::Error>(&mut self) -> Result<(), E> {
        self.observed = self.observed.saturating_add(1);
        if self.observed <= self.limit {
            return Ok(());
        }
        self.overflow_observed = Some(self.observed);
        Err(E::custom("JSON item limit exceeded"))
    }
}

struct JsonItemSeed<'a> {
    counter: &'a mut JsonItemCounter,
}

impl<'de> DeserializeSeed<'de> for JsonItemSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.counter.observe::<D::Error>()?;
        deserializer.deserialize_any(JsonItemVisitor {
            counter: self.counter,
        })
    }
}

struct JsonItemVisitor<'a> {
    counter: &'a mut JsonItemCounter,
}

impl<'de> Visitor<'de> for JsonItemVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_borrowed_str<E>(self, _value: &'de str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
        }
        .deserialize(deserializer)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        JsonItemSeed {
            counter: self.counter,
        }
        .deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence
            .next_element_seed(JsonItemSeed {
                counter: self.counter,
            })?
            .is_some()
        {}
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_key::<IgnoredAny>()?.is_some() {
            map.next_value_seed(JsonItemSeed {
                counter: self.counter,
            })?;
        }
        Ok(())
    }
}

pub(in crate::dynamic_api) fn json_value_item_count(value: &serde_json::Value) -> usize {
    let mut count = 0_usize;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        count = count.saturating_add(1);
        match value {
            serde_json::Value::Array(values) => pending.extend(values),
            serde_json::Value::Object(values) => pending.extend(values.values()),
            _ => {}
        }
    }
    count
}
