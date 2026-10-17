//! 验证解析结果借用输入存储，并保留解析器的源码契约。

use std::collections::BTreeMap;

use super::*;

/// 比较底层地址，证明文本队列标识没有额外复制。
#[test]
fn optimization_batch_20260909_runtime274_queue_string_parser_borrows_id_storage() {
    let value = UiValue::String("save|autoHideDuration=40".to_string());
    let source = match &value {
        UiValue::String(source) => source.as_str(),
        _ => unreachable!(),
    };

    let (toast_id, timeout_ms) =
        toast_timer_from_queue_value(&value).expect("valid toast queue string");

    assert_eq!(toast_id, "save");
    assert_eq!(timeout_ms, 40);
    assert!(
        std::ptr::eq(toast_id.as_ptr(), source.as_ptr()),
        "queue parsing should return a slice into the event value"
    );
}

/// 映射标识也必须指向原值内存，而不是仅比较内容相等。
#[test]
fn optimization_batch_20260909_runtime274_queue_map_parser_borrows_id_storage() {
    let toast_id = "export".to_string();
    let mut values = BTreeMap::new();
    values.insert("id".to_string(), UiValue::String(toast_id.clone()));
    values.insert("duration_ms".to_string(), UiValue::Int(80));
    let value = UiValue::Map(values);

    let (parsed_id, timeout_ms) =
        toast_timer_from_queue_value(&value).expect("valid toast queue map");

    let source = match &value {
        UiValue::Map(values) => match values.get("id") {
            Some(UiValue::String(source)) => source.as_str(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    assert_eq!(parsed_id, "export");
    assert_eq!(timeout_ms, 80);
    assert!(std::ptr::eq(parsed_id.as_ptr(), source.as_ptr()));
}

/// 拼接旧形式作为禁用探针，避免探针文本本身误命中被扫描源码。
#[test]
fn optimization_batch_20260909_runtime274_queue_parser_source_contract_is_borrowed() {
    let source = include_str!("../toast_timer_queue.rs");
    let legacy_id_clone = ["parts.next()?.trim()", "to_string()"].concat();
    let legacy_value_clone = ["Some(", "value.clone())"].concat();
    assert!(source.contains("Option<(&str, u64)>"));
    assert!(!source.contains(&legacy_id_clone));
    assert!(!source.contains(&legacy_value_clone));
}
