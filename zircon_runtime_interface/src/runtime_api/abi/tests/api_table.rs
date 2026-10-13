//! 将生成的 InterfaceSpec 槽位清单与两个具体 ABI 表的源码字段交叉核对。
//!
//! 分区列表先按顺序比较，完整槽位再按集合比较；这只检查声明清单，不调用函数指针。
//! 扫描使用源码文本，表字段必须继续逐行声明，注释也不能伪装成字段行。

use std::collections::BTreeSet;

use super::{
    ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES, ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES,
    ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES,
};

#[test]
/// 生成分区与手写清单一致，且除两个表头外没有遗漏或额外函数槽。
fn interface_spec_slot_partitions_match_the_abi_table_fields() {
    assert_eq!(
        crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES,
        ZR_RUNTIME_API_V8_REQUIRED_FIELD_NAMES,
    );
    assert_eq!(
        crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES,
        ZR_RUNTIME_API_V8_OPTIONAL_FIELD_NAMES,
    );
    assert_eq!(
        crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES,
        ZR_HOST_API_V1_OPTIONAL_FIELD_NAMES,
    );

    // 表头属于必需字段清单，却不是函数槽；合并分区后仅排除这两个名称。
    let expected_runtime_slots = crate::runtime_build_set::ZR_RUNTIME_API_V8_REQUIRED_SLOT_NAMES
        .iter()
        .chain(crate::runtime_build_set::ZR_RUNTIME_API_V8_OPTIONAL_SLOT_NAMES.iter())
        .filter(|field| **field != "abi_version" && **field != "size_bytes")
        .map(|field| (*field).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        concrete_table_slot_names("ZrRuntimeApiV8"),
        expected_runtime_slots,
    );

    let expected_host_slots = crate::runtime_build_set::ZR_HOST_API_V1_OPTIONAL_SLOT_NAMES
        .iter()
        .map(|field| (*field).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        concrete_table_slot_names("ZrHostApiV1"),
        expected_host_slots,
    );
}

// 扫描器依赖具体 ABI 表逐行声明 pub 字段，并以独占一行的结束括号确定字段区间。
// 先确认两个表头的位置再排除它们，其余字段名才参与函数槽集合核对。
/// 从指定表中返回除版本和尺寸外的字段名集合。
///
/// `table_name` 必须在相邻 ABI 源文件中有具体声明；缺失声明、表头或字段类型时断言失败。
/// 此文本扫描器不解析 Rust 语法，也不验证字段的函数签名或指针可调用性。
fn concrete_table_slot_names(table_name: &str) -> BTreeSet<String> {
    let source = include_str!("../api_table.rs");
    let struct_needle = format!("pub struct {table_name} ");
    let struct_index = source
        .find(&struct_needle)
        .unwrap_or_else(|| panic!("{table_name} must remain a concrete ABI table declaration"));
    let body_start = source[struct_index..]
        .find('{')
        .map(|offset| struct_index + offset + 1)
        .expect("concrete ABI table must have a field body");
    let fields = source[body_start..]
        .lines()
        .map(str::trim)
        .take_while(|line| *line != "}")
        .filter_map(|line| line.strip_prefix("pub "))
        .map(|field| {
            field
                .split_once(':')
                .map(|(field, _)| field.trim().to_owned())
                .expect("concrete ABI table field must have a type")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        fields.first().map(String::as_str),
        Some("abi_version"),
        "{table_name} must begin with the ABI version header"
    );
    assert_eq!(
        fields.get(1).map(String::as_str),
        Some("size_bytes"),
        "{table_name} must retain the byte-size header after its ABI version"
    );
    fields.into_iter().skip(2).collect()
}
