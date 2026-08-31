use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    zircon_reflect_derive::ZrReflect,
)]
#[zr_reflect(
    component,
    type_path = "zircon_runtime::core::framework::scene::Mobility",
    script_visibility = "public",
    field(
        name = "kind",
        value_type_path = "Enum",
        editor_hint = "Enum",
        read = "reflection::read_kind",
        write = "reflection::write_kind"
    )
)]
pub enum Mobility {
    Dynamic,
    Static,
}

mod reflection {
    use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

    use super::Mobility;

    const TYPE_PATH: &str = "zircon_runtime::core::framework::scene::Mobility";

    pub(super) fn read_kind(value: &Mobility) -> Result<ReflectedValue, ReflectError> {
        Ok(ReflectedValue::Enum(
            match value {
                Mobility::Dynamic => "dynamic",
                Mobility::Static => "static",
            }
            .to_string(),
        ))
    }

    pub(super) fn write_kind(
        current: &mut Mobility,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError> {
        let ReflectedValue::Enum(value) = value else {
            return Err(ReflectError::TypeMismatch {
                type_path: TYPE_PATH.to_string(),
                field_name: "kind".to_string(),
                expected: "Enum".to_string(),
                actual: value.type_name().to_string(),
            });
        };
        let Some(next) = parse_mobility(&value) else {
            return Err(ReflectError::UnsupportedConversion {
                source: value,
                target: "Mobility".to_string(),
            });
        };
        if *current == next {
            return Ok(false);
        }
        *current = next;
        Ok(true)
    }

    pub(super) fn parse_mobility(value: &str) -> Option<Mobility> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("dynamic") {
            Some(Mobility::Dynamic)
        } else if value.eq_ignore_ascii_case("static") {
            Some(Mobility::Static)
        } else {
            None
        }
    }
}

impl Default for Mobility {
    fn default() -> Self {
        Self::Dynamic
    }
}

#[cfg(test)]
#[path = "mobility/borrowed_parse_tests.rs"]
mod borrowed_parse_tests;
