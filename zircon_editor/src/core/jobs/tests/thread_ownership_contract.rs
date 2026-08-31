use std::fs;
use std::path::{Path, PathBuf};

#[path = "thread_ownership_contract/scanner.rs"]
mod scanner;

use scanner::has_bare_thread_owner;

#[test]
fn editor_production_sources_do_not_create_bare_threads() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations: Vec<PathBuf> = Vec::new();

    visit_production_rust_sources(&source_root, &mut |path, source| {
        if has_bare_thread_owner(source) {
            violations.push(path.to_path_buf());
        }
    });

    assert!(
        violations.is_empty(),
        "editor production sources must submit work through EditorJobSystem; bare thread owners: {violations:?}"
    );
}

#[test]
fn production_source_scope_excludes_crate_and_module_test_files() {
    let production_directory = Path::new("src/ui/retained_host/host_contract/surface_hit_test");
    assert!(
        !is_test_directory_path(production_directory),
        "production directories ending in `_test` must remain in the ownership scan: {}",
        production_directory.display()
    );
    assert!(
        !is_test_source_path(production_directory),
        "file-only test suffixes must not classify a production directory as a test source: {}",
        production_directory.display()
    );

    let test_directory = Path::new("src/ui/retained_host/host_contract/tests");
    assert!(
        is_test_directory_path(test_directory),
        "a canonical `tests` directory must stay outside the production ownership scan: {}",
        test_directory.display()
    );

    for test_source in [
        Path::new("src/tests/editing/locking.rs"),
        Path::new("src/core/jobs/tests/thread_ownership_contract.rs"),
        Path::new("src/ui/host/widget/tests.rs"),
        Path::new("src/ui/host/widget/cancellation_tests.rs"),
        Path::new("src/ui/host/widget/single_test.rs"),
    ] {
        assert!(
            is_test_source_path(test_source),
            "test source must stay outside the production ownership scan: {}",
            test_source.display()
        );
    }

    assert!(!is_test_source_path(Path::new(
        "src/ui/host/export_cargo_process.rs"
    )));
}

#[test]
fn editor_context_jobs_reuse_the_runtime_owned_scheduler() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let builder = read_source(
        &manifest_dir.join("src/core/context/builder.rs"),
        "editor context builder",
    );
    let manager = read_source(
        &manifest_dir.join("src/ui/host/editor_manager.rs"),
        "editor manager",
    );
    let jobs = read_source(
        &manifest_dir.join("src/core/jobs/system/mod.rs"),
        "editor job system",
    );
    let settings_persistence = read_source(
        &manifest_dir.join("src/core/settings/persistence.rs"),
        "editor settings persistence",
    );

    assert!(
        builder
            .contains("pub fn new(scheduler: JobScheduler, settings_io_scheduler: JobScheduler)"),
        "EditorContextBuilder must require compute and settings I/O schedulers explicitly"
    );
    assert!(
        manager.contains("core.scheduler().clone()"),
        "EditorManager must reuse the scheduler owned by its CoreHandle"
    );
    assert!(
        !jobs.contains("JobScheduler::default()")
            && !jobs.contains("impl Default for EditorJobSystem"),
        "EditorJobSystem must not hide a worker pool behind default construction"
    );
    assert!(
        manager.contains("JobScheduler::from_pool(core.task_graph().worker_pool().clone())"),
        "EditorManager must derive settings persistence from the Core task graph worker owner"
    );
    assert!(
        !settings_persistence.contains("JobScheduler::process_io()"),
        "settings persistence must not hide the process-default I/O owner"
    );
}

#[test]
fn bare_thread_guard_rejects_single_imports_and_aliases() {
    let spawn_word = ["sp", "awn"].concat();
    let scope_word = ["sc", "ope"].concat();
    let scoped_spawn_word = [spawn_word.as_str(), "_scoped"].concat();
    let builder_word = ["Build", "er"].concat();
    let scope_type_word = ["Sc", "ope"].concat();
    let samples = [
        format!("use std::thread::{spawn_word}; {spawn_word}(work);"),
        format!("::std::thread::{spawn_word}(work);"),
        format!("use ::std::thread::{spawn_word}; {spawn_word}(work);"),
        format!("use std as standard; standard::thread::{spawn_word}(work);"),
        format!("use std::{{self as standard}}; standard::thread::{spawn_word}(work);"),
        format!("use ::std::{{self as standard}}; standard::thread::{scope_word}(work);"),
        format!("let quote = '\"'; std::thread::{spawn_word}(work);"),
        format!("let quote = b'\"'; std::thread::{spawn_word}(work);"),
        format!("let quote = '\\''; std::thread::{spawn_word}(work);"),
        format!("let quote = b'\\''; std::thread::{spawn_word}(work);"),
        format!("use std::thread::{spawn_word} as launch; launch(work);"),
        format!("use std::thread::{{{spawn_word} as launch}}; launch(work);"),
        format!("use std::thread::{builder_word}; {builder_word}::new().{spawn_word}(work);"),
        format!(
            "use std::thread::{builder_word} as ThreadBuilder; ThreadBuilder::new().{spawn_word}(work);"
        ),
        format!("use std::thread as t; t::{builder_word}::new().{spawn_word}(work);"),
        format!("use std::{{thread as t}}; t::{spawn_word}(work);"),
        format!("use std::thread::{{self as t}}; t::{builder_word}::new().{spawn_word}(work);"),
        format!("use std::thread::{{self}}; thread::{spawn_word}(work);"),
        format!("std::thread::{scope_word}(|task_scope| task_scope.{spawn_word}(work));"),
        format!(
            "use std::thread as t; t::{scope_word}(|task_scope| task_scope.{spawn_word}(work));"
        ),
        format!(
            "use std::thread::{scope_word}; {scope_word}(|task_scope| task_scope.{spawn_word}(work));"
        ),
        format!(
            "use std::thread::{scope_word} as with_scope; with_scope(|task_scope| task_scope.{spawn_word}(work));"
        ),
        format!(
            "use std::thread::{{{scope_word} as with_scope}}; with_scope(|task_scope| task_scope.{spawn_word}(work));"
        ),
        format!("std::thread::{builder_word}::{scoped_spawn_word}(&builder, scope_ref, work);"),
        format!(
            "use std::thread::{builder_word} as ThreadBuilder; ThreadBuilder::{scoped_spawn_word}(&builder, scope_ref, work);"
        ),
        format!(
            "use std::thread::{{{builder_word} as ThreadBuilder}}; ThreadBuilder::{scoped_spawn_word}(&builder, scope_ref, work);"
        ),
        format!(
            "use std::thread::{{{builder_word} as ThreadBuilder}}; fn launch(builder: ThreadBuilder, scope_ref: ScopeRef) {{ builder.{scoped_spawn_word}(scope_ref, work); }}"
        ),
        format!(
            "use std::thread::{{self as t}}; t::{builder_word}::new().{scoped_spawn_word}(scope_ref, work);"
        ),
        format!(
            "fn launch(task_scope: &std::thread::{scope_type_word}<'_, '_>) {{ task_scope.{spawn_word}(work); }}"
        ),
        format!("std::thread::{scope_type_word}::{spawn_word}(task_scope, work);"),
        format!(
            "use std::thread::{{{scope_type_word}}}; fn launch(task_scope: &{scope_type_word}<'_, '_>) {{ task_scope.{spawn_word}(work); }}"
        ),
        format!(
            "use std::thread::{{{scope_type_word} as ThreadScope}}; fn launch(task_scope: &ThreadScope<'_, '_>) {{ task_scope.{spawn_word}(work); }}"
        ),
        format!(
            "use std::thread as t; fn launch(task_scope: &t::{scope_type_word}<'_, '_>) {{ task_scope.{spawn_word}(work); }}"
        ),
        format!(
            "use std::thread::{{self}}; fn launch(task_scope: &thread::{scope_type_word}<'_, '_>) {{ task_scope.{spawn_word}(work); }}"
        ),
    ];

    for sample in samples {
        assert!(
            has_bare_thread_owner(&sample),
            "guard must reject imported or aliased thread owners: {sample}"
        );
    }
}

#[test]
fn bare_thread_guard_ignores_threads_inside_cfg_test_modules() {
    let source = r#"
        fn production_entry() {}

        #[cfg(test)]
        mod tests {
            #[test]
            fn concurrent_fixture() {
                let worker = std::thread::spawn(|| {});
                worker.join().unwrap();
            }
        }
    "#;

    assert!(
        !has_bare_thread_owner(source),
        "the production ownership guard must not inspect cfg(test) fixtures"
    );
}

#[test]
fn bare_thread_guard_keeps_production_threads_visible_after_cfg_test_modules() {
    let source = r#"
        #[cfg(test)]
        mod tests {
            #[test]
            fn concurrent_fixture() {
                let worker = std::thread::spawn(|| {});
                worker.join().unwrap();
            }
        }

        fn production_worker() {
            let worker = std::thread::spawn(|| {});
            worker.join().unwrap();
        }
    "#;

    assert!(
        has_bare_thread_owner(source),
        "the production ownership guard must still reject non-test thread owners"
    );
}

#[test]
fn bare_thread_guard_ignores_visible_cfg_test_modules_without_hiding_production_owners() {
    let fixture_source = r#"
        #[cfg(test)]
        #[allow(dead_code)]
        pub(crate) mod fixtures {
            fn concurrent_fixture() {
                let worker = std::thread::Builder::new().spawn(|| {}).unwrap();
                worker.join().unwrap();
            }
        }
    "#;

    assert!(
        !has_bare_thread_owner(fixture_source),
        "the guard must not inspect visible cfg(test) fixture threads"
    );

    for visible_fixture in [
        r#"#[cfg(test)] pub mod fixtures { fn fixture() { std::thread::spawn(|| {}); } }"#,
        r#"#[cfg(test)] pub(in crate) mod fixtures { fn fixture() { std::thread::spawn(|| {}); } }"#,
        r#"#[allow(dead_code)] #[cfg(test)] mod fixtures { fn fixture() { std::thread::spawn(|| {}); } }"#,
    ] {
        assert!(
            !has_bare_thread_owner(visible_fixture),
            "the guard must skip every visible cfg(test) module form: {visible_fixture}"
        );
    }

    assert!(
        has_bare_thread_owner(
            r#"#[cfg(test)] fn fixture() { let worker = std::thread::spawn(|| {}); worker.join().unwrap(); }"#
        ),
        "only cfg(test) module bodies may be skipped by the production ownership guard"
    );

    let builder_source = format!(
        "{fixture_source}\nfn production_worker() {{ let worker = std::thread::Builder::new().spawn(|| {{}}).unwrap(); worker.join().unwrap(); }}"
    );
    assert!(
        has_bare_thread_owner(&builder_source),
        "the guard must not hide a production builder after a visible cfg(test) module"
    );

    let scope_source = format!(
        "{fixture_source}\nfn production_worker() {{ std::thread::scope(|scope| scope.spawn(|| {{}})); }}"
    );
    assert!(
        has_bare_thread_owner(&scope_source),
        "the guard must not hide a production scope after a visible cfg(test) module"
    );
}

#[test]
fn bare_thread_guard_allows_unrelated_spawn_and_scope_names() {
    let spawn_word = ["sp", "awn"].concat();
    let scope_word = ["sc", "ope"].concat();
    let scoped_spawn_word = [spawn_word.as_str(), "_scoped"].concat();
    let builder_word = ["Build", "er"].concat();
    let scope_type_word = ["Sc", "ope"].concat();
    let samples = [
        format!("executor.{spawn_word}(work);"),
        format!("task_scope.{spawn_word}(work);"),
        format!("database::{scope_word}(|transaction| transaction.commit());"),
        format!("use other::{scope_word}; {scope_word}(work);"),
        format!("use std::thread::{{{scope_word} as with_scope}}; database::{scope_word}(work);"),
        format!(
            "use std::thread::{{{scope_word} as with_scope}}; let _unused = with_scope; database::{scope_word}(work);"
        ),
        format!(
            "use std::thread::{{{spawn_word} as launch}}; let _unused = launch; executor.{spawn_word}(work);"
        ),
        format!(
            "use std::thread::{scope_word} as with_scope; let _unused = with_scope; database::{scope_word}(work);"
        ),
        format!(
            "use std::thread::{spawn_word} as launch; let _unused = launch; executor.{spawn_word}(work);"
        ),
        format!(
            "use other::{builder_word} as ThreadBuilder; ThreadBuilder::{scoped_spawn_word}(&builder, scope_ref, work);"
        ),
        format!(
            "use other::{{{builder_word} as ThreadBuilder}}; fn launch(builder: ThreadBuilder, scope_ref: ScopeRef) {{ builder.{scoped_spawn_word}(scope_ref, work); }}"
        ),
        format!(
            "mod thread {{ pub fn {spawn_word}(work: Work) {{ work.run(); }} }} thread::{spawn_word}(work);"
        ),
        format!("crate::thread::{scope_word}(work);"),
        format!("use other::{{self as standard}}; standard::thread::{spawn_word}(work);"),
        format!(
            "use std::thread::{builder_word}; fn launch(executor: Executor) {{ executor.{spawn_word}(work); }}"
        ),
        format!("// std::thread::{spawn_word}(work);\nexecutor.run(work);"),
        format!("let example = \"std::thread::{scope_word}(work)\"; executor.run(work);"),
        format!(
            "use other::{{{scope_type_word} as ThreadScope}}; fn launch(task_scope: &ThreadScope<'_, '_>) {{ task_scope.{spawn_word}(work); }}"
        ),
    ];

    for sample in samples {
        assert!(
            !has_bare_thread_owner(&sample),
            "guard must allow unrelated spawn or scope names: {sample}"
        );
    }
}

fn visit_production_rust_sources(root: &Path, visit: &mut impl FnMut(&Path, &str)) {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        panic!(
            "failed to enumerate editor source directory {}: {error}",
            root.display()
        )
    });
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to enumerate editor source: {error}"));
        let path = entry.path();
        if path.is_dir() {
            if !is_test_directory_path(&path) {
                visit_production_rust_sources(&path, visit);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && !is_test_source_path(&path)
        {
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!("failed to read editor source {}: {error}", path.display())
            });
            visit(&path, &source);
        }
    }
}

fn is_test_source_path(path: &Path) -> bool {
    if is_test_directory_path(path) {
        return true;
    }
    if !path.extension().is_some_and(|extension| extension == "rs") {
        return false;
    }

    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };
    stem == "tests" || stem.ends_with("_tests") || stem.ends_with("_test")
}

fn is_test_directory_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
}

fn read_source(path: &Path, label: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {label} {}: {error}", path.display()))
}
