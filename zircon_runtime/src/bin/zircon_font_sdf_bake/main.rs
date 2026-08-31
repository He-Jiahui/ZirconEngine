mod args;
mod write;

use std::fs;
use std::time::Duration;

use zircon_runtime::core::{EngineTaskGraph, EngineTaskGraphOptions};
use zircon_runtime::text::font_sdf_build_tool::bake_font_sdf_artifact;

const TASK_GRAPH_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    if let Err(error) = run() {
        eprintln!("zircon_font_sdf_bake: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::FontSdfCliArgs::parse(std::env::args_os().skip(1))?;
    let font_bytes = fs::read(&args.font)?;
    let task_graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::default())?;
    let artifact = bake_font_sdf_artifact(task_graph.worker_pool(), &font_bytes, &args.request);
    task_graph.shutdown(TASK_GRAPH_SHUTDOWN_TIMEOUT)?;
    let artifact = artifact?;
    let path = artifact.artifact_path(&args.cache_root);
    write::atomic_write(&path, artifact.bytes())?;
    let report = artifact.report();
    println!(
        "wrote {} ({} bytes, {} pages, {} glyphs, {} skipped)",
        path.display(),
        report.encoded_len,
        report.page_count,
        report.generated_glyph_count,
        report.skipped_glyph_count,
    );
    Ok(())
}
