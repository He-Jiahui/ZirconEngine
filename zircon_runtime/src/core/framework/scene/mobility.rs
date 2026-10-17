use serde::{Deserialize, Serialize};

/// 实体的移动性声明，默认动态；World 层负责切换时的层级约束。
///
/// # 序列化与反射
/// serde 使用变体原名；反射字段 `kind` 使用小写文本，并接纳裁剪后的 ASCII 大小写变体。
/// 派生反射适配器先修改克隆候选，再经 `World::set_mobility` 提交领域校验。
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
    /// 动态移动性状态，也是本类型的默认值。
    Dynamic,
    /// 静态移动性状态，与父子层级的兼容性由 World 写入口判断。
    Static,
}

mod reflection {
    use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

    use super::Mobility;

    const TYPE_PATH: &str = "zircon_runtime::core::framework::scene::Mobility";

    /// 同步借用枚举并返回拥有型的小写反射值，不保留组件引用。
    pub(super) fn read_kind(value: &Mobility) -> Result<ReflectedValue, ReflectError> {
        Ok(ReflectedValue::Enum(
            match value {
                Mobility::Dynamic => "dynamic",
                Mobility::Static => "static",
            }
            .to_string(),
        ))
    }

    /// 消费枚举反射值并更新当前候选。
    ///
    /// 类型错误或未知文本在赋值前返回错误；相同值返回 `false`，成功改变返回 `true`。
    /// 此助手只处理枚举转换，World 的实际提交见派生组件适配器与受检写入口。
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

    /// 只读解析裁剪后的动态/静态文本，成功返回可复制枚举，未知文本返回 `None`。
    /// 输入借用仅用于比较，不保存字符串。
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
#[path = "mobility/tests/borrowed_parse_tests.rs"]
mod borrowed_parse_tests;
