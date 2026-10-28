use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue, ZrReflectValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

impl Default for ProjectionMode {
    fn default() -> Self {
        Self::Perspective
    }
}

impl ZrReflectValue for ProjectionMode {
    fn to_reflected_value(&self) -> ReflectedValue {
        // Keep reflected names aligned with the serialized scene variants.
        ReflectedValue::Enum(
            match self {
                Self::Perspective => "Perspective",
                Self::Orthographic => "Orthographic",
            }
            .to_string(),
        )
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match &value {
            ReflectedValue::Enum(name) if name == "Perspective" => Ok(Self::Perspective),
            ReflectedValue::Enum(name) if name == "Orthographic" => Ok(Self::Orthographic),
            _ => Err(ReflectError::TypeMismatch {
                type_path: owner_type_path.to_string(),
                field_name: field_name.to_string(),
                expected: "ProjectionMode (Perspective or Orthographic)".to_string(),
                actual: match value {
                    ReflectedValue::Enum(name) => format!("Enum({name})"),
                    value => value.type_name().to_string(),
                },
            }),
        }
    }
}
