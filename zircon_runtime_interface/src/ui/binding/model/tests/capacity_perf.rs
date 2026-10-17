//! 事件绑定容量提示的文本等价测试与显式忽略的微基准。
//!
//! 普通测试覆盖带参数、无参数和无动作三种状态；相等断言不统计实际分配次数。
//! 微基准仅测量此处生成的无参数夹具，不代表任意参数规模的性能结论。

use std::{hint::black_box, time::Instant};

use super::UiEventBinding;
use crate::ui::binding::model::{UiBindingCall, UiBindingValue, UiEventKind, UiEventPath};

/// 按开关构造长标识夹具；参数分支另加入字符串与整数值。
fn sample_binding(with_argument: bool) -> UiEventBinding {
    let mut action = UiBindingCall::new(format!("Apply{}", "Action".repeat(24)));
    if with_argument {
        action = action.with_argument(UiBindingValue::Array(vec![
            UiBindingValue::String("value".repeat(32)),
            UiBindingValue::Unsigned(42),
        ]));
    }
    UiEventBinding::new(
        UiEventPath::new(
            format!("Editor.{}", "Workspace".repeat(24)),
            format!("Panel.{}", "Selection".repeat(24)),
            UiEventKind::Change,
        ),
        action,
    )
}

/// 不作容量预估的历史写入方式，供文本结果与局部耗时比较。
fn native_binding_unpresized(binding: &UiEventBinding) -> String {
    let mut output = String::new();
    binding.path.native_prefix_into(&mut output);
    if let Some(action) = &binding.action {
        output.push('(');
        action.native_repr_into(&mut output);
        output.push(')');
    }
    output
}

/// 比较三种载荷状态的文本输出；没有对分配器或 Runtime 执行作断言。
#[test]
fn runtime_interface03_batch44_46_presized_event_binding_preserves_unpresized_output() {
    for binding in [
        sample_binding(false),
        sample_binding(true),
        UiEventBinding::without_action(UiEventPath::new("Editor", "Canvas", UiEventKind::Click)),
    ] {
        assert_eq!(
            binding.native_binding(),
            native_binding_unpresized(&binding)
        );
    }
}

/// 显式忽略的无参数夹具微基准；交替测量顺序并比较采样分位值。
#[test]
#[ignore = "release-only presized event binding benchmark"]
fn runtime_interface03_batch44_46_presized_event_binding_release_benchmark() {
    const ITERATIONS: usize = 500_000;
    const SAMPLE_COUNT: usize = 11;
    let binding = sample_binding(false);
    let mut unpresized_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut presized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_unpresized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(native_binding_unpresized(black_box(&binding)));
            }
            started.elapsed().as_nanos()
        };
        let measure_presized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(binding.native_binding());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unpresized_samples.push(measure_unpresized());
            presized_samples.push(measure_presized());
        } else {
            presized_samples.push(measure_presized());
            unpresized_samples.push(measure_unpresized());
        }
    }

    unpresized_samples.sort_unstable();
    presized_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_PRESIZED_EVENT_BINDING_BENCH_V1 bytes={} iterations={ITERATIONS} samples={SAMPLE_COUNT} unpresized_p95_ns={} presized_p95_ns={}",
        binding.native_binding().len(), unpresized_samples[p95], presized_samples[p95],
    );
    assert!(
        presized_samples[p95].saturating_mul(5) <= unpresized_samples[p95].saturating_mul(4),
        "presized event binding output must improve P95 by at least 20%: unpresized={}ns presized={}ns",
        unpresized_samples[p95],
        presized_samples[p95],
    );
}
