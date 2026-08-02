use super::{assert_contains_all, read_runtime_src};

#[test]
fn runtime_15_text_raster_worker_pool_diagnostics_are_child_owner() {
    let raster_pool = read_runtime_src("text/parallel/raster_pool.rs");
    let diagnostics = read_runtime_src("text/parallel/raster_pool/diagnostics.rs");

    assert!(
        raster_pool.lines().count() < 800,
        "text/parallel/raster_pool.rs must stay within the production soft budget"
    );
    assert_contains_all(
        "text raster worker pool root keeps queue lifecycle and mounts diagnostics",
        &raster_pool,
        &[
            "mod diagnostics;",
            "mod worker;",
            "pub(crate) use diagnostics::{",
            "pub(crate) struct TextRasterWorkerPool",
        ],
    );
    assert!(
        !raster_pool.contains("pub(crate) fn record_diagnostics("),
        "text raster worker pool root must delegate diagnostic recording"
    );
    assert_contains_all(
        "text raster worker diagnostics owns telemetry projection",
        &diagnostics,
        &[
            "pub(crate) struct TextRasterWorkerPoolDiagnostics",
            "pub(crate) struct TextRasterWorkerPoolFrameDiagnostics",
            "pub(crate) struct TextRasterWorkerPoolFrameSampler",
            "impl TextRasterWorkerPool {",
            "pub(crate) fn record_diagnostics(",
        ],
    );
}
