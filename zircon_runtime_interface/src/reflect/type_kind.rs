use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 注册元数据描述的类型形状；此标签不单独定义字段访问或编码行为。
pub enum ReflectTypeKind {
    /// 具名字段的结构体。
    Struct,
    /// 元组成员构成的结构体。
    TupleStruct,
    /// 元组形状。
    Tuple,
    /// 枚举类型；变体不等同于普通字段记录。
    Enum,
    /// 列表类型。
    List,
    /// 映射类型。
    Map,
    /// 标量类型。
    Scalar,
    /// 访问器不展开内部字段的不透明类型。
    Opaque,
    /// 由 JSON 值表示的类型。
    Json,
}
