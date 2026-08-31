mod app;
mod args;
mod background_load;
mod camera;
mod evidence_identity;
mod frame_io;
mod gpu_timing_evidence;
mod hdri;
mod material_fixture;
mod presenter;
mod project_asset_fixture_validation;
mod project_assets;
mod renderdoc;
mod scene;
mod terminal_outcome;
mod work_paths;

use std::process::ExitCode;

use app::PbrMirrorViewerApp;
use args::{print_help, ViewerConfig};
use evidence_identity::load_ready_frame_evidence_identity;
use terminal_outcome::{
    write_terminal_outcome, TerminalErrorCategory, TerminalOutcome, TerminalPhase,
};
use winit::event_loop::EventLoop;
use work_paths::ViewerWorkPaths;

fn main() -> ExitCode {
    match run() {
        Ok(exit_code) => ExitCode::from(exit_code),
        Err(error) => {
            eprintln!(
                "PBR viewer startup failed before a terminal record could be written: {error}"
            );
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, String> {
    let config =
        ViewerConfig::from_args(std::env::args().skip(1)).map_err(|error| error.to_string())?;
    if config.help_requested {
        print_help();
        return Ok(0);
    }

    #[cfg(feature = "profiling")]
    let profile_capture_started =
        zircon_runtime::core::diagnostics::profiling::start_capture_from_env("shader-pbr-viewer")
            .is_some();
    let result = run_viewer(config);
    #[cfg(feature = "profiling")]
    return finish_profile_capture(result, profile_capture_started);
    #[cfg(not(feature = "profiling"))]
    result
}

fn run_viewer(config: ViewerConfig) -> Result<u8, String> {
    let work_paths = ViewerWorkPaths::new(&config.work_dir, config.ibl_cache_dir.as_deref());
    let terminal_outcome_path = work_paths.terminal_outcome_path().to_path_buf();
    let ready_frame_evidence_identity = match config.evidence_identity_path.as_deref() {
        Some(identity_path) => {
            match load_ready_frame_evidence_identity(identity_path, &config.hdri_path) {
                Ok(identity) => Some(identity),
                Err(error) => {
                    let outcome = TerminalOutcome::failure(
                        TerminalPhase::Startup,
                        TerminalErrorCategory::Artifact,
                        error,
                    )
                    .with_source_chain(viewer_terminal_source_chain(&config));
                    write_terminal_outcome(&terminal_outcome_path, &outcome)?;
                    return Ok(outcome.exit_code());
                }
            }
        }
        None => None,
    };
    let renderdoc_capture_path = config
        .renderdoc_capture_path
        .as_deref()
        .unwrap_or_else(|| work_paths.renderdoc_capture_template());
    let renderdoc_bridge = match renderdoc::preload_renderdoc_dll(
        config.renderdoc_dll.as_deref(),
        Some(renderdoc_capture_path),
    ) {
        Ok(bridge) => bridge,
        Err(error) => {
            let outcome = TerminalOutcome::failure(
                TerminalPhase::Startup,
                TerminalErrorCategory::Startup,
                error.to_string(),
            )
            .with_source_chain(viewer_terminal_source_chain(&config));
            write_terminal_outcome(&terminal_outcome_path, &outcome)?;
            return Ok(outcome.exit_code());
        }
    };
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            let outcome = TerminalOutcome::failure(
                TerminalPhase::EventLoop,
                TerminalErrorCategory::EventLoop,
                error.to_string(),
            )
            .with_source_chain(viewer_terminal_source_chain(&config));
            write_terminal_outcome(&terminal_outcome_path, &outcome)?;
            return Ok(outcome.exit_code());
        }
    };
    let event_loop_proxy = event_loop.create_proxy();
    let mut app = PbrMirrorViewerApp::new(
        config,
        event_loop_proxy,
        renderdoc_bridge,
        ready_frame_evidence_identity,
    );
    if let Err(error) = event_loop.run_app(&mut app) {
        app.record_event_loop_failure(error.to_string());
    }
    let outcome = app.finish_after_event_loop();
    write_terminal_outcome(&terminal_outcome_path, &outcome)?;
    Ok(outcome.exit_code())
}

#[cfg(feature = "profiling")]
fn finish_profile_capture(
    viewer_result: Result<u8, String>,
    profile_capture_started: bool,
) -> Result<u8, String> {
    if !profile_capture_started {
        return viewer_result;
    }
    match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env() {
        Some(Ok(report)) => {
            eprintln!("PBR viewer profile report exported: {}", report.export_dir);
            viewer_result
        }
        Some(Err(error)) if matches!(viewer_result.as_ref(), Ok(0)) => {
            Err(format!("export PBR viewer profile report: {error}"))
        }
        Some(Err(error)) => {
            eprintln!("PBR viewer profile report export also failed: {error}");
            viewer_result
        }
        None => viewer_result,
    }
}

fn viewer_terminal_source_chain(config: &ViewerConfig) -> [String; 2] {
    [
        "zircon_shader_pbr_viewer".to_owned(),
        format!("hdri:{}", config.hdri_path.display()),
    ]
}

#[cfg(test)]
mod tests {
    const MAIN_SOURCE: &str = include_str!("main.rs");

    #[test]
    fn early_fatal_paths_write_a_terminal_source_chain() {
        let production = MAIN_SOURCE
            .split("#[cfg(test)]")
            .next()
            .expect("main source should retain production code before tests");

        assert!(production.contains("fn viewer_terminal_source_chain(config: &ViewerConfig)"));
        assert_eq!(
            production
                .matches(".with_source_chain(viewer_terminal_source_chain(&config))")
                .count(),
            2,
            "RenderDoc preload and EventLoop creation failures must retain input provenance"
        );
    }

    #[test]
    fn profiling_build_uses_environment_capture_lifecycle() {
        let production = MAIN_SOURCE
            .split("#[cfg(test)]")
            .next()
            .expect("main source should retain production code before tests");

        assert!(production.contains("start_capture_from_env(\"shader-pbr-viewer\")"));
        assert!(production.contains("stop_and_export_capture_from_env()"));
        assert!(
            production
                .find("start_capture_from_env")
                .expect("capture start")
                < production
                    .find("run_viewer(config)")
                    .expect("viewer execution")
        );
        assert!(
            production
                .find("run_viewer(config)")
                .expect("viewer execution")
                < production
                    .find("stop_and_export_capture_from_env")
                    .expect("capture export")
        );
    }
}
