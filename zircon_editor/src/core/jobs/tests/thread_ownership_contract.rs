use std::fs;
use std::path::{Path, PathBuf};

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

    assert!(
        builder.contains("pub fn new(scheduler: JobScheduler)"),
        "EditorContextBuilder must require the runtime scheduler explicitly"
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
        format!(
            "use std::{{self as standard}}; standard::thread::{spawn_word}(work);"
        ),
        format!(
            "use ::std::{{self as standard}}; standard::thread::{scope_word}(work);"
        ),
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
        format!(
            "use std::thread::{{self as t}}; t::{builder_word}::new().{spawn_word}(work);"
        ),
        format!("use std::thread::{{self}}; thread::{spawn_word}(work);"),
        format!(
            "std::thread::{scope_word}(|task_scope| task_scope.{spawn_word}(work));"
        ),
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
        format!(
            "std::thread::{builder_word}::{scoped_spawn_word}(&builder, scope_ref, work);"
        ),
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
        format!(
            "std::thread::{scope_type_word}::{spawn_word}(task_scope, work);"
        ),
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
        format!(
            "use std::thread::{{{scope_word} as with_scope}}; database::{scope_word}(work);"
        ),
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
        format!(
            "use other::{{self as standard}}; standard::thread::{spawn_word}(work);"
        ),
        format!(
            "use std::thread::{builder_word}; fn launch(executor: Executor) {{ executor.{spawn_word}(work); }}"
        ),
        format!("// std::thread::{spawn_word}(work);\nexecutor.run(work);"),
        format!(
            "let example = \"std::thread::{scope_word}(work)\"; executor.run(work);"
        ),
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

fn has_bare_thread_owner(source: &str) -> bool {
    let spawn_word = ["sp", "awn"].concat();
    let scope_word = ["sc", "ope"].concat();
    let scoped_spawn_word = [spawn_word.as_str(), "_scoped"].concat();
    let builder_word = ["Build", "er"].concat();
    let scope_type_word = ["Sc", "ope"].concat();
    let tokens = rust_code_tokens(source);
    let mut imports = ThreadImports::default();
    collect_thread_imports(
        &tokens,
        &spawn_word,
        &scope_word,
        &builder_word,
        &scope_type_word,
        &mut imports,
    );

    [spawn_word.as_str(), scope_word.as_str()]
        .iter()
        .any(|function_name| {
            let function_name = *function_name;
            path_is_called(&tokens, &["std", "::", "thread", "::", function_name])
                || path_is_called(&tokens, &["::", "std", "::", "thread", "::", function_name])
                || imports.std_root_names.iter().any(|root| {
                    path_is_called(
                        &tokens,
                        &[root.as_str(), "::", "thread", "::", function_name],
                    )
                })
                || imports
                    .module_names
                    .iter()
                    .any(|module| path_is_called(&tokens, &[module.as_str(), "::", function_name]))
                || imports
                    .function_names
                    .iter()
                    .any(|name| unqualified_function_is_called(&tokens, name))
        })
        || thread_types_own_threads(
            &tokens,
            &imports,
            &builder_word,
            &scope_type_word,
            &spawn_word,
            &scoped_spawn_word,
        )
}

#[derive(Default)]
struct ThreadImports {
    std_root_names: Vec<String>,
    module_names: Vec<String>,
    function_names: Vec<String>,
    builder_names: Vec<String>,
    scope_type_names: Vec<String>,
}

fn thread_types_own_threads(
    tokens: &[String],
    imports: &ThreadImports,
    builder_name: &str,
    scope_type_name: &str,
    spawn_name: &str,
    scoped_spawn_name: &str,
) -> bool {
    for index in 0..tokens.len() {
        if let Some(type_len) = thread_builder_type_len(tokens, index, imports, builder_name) {
            if associated_call_at(tokens, index + type_len, spawn_name)
                || associated_call_at(tokens, index + type_len, scoped_spawn_name)
                || builder_constructor_chain_spawns(
                    tokens,
                    index + type_len,
                    spawn_name,
                    scoped_spawn_name,
                )
            {
                return true;
            }
        }
        if let Some(type_len) = thread_scope_type_len(tokens, index, imports, scope_type_name) {
            if associated_call_at(tokens, index + type_len, spawn_name) {
                return true;
            }
        }
    }

    let mut builder_receivers = typed_receivers(tokens, |tokens, index| {
        thread_builder_type_len(tokens, index, imports, builder_name)
    });
    collect_builder_assignments(tokens, imports, builder_name, &mut builder_receivers);
    let scope_receivers = typed_receivers(tokens, |tokens, index| {
        thread_scope_type_len(tokens, index, imports, scope_type_name)
    });

    builder_receivers.iter().any(|receiver| {
        receiver_method_is_called(tokens, receiver, spawn_name)
            || receiver_method_is_called(tokens, receiver, scoped_spawn_name)
    }) || scope_receivers
        .iter()
        .any(|receiver| receiver_method_is_called(tokens, receiver, spawn_name))
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

fn collect_thread_imports(
    tokens: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "use" {
            index += 1;
            continue;
        }
        let Some(statement_end) = tokens[index + 1..]
            .iter()
            .position(|token| token == ";")
            .map(|offset| index + 1 + offset)
        else {
            break;
        };
        parse_thread_import(
            &tokens[index + 1..statement_end],
            spawn_name,
            scope_name,
            builder_name,
            scope_type_name,
            imports,
        );
        index = statement_end + 1;
    }
}

fn parse_thread_import(
    path: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    let path = if path.first().is_some_and(|token| token == "::") {
        &path[1..]
    } else {
        path
    };
    if path.len() == 3 && sequence_at(path, 0, &["std", "as"]) {
        push_unique(&mut imports.std_root_names, &path[2]);
        return;
    }
    if sequence_at(path, 0, &["std", "::", "thread"]) {
        let tail = &path[3..];
        if tail.is_empty() {
            push_unique(&mut imports.module_names, "thread");
        } else if tail.first().is_some_and(|token| token == "as") {
            if let Some(alias) = tail.get(1) {
                push_unique(&mut imports.module_names, alias);
            }
        } else if tail.first().is_some_and(|token| token == "::") {
            parse_thread_import_items(
                &tail[1..],
                spawn_name,
                scope_name,
                builder_name,
                scope_type_name,
                imports,
            );
        }
        return;
    }

    if path.len() >= 5
        && sequence_at(path, 0, &["std", "::", "{"])
        && path.last().is_some_and(|token| token == "}")
    {
        for item in top_level_items(&path[3..path.len() - 1]) {
            if item.first().is_some_and(|token| token == "self") {
                let root_name = if item.get(1).is_some_and(|token| token == "as") {
                    item.get(2).map(String::as_str).unwrap_or("std")
                } else {
                    "std"
                };
                push_unique(&mut imports.std_root_names, root_name);
                continue;
            }
            if item.first().is_none_or(|token| token != "thread") {
                continue;
            }
            if item.len() == 1 {
                push_unique(&mut imports.module_names, "thread");
            } else if item.get(1).is_some_and(|token| token == "as") {
                if let Some(alias) = item.get(2) {
                    push_unique(&mut imports.module_names, alias);
                }
            } else if item.get(1).is_some_and(|token| token == "::") {
                parse_thread_import_items(
                    &item[2..],
                    spawn_name,
                    scope_name,
                    builder_name,
                    scope_type_name,
                    imports,
                );
            }
        }
    }
}

fn parse_thread_import_items(
    items: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    if items.first().is_some_and(|token| token == "{")
        && items.last().is_some_and(|token| token == "}")
    {
        for item in top_level_items(&items[1..items.len() - 1]) {
            register_thread_import_item(
                item,
                spawn_name,
                scope_name,
                builder_name,
                scope_type_name,
                imports,
            );
        }
    } else {
        register_thread_import_item(
            items,
            spawn_name,
            scope_name,
            builder_name,
            scope_type_name,
            imports,
        );
    }
}

fn register_thread_import_item(
    item: &[String],
    spawn_name: &str,
    scope_name: &str,
    builder_name: &str,
    scope_type_name: &str,
    imports: &mut ThreadImports,
) {
    let Some(imported_name) = item.first() else {
        return;
    };
    let resolved_name = if item.get(1).is_some_and(|token| token == "as") {
        let Some(alias) = item.get(2) else {
            return;
        };
        alias
    } else {
        imported_name
    };

    if imported_name == "self" {
        let module_name = if item.get(1).is_some_and(|token| token == "as") {
            resolved_name
        } else {
            "thread"
        };
        push_unique(&mut imports.module_names, module_name);
    } else if imported_name == spawn_name || imported_name == scope_name {
        push_unique(&mut imports.function_names, resolved_name);
    } else if imported_name == builder_name {
        push_unique(&mut imports.builder_names, resolved_name);
    } else if imported_name == scope_type_name {
        push_unique(&mut imports.scope_type_names, resolved_name);
    }
}

fn top_level_items(tokens: &[String]) -> Vec<&[String]> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token.as_str() {
            "{" | "(" | "[" | "<" => depth += 1,
            "}" | ")" | "]" | ">" => depth = depth.saturating_sub(1),
            "," if depth == 0 => {
                items.push(&tokens[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    items.push(&tokens[start..]);
    items
}

fn path_is_called(tokens: &[String], path: &[&str]) -> bool {
    (0..tokens.len()).any(|index| {
        sequence_at(tokens, index, path)
            && path_boundary_before(tokens, index)
            && tokens
                .get(index + path.len())
                .is_some_and(|token| token == "(")
    })
}

fn unqualified_function_is_called(tokens: &[String], function_name: &str) -> bool {
    tokens.iter().enumerate().any(|(index, token)| {
        token == function_name
            && tokens.get(index + 1).is_some_and(|next| next == "(")
            && !matches!(
                tokens.get(index.wrapping_sub(1)).map(String::as_str),
                Some(".") | Some("::")
            )
    })
}

fn thread_builder_type_len(
    tokens: &[String],
    index: usize,
    imports: &ThreadImports,
    builder_name: &str,
) -> Option<usize> {
    thread_type_len(tokens, index, imports, builder_name, &imports.builder_names)
}

fn thread_scope_type_len(
    tokens: &[String],
    index: usize,
    imports: &ThreadImports,
    scope_type_name: &str,
) -> Option<usize> {
    thread_type_len(
        tokens,
        index,
        imports,
        scope_type_name,
        &imports.scope_type_names,
    )
}

fn thread_type_len(
    tokens: &[String],
    index: usize,
    imports: &ThreadImports,
    canonical_name: &str,
    imported_names: &[String],
) -> Option<usize> {
    if sequence_at(
        tokens,
        index,
        &["std", "::", "thread", "::", canonical_name],
    ) && path_boundary_before(tokens, index)
    {
        return Some(5);
    }
    if sequence_at(
        tokens,
        index,
        &["::", "std", "::", "thread", "::", canonical_name],
    ) && path_boundary_before(tokens, index)
    {
        return Some(6);
    }
    if imports.std_root_names.iter().any(|root| {
        sequence_at(
            tokens,
            index,
            &[root.as_str(), "::", "thread", "::", canonical_name],
        ) && path_boundary_before(tokens, index)
    }) {
        return Some(5);
    }
    if imports.module_names.iter().any(|module| {
        sequence_at(tokens, index, &[module.as_str(), "::", canonical_name])
            && path_boundary_before(tokens, index)
    }) {
        return Some(3);
    }
    if imported_names
        .iter()
        .any(|name| tokens.get(index).is_some_and(|token| token == name))
        && path_boundary_before(tokens, index)
    {
        return Some(1);
    }
    None
}

fn associated_call_at(tokens: &[String], after_type: usize, method_name: &str) -> bool {
    sequence_at(tokens, after_type, &["::", method_name, "("])
}

fn builder_constructor_chain_spawns(
    tokens: &[String],
    after_type: usize,
    spawn_name: &str,
    scoped_spawn_name: &str,
) -> bool {
    if !sequence_at(tokens, after_type, &["::", "new", "("]) {
        return false;
    }
    let Some(mut cursor) = matching_close(tokens, after_type + 2, "(", ")").map(|end| end + 1)
    else {
        return false;
    };
    while sequence_at(tokens, cursor, &["."])
        && tokens
            .get(cursor + 1)
            .is_some_and(|token| is_identifier(token))
        && tokens.get(cursor + 2).is_some_and(|token| token == "(")
    {
        let method = &tokens[cursor + 1];
        if method == spawn_name || method == scoped_spawn_name {
            return true;
        }
        let Some(end) = matching_close(tokens, cursor + 2, "(", ")") else {
            return false;
        };
        cursor = end + 1;
    }
    false
}

fn typed_receivers(
    tokens: &[String],
    type_len_at: impl Fn(&[String], usize) -> Option<usize>,
) -> Vec<String> {
    let mut receivers = Vec::new();
    for index in 0..tokens.len().saturating_sub(2) {
        if !is_identifier(&tokens[index]) || tokens[index + 1] != ":" {
            continue;
        }
        let mut type_index = index + 2;
        while type_index < tokens.len()
            && !matches!(tokens[type_index].as_str(), "," | ")" | "=" | ";" | "{")
        {
            if type_len_at(tokens, type_index).is_some() {
                push_unique(&mut receivers, &tokens[index]);
                break;
            }
            type_index += 1;
        }
    }
    receivers
}

fn collect_builder_assignments(
    tokens: &[String],
    imports: &ThreadImports,
    builder_name: &str,
    receivers: &mut Vec<String>,
) {
    let mut index = 0;
    while index + 3 < tokens.len() {
        if tokens[index] != "let" {
            index += 1;
            continue;
        }
        let name_index = if tokens.get(index + 1).is_some_and(|token| token == "mut") {
            index + 2
        } else {
            index + 1
        };
        if !tokens
            .get(name_index)
            .is_some_and(|token| is_identifier(token))
        {
            index += 1;
            continue;
        }
        let Some(equals_index) = tokens[name_index + 1..]
            .iter()
            .position(|token| token == "=" || token == ";")
            .map(|offset| name_index + 1 + offset)
        else {
            break;
        };
        if tokens[equals_index] == "=" {
            let type_index = equals_index + 1;
            if let Some(type_len) =
                thread_builder_type_len(tokens, type_index, imports, builder_name)
            {
                if sequence_at(tokens, type_index + type_len, &["::", "new", "("]) {
                    push_unique(receivers, &tokens[name_index]);
                }
            }
        }
        index = equals_index + 1;
    }
}

fn receiver_method_is_called(tokens: &[String], receiver: &str, method_name: &str) -> bool {
    (0..tokens.len()).any(|index| {
        sequence_at(tokens, index, &[receiver, ".", method_name, "("])
            && path_boundary_before(tokens, index)
    })
}

fn matching_close(tokens: &[String], open_index: usize, open: &str, close: &str) -> Option<usize> {
    if tokens.get(open_index).is_none_or(|token| token != open) {
        return None;
    }
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open_index) {
        if token == open {
            depth += 1;
        } else if token == close {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn sequence_at(tokens: &[String], index: usize, sequence: &[&str]) -> bool {
    tokens
        .get(index..index + sequence.len())
        .is_some_and(|candidate| {
            candidate
                .iter()
                .map(String::as_str)
                .eq(sequence.iter().copied())
        })
}

fn path_boundary_before(tokens: &[String], index: usize) -> bool {
    !matches!(
        index
            .checked_sub(1)
            .and_then(|before| tokens.get(before))
            .map(String::as_str),
        Some(".") | Some("::")
    )
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !value.is_empty() && !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

fn is_identifier(token: &str) -> bool {
    token
        .bytes()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn rust_code_tokens(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
        } else if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index);
        } else if let Some(end) = char_or_byte_literal_end(bytes, index) {
            index = end;
        } else if let Some(end) = raw_string_end(bytes, index) {
            index = end;
        } else if bytes[index] == b'"' {
            index = skip_quoted_string(bytes, index);
        } else if matches!(bytes[index], b'b' | b'c')
            && bytes.get(index + 1).is_some_and(|byte| *byte == b'"')
        {
            index = skip_quoted_string(bytes, index + 1);
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(String::from_utf8_lossy(&bytes[start..index]).into_owned());
        } else if index + 1 < bytes.len()
            && matches!(&bytes[index..index + 2], b"::" | b"->" | b"=>")
        {
            tokens.push(String::from_utf8_lossy(&bytes[index..index + 2]).into_owned());
            index += 2;
        } else {
            tokens.push((bytes[index] as char).to_string());
            index += 1;
        }
    }
    tokens
}

fn char_or_byte_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let quote_index = if bytes.get(start) == Some(&b'\'') {
        start
    } else if bytes.get(start) == Some(&b'b') && bytes.get(start + 1) == Some(&b'\'') {
        start + 1
    } else {
        return None;
    };
    let mut cursor = quote_index + 1;
    if bytes.get(cursor) == Some(&b'\\') {
        cursor += 1;
        match bytes.get(cursor).copied()? {
            b'x' => cursor += 3,
            b'u' if bytes.get(cursor + 1) == Some(&b'{') => {
                cursor += 2;
                while bytes.get(cursor).is_some_and(|byte| *byte != b'}') {
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'}') {
                    return None;
                }
                cursor += 1;
            }
            _ => cursor += 1,
        }
    } else {
        let character = std::str::from_utf8(bytes.get(cursor..)?)
            .ok()?
            .chars()
            .next()?;
        cursor += character.len_utf8();
    }
    (bytes.get(cursor) == Some(&b'\'')).then_some(cursor + 1)
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start + 2;
    while index < bytes.len() && depth > 0 {
        if bytes[index..].starts_with(b"/*") {
            depth += 1;
            index += 2;
        } else if bytes[index..].starts_with(b"*/") {
            depth -= 1;
            index += 2;
        } else {
            index += 1;
        }
    }
    index
}

fn skip_quoted_string(bytes: &[u8], quote_index: usize) -> usize {
    let mut index = quote_index + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == b'"' {
            return index + 1;
        } else {
            index += 1;
        }
    }
    index
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut marker = start;
    if matches!(bytes.get(marker), Some(b'b' | b'c')) {
        marker += 1;
    }
    if bytes.get(marker) != Some(&b'r') {
        return None;
    }
    marker += 1;
    let hashes_start = marker;
    while bytes.get(marker) == Some(&b'#') {
        marker += 1;
    }
    if bytes.get(marker) != Some(&b'"') {
        return None;
    }
    let hash_count = marker - hashes_start;
    let mut index = marker + 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && bytes
                .get(index + 1..index + 1 + hash_count)
                .is_some_and(|hashes| hashes.iter().all(|byte| *byte == b'#'))
        {
            return Some(index + 1 + hash_count);
        }
        index += 1;
    }
    Some(bytes.len())
}
