//! 固定 ReflectValueBudget 与 ReflectedValue::validate_with_budget 的契约样例。
//!
//! 覆盖上限读取、各维度错误和嵌入 JSON 深度；不把预算解释为 schema、字段类型或权限验证。

use std::collections::BTreeMap;

use crate::reflect::{
    ReflectValueBudget, ReflectValueFloatKind, ReflectValueValidationError, ReflectedValue,
};

/// 用测试样本显式指定四个独立预算维度。
///
/// # 参数
/// 参数依次为最大深度、节点数、字符串 UTF-8 字节数和单容器直接条目数。
///
/// # 返回值
/// 通过 ReflectValueBudget::new 原样构造的配额；不应用默认值或二次校验。

fn budget(
    max_depth: usize,
    max_nodes: usize,
    max_string_bytes: usize,
    max_container_entries: usize,
) -> ReflectValueBudget {
    ReflectValueBudget::new(
        max_depth,
        max_nodes,
        max_string_bytes,
        max_container_entries,
    )
}

/// 确认四个 getter 分别返回构造时传入的调用方上限。
#[test]
fn reflected_value_budget_exposes_caller_owned_limits() {
    let budget = budget(7, 11, 13, 17);

    assert_eq!(budget.max_depth(), 7);
    assert_eq!(budget.max_nodes(), 11);
    assert_eq!(budget.max_string_bytes(), 13);
    assert_eq!(budget.max_container_entries(), 17);
}

/// 构造含反射 Map、嵌入 JSON 和 List 的值并在宽裕预算下通过；不声称预算等于实际计数。
#[test]
fn reflected_value_budget_counts_reflected_and_embedded_json_graphs() {
    let value = ReflectedValue::Map(BTreeMap::from([
        (
            "json".to_string(),
            ReflectedValue::Json(serde_json::json!({ "label": ["ok"] })),
        ),
        (
            "list".to_string(),
            ReflectedValue::List(vec![ReflectedValue::Unsigned(3)]),
        ),
    ]));

    assert_eq!(value.validate_with_budget(budget(6, 9, 19, 2)), Ok(()));
}

/// 分别验证深度、节点、累计 UTF-8 字节和单容器条目数越界及其计数。
#[test]
fn reflected_value_budget_rejects_each_bounded_dimension() {
    // 四个样本分别越过深度、节点、字符串和容器上限，确保错误来源不会互相替代。
    let depth = ReflectedValue::List(vec![ReflectedValue::List(vec![ReflectedValue::Null])]);
    assert_eq!(
        depth.validate_with_budget(budget(2, 8, 8, 8)),
        Err(ReflectValueValidationError::DepthExceeded {
            actual: 3,
            maximum: 2,
        })
    );

    let nodes = ReflectedValue::List(vec![ReflectedValue::Null, ReflectedValue::Null]);
    assert_eq!(
        nodes.validate_with_budget(budget(4, 2, 8, 8)),
        Err(ReflectValueValidationError::NodeBudgetExceeded {
            actual: 3,
            maximum: 2,
        })
    );

    let strings = ReflectedValue::Map(BTreeMap::from([(
        "aa".to_string(),
        ReflectedValue::String("bbb".to_string()),
    )]));
    assert_eq!(
        strings.validate_with_budget(budget(4, 8, 4, 8)),
        Err(ReflectValueValidationError::StringBudgetExceeded {
            actual: 5,
            maximum: 4,
        })
    );

    let container = ReflectedValue::List(vec![ReflectedValue::Null, ReflectedValue::Null]);
    assert_eq!(
        container.validate_with_budget(budget(4, 8, 8, 1)),
        Err(ReflectValueValidationError::ContainerEntriesExceeded {
            actual: 2,
            maximum: 1,
        })
    );
}

/// 确认四元数中首个非有限分量以零基索引返回。
#[test]
fn reflected_value_budget_rejects_non_finite_vector_components() {
    let value = ReflectedValue::Quaternion([0.0, f32::NAN, 0.0, 1.0]);

    assert_eq!(
        value.validate_with_budget(budget(2, 2, 0, 0)),
        Err(ReflectValueValidationError::NonFiniteFloat {
            kind: ReflectValueFloatKind::Quaternion,
            component: 1,
        })
    );
}

/// 固定 Json 包装节点和嵌入 JSON 根分别计节点且根深度加一的契约。
#[test]
fn embedded_json_root_is_a_child_of_the_reflected_json_wrapper() {
    let value = ReflectedValue::Json(serde_json::json!([null]));

    assert_eq!(
        value.validate_with_budget(budget(2, 8, 0, 8)),
        Err(ReflectValueValidationError::DepthExceeded {
            actual: 3,
            maximum: 2,
        })
    );
}
