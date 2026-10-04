//! 对照每次新建遍历栈的结果，覆盖有效引用、越界引用、非有限值和超深树，并确认复用栈每次调用后为空。
use super::*;

fn expression_program() -> UiCompiledBindingProgram {
    UiCompiledBindingProgram::new(
        UiCompiledBindingGeneration::new(1),
        Vec::new(),
        vec!["property_0000".to_string()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn benchmark_expression() -> UiCompiledBindingExpression {
    UiCompiledBindingExpression::And(
        Box::new(UiCompiledBindingExpression::Property(UiPropertyId::new(0))),
        Box::new(UiCompiledBindingExpression::Not(Box::new(
            UiCompiledBindingExpression::Literal(UiValue::Bool(true)),
        ))),
    )
}

#[test]
fn reused_expression_stack_preserves_allocating_results_and_clears_failures() {
    let program = expression_program();
    let mut excessive_depth = UiCompiledBindingExpression::Literal(UiValue::Bool(true));
    for _ in 0..UI_BINDING_EXPRESSION_MAX_DEPTH {
        excessive_depth = UiCompiledBindingExpression::Not(Box::new(excessive_depth));
    }
    let expressions = [
        benchmark_expression(),
        UiCompiledBindingExpression::Property(UiPropertyId::new(1)),
        UiCompiledBindingExpression::Literal(UiValue::Float(f64::NAN)),
        excessive_depth,
        UiCompiledBindingExpression::Literal(UiValue::Bool(false)),
    ];
    let mut pending = Vec::with_capacity(UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY);

    for expression in &expressions {
        assert_eq!(
            program.expression_is_well_formed(expression, &mut pending),
            program.expression_is_well_formed_allocating(expression),
        );
        assert!(pending.is_empty());
    }
}

#[test]
#[ignore = "release-only reused binding expression stack benchmark"]
fn runtime_interface03_batch33_reused_binding_expression_stack_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let program = expression_program();
    let expression = benchmark_expression();
    let mut pending = Vec::with_capacity(UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY);
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(program.expression_is_well_formed_allocating(black_box(&expression)));
            }
            started.elapsed().as_nanos()
        };
        let mut measure_reused = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(
                    program
                        .expression_is_well_formed(black_box(&expression), black_box(&mut pending)),
                );
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            reused_samples.push(measure_reused());
        } else {
            reused_samples.push(measure_reused());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    reused_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_REUSED_BINDING_EXPRESSION_STACK_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} reused_p95_ns={}",
        allocating_samples[p95], reused_samples[p95],
    );
    assert!(
        reused_samples[p95].saturating_mul(2) <= allocating_samples[p95],
        "reused binding expression stack must improve P95 by at least 50%: allocating={}ns reused={}ns",
        allocating_samples[p95],
        reused_samples[p95],
    );
}
