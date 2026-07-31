use super::{ReflectError, ReflectTypeRegistration, ReflectedValue};

/// Compile-time reflection contract implemented by `#[derive(ZrReflect)]`.
pub trait ZrReflect: Sized {
    fn reflect_type_registration() -> Result<ReflectTypeRegistration, ReflectError>;

    fn read_reflected_field(&self, field_name: &str) -> Result<ReflectedValue, ReflectError>;

    fn write_reflected_field(
        &mut self,
        field_name: &str,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError>;

    fn read_reflected_field_by_slot(&self, field_slot: u32)
    -> Result<ReflectedValue, ReflectError>;

    fn write_reflected_field_by_slot(
        &mut self,
        field_slot: u32,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError>;
}
