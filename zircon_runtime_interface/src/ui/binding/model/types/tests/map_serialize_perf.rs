//! 异构映射条目序列的 wire 等价和遍历微基准。
//!
//! 普通测试比较新旧序列化写法的 JSON 与二进制字节，未在这里解码或执行 Runtime。
//! 显式忽略的基准只比较条目视图收集与流式遍历的校验和，计时不包含真正编码调用。

use std::{hint::black_box, time::Instant};

use serde::{Serialize, Serializer};

use super::{UiBindingMap, UiBindingMapEntryRef, UiBindingMapKey};
use crate::ui::binding::model::UiBindingValue;

/// 借用映射的历史序列化包装；只在一次编码期间建立临时条目视图向量。
struct AllocatingMap<'a>(&'a UiBindingMap);

impl Serialize for AllocatingMap<'_> {
    /// 将借用条目视图暂存后交给同一序列化器；包装和源映射均不转移条目所有权。
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
             .0
            .iter()
            .map(|(key, value)| UiBindingMapEntryRef { key, value })
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

/// 构造固定键序、无重复的满标准条目数夹具；不代表任意映射树已校验。
fn sample_map() -> UiBindingMap {
    UiBindingMap::try_from_entries((0_u64..256).map(|index| {
        (
            UiBindingMapKey::Unsigned(index),
            UiBindingValue::Unsigned(index * 7),
        )
    }))
    .unwrap()
}

/// 建立临时借用条目向量再计算校验和的基线，源映射保持不变。
fn map_entries_allocating(map: &UiBindingMap) -> u64 {
    let entries = map
        .0
        .iter()
        .map(|(key, value)| UiBindingMapEntryRef { key, value })
        .collect::<Vec<_>>();
    entries
        .iter()
        .map(|entry| match (entry.key, entry.value) {
            (UiBindingMapKey::Unsigned(key), UiBindingValue::Unsigned(value)) => key ^ value,
            _ => 0,
        })
        .fold(0, u64::wrapping_add)
}

/// 直接借用遍历同一条目并计算校验和，不进行序列化。
fn map_entries_streaming(map: &UiBindingMap) -> u64 {
    map.0
        .iter()
        .map(|(key, value)| match (key, value) {
            (UiBindingMapKey::Unsigned(key), UiBindingValue::Unsigned(value)) => key ^ value,
            _ => 0,
        })
        .fold(0, u64::wrapping_add)
}

#[test]
/// 比较固定映射的新旧 JSON 与二进制编码字节，不声称通用往返或跨目标一致性已验证。
fn streamed_binding_map_preserves_json_and_binary_wire_bytes() {
    let map = sample_map();
    assert_eq!(
        serde_json::to_vec(&map).unwrap(),
        serde_json::to_vec(&AllocatingMap(&map)).unwrap(),
    );
    assert_eq!(
        crate::serialization::PositionalEncoding::FixedLittleEndian
            .encode(&map)
            .unwrap(),
        crate::serialization::PositionalEncoding::FixedLittleEndian
            .encode(&AllocatingMap(&map))
            .unwrap(),
    );
}

/// 显式忽略的条目遍历基准；原有分位比例断言保留，本批未执行。
#[test]
#[ignore = "release-only streamed binding map entry benchmark"]
fn runtime_interface03_batch38_streamed_binding_map_release_benchmark() {
    const ITERATIONS: usize = 50_000;
    const SAMPLE_COUNT: usize = 11;
    let map = sample_map();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streaming_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(map_entries_allocating(black_box(&map)));
            }
            started.elapsed().as_nanos()
        };
        let measure_streaming = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(map_entries_streaming(black_box(&map)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            streaming_samples.push(measure_streaming());
        } else {
            streaming_samples.push(measure_streaming());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    streaming_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STREAMED_BINDING_MAP_BENCH_V1 entries={} iterations={ITERATIONS} samples={SAMPLE_COUNT} allocating_p95_ns={} streaming_p95_ns={}",
        map.len(), allocating_samples[p95], streaming_samples[p95],
    );
    assert!(
        streaming_samples[p95].saturating_mul(2) <= allocating_samples[p95],
        "streamed binding map entry projection must improve P95 by at least 50%: allocating={}ns streaming={}ns",
        allocating_samples[p95],
        streaming_samples[p95],
    );
}
