use super::{ReflectError, ReflectTypeRegistration, ReflectedValue};

/// Compile-time reflection contract implemented by `#[derive(ZrReflect)]`.
/// 由派生宏实现的静态反射契约，运行时组件适配器据此建立注册并访问字段。
pub trait ZrReflect: Sized {
    fn reflect_type_registration() -> Result<ReflectTypeRegistration, ReflectError>;

    fn read_reflected_field(&self, field_name: &str) -> Result<ReflectedValue, ReflectError>;

    /// 返回字段值是否实际变化；派生组件适配器仅在变化后重新插入组件。
    fn write_reflected_field(
        &mut self,
        field_name: &str,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError>;

    /// 槽位取自当期注册字段顺序，持久化引用应使用稳定字段 ID。
    fn read_reflected_field_by_slot(&self, field_slot: u32)
        -> Result<ReflectedValue, ReflectError>;

    fn write_reflected_field_by_slot(
        &mut self,
        field_slot: u32,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError>;
}
