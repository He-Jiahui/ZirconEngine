use serde_json::Value;

/// Recursively orders every object key before the text formatter runs.
///
/// 原地遍历拥有的 JSON 树并排序各层对象键；数组元素的次序和标量值保持不变。
///
/// # 参数与返回
/// `value` 的所有权移入，返回排序后树的所有权，不从外部输入保留借用。
///
/// # 当前调用边界
/// 二进制 payload 编码先使用本函数，再转换为受限二进制值；流式文本写入走独立的序列化器。
/// 本函数不执行 schema、深度或字节预算检查，不能替代调用入口的验证。
///
/// # 参考
/// 二进制入口 `super::super::binary::encode::encode_binary_payload`。
pub(in crate::serialization) fn canonicalize_value(mut value: Value) -> Value {
    canonicalize_value_in_place(&mut value);
    value
}

/// 只递归进入容器；对象在子值处理完后排序，数组仅依序访问其元素。
fn canonicalize_value_in_place(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                canonicalize_nested_value(value);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                canonicalize_nested_value(value);
            }
            values.sort_keys();
        }
        _ => {}
    }
}

#[inline]
/// 跳过标量叶，避免为不含子节点的值再次进入容器分派。
fn canonicalize_nested_value(value: &mut Value) {
    if matches!(value, Value::Array(_) | Value::Object(_)) {
        canonicalize_value_in_place(value);
    }
}

#[cfg(test)]
#[path = "tests/canonical_perf.rs"]
mod performance_tests;
