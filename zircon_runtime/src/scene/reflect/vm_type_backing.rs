//! 选择 VM 反射元数据在 Runtime 场景中的承载方式。
//! 承载类型参与注册时的角色、字段语法和动态组件描述符检查。

use serde::{Deserialize, Serialize};

/// VM 类型注册的承载选择；序列化名称采用 `snake_case`。
///
/// 由 `TypeRegistry` 构造动态描述符和操作表，`World::register_vm_type` 另同步 World 的目录与 VM 类型标识。
/// 枚举值本身不拥有实例、目录或 World；参见这两个注册入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmTypeBacking {
    /// 以 World 的 JSON 动态组件存储承载组件角色；线名为 `dynamic_component`。
    DynamicComponent,
}

#[cfg(test)]
#[path = "tests/vm_type_backing.rs"]
mod tests;
