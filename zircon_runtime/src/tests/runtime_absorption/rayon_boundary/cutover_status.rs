use super::support::{collect_rayon_references, rust_source_files};

const JOB_SYSTEM_DOC: &str = include_str!("../../../../../docs/zircon_runtime/core/job_system.md");
const RUNTIME_11_PLAN: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/11/2026-07-09-job-system-task-model-output-records.md")
);
const RUNTIME_INDEX: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md")
);

#[test]
fn rayon_render_exception_cutover_is_recorded_in_runtime_11_m2_1_status() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
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
