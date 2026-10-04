use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if !arguments.is_empty() {
        return run_snapshot_command(&arguments);
    }
    match zircon_editor::export_zui_visual_evidence() {
        Ok(summary) => {
            println!(
                "editor-zui-capture captured={} failed={} pending={} ready={} accepted={} report={}",
                summary.captured(),
                summary.failed(),
                summary.pending(),
                summary.is_ready(),
                summary.is_accepted(),
                summary.report_path().display()
            );
            if summary.is_ready() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("editor-zui-capture failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_snapshot_command(arguments: &[String]) -> ExitCode {
    let (output, repo_root) = match arguments {
        [option, output, repo_option, repo_root]
            if option == "--export-product-snapshots" && repo_option == "--repo-root" =>
        {
            (output, repo_root)
        }
        [option] if option == "--help" => {
            println!(
                "Usage: zircon_editor_zui_capture [--export-product-snapshots PATH --repo-root PATH]"
            );
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!(
                "Usage: zircon_editor_zui_capture [--export-product-snapshots PATH --repo-root PATH]"
            );
            return ExitCode::FAILURE;
        }
    };
    match zircon_editor::export_zui_workbench_product_snapshots(
        std::path::Path::new(repo_root),
        std::path::Path::new(output),
    ) {
        Ok(()) => {
            println!("editor-zui-snapshots source_root={repo_root} output={output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("editor-zui-snapshots failed: {error}");
            ExitCode::FAILURE
        }
    }
}
