use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldValue, ReflectValueBudget, ReflectValueValidationError,
    ReflectedValue,
};

// Runtime 共用的单值树预算：根深度从 1 起，节点及 UTF-8 文本字节按该值树累计，容器条目按各单容器直接成员计。ReflectedValue 包装及其中 JSON 内容均参与检查；配置不持有跨调用计数。
const RUNTIME_REFLECT_VALUE_MAX_DEPTH: usize = 128;
const RUNTIME_REFLECT_VALUE_MAX_NODES: usize = 16_384;
const RUNTIME_REFLECT_VALUE_MAX_STRING_BYTES: usize = 1024 * 1024;
const RUNTIME_REFLECT_VALUE_MAX_CONTAINER_ENTRIES: usize = 4_096;

pub(crate) const RUNTIME_REFLECT_VALUE_BUDGET: ReflectValueBudget = ReflectValueBudget::new(
    RUNTIME_REFLECT_VALUE_MAX_DEPTH,
    RUNTIME_REFLECT_VALUE_MAX_NODES,
    RUNTIME_REFLECT_VALUE_MAX_STRING_BYTES,
    RUNTIME_REFLECT_VALUE_MAX_CONTAINER_ENTRIES,
);

// 远程反射值进入 World 前的统一资源预算与类型校验；读回也验证，避免发布超限的插件数据。
// 这里的校验范围是单值树预算与反射浮点有限性；type_path/field_name 只用于错误定位，字段类型匹配和具体写入准入由后续目录/适配器消费路径承担。
/// 按 Runtime 值树预算验证一个反射值，并补充错误上下文。
///
/// # 参数
///
/// `value` 为共享借用；`type_path`、`field_name` 仅定位错误，不用于解析 schema 或匹配声明字段类型。
///
/// # 返回值
///
/// 值树规模和适用的反射浮点有限性检查通过时返回成功，否则返回拥有上下文的 `InvalidValue`。
///
/// # 约束与生命周期
///
/// 不修改或克隆输入值。字段存在性、声明类型及领域写入条件由目录、转换适配器和 World writer 后续处理。
///
/// # 参考
///
/// `validate_reflected_value_contract`；`TypeRegistry` 的默认值类型检查和动态组件 VM 写入的 `ensure_reflected_value_type`。
pub(crate) fn validate_reflected_value(
    type_path: &str,
    field_name: &str,
    value: &ReflectedValue,
) -> Result<(), ReflectError> {
    validate_reflected_value_contract(value)
        .map_err(|error| invalid_value(type_path, field_name, error))
}

fn validate_reflected_field_value(
    type_path: &str,
    field: &ReflectFieldValue,
) -> Result<(), ReflectError> {
    validate_reflected_value(type_path, &field.field_name, &field.value)
}

/// 逐字段应用单值树预算检查。
///
/// # 参数
///
/// `type_path` 提供错误上下文；`fields` 是待检查字段值的共享切片，每个字段使用自己的名称定位错误。
///
/// # 返回值
///
/// 全部通过时成功；首个无效值返回错误。
///
/// # 约束与生命周期
///
/// 预算计数在各字段之间独立，不是整个批次的总节点或文本预算。本入口不验证字段 ID、名称、声明类型或唯一性；调用者继续完成相应 schema/写入准入。
///
/// # 参考
///
/// `validate_reflected_value`；动态场景 capture 与资源 spawn 预检消费路径。
pub(crate) fn validate_reflected_field_values(
    type_path: &str,
    fields: &[ReflectFieldValue],
) -> Result<(), ReflectError> {
    for field in fields {
        validate_reflected_field_value(type_path, field)?;
    }
    Ok(())
}

/// 使用共用 Runtime 配置执行 Interface 反射值树验证。
///
/// # 参数
///
/// `value` 为本次同步检查期间有效的共享借用。
///
/// # 返回值
///
/// 返回原始 `ReflectValueValidationError`，由调用者决定字段注册或值访问错误的上下文映射。
///
/// # 约束与生命周期
///
/// 每次调用建立新的预算状态，以借用工作栈遍历值树；不匹配目录中的声明字段类型。
///
/// # 参考
///
/// `RUNTIME_REFLECT_VALUE_BUDGET`、`ReflectedValue::validate_with_budget`、`validate_reflected_value`。
pub(crate) fn validate_reflected_value_contract(
    value: &ReflectedValue,
) -> Result<(), ReflectValueValidationError> {
    value.validate_with_budget(RUNTIME_REFLECT_VALUE_BUDGET)
}

fn invalid_value(
    type_path: &str,
    field_name: &str,
    error: ReflectValueValidationError,
) -> ReflectError {
    ReflectError::InvalidValue {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        reason: error.to_string(),
    }
}
