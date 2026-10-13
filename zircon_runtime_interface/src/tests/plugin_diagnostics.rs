//! 诊断字段/消息回归及显式忽略的 release 性能对照；不作为注册权限或实际插件加载验收。
use std::{hint::black_box, time::Instant};

use super::{RegistrationDiagnostic, RegistrationDiagnosticSeverity};

/// 性能夹具的字符串构造预算；前缀长度由现有表达式决定，不代表运行期输入上限。
const PLUGIN_ID_BYTES: usize = 16_384;
/// 固定 21 对测量，使 p95 的非空样本前提由调用方建立。
const SAMPLE_PAIRS: usize = 21;

/// 性能对照实现：克隆一次拥有的插件 ID，并用 format 构造同样的诊断内容。
fn legacy_missing_capability(plugin_id: String, capability: String) -> RegistrationDiagnostic {
    RegistrationDiagnostic::new(
        RegistrationDiagnosticSeverity::Error,
        "editor.capability.missing",
        plugin_id.clone(),
        format!("editor plugin `{plugin_id}` requires missing capability `{capability}`"),
    )
}

/// 对非空耗时样本排序并按向上取整索引读取 p95；唯一调用方传入完整的 21 项。
fn p95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

/// 执行一次 FnOnce 并返回纳秒耗时和原始结果；不把调用前准备输入的克隆计入耗时。
fn measure<T>(run: impl FnOnce() -> T) -> (u128, T) {
    let start = Instant::now();
    let output = run();
    (start.elapsed().as_nanos(), output)
}

#[test]
/// 核对 Error、固定代码、插件身份和精确消息；不测量时延或注册表状态。
fn missing_capability_preserves_fields_and_message() {
    let diagnostic = RegistrationDiagnostic::missing_capability("sample.plugin", "asset.import");

    assert_eq!(diagnostic.severity, RegistrationDiagnosticSeverity::Error);
    assert_eq!(diagnostic.code, "editor.capability.missing");
    assert_eq!(diagnostic.plugin_id, "sample.plugin");
    assert_eq!(
        diagnostic.message,
        "editor plugin `sample.plugin` requires missing capability `asset.import`"
    );
}

#[test]
#[ignore = "release performance evidence"]
/// ignored 的性能对照：先预热，再交替两种测量顺序，报告各自 p95 并检查既有阈值。
/// 普通 test 默认跳过此用例；该阈值是计时观察，不单独证明分配数或所有输入的收益。
fn missing_capability_moves_owned_plugin_id_without_clone() {
    let plugin_id = "plugin.".to_owned() + &"x".repeat(PLUGIN_ID_BYTES - 7);
    let capability = "asset.import".to_owned();

    // 预热也构造两边的拥有输入；不把这些样本加入后面的计时分位数。
    for _ in 0..5 {
        black_box(legacy_missing_capability(
            plugin_id.clone(),
            capability.clone(),
        ));
        black_box(RegistrationDiagnostic::missing_capability(
            plugin_id.clone(),
            capability.clone(),
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        // 四份输入都在计时前准备，比较范围只含诊断构造；交替顺序减轻固定先后偏差。
        let legacy_plugin_id = plugin_id.clone();
        let legacy_capability = capability.clone();
        let optimized_plugin_id = plugin_id.clone();
        let optimized_capability = capability.clone();
        if sample % 2 == 0 {
            let (elapsed, diagnostic) =
                measure(|| legacy_missing_capability(legacy_plugin_id, legacy_capability));
            black_box(diagnostic);
            legacy_samples.push(elapsed);
            let (elapsed, diagnostic) = measure(|| {
                RegistrationDiagnostic::missing_capability(
                    optimized_plugin_id,
                    optimized_capability,
                )
            });
            black_box(diagnostic);
            optimized_samples.push(elapsed);
        } else {
            let (elapsed, diagnostic) = measure(|| {
                RegistrationDiagnostic::missing_capability(
                    optimized_plugin_id,
                    optimized_capability,
                )
            });
            black_box(diagnostic);
            optimized_samples.push(elapsed);
            let (elapsed, diagnostic) =
                measure(|| legacy_missing_capability(legacy_plugin_id, legacy_capability));
            black_box(diagnostic);
            legacy_samples.push(elapsed);
        }
    }

    // 两个分支各收集一个样本，21 轮后 p95 才满足非空前提；输出保留既有性能证据格式。
    let legacy_p95_ns = p95(legacy_samples);
    let optimized_p95_ns = p95(optimized_samples);
    println!(
        "PERF_RESULT runtime_interface04_registration_diagnostic \
             plugin_id_bytes={PLUGIN_ID_BYTES} sample_pairs={SAMPLE_PAIRS} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_plugin_id_clones=1 optimized_plugin_id_clones=0"
    );
    assert!(
        optimized_p95_ns * 100 <= legacy_p95_ns * 80,
        "clone-free diagnostic construction must be <=80% of legacy P95: \
             optimized={optimized_p95_ns}ns legacy={legacy_p95_ns}ns"
    );
}
