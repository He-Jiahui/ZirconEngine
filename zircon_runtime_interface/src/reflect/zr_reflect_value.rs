use crate::math::{Vec2, Vec3, Vec4};

use super::{ReflectError, ReflectedValue};

/// Converts a Rust field value to and from the unified reflection value model.
pub trait ZrReflectValue: Sized {
    fn to_reflected_value(&self) -> ReflectedValue;

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError>;
}

macro_rules! impl_integer_value {
    ($type:ty, $variant:ident, $expected:literal) => {
        impl ZrReflectValue for $type {
            fn to_reflected_value(&self) -> ReflectedValue {
                ReflectedValue::$variant((*self).into())
            }

            fn from_reflected_value(
                value: ReflectedValue,
                owner_type_path: &str,
                field_name: &str,
            ) -> Result<Self, ReflectError> {
                match value {
                    ReflectedValue::$variant(value) => <$type>::try_from(value).map_err(|_| {
                        type_mismatch(
                            owner_type_path,
                            field_name,
                            $expected,
                            concat!("out-of-range ", $expected),
                        )
                    }),
                    value => Err(value_type_mismatch(
                        owner_type_path,
                        field_name,
                        $expected,
                        &value,
                    )),
                }
            }
        }
    };
}

impl ZrReflectValue for bool {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::Bool(*self)
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::Bool(value) => Ok(value),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "Bool",
                &value,
            )),
        }
    }
}

impl_integer_value!(i8, Integer, "Integer");
impl_integer_value!(i16, Integer, "Integer");
impl_integer_value!(i32, Integer, "Integer");
impl_integer_value!(i64, Integer, "Integer");
impl_integer_value!(u8, Unsigned, "Unsigned");
impl_integer_value!(u16, Unsigned, "Unsigned");
impl_integer_value!(u32, Unsigned, "Unsigned");
impl_integer_value!(u64, Unsigned, "Unsigned");

impl ZrReflectValue for f32 {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::Scalar(*self)
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::Scalar(value) if value.is_finite() => Ok(value),
            ReflectedValue::Scalar(_) => Err(type_mismatch(
                owner_type_path,
                field_name,
                "finite Scalar",
                "non-finite Scalar",
            )),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "Scalar",
                &value,
            )),
        }
    }
}

impl ZrReflectValue for String {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::String(self.clone())
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::String(value) => Ok(value),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "String",
                &value,
            )),
        }
    }
}

impl ZrReflectValue for Option<u64> {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::Entity(*self)
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::Entity(value) => Ok(value),
            ReflectedValue::Null => Ok(None),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "Entity",
                &value,
            )),
        }
    }
}

impl ZrReflectValue for Vec2 {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::Vec2(self.to_array())
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::Vec2(value) if values_are_finite(&value) => Ok(Self::from_array(value)),
            ReflectedValue::Vec2(_) => Err(non_finite_vector(owner_type_path, field_name, "Vec2")),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "Vec2",
                &value,
            )),
        }
    }
}

impl ZrReflectValue for Vec3 {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::Vec3(self.to_array())
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::Vec3(value) if values_are_finite(&value) => Ok(Self::from_array(value)),
            ReflectedValue::Vec3(_) => Err(non_finite_vector(owner_type_path, field_name, "Vec3")),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "Vec3",
                &value,
            )),
        }
    }
}

impl ZrReflectValue for Vec4 {
    fn to_reflected_value(&self) -> ReflectedValue {
        ReflectedValue::Vec4(self.to_array())
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        match value {
            ReflectedValue::Vec4(value) if values_are_finite(&value) => Ok(Self::from_array(value)),
            ReflectedValue::Vec4(_) => Err(non_finite_vector(owner_type_path, field_name, "Vec4")),
            value => Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "Vec4",
                &value,
            )),
        }
    }
}

impl<T> ZrReflectValue for Vec<T>
where
    T: ZrReflectValue,
{
    fn to_reflected_value(&self) -> ReflectedValue {
        let mut reflected = Vec::with_capacity(self.len());
        for value in self {
            reflected.push(value.to_reflected_value());
        }
        ReflectedValue::List(reflected)
    }

    fn from_reflected_value(
        value: ReflectedValue,
        owner_type_path: &str,
        field_name: &str,
    ) -> Result<Self, ReflectError> {
        let ReflectedValue::List(values) = value else {
            return Err(value_type_mismatch(
                owner_type_path,
                field_name,
                "List",
                &value,
            ));
        };
        let mut converted = Vec::with_capacity(values.len());
        for value in values {
            converted.push(T::from_reflected_value(value, owner_type_path, field_name)?);
        }
        Ok(converted)
    }
}

fn value_type_mismatch(
    owner_type_path: &str,
    field_name: &str,
    expected: &str,
    actual: &ReflectedValue,
) -> ReflectError {
    type_mismatch(owner_type_path, field_name, expected, actual.type_name())
}

fn type_mismatch(
    owner_type_path: &str,
    field_name: &str,
    expected: &str,
    actual: &str,
) -> ReflectError {
    ReflectError::TypeMismatch {
        type_path: owner_type_path.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}

fn non_finite_vector(owner_type_path: &str, field_name: &str, kind: &str) -> ReflectError {
    type_mismatch(
        owner_type_path,
        field_name,
        &format!("finite {kind}"),
        &format!("non-finite {kind}"),
    )
}

fn values_are_finite<const N: usize>(values: &[f32; N]) -> bool {
    for value in values {
        if !value.is_finite() {
            return false;
        }
    }
    true
}
