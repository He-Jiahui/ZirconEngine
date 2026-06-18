use std::fs;
use std::path::{Path, PathBuf};

const JOB_SYSTEM_DOC: &str = include_str!("../../../../docs/zircon_runtime/core/job_system.md");
const RUNTIME_11_PLAN: &str =
    include_str!("../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md");
const RUNTIME_INDEX: &str = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

#[derive(Debug)]
struct RayonReference {
    path: String,
    line: usize,
    snippet: String,
}

#[test]
fn rayon_is_only_reachable_through_core_task_primitives() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);
    let references = collect_rayon_references(manifest_root, &files);

    assert!(
        references
            .iter()
            .any(|reference| reference.path == "src/core/runtime/tasks/pool.rs"),
        "rayon boundary guard should see the task pool owner"
    );
    assert!(
        references
            .iter()
            .any(|reference| reference.path == "src/core/runtime/tasks/parallel_for.rs"),
        "rayon boundary guard should see the parallel_for owner"
    );

    let unclassified = references
        .iter()
        .filter(|reference| classify_rayon_reference(&reference.path).is_none())
        .map(|reference| {
            format!(
                "{}:{}: {}",
                reference.path, reference.line, reference.snippet
            )
        })
        .collect::<Vec<_>>();

    assert!(
        unclassified.is_empty(),
        "direct rayon usage must be routed through core task primitives:\n{}",
        unclassified.join("\n")
    );
}

#[test]
fn rayon_boundary_guard_rejects_unclassified_runtime_source() {
    assert_eq!(
        classify_rayon_reference("src/core/runtime/tasks/pool.rs"),
        Some("core-task-pool-rayon-owner")
    );
    assert_eq!(
        classify_rayon_reference("src/core/runtime/tasks/parallel_for.rs"),
        Some("core-task-parallel-for-owner")
    );
    assert_eq!(
        classify_rayon_reference("src/graphics/visibility/culling/parallel_frustum.rs"),
        None
    );
    assert_eq!(
        classify_rayon_reference("src/scene/ecs/schedule_parallel_executor.rs"),
        None
    );
}

#[test]
fn rayon_render_exception_cutover_is_recorded_in_runtime_11_m2_1_status() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);
    let references = collect_rayon_references(manifest_root, &files);

    let production_rayon_paths = references
        .iter()
        .map(|reference| reference.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        production_rayon_paths,
        std::collections::BTreeSet::from([
            "src/core/runtime/tasks/parallel_for.rs",
            "src/core/runtime/tasks/pool.rs",
        ]),
        "Runtime 11 M2.1 cutover allows direct Rayon only in core task owners"
    );

    for required_doc_anchor in [
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
        "direct_rayon_paths = 2",
        "parallel_frustum.rs",
        "compute_task_pool",
    ] {
        assert!(
            JOB_SYSTEM_DOC.contains(required_doc_anchor),
            "JobSystem doc must record Runtime 11 M2.1 cutover anchor `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
        "direct_rayon_paths = 2",
        "parallel_frustum.rs",
        "compute_task_pool",
    ] {
        assert!(
            RUNTIME_11_PLAN.contains(required_plan_anchor),
            "Runtime 11 plan must record M2.1 cutover anchor `{required_plan_anchor}`"
        );
    }

    for required_index_anchor in [
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
        "direct_rayon_paths = 2",
        "parallel_frustum",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must record Runtime 11 M2.1 cutover anchor `{required_index_anchor}`"
        );
    }
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("runtime source directory should be readable") {
        let entry = entry.expect("runtime source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn collect_rayon_references(manifest_root: &Path, files: &[PathBuf]) -> Vec<RayonReference> {
    let mut references = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, path);
        if is_test_path(&relative) {
            continue;
        }

        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in source.lines().enumerate() {
            if line_mentions_rayon(line) {
                references.push(RayonReference {
                    path: relative.clone(),
                    line: line_index + 1,
                    snippet: line.trim().to_string(),
                });
            }
        }
    }
    references
}

fn line_mentions_rayon(line: &str) -> bool {
    line.contains("use rayon")
        || line.contains("rayon::")
        || line.contains(".par_iter(")
        || line.contains(".par_chunks")
        || line.contains(".into_par_iter(")
}

fn classify_rayon_reference(relative_path: &str) -> Option<&'static str> {
    match relative_path {
        "src/core/runtime/tasks/pool.rs" => Some("core-task-pool-rayon-owner"),
        "src/core/runtime/tasks/parallel_for.rs" => Some("core-task-parallel-for-owner"),
        _ => None,
    }
}

fn is_test_path(relative_path: &str) -> bool {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    relative_path.split('/').any(|part| part == "tests")
        || file_name == "tests.rs"
        || file_name.ends_with("_tests.rs")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
