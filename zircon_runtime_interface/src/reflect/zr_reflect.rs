use super::{ReflectError, ReflectTypeRegistration, ReflectedValue};

/// Compile-time reflection contract implemented by `#[derive(ZrReflect)]`.
/// 由派生宏实现的静态反射契约，运行时组件适配器据此建立注册并访问字段。
pub trait ZrReflect: Sized {
    /// 返回派生类型的注册元数据；注册表调用此契约获取声明，之后再校验并接纳注册项。
    fn reflect_type_registration() -> Result<ReflectTypeRegistration, ReflectError>;

    /// 按本类型字段名读取当前值；未知字段返回 `UnknownField`。
    fn read_reflected_field(&self, field_name: &str) -> Result<ReflectedValue, ReflectError>;

    /// 返回字段值是否实际变化；派生组件适配器仅在变化后重新插入组件。
    /// `field_name` 为派生注册使用的字段名，`value` 会按 Rust 字段类型反向转换；名称、类型或取值无效时返回错误。
    fn write_reflected_field(
        &mut self,
        field_name: &str,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError>;

    /// 槽位取自当期注册字段顺序，持久化引用应使用稳定字段 ID。
    /// `field_slot` 是当前注册中的稠密零起始索引；不存在的槽位返回 `UnknownField`。
    fn read_reflected_field_by_slot(&self, field_slot: u32)
        -> Result<ReflectedValue, ReflectError>;

    /// 按当前注册槽位写入字段；返回值表示实际值是否变化，适配器据此决定是否提交对象副本。
    /// 槽位仅在当前注册顺序内有效；槽位不存在或值无法转换时返回错误。
    fn write_reflected_field_by_slot(
        &mut self,
        field_slot: u32,
        value: ReflectedValue,
    ) -> Result<bool, ReflectError>;
}
