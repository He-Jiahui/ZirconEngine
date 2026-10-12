//! 检查射线映射的空输入结果与容量预留表达式，并提供忽略的固定场景性能采样。
//!
//! 普通测试分别调用 [`RayMap::rebuild`] 和检查父模块源码文本；
//! 性能测试通过 [`measure`] 比较当前重建与本地 [`legacy_rebuild`] 基线。
//! 样本每轮新建映射，所有夹具和统计在本模块局部拥有；它不测同一映射跨帧的容量复用。
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use crate::core::math::Vec2;

use super::*;

/// 每个 [`measure`] 样本执行的映射创建、重建与销毁次数。
///
/// 当前为 256 次；返回的纳秒值覆盖整批循环，未换算为单次重建平均耗时。
const CHECKS_PER_SAMPLE: usize = 256;
/// 性能夹具中的位置数量，每个位置使用手选的互异指针键和同一视口。
///
/// 当前 256 个位置与活动源的潜在组合数由此值和 [`CAMERA_COUNT`] 相乘估计；
/// 该数量不是设备登记结果或实际散列表容量。
const POINTER_COUNT: usize = 256;
/// 性能夹具中的活动相机源数量，所有源共享配对的视口尺寸而拥有不同相机键。
///
/// 当前为 8，源在计时前创建并共享借给两种构建路径。
/// 与 [`POINTER_COUNT`] 的乘积只表示固定夹具的潜在组合数量。
const CAMERA_COUNT: usize = 8;
/// 基线与候选各自采集的批次样本数，同时也是两种测量配对的组数。
///
/// 当前 31 组交替测量顺序，其中基线先测 16 组、候选先测 15 组。
/// [`percentile`] 对各自完整样本序列选取最近秩，原始序列仍按采样顺序保存。
const SAMPLE_PAIRS: usize = 31;

/// 测量一批新映射的构建、结果消费与销毁，返回整个样本的耗时。
///
/// # 参数
///
/// - `pointers`：本次借用的固定位置夹具，应与相机源的视口及局部像素范围配对。
/// - `cameras`：本次借用的固定相机源夹具，两种测量路径使用相同输入。
/// - `optimized`：为 `true` 时每轮创建 `RayMap` 并调用 [`RayMap::rebuild`]，
///   为 `false` 时调用本地 [`legacy_rebuild`] 创建未预留的新表。
///
/// # 返回值
///
/// 返回 [`CHECKS_PER_SAMPLE`] 次循环的总纳秒数，最小记为 `1`，不除以循环次数。
/// 计时包含新映射创建、构建、条目计数、黑盒消费及映射销毁；
/// 输入夹具准备与后续统计、格式化位于这段计时之外。
///
/// # 调用时机与生命周期
///
/// 由固定场景性能测试逐样本调用。每轮重新创建并销毁映射，不沿样本保留表容量。
/// 累积条目数只交给黑盒消费，调用方须使计数可用 `usize` 表示；
/// 这里没有输出等价断言或容量读数。返回计时值独立于输入借用。
///
/// # 参考
///
/// 本地未预留基线见 [`legacy_rebuild`]，生产重建见 [`RayMap::rebuild`]。
fn measure(pointers: &[PointerLocation], cameras: &[CameraRaySource], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut count = 0;
    // 每轮新建表并在黑盒消费后销毁，销毁也计入当前样本；这里不沿循环保留表容量。
    // 累积len只用于消费结果，不是两种实现输出等价的断言。
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            let mut map = RayMap::default();
            map.rebuild(black_box(pointers), black_box(cameras));
            count += map.len();
            black_box(map);
        } else {
            let map = legacy_rebuild(black_box(pointers), black_box(cameras));
            count += map.len();
            black_box(map);
        }
    }
    black_box(count);
    started.elapsed().as_nanos().max(1)
}

/// 为性能采样创建未提前预留容量的本地射线映射基线。
///
/// # 参数
///
/// - `pointers`：借用的位置切片，位置与视口配对责任和生产重建相同。
/// - `cameras`：借用的相机源切片，按源的 `active` 字段决定参与。
///
/// # 返回值
///
/// 返回新建且拥有完整射线键与数值的 `HashMap`。
/// 活动、视口与位置准入及 [`ray_from_viewport_point`] 检查和 [`RayMap::rebuild`] 相同；
/// 本地基线不执行容量预留。同键按相机在外、位置在内的顺序保留最后一次成功写入，
/// 最终散列表遍历顺序未规定。
///
/// # 调用时机与生命周期
///
/// 仅供 [`measure`] 的基线分支调用。函数不修改输入或保留其借用，
/// 返回表由测量函数在计时循环内消费并销毁。
///
/// # 参考
///
/// 计时范围见 [`measure`]；当前生产算法见 [`RayMap::rebuild`]。
fn legacy_rebuild(
    pointers: &[PointerLocation],
    cameras: &[CameraRaySource],
) -> HashMap<RayId, PointerRay> {
    let mut map = HashMap::new();
    for camera in cameras {
        if !camera.active {
            continue;
        }
        for pointer in pointers {
            if pointer.viewport != camera.viewport
                || !pointer.is_inside_viewport(camera.viewport_size)
            {
                continue;
            }
            if let Some(ray) =
                ray_from_viewport_point(&camera.snapshot, camera.viewport_size, pointer.position)
            {
                map.insert(
                    RayId::new(camera.camera, pointer.pointer, camera.viewport),
                    ray,
                );
            }
        }
    }
    map
}

/// 在样本副本排序后，以整数最近秩选取一个已观测的耗时值。
///
/// # 调用时机与生命周期
///
/// 在全部样本采集后计算统计。排序副本局部拥有并随后释放，返回 `u128` 独立于输入。
///
/// # 参数
///
/// - `samples`：借用的非空耗时序列；原序列保持采样顺序，不被排序或改写。
/// - `percentile`：调用方提供的百分位，应在 `1..=100`，且样本数与它的乘积须能用 `usize` 表示。
///
/// # 返回值与约束
///
/// 返回排序后第 `ceil(len * percentile / 100)` 个样本，不做插值。
/// 当前 [`SAMPLE_PAIRS`] 为 31，调用的 p50 与 p95 分别取第 16 和第 30 个值。
/// 函数未验证上述前提：空输入或越界秩可引发索引 panic，乘积也可能溢出；
/// 下标的饱和减法只处理减一，不把任意百分位截断为有效范围。
///
/// # 参考
///
/// 原始采样时序的输出见 [`sample_csv`]。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

/// 将耗时样本按输入顺序连接成逗号分隔的十进制字符串。
///
/// # 参数
///
/// - `samples`：借用的 `u128` 序列；性能测试传入 [`measure`] 的整批纳秒值。
///
/// # 返回值
///
/// 返回拥有的字符串，不排序、不换算单位，也不添加标题或尾逗号；空序列返回空字符串。
///
/// # 调用时机与生命周期
///
/// 在性能测试打印原始样本时使用。函数不改输入或保留其借用，
/// 临时数字字符串在拼接后释放，最终字符串由调用方拥有。
///
/// # 参考
///
/// 在副本上排序并选取统计值的接口见 [`percentile`]。
fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
/// 验证新映射在没有位置和相机源的场景下，重建后仍没有条目。
///
/// # 调用时机与参数
///
/// 无参数；由测试框架在本模块的测试条件满足后调用，夹具在函数内局部构造。
///
/// # 场景与断言
///
/// 创建空的两个输入向量和默认空映射，调用 [`RayMap::rebuild`] 后断言 [`RayMap::is_empty`]。
/// 本场景没有非空结果比较、旧条目或实际容量读取，断言仅覆盖空输入结果。
///
/// # 返回值与生命周期
///
/// 普通测试正常结束时返回 `()`；断言失败会 panic。
/// 所有夹具与映射局部拥有，测试结束即释放。
///
/// # 参考
///
/// 空场景调用入口见 [`RayMap::rebuild`]。
fn optimization_batch_20260829bv_runtime349_ray_map_capacity_preserves_results() {
    let pointers = Vec::<PointerLocation>::new();
    let cameras = Vec::<CameraRaySource>::new();
    let mut map = RayMap::default();
    map.rebuild(&pointers, &cameras);
    assert!(map.is_empty());
}

#[test]
/// 检查生产重建源码文本保留活动源计数与饱和组合容量预留表达式。
///
/// # 调用时机与参数
///
/// 无参数；由测试框架在本模块的测试条件满足后调用，夹具在函数内局部构造。
///
/// # 场景与断言
///
/// 编译期嵌入父模块源码，按第一次测试条件分界取得生产文本前缀，
/// 检查 [`RayMap::rebuild`] 使用的活动源计数和容量预留两个字串存在。
/// 这是源码表达式守卫，不读取实际桶容量，也不比较射线结果或测量性能。
///
/// # 返回值与生命周期
///
/// 正常结束返回 `()`；缺少文本分界或断言不成立会 panic。
/// 输入是测试中嵌入的静态文本，没有运行时文件句柄或相机资源需要保留。
///
/// # 参考
///
/// 受守卫的容量表达式属于 [`RayMap::rebuild`]。
fn optimization_batch_20260829bv_runtime349_ray_map_reserves_active_pair_capacity() {
    let source = include_str!("../../ray_map.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains(
        "let active_camera_count = cameras.iter().filter(|camera| camera.active).count()"
    ));
    assert!(production.contains("reserve(pointers.len().saturating_mul(active_camera_count))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
/// 在固定单视口多指针、多相机场景采样新映射构建的批次耗时，并检查 p95 阈值。
///
/// # 调用时机与参数
///
/// 无参数；由测试框架调用，夹具在函数内局部构造。
/// 此测试带有忽略标记，只有被显式选中时才采样，供受控 Windows release 场景使用。
///
/// # 场景与测量
///
/// 计时前准备 256 个位置：手选指针键 `0..255`、视口值 `1`、局部像素点 `(index, 2)`；
/// 准备 8 个活动源：手选相机键 `1..8`、相同视口值、`1024 × 768` 尺寸和独立默认快照。
/// 两种实现各采集 31 个 [`measure`] 样本，每样本包含 256 次新建映射的完整循环。
/// 偶数组先测基线、奇数组先测候选，因此分别有 16 与 15 组先测；没有随机化或显式预热。
///
/// # 输出与断言
///
/// 通过 [`percentile`] 得到整批纳秒样本的 p50/p95，并用 [`sample_csv`] 打印采样顺序的原始值。
/// 输出中的候选预留容量为输入数量的乘积 `2048`，没有读取 `HashMap::capacity`；
/// 条目计数只被黑盒消费，未断言结果等价或实际条目数。
/// 阈值按 `candidate_p95.saturating_mul(100) <= baseline_p95.saturating_mul(70)` 比较；
/// 在乘法未饱和的计时范围内，要求候选 p95 不高于基线的 70%。
///
/// # 返回值与生命周期
///
/// 正常结束返回 `()`；p95 阈值断言失败会 panic。夹具和样本向量局部拥有，函数结束时释放。
/// 每轮映射销毁包含在计时内。测量针对新容器路径，
/// 不能据此判定 [`RayMap::rebuild`] 在长期持有的同一实例上复用容量的性能。
///
/// # 参考
///
/// 整批计时见 [`measure`]，统计秩见 [`percentile`]，原序列格式见 [`sample_csv`]。
fn optimization_batch_20260829bv_runtime349_ray_map_rebuild_capacity_bench() {
    let pointers = (0..POINTER_COUNT)
        .map(|index| PointerLocation {
            pointer: PointerId::new(index as u64),
            viewport: RenderViewportHandle::new(1),
            position: Vec2::new(index as f32, 2.0),
        })
        .collect::<Vec<_>>();
    let cameras = (0..CAMERA_COUNT)
        .map(|index| {
            CameraRaySource::new(
                index as EntityId + 1,
                RenderViewportHandle::new(1),
                UVec2::new(1024, 768),
                ViewportCameraSnapshot::default(),
            )
        })
        .collect::<Vec<_>>();
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    // 交替谁先测，当前31组中基线先16组、候选先15组；输入相同但顺序并不随机。
    // 每次测量都重新创建映射，样本之间只保存时间值。
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&pointers, &cameras, false));
            candidate.push(measure(&pointers, &cameras, true));
        } else {
            candidate.push(measure(&pointers, &cameras, true));
            baseline.push(measure(&pointers, &cameras, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    // candidate_reserved_capacity报告的是固定输入的潜在组合数，没有读取实际桶容量；
    // 原始纳秒序列仍是每样本整批循环的时间，p50/p95也采用这个单位。
    println!(
        "RUNTIME349_RAY_MAP_REBUILD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} pointer_count={POINTER_COUNT} camera_count={CAMERA_COUNT} baseline_initial_capacity=0 candidate_reserved_capacity={} baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        POINTER_COUNT * CAMERA_COUNT,
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
