//! 原生文本追加实现与历史分配型写法的直接对照测试。
//!
//! 普通测试比较固定嵌套值和动作的文本输出，不断言重新解析后的类别无损往返。
//! 显式忽略的微基准比较相同夹具的文本生成；源码中的阈值不是本批已执行的性能结果。

use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::super::UiBindingValue;
use crate::ui::binding::model::{
    UiBindingAssetReference, UiBindingCall, UiBindingEntityReference, UiBindingMapKey,
};

/// 构造覆盖多种拥有载荷的固定值树；由当前标准预算构造入口确认夹具可用。
fn sample_value() -> UiBindingValue {
    let mut fields = BTreeMap::new();
    fields.insert(
        "message".to_string(),
        UiBindingValue::String("quoted\\value\nwith\"escapes".to_string()),
    );
    fields.insert(
        "items".to_string(),
        UiBindingValue::Array(vec![
            UiBindingValue::Unsigned(17),
            UiBindingValue::Signed(-9),
            UiBindingValue::Float(2.5),
            UiBindingValue::Bool(true),
            UiBindingValue::Null,
        ]),
    );
    let map = UiBindingValue::map([
        (
            UiBindingMapKey::String("left".to_string()),
            UiBindingValue::String("L".to_string()),
        ),
        (UiBindingMapKey::Unsigned(3), UiBindingValue::Signed(4)),
    ])
    .unwrap();
    fields.insert("map".to_string(), map);
    fields.insert(
        "enum".to_string(),
        UiBindingValue::Enum(
            crate::ui::binding::model::UiBindingEnumValue::try_new(
                "demo.type",
                "Ready",
                Some(UiBindingValue::String("payload".to_string())),
            )
            .unwrap(),
        ),
    );
    fields.insert(
        "asset".to_string(),
        UiBindingValue::Asset(UiBindingAssetReference::try_new("ui/button").unwrap()),
    );
    fields.insert(
        "entity".to_string(),
        UiBindingValue::Entity(UiBindingEntityReference::try_new(8, 2).unwrap()),
    );
    UiBindingValue::record(fields).unwrap()
}

/// 历史多次替换的转义对照，仅用于生成比较基线，不作为产品入口。
fn quote_allocating(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

/// 按各变体递归分配中间文本的历史基线；保持旧比较逻辑和错误前提不变。
fn native_repr_allocating(value: &UiBindingValue) -> String {
    match value {
        UiBindingValue::String(value) => quote_allocating(value),
        UiBindingValue::Unsigned(value) => value.to_string(),
        UiBindingValue::Signed(value) => value.to_string(),
        UiBindingValue::Float(value) => {
            let mut rendered = value.to_string();
            if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                rendered.push_str(".0");
            }
            rendered
        }
        UiBindingValue::Bool(value) => value.to_string(),
        UiBindingValue::Null => "null".to_string(),
        UiBindingValue::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(native_repr_allocating)
                .collect::<Vec<_>>()
                .join(",")
        ),
        UiBindingValue::Record(fields) => format!(
            "record({})",
            fields
                .iter()
                .flat_map(|(field, value)| [quote_allocating(field), native_repr_allocating(value)])
                .collect::<Vec<_>>()
                .join(",")
        ),
        UiBindingValue::Map(values) => format!(
            "map({})",
            values
                .iter()
                .flat_map(|(key, value)| [map_key_allocating(key), native_repr_allocating(value)])
                .collect::<Vec<_>>()
                .join(",")
        ),
        UiBindingValue::Enum(value) => {
            let mut arguments = vec![
                quote_allocating(value.type_id()),
                quote_allocating(value.variant()),
            ];
            if let Some(payload) = value.payload() {
                arguments.push(native_repr_allocating(payload));
            }
            format!("enum({})", arguments.join(","))
        }
        UiBindingValue::Asset(value) => format!("asset({})", quote_allocating(value.locator())),
        UiBindingValue::Entity(value) => {
            format!("entity({},{})", value.entity_id(), value.generation())
        }
        UiBindingValue::Optional(value) => format!(
            "optional({})",
            value
                .as_deref()
                .map(native_repr_allocating)
                .unwrap_or_default()
        ),
        UiBindingValue::CollectionView(value) => format!(
            "collection_view({},{},{},{},{},{},{},{})",
            quote_allocating(value.provider().id.as_str()),
            value.provider().version.get(),
            quote_allocating(value.item_schema().id.as_str()),
            value.item_schema().version.get(),
            value.revision(),
            value.offset(),
            value.length(),
            value.total_length(),
        ),
    }
}

/// 历史键文本基线；同样不在文本中另存整数类别。
fn map_key_allocating(key: &UiBindingMapKey) -> String {
    match key {
        UiBindingMapKey::String(value) => quote_allocating(value),
        UiBindingMapKey::Unsigned(value) => value.to_string(),
        UiBindingMapKey::Signed(value) => value.to_string(),
        UiBindingMapKey::Bool(value) => value.to_string(),
    }
}

/// 历史动作文本基线，借用载荷但为各段分配结果。
fn call_allocating(call: &UiBindingCall) -> String {
    format!(
        "{}({})",
        call.symbol,
        call.arguments
            .iter()
            .map(native_repr_allocating)
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[test]
/// 比较固定值和单参数动作的两种文本输出；不执行处理器或资产解析。
fn single_buffer_native_projection_preserves_nested_output() {
    let value = sample_value();
    assert_eq!(value.native_repr(), native_repr_allocating(&value));

    let call = UiBindingCall::new("activate").with_argument(value);
    assert_eq!(call.native_repr(), call_allocating(&call));
}

/// 仅显式选择时运行的文本生成微基准；固定样本排序后检查原比例断言。
#[test]
#[ignore = "release-only single-buffer native binding projection benchmark"]
fn runtime_interface03_batch37_single_buffer_native_binding_release_benchmark() {
    const ITERATIONS: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let value = sample_value();
    let call = UiBindingCall::new("activate").with_argument(value);
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut single_buffer_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(call_allocating(black_box(&call)));
            }
            started.elapsed().as_nanos()
        };
        let measure_single_buffer = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(call.native_repr());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            single_buffer_samples.push(measure_single_buffer());
        } else {
            single_buffer_samples.push(measure_single_buffer());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    single_buffer_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_BUFFER_NATIVE_BINDING_BENCH_V1 iterations={ITERATIONS} samples={SAMPLE_COUNT} allocating_p95_ns={} single_buffer_p95_ns={}",
        allocating_samples[p95], single_buffer_samples[p95],
    );
    assert!(
        single_buffer_samples[p95].saturating_mul(2) <= allocating_samples[p95],
        "single-buffer native binding projection must improve P95 by at least 50%: allocating={}ns single_buffer={}ns",
        allocating_samples[p95],
        single_buffer_samples[p95],
    );
}
