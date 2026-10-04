use super::*;

#[test]
fn typed_binding_literal_parser_preserves_supported_value_kinds_and_escapes() {
    let cases = [
        (
            r#"color("quote\" slash\\ newline\n return\r tab\t back\b form\f unit\u001f")"#,
            UiValue::Color(
                "quote\" slash\\ newline\n return\r tab\t back\u{0008} form\u{000c} unit\u{001f}"
                    .to_string(),
            ),
        ),
        (
            r#"asset_ref("asset://ui/status")"#,
            UiValue::AssetRef("asset://ui/status".to_string()),
        ),
        (
            r#"instance_ref("StatusRoot")"#,
            UiValue::InstanceRef("StatusRoot".to_string()),
        ),
        (
            r#"enum("layout.horizontal")"#,
            UiValue::Enum("layout.horizontal".to_string()),
        ),
        ("vec2(1, -2.5)", UiValue::Vec2([1.0, -2.5])),
        ("vec3(1, 2.25, -3)", UiValue::Vec3([1.0, 2.25, -3.0])),
        (
            "vec4(0, 0.5, 1, -4.75)",
            UiValue::Vec4([0.0, 0.5, 1.0, -4.75]),
        ),
        (
            r#"flags("read", "write")"#,
            UiValue::Flags(vec!["read".to_string(), "write".to_string()]),
        ),
        ("flags()", UiValue::Flags(Vec::new())),
    ];

    for (source, value) in cases {
        assert_eq!(
            UiBindingExpression::parse(source).unwrap(),
            UiBindingExpression::Literal(value),
            "typed literal source: {source}"
        );
    }
}

#[test]
fn typed_binding_literal_param_probe_requires_a_path_root() {
    assert!(UiBindingExpression::contains_param_reference("param.title"));
    assert!(UiBindingExpression::contains_param_reference(
        "=concat(param.title, prop.text)"
    ));
    assert!(!UiBindingExpression::contains_param_reference(
        r#"=concat("param.title", prop.text)"#
    ));
    assert!(!UiBindingExpression::contains_param_reference(
        "control.param.prop.value"
    ));
}

#[test]
fn control_reference_probe_ignores_quoted_preview_text() {
    assert!(UiBindingExpression::contains_control_reference(
        "control.Value.prop.text"
    ));
    assert!(UiBindingExpression::contains_control_reference(
        "=control.Value.prop.text == \"Ready\""
    ));
    assert!(!UiBindingExpression::contains_control_reference(
        r#"=concat("control.Value.prop.text", prop.text)"#
    ));
    assert!(!UiBindingExpression::contains_control_reference(
        "model.control.Value.prop.text"
    ));
}

#[test]
fn binding_expression_parse_budgets_reject_oversized_or_deep_input() {
    let oversized = "x".repeat(UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES + 1);
    assert_eq!(
        UiBindingExpression::parse(&oversized),
        Err(UiBindingExpressionParseError::BudgetExceeded {
            budget: "source bytes",
            limit: UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        })
    );
    assert_eq!(
        UiBindingExpression::probe_param_reference(&oversized),
        Err(UiBindingExpressionParseError::BudgetExceeded {
            budget: "source bytes",
            limit: UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        })
    );
    assert_eq!(
        UiBindingExpression::probe_control_reference(&oversized),
        Err(UiBindingExpressionParseError::BudgetExceeded {
            budget: "source bytes",
            limit: UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES,
        })
    );

    let excessive_tokens = format!("{}true", "!".repeat(UI_BINDING_EXPRESSION_MAX_TOKENS));
    assert_eq!(
        UiBindingExpression::parse(&excessive_tokens),
        Err(UiBindingExpressionParseError::BudgetExceeded {
            budget: "tokens",
            limit: UI_BINDING_EXPRESSION_MAX_TOKENS,
        })
    );

    let excessive_depth = format!("{}true", "!".repeat(UI_BINDING_EXPRESSION_MAX_DEPTH));
    assert_eq!(
        UiBindingExpression::parse(&excessive_depth),
        Err(UiBindingExpressionParseError::BudgetExceeded {
            budget: "depth",
            limit: UI_BINDING_EXPRESSION_MAX_DEPTH,
        })
    );
}

#[test]
fn early_token_admission_preserves_unbounded_scan_error() {
    let input = "!".repeat(UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES);
    assert_eq!(
        tokenize_with_budget(&input),
        tokenize_with_budget_unbounded_scan(&input),
    );
    assert_eq!(
        tokenize_with_budget(&input),
        Err(UiBindingExpressionParseError::BudgetExceeded {
            budget: "tokens",
            limit: UI_BINDING_EXPRESSION_MAX_TOKENS,
        })
    );
}

#[test]
#[ignore = "release-only early binding token admission benchmark"]
fn runtime_interface03_batch25_early_token_admission_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 32;
    const SAMPLE_COUNT: usize = 11;
    let input = "!".repeat(UI_BINDING_EXPRESSION_MAX_SOURCE_BYTES);
    let mut unbounded_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut early_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_unbounded = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(tokenize_with_budget_unbounded_scan(black_box(&input)));
            }
            started.elapsed().as_nanos()
        };
        let measure_early = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(tokenize_with_budget(black_box(&input)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unbounded_samples.push(measure_unbounded());
            early_samples.push(measure_early());
        } else {
            early_samples.push(measure_early());
            unbounded_samples.push(measure_unbounded());
        }
    }

    unbounded_samples.sort_unstable();
    early_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_EARLY_TOKEN_ADMISSION_BENCH_V1 source_bytes={} token_limit={} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} unbounded_p95_ns={} early_p95_ns={}",
        input.len(),
        UI_BINDING_EXPRESSION_MAX_TOKENS,
        unbounded_samples[p95],
        early_samples[p95],
    );
    assert!(
        early_samples[p95].saturating_mul(2) <= unbounded_samples[p95],
        "early token admission must improve hostile-input P95 by at least 50%: unbounded={}ns early={}ns",
        unbounded_samples[p95],
        early_samples[p95],
    );
}

#[test]
fn stack_unicode_escape_preserves_allocating_decoder_results() {
    for source in [
        "0041",
        "d7ff",
        "e000",
        "ffff",
        "+001",
        "zzzz",
        "\u{00e9}001",
    ] {
        let chars = source.chars().collect::<Vec<_>>();
        let mut allocating_index = 0;
        let mut stack_index = 0;
        assert_eq!(
            parse_unicode_escape(&chars, &mut stack_index),
            parse_unicode_escape_allocating(&chars, &mut allocating_index),
            "escape source: {source}",
        );
        assert_eq!(stack_index, allocating_index, "escape source: {source}");
    }
}

#[test]
#[ignore = "release-only stack unicode escape benchmark"]
fn runtime_interface03_batch26_stack_unicode_escape_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let chars = "20ac".chars().collect::<Vec<_>>();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                let mut index = 0;
                black_box(parse_unicode_escape_allocating(
                    black_box(&chars),
                    black_box(&mut index),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_stack = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                let mut index = 0;
                black_box(parse_unicode_escape(
                    black_box(&chars),
                    black_box(&mut index),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            stack_samples.push(measure_stack());
        } else {
            stack_samples.push(measure_stack());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    stack_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STACK_UNICODE_ESCAPE_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} stack_p95_ns={}",
        allocating_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(2) <= allocating_samples[p95],
        "stack unicode escape must improve P95 by at least 50%: allocating={}ns stack={}ns",
        allocating_samples[p95],
        stack_samples[p95],
    );
}

#[test]
fn borrowed_keyword_tokens_preserve_allocating_identifier_results() {
    for source in ["true", "false", "null", "True", "true_value", "custom", "-"] {
        let chars = source.chars().collect::<Vec<_>>();
        let mut allocating_index = 0;
        let mut borrowed_index = 0;
        assert_eq!(
            parse_ident(&chars, &mut borrowed_index),
            parse_ident_allocating(&chars, &mut allocating_index),
            "identifier source: {source}",
        );
        assert_eq!(borrowed_index, allocating_index);
    }
}

#[test]
#[ignore = "release-only borrowed binding keyword token benchmark"]
fn runtime_interface03_batch27_borrowed_keyword_token_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let chars = "false".chars().collect::<Vec<_>>();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                let mut index = 0;
                black_box(parse_ident_allocating(
                    black_box(&chars),
                    black_box(&mut index),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_borrowed = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                let mut index = 0;
                black_box(parse_ident(black_box(&chars), black_box(&mut index)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BORROWED_KEYWORD_TOKEN_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} borrowed_p95_ns={}",
        allocating_samples[p95], borrowed_samples[p95],
    );
    assert!(
        borrowed_samples[p95].saturating_mul(2) <= allocating_samples[p95],
        "borrowed keyword token must improve P95 by at least 50%: allocating={}ns borrowed={}ns",
        allocating_samples[p95],
        borrowed_samples[p95],
    );
}

fn float_argument_parser() -> Parser {
    Parser {
        tokens: vec![
            Token::Integer(1),
            Token::Comma,
            Token::Float(2.25),
            Token::Comma,
            Token::Integer(-3),
            Token::Comma,
            Token::Float(4.75),
            Token::RightParen,
        ],
        index: 0,
    }
}

#[test]
fn stack_float_arguments_preserve_allocating_parser_results() {
    let mut allocating = float_argument_parser();
    let mut stack = float_argument_parser();
    assert_eq!(
        stack.expect_float_arguments::<4>("vec4"),
        allocating.expect_float_arguments_allocating::<4>("vec4"),
    );
    assert_eq!(stack.index, allocating.index);
}

#[test]
#[ignore = "release-only stack float argument benchmark"]
fn runtime_interface03_batch28_stack_float_arguments_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let mut allocating_parser = float_argument_parser();
    let mut stack_parser = float_argument_parser();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut stack_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let mut measure_allocating = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                allocating_parser.index = 0;
                black_box(
                    allocating_parser.expect_float_arguments_allocating::<4>(black_box("vec4")),
                );
            }
            started.elapsed().as_nanos()
        };
        let mut measure_stack = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                stack_parser.index = 0;
                black_box(stack_parser.expect_float_arguments::<4>(black_box("vec4")));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            stack_samples.push(measure_stack());
        } else {
            stack_samples.push(measure_stack());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    stack_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STACK_FLOAT_ARGUMENTS_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} allocating_p95_ns={} stack_p95_ns={}",
        allocating_samples[p95], stack_samples[p95],
    );
    assert!(
        stack_samples[p95].saturating_mul(5) <= allocating_samples[p95].saturating_mul(4),
        "stack float arguments must improve P95 by at least 20%: allocating={}ns stack={}ns",
        allocating_samples[p95],
        stack_samples[p95],
    );
}
