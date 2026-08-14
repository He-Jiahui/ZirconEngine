use super::*;

#[test]
fn runtime_15_production_sources_do_not_directly_unwrap_mutex_locks() {
    let runtime_src = runtime_src_path("");
    let mut runtime_sources = Vec::new();
    collect_runtime_rust_sources(&runtime_src, &runtime_src, &mut runtime_sources);

    assert!(
        !runtime_sources.is_empty(),
        "runtime lock-poison global gate should scan runtime production sources"
    );

    let mut violations = Vec::new();
    for source_path in runtime_sources {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read runtime source: {error}"));
        let production = production_code_view(&source);
        for (line_index, line) in production.lines().enumerate() {
            if line.contains(LOCK_UNWRAP_CALL) {
                violations.push(format!(
                    "{}:{}: {}",
                    runtime_source_display_path(&runtime_src, &source_path),
                    line_index + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime production code should use poison-safe lock helpers instead of direct {LOCK_UNWRAP_CALL}:\n{}",
        violations.join("\n")
    );

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}

#[test]
fn runtime_15_production_view_preserves_items_after_test_only_helpers() {
    let source = r###"
fn before() {
    poison_safe_lock();
}

#[cfg(test)]
fn direct_test_helper() {
    let _guard = state.lock().unwrap();
}

#[cfg(all(feature = "diagnostics", test))]
fn all_test_helper() {
    let _guard = state.lock().unwrap();
}

#[cfg(any(test, feature = "diagnostics"))]
fn conditionally_production() {
    poison_safe_lock();
}

#[cfg(not(test))]
fn explicitly_production() {
    poison_safe_lock();
}

fn after() {
    let _guard = state.lock().unwrap();
}

const FALSE_POSITIVE: &str = "#[cfg(test)] fn fake() { state.lock().unwrap(); }";
// #[cfg(test)] fn commented_out() { state.lock().unwrap(); }
"###;

    let production = production_code_view(source);
    assert_eq!(production.lines().count(), source.lines().count());
    assert!(production.contains("fn before()"));
    assert!(!production.contains("fn direct_test_helper()"));
    assert!(!production.contains("fn all_test_helper()"));
    assert!(production.contains("fn conditionally_production()"));
    assert!(production.contains("fn explicitly_production()"));
    assert!(production.contains("fn after()"));
    assert_eq!(production.matches(LOCK_UNWRAP_CALL).count(), 1);

    let lexical_edges = r###"
fn lifetime_pair<'a, 'b>(left: &'a str, right: &'b str) -> (&'a str, &'b str) {
    (left, right)
}
const BYTE: u8 = b'\'';
const NORMAL: &str = "#[cfg(test)] fake.lock().unwrap()";
const RAW: &str = r##"#[cfg(test)] fake.lock().unwrap()"##;
const BYTE_STRING: &[u8] = b"#[cfg(test)] fake.lock().unwrap()";
const BYTE_RAW: &[u8] = br##"#[cfg(test)] fake.lock().unwrap()"##;
const C_RAW: &core::ffi::CStr = cr##"#[cfg(test)] fake.lock().unwrap()"##;
/* outer /* nested #[cfg(test)] fake.lock().unwrap() */ still comment */
#[allow(dead_code)]
#[cfg(test)]
fn attributed_test_helper() { fake.lock().unwrap(); }
fn visible_after_edges() { poison_safe_lock(); }
"###;
    let lexical_production = production_code_view(lexical_edges);
    assert!(lexical_production.contains("fn lifetime_pair<'a, 'b>"));
    assert!(lexical_production.contains("fn visible_after_edges()"));
    assert!(!lexical_production.contains("fn attributed_test_helper()"));
    assert_eq!(lexical_production.matches(LOCK_UNWRAP_CALL).count(), 0);

    let cfg_contract = r###"
#[cfg_attr(not(test), cfg(test))]
fn cfg_attr_test_helper() { fake.lock().unwrap(); }
#[cfg_attr(test, allow(dead_code))]
fn cfg_attr_production_item() { poison_safe_lock(); }
#[cfg(feature = "diagnostics,trace")]
#[cfg(not(feature = "diagnostics,trace"))]
fn unreachable_in_production() { fake.lock().unwrap(); }
fn visible_after_cfg_contract() { poison_safe_lock(); }
"###;
    let cfg_production = production_code_view(cfg_contract);
    assert!(!cfg_production.contains("fn cfg_attr_test_helper()"));
    assert!(cfg_production.contains("fn cfg_attr_production_item()"));
    assert!(!cfg_production.contains("fn unreachable_in_production()"));
    assert!(cfg_production.contains("fn visible_after_cfg_contract()"));
    assert_eq!(cfg_production.matches(LOCK_UNWRAP_CALL).count(), 0);

    let local_grammar = r###"
struct LocalFields {
    #[cfg(test)]
    observer: Result<Left, Right>,
    production: usize,
}
enum LocalVariants {
    #[cfg(test)] Tuple(Result<Left, Right>),
    #[cfg(test)] Unit,
    #[cfg(test)] Struct { left: Left, right: Right },
    Production,
}
fn local_statements(value: Value) {
    #[cfg(test)]
    let observer = value.lock().unwrap();
    match value {
        #[cfg(test)]
        Value::Test => observer.lock().unwrap(),
        Value::Production => poison_safe_lock(),
    }
}
fn visible_after_local_grammar() { poison_safe_lock(); }
"###;
    let local_production = production_code_view(local_grammar);
    assert!(local_production.contains("observer: Result<Left, Right>"));
    assert!(local_production.contains("Tuple(Result<Left, Right>)"));
    assert!(local_production.contains("Struct { left: Left, right: Right }"));
    assert!(local_production.contains("let observer = value.lock().unwrap()"));
    assert!(local_production.contains("Value::Test => observer.lock().unwrap()"));
    assert!(local_production.contains("fn visible_after_local_grammar()"));

    let event_stream = read_runtime_src("core/resource/event_stream.rs");
    assert!(event_stream.contains("pub(crate) fn poison_state"));
    let event_stream_production = production_code_view(&event_stream);
    assert!(!event_stream_production.contains("pub(crate) fn poison_state"));
    assert!(event_stream_production.contains("pub struct ResourceEventReceiver"));
    assert_eq!(event_stream_production.matches(LOCK_UNWRAP_CALL).count(), 0);

    let diagnostic_source = r###"
fn production_diagnostic() { panic!("lock poisoned"); }
// comment-only lock poisoned diagnostic must not be treated as production code.
#[cfg(test)]
fn test_diagnostic() { panic!("test lock poisoned"); }
"###;
    let diagnostic_production = production_section(diagnostic_source);
    assert!(diagnostic_production.contains("lock poisoned"));
    assert!(!diagnostic_production.contains("comment-only lock poisoned"));
    assert!(!diagnostic_production.contains("test lock poisoned"));
}

fn collect_runtime_rust_sources(
    runtime_src: &std::path::Path,
    root: &std::path::Path,
    sources: &mut Vec<std::path::PathBuf>,
) {
    for entry in std::fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read runtime source directory: {error}"))
    {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read runtime source entry: {error}"))
            .path();
        if path.is_dir() {
            collect_runtime_rust_sources(runtime_src, &path, sources);
        } else if is_runtime_production_source(runtime_src, &path) {
            sources.push(path);
        }
    }
}

fn is_runtime_production_source(runtime_src: &std::path::Path, path: &std::path::Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return false;
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if file_name == "tests.rs" || file_name.ends_with("_tests.rs") {
        return false;
    }

    let Ok(relative) = path.strip_prefix(runtime_src) else {
        return false;
    };

    !relative
        .components()
        .any(|component| component.as_os_str().to_string_lossy() == "tests")
}

fn runtime_source_display_path(runtime_src: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(runtime_src)
        .map(|relative| format!("zircon_runtime/src/{}", relative.display()).replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string().replace('\\', "/"))
}
