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
        let next = match value.trim().to_ascii_lowercase().as_str() {
            "dynamic" => Mobility::Dynamic,
            "static" => Mobility::Static,
            _ => {
                return Err(ReflectError::UnsupportedConversion {
                    source: value,
                    target: "Mobility".to_string(),
                })
            }
        };
        if *current == next {
            return Ok(false);
        }
        *current = next;
        Ok(true)
    }
}

impl Default for Mobility {
    fn default() -> Self {
        Self::Dynamic
    }
}
