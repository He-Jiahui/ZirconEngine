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
