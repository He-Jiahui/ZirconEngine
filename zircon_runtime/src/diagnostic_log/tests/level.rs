use super::{
    selected_filter_env_override, DiagnosticLogFilter, DiagnosticLogFilterConfig,
    DiagnosticLogLevel, DIAGNOSTIC_LOG_ENV, DIAGNOSTIC_LOG_FILTER_ENV, RUST_LOG_ENV,
};

#[test]
fn default_filter_matches_build_profile_policy() {
    assert_eq!(
        DiagnosticLogFilter::default_for_debug_assertions(true),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)
    );
    assert_eq!(
        DiagnosticLogFilter::default_for_debug_assertions(false),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log)
    );
}

#[test]
fn filter_parse_accepts_level_names_and_common_aliases() {
    assert_eq!(
        DiagnosticLogFilter::parse("verbose").unwrap(),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)
    );
    assert_eq!(
        DiagnosticLogFilter::parse("trace").unwrap(),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Verbose)
    );
    assert_eq!(
        DiagnosticLogFilter::parse("debug").unwrap(),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug)
    );
    assert_eq!(
        DiagnosticLogFilter::parse("info").unwrap(),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log)
    );
    assert_eq!(
        DiagnosticLogFilter::parse("warning").unwrap(),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)
    );
    assert_eq!(
        DiagnosticLogFilter::parse("err").unwrap(),
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Error)
    );
    assert_eq!(
        DiagnosticLogFilter::parse("off").unwrap(),
        DiagnosticLogFilter::Off
    );
}

#[test]
fn filter_allows_only_level_at_or_above_minimum() {
    let release_default = DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log);

    assert!(!release_default.allows(DiagnosticLogLevel::Verbose));
    assert!(!release_default.allows(DiagnosticLogLevel::Debug));
    assert!(release_default.allows(DiagnosticLogLevel::Log));
    assert!(release_default.allows(DiagnosticLogLevel::Warn));
    assert!(release_default.allows(DiagnosticLogLevel::Error));
    assert!(!DiagnosticLogFilter::Off.allows(DiagnosticLogLevel::Error));
}

#[test]
fn compiled_filter_preserves_longest_scope_prefix_semantics() {
    let config = DiagnosticLogFilterConfig::parse(
        "warn,runtime=debug,runtime::asset=off,runtime::asset::loader=verbose",
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
    )
    .unwrap();
    let compiled = super::CompiledDiagnosticLogFilter::new(&config);

    for scope in [
        "",
        "editor",
        "runtime",
        "runtime::scene",
        "runtime::asset",
        "runtime::asset::database",
        "runtime::asset::loader",
        "runtime::asset::loader::io",
    ] {
        for level in [
            DiagnosticLogLevel::Verbose,
            DiagnosticLogLevel::Debug,
            DiagnosticLogLevel::Log,
            DiagnosticLogLevel::Warn,
            DiagnosticLogLevel::Error,
        ] {
            assert_eq!(
                compiled.allows(level, scope),
                config.allows(level, scope),
                "scope={scope} level={level}"
            );
        }
    }
}

#[test]
fn filter_parse_rejects_unknown_values() {
    let error = DiagnosticLogFilter::parse("chatty").unwrap_err();

    assert_eq!(error.value(), "chatty");
    assert!(error.to_string().contains("unknown diagnostic log level"));
}

#[test]
fn filter_config_parse_supports_global_and_scoped_rules() {
    let config = DiagnosticLogFilterConfig::parse(
        "warn,zircon_runtime::asset=debug,zircon_runtime::asset::import=verbose",
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
    )
    .unwrap();

    assert!(config.allows(
        DiagnosticLogLevel::Verbose,
        "zircon_runtime::asset::import::native"
    ));
    assert!(config.allows(DiagnosticLogLevel::Debug, "zircon_runtime::asset::path"));
    assert!(!config.allows(DiagnosticLogLevel::Log, "zircon_runtime::ui"));
    assert!(config.allows(DiagnosticLogLevel::Warn, "zircon_runtime::ui"));
}

#[test]
fn filter_config_env_precedence_prefers_zircon_filter_alias_before_rust_log() {
    assert_eq!(
        selected_filter_env_override(
            Some("warn".to_string()),
            Some("debug".to_string()),
            Some("error".to_string())
        ),
        Some((DIAGNOSTIC_LOG_FILTER_ENV, "warn".to_string()))
    );
    assert_eq!(
        selected_filter_env_override(None, Some("debug".to_string()), Some("error".to_string())),
        Some((DIAGNOSTIC_LOG_ENV, "debug".to_string()))
    );
    assert_eq!(
        selected_filter_env_override(None, None, Some("trace".to_string())),
        Some((RUST_LOG_ENV, "trace".to_string()))
    );
    assert_eq!(selected_filter_env_override(None, None, None), None);
}

fn assert_public_compiled_prefix_levels(
    config: &DiagnosticLogFilterConfig,
    expectations: &[(&str, DiagnosticLogFilter, [bool; 5])],
) {
    let compiled = super::CompiledDiagnosticLogFilter::new(config);
    for (scope, expected_filter, expected_levels) in expectations {
        assert_eq!(
            config.filter_for_scope(scope),
            *expected_filter,
            "scope={scope}"
        );
        for (level, expected) in [
            DiagnosticLogLevel::Verbose,
            DiagnosticLogLevel::Debug,
            DiagnosticLogLevel::Log,
            DiagnosticLogLevel::Warn,
            DiagnosticLogLevel::Error,
        ]
        .into_iter()
        .zip(expected_levels)
        {
            assert_eq!(
                config.allows(level, scope),
                *expected,
                "public scope={scope} level={level}"
            );
            assert_eq!(
                compiled.allows(level, scope),
                *expected,
                "compiled scope={scope} level={level}"
            );
        }
    }
}

#[test]
fn compiled_empty_prefix_overrides_global_for_all_scopes_and_levels() {
    use super::{DiagnosticLogFilter::*, DiagnosticLogLevel::*, DiagnosticLogModuleFilter};

    for (minimum, root_filter, expected_levels) in [
        (Minimum(Log), Off, [false, false, false, false, false]),
        (Off, Minimum(Debug), [false, true, true, true, true]),
        (
            Minimum(Warn),
            Minimum(Verbose),
            [true, true, true, true, true],
        ),
    ] {
        let config = DiagnosticLogFilterConfig {
            minimum,
            module_filters: vec![DiagnosticLogModuleFilter {
                scope_prefix: String::new(),
                filter: root_filter,
            }],
        };
        assert_public_compiled_prefix_levels(
            &config,
            &[
                ("", root_filter, expected_levels),
                ("asset", root_filter, expected_levels),
                ("unrelated::scope", root_filter, expected_levels),
                ("界🙂", root_filter, expected_levels),
            ],
        );
    }
}

#[test]
fn compiled_empty_prefix_is_overridden_only_by_longer_literal_byte_prefixes() {
    use super::{DiagnosticLogFilter::*, DiagnosticLogLevel::*, DiagnosticLogModuleFilter};

    let config = DiagnosticLogFilterConfig {
        minimum: Minimum(Verbose),
        module_filters: [
            ("asset", Minimum(Debug)),
            ("asset::ui", Minimum(Warn)),
            ("界", Minimum(Debug)),
            ("界面", Minimum(Error)),
            ("", Off),
        ]
        .into_iter()
        .map(|(scope_prefix, filter)| DiagnosticLogModuleFilter {
            scope_prefix: scope_prefix.to_string(),
            filter,
        })
        .collect(),
    };
    assert_public_compiled_prefix_levels(
        &config,
        &[
            ("", Off, [false, false, false, false, false]),
            ("unrelated", Off, [false, false, false, false, false]),
            ("xasset", Off, [false, false, false, false, false]),
            ("asset", Minimum(Debug), [false, true, true, true, true]),
            ("assets", Minimum(Debug), [false, true, true, true, true]),
            (
                "asset::ui",
                Minimum(Warn),
                [false, false, false, true, true],
            ),
            (
                "asset::uix",
                Minimum(Warn),
                [false, false, false, true, true],
            ),
            ("畀", Off, [false, false, false, false, false]),
            ("界", Minimum(Debug), [false, true, true, true, true]),
            ("界🙂", Minimum(Debug), [false, true, true, true, true]),
            ("界面", Minimum(Error), [false, false, false, false, true]),
            ("界面板", Minimum(Error), [false, false, false, false, true]),
        ],
    );
}

#[test]
fn compiled_duplicate_empty_prefix_rules_preserve_last_wins() {
    use super::{DiagnosticLogFilter::*, DiagnosticLogLevel::*, DiagnosticLogModuleFilter};

    for (first_root, last_root, expected_levels) in [
        (Off, Minimum(Debug), [false, true, true, true, true]),
        (Minimum(Verbose), Off, [false, false, false, false, false]),
    ] {
        let config = DiagnosticLogFilterConfig {
            minimum: Minimum(Log),
            module_filters: [
                ("", first_root),
                ("asset", Minimum(Verbose)),
                ("", last_root),
                ("asset", Minimum(Error)),
            ]
            .into_iter()
            .map(|(scope_prefix, filter)| DiagnosticLogModuleFilter {
                scope_prefix: scope_prefix.to_string(),
                filter,
            })
            .collect(),
        };
        assert_public_compiled_prefix_levels(
            &config,
            &[
                ("", last_root, expected_levels),
                ("unrelated", last_root, expected_levels),
                ("asset", Minimum(Error), [false, false, false, false, true]),
                ("assets", Minimum(Error), [false, false, false, false, true]),
            ],
        );
    }
}

#[test]
fn filter_parser_rejects_empty_prefix_and_preserves_literal_wildcards() {
    use super::{DiagnosticLogFilter::*, DiagnosticLogLevel::*};

    for (input, rejected_rule) in [
        ("=off", "=off"),
        ("  =debug", "=debug"),
        ("verbose, =warn", "=warn"),
    ] {
        let error = DiagnosticLogFilterConfig::parse(input, Minimum(Log)).unwrap_err();
        assert_eq!(error.value(), rejected_rule);
    }

    let config =
        DiagnosticLogFilterConfig::parse("off,asset*=debug,asset::*=warn", Minimum(Log)).unwrap();
    assert_public_compiled_prefix_levels(
        &config,
        &[
            ("", Off, [false, false, false, false, false]),
            ("asset", Off, [false, false, false, false, false]),
            ("assets", Off, [false, false, false, false, false]),
            ("asset::loader", Off, [false, false, false, false, false]),
            ("asset*", Minimum(Debug), [false, true, true, true, true]),
            (
                "asset*more",
                Minimum(Debug),
                [false, true, true, true, true],
            ),
            ("asset::*", Minimum(Warn), [false, false, false, true, true]),
            (
                "asset::*loader",
                Minimum(Warn),
                [false, false, false, true, true],
            ),
        ],
    );
}
