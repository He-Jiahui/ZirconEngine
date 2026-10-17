//! 绑定值的 Editor JSON 投影与原生文本追加。
//!
//! 两种投影均只读借用拥有值树，返回新 JSON 或追加到调用方字符串，不改变树本身。
//! 回调 JSON 将扩展类型写为标签对象，空可选值会折叠为空值；它不同于 Serde 枚举编码。
//! 文本路径按静态变体递归写入一个缓冲区；此处不执行预算、身份或有限性准入。

use std::fmt::Write as _;

use serde_json::{json, Map, Value};

use super::{types::quoted_into, UiBindingValue};

#[cfg(test)]
#[path = "../tests/native_repr_perf.rs"]
mod native_repr_performance_tests;

impl UiBindingValue {
    /// 投影为编辑器回调边界使用的 JSON；扩展类型用标签对象承载，空可选值与非有限浮点按契约投影为 null。
    ///
    /// # 使用时机与返回
    /// 供 Editor 回调边界消费，返回独立拥有的 JSON 树；借用在本次遍历结束后即可释放。
    /// 数组和记录递归投影，扩展载荷写成标签对象；可选载荷不保留独立类别标签。
    ///
    /// # 约束
    /// 非有限浮点会成为空值，结果不保证可恢复原始绑定值类别；需要保留类别时使用结构化编码。
    /// 本方法不做整树预算检查，也不获取资产、实体或集合内容。
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Unsigned(value) => Value::Number((*value).into()),
            Self::Signed(value) => Value::Number((*value).into()),
            Self::Float(value) => serde_json::Number::from_f64(*value)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            Self::Bool(value) => Value::Bool(*value),
            Self::Null => Value::Null,
            Self::Array(values) => Value::Array(values.iter().map(Self::to_json_value).collect()),
            Self::Record(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_json_value()))
                    .collect::<Map<_, _>>(),
            ),
            Self::Map(values) => json!({
                "$map": values
                    .iter()
                    .map(|(key, value)| json!({
                        "key": key.to_json_value(),
                        "value": value.to_json_value(),
                    }))
                    .collect::<Vec<_>>()
            }),
            Self::Enum(value) => {
                let mut projected = Map::new();
                projected.insert(
                    "type".to_string(),
                    Value::String(value.type_id().to_string()),
                );
                projected.insert(
                    "variant".to_string(),
                    Value::String(value.variant().to_string()),
                );
                if let Some(payload) = value.payload() {
                    projected.insert("payload".to_string(), payload.to_json_value());
                }
                json!({"$enum": projected})
            }
            Self::Asset(value) => json!({"$asset": value.locator()}),
            Self::Entity(value) => json!({
                "$entity": {
                    "id": value.entity_id(),
                    "generation": value.generation(),
                }
            }),
            Self::Optional(value) => value
                .as_deref()
                .map(Self::to_json_value)
                .unwrap_or(Value::Null),
            Self::CollectionView(value) => json!({
                "$collection_view": {
                    "provider_id": value.provider().id.as_str(),
                    "provider_version": value.provider().version.get(),
                    "item_schema_id": value.item_schema().id.as_str(),
                    "item_schema_version": value.item_schema().version.get(),
                    "revision": value.revision(),
                    "offset": value.offset(),
                    "length": value.length(),
                    "total_length": value.total_length(),
                }
            }),
        }
    }

    /// 为已校验的绑定值生成原生文本；解析器按文本语法重建值，整数类别不在文本中单独编码。
    ///
    /// # 约束
    /// 仅供本 crate 的测试对照使用，返回独立文本；不证明类别无损往返或性能阈值已通过。
    #[cfg(test)]
    pub(crate) fn native_repr(&self) -> String {
        let mut output = String::new();
        self.native_repr_into(&mut output);
        output
    }

    /// 将当前值的原生文本追加到既有缓冲区。
    ///
    /// # 参数与生命周期
    /// `output` 可已有前缀；递归子值共享该独占借用，退出后由调用方继续拥有全部文本。
    /// 不清空缓冲区，也不留下对树或输出的借用。
    ///
    /// # 约束
    /// 调用方需要受控且可重新解析的输入时，应先校验整个根值。
    /// 数字字面量不为非负整型单独编码类别；字符串只使用与解析器对应的有限转义集合。
    pub(crate) fn native_repr_into(&self, output: &mut String) {
        match self {
            Self::String(value) => quoted_into(value, output),
            Self::Unsigned(value) => {
                let _ = write!(output, "{value}");
            }
            Self::Signed(value) => {
                let _ = write!(output, "{value}");
            }
            Self::Float(value) => {
                let start = output.len();
                let _ = write!(output, "{value}");
                if !output[start..].contains('.')
                    && !output[start..].contains('e')
                    && !output[start..].contains('E')
                {
                    output.push_str(".0");
                }
            }
            Self::Bool(value) => {
                let _ = write!(output, "{value}");
            }
            Self::Null => output.push_str("null"),
            Self::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    value.native_repr_into(output);
                }
                output.push(']');
            }
            Self::Record(fields) => {
                output.push_str("record(");
                for (index, (field, value)) in fields.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    quoted_into(field, output);
                    output.push(',');
                    value.native_repr_into(output);
                }
                output.push(')');
            }
            Self::Map(values) => {
                output.push_str("map(");
                for (index, (key, value)) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    key.native_repr_into(output);
                    output.push(',');
                    value.native_repr_into(output);
                }
                output.push(')');
            }
            Self::Enum(value) => {
                output.push_str("enum(");
                quoted_into(value.type_id(), output);
                output.push(',');
                quoted_into(value.variant(), output);
                if let Some(payload) = value.payload() {
                    output.push(',');
                    payload.native_repr_into(output);
                }
                output.push(')');
            }
            Self::Asset(value) => {
                output.push_str("asset(");
                quoted_into(value.locator(), output);
                output.push(')');
            }
            Self::Entity(value) => {
                let _ = write!(
                    output,
                    "entity({},{})",
                    value.entity_id(),
                    value.generation()
                );
            }
            Self::Optional(value) => {
                output.push_str("optional(");
                if let Some(value) = value.as_deref() {
                    value.native_repr_into(output);
                }
                output.push(')');
            }
            Self::CollectionView(value) => {
                output.push_str("collection_view(");
                quoted_into(value.provider().id.as_str(), output);
                let _ = write!(output, ",{},", value.provider().version.get());
                quoted_into(value.item_schema().id.as_str(), output);
                let _ = write!(
                    output,
                    ",{},{},{},{},{})",
                    value.item_schema().version.get(),
                    value.revision(),
                    value.offset(),
                    value.length(),
                    value.total_length(),
                );
            }
        }
    }
}
