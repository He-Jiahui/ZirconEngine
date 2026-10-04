//! 为消息反序列化使用的稳定标识查询保存发布模式性能证据，先核对查询语义，再比较延迟与分配。

use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Instant,
};

use zircon_hub::state::HubMessageId;

// 固定工作负载、成对样本和验收阈值属于受管理的发布性能契约。
const LOOKUPS: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;
const THRESHOLD_PERCENT: u128 = 60;

/// 性能目标专用的进程全局分配观察器；计数只描述请求量，内存所有权和释放职责仍由系统分配器承担。
struct CountingAllocator;

// 计数窗口由性能测试串行开启；这些静态量对进程内所有线程可见。
static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static COUNTING_ALLOCATOR: CountingAllocator = CountingAllocator;

// SAFETY: 计数路径仅使用原子操作，所有内存请求以未改变的指针、布局和大小交给同一个 System 分配器。
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        // SAFETY: 调用方传入的有效 Layout 原样传递给 System；返回值直接来自该分配器。
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation(layout.size());
        // SAFETY: 调用方传入的有效 Layout 原样传递给 System；返回值直接来自该分配器。
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: 属于本分配器的原指针和原 Layout 未经转换，仍由 System 释放。
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation(size);
        // SAFETY: 原指针、原 Layout 和新大小未改变，重分配仍由同一个 System 完成。
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[derive(Clone, Copy)]
/// 保存同一工作负载的延迟、请求分配量与语义校验和，使性能收益与查询结果可分别审查。
struct Measurement {
    elapsed_ns: u128,
    allocations: usize,
    allocated_bytes: usize,
    checksum: usize,
}

/// 由受管理的发布性能批次显式启用；交替测量顺序降低热身偏差，校验结果与分配后才判定分位延迟收益。
#[test]
#[ignore = "managed release performance contract"]
fn hub04_message_id_lookup_release_benchmark_evidence() {
    let ids = HubMessageId::all()
        .into_iter()
        .map(HubMessageId::as_str)
        .collect::<Vec<_>>();
    for id in &ids {
        assert_eq!(legacy_from_str_id(id), HubMessageId::from_str_id(id));
    }
    for id in ["unknown.id", "shell", ".missing"] {
        assert_eq!(legacy_from_str_id(id), HubMessageId::from_str_id(id));
    }

    for _ in 0..4 {
        black_box(measure(&ids, legacy_from_str_id));
        black_box(measure(&ids, HubMessageId::from_str_id));
    }

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&ids, legacy_from_str_id));
            optimized.push(measure(&ids, HubMessageId::from_str_id));
        } else {
            optimized.push(measure(&ids, HubMessageId::from_str_id));
            legacy.push(measure(&ids, legacy_from_str_id));
        }
    }

    let checksum = legacy[0].checksum;
    assert!(legacy.iter().all(|sample| sample.checksum == checksum));
    assert!(optimized.iter().all(|sample| sample.checksum == checksum));

    let legacy_allocations = legacy[0].allocations;
    let optimized_allocations = optimized[0].allocations;
    assert!(legacy_allocations >= LOOKUPS);
    assert!(legacy
        .iter()
        .all(|sample| sample.allocations == legacy_allocations));
    assert_eq!(optimized_allocations, 0);
    assert!(optimized.iter().all(|sample| sample.allocations == 0));
    assert!(optimized.iter().all(|sample| sample.allocated_bytes == 0));

    let legacy_ns = elapsed_samples(&legacy);
    let optimized_ns = elapsed_samples(&optimized);
    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    let p50_reduction_percent = reduction_percent(legacy_p50_ns, optimized_p50_ns);
    let p95_reduction_percent = reduction_percent(legacy_p95_ns, optimized_p95_ns);

    println!(
        "PERF_RESULT hub04_message_id_lookup lookups=8192 sample_pairs=21 \
         threshold_percent=60 checksum={checksum} \
         legacy_allocations={legacy_allocations} optimized_allocations={optimized_allocations} \
         legacy_allocated_bytes={} optimized_allocated_bytes={} allocation_reduction_percent=100 \
         legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
         p50_reduction_percent={p50_reduction_percent} \
         legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
         p95_reduction_percent={p95_reduction_percent} \
         legacy_raw_ns={} optimized_raw_ns={}",
        legacy[0].allocated_bytes,
        optimized[0].allocated_bytes,
        raw_samples(&legacy_ns),
        raw_samples(&optimized_ns),
    );

    assert!(
        p50_reduction_percent >= THRESHOLD_PERCENT,
        "optimized lookup must improve P50 by at least {THRESHOLD_PERCENT}%: \
         legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns"
    );
    assert!(
        p95_reduction_percent >= THRESHOLD_PERCENT,
        "optimized lookup must improve P95 by at least {THRESHOLD_PERCENT}%: \
         legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

/// 保留优化前遍历全部标识的语义与分配基线，只用于性能对照，不能作为生产查询入口。
fn legacy_from_str_id(id: &str) -> Option<HubMessageId> {
    HubMessageId::all()
        .into_iter()
        .find(|candidate| candidate.as_str() == id)
}

// TODO: [CR-HUBTESTA-0015] 确认进程全局分配计数在测量窗口内不会包含其他线程请求；缺少并行隔离证据；下一步在受管理性能批次核对。
/// 在非空标识集合上测量同一查询工作负载；输入构造、结果输出和统计整理留在计时窗口之外。
fn measure(ids: &[&str], lookup: fn(&str) -> Option<HubMessageId>) -> Measurement {
    reset_allocation_counters();
    TRACK_ALLOCATIONS.store(true, Ordering::Relaxed);
    let started = Instant::now();
    let mut checksum = 0usize;
    for index in 0..LOOKUPS {
        let id = ids[index % ids.len()];
        let result = lookup(black_box(id));
        checksum = checksum.wrapping_add(match black_box(result) {
            Some(message_id) => message_id.as_str().len().wrapping_add(index),
            None => index,
        });
    }
    let elapsed_ns = started.elapsed().as_nanos().max(1);
    TRACK_ALLOCATIONS.store(false, Ordering::Relaxed);

    Measurement {
        elapsed_ns,
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        checksum,
    }
}

/// 全局分配器仅在计量窗口登记请求，不在钩子中分配内存，以免统计自身递归进入分配器。
fn record_allocation(size: usize) {
    if TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size, Ordering::Relaxed);
    }
}

/// 只在关闭计量窗口后准备下一样本，不能与其他测量窗口并发使用这些共享计数。
fn reset_allocation_counters() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
}

/// 将已完成的样本投影为分位统计输入，分配计数和校验和保留在原样本供独立核对。
fn elapsed_samples(measurements: &[Measurement]) -> Vec<u128> {
    measurements
        .iter()
        .map(|measurement| measurement.elapsed_ns)
        .collect()
}

/// 供固定非空样本集计算最近秩分位值；调用方使用有效百分位，不能把此助手当通用统计 API。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

/// 把对照延迟转换成非负收益百分比，退化时由上层阈值断言报告失败。
fn reduction_percent(legacy: u128, optimized: u128) -> u128 {
    legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
}

/// 按测量顺序保存机器可读原始延迟证据，便于后续复核噪声而不只接受汇总分位值。
fn raw_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
