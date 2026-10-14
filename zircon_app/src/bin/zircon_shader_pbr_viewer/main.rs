//! 独立 PBR 诊断查看器的进程入口：配置与早期失败发布在主线程，场景/窗口状态交给事件循环宿主。
//! 终态发布成功才返回记录中的退出码；无法建立记录的启动错误由最外层报告。

#[path = "app.rs"]
mod app;
#[path = "args.rs"]
mod args;
#[path = "background_load.rs"]
mod background_load;
#[path = "camera.rs"]
mod camera;
#[path = "evidence_identity.rs"]
mod evidence_identity;
#[path = "frame_io.rs"]
mod frame_io;
#[path = "gpu_timing_evidence.rs"]
mod gpu_timing_evidence;
#[path = "hdri.rs"]
mod hdri;
#[path = "material_fixture.rs"]
mod material_fixture;
#[path = "presenter.rs"]
mod presenter;
#[path = "project_asset_fixture_validation.rs"]
mod project_asset_fixture_validation;
#[path = "project_assets.rs"]
mod project_assets;
#[path = "renderdoc.rs"]
mod renderdoc;
#[path = "scene.rs"]
mod scene;
#[path = "terminal_outcome.rs"]
mod terminal_outcome;
#[path = "work_paths.rs"]
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

/// 在主线程建立诊断宿主；事件循环返回后先收集清理结果，再发布终态与退出码。
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
    let renderdoc_bridge_result = (|| {
        if config.renderdoc_dll.is_some() {
            work_paths.prepare_renderdoc_capture_template()?;
        }
        renderdoc::preload_renderdoc_dll(
            config.renderdoc_dll.as_deref(),
            Some(renderdoc_capture_path),
        )
        .map_err(|error| error.to_string())
    })();
    let renderdoc_bridge = match renderdoc_bridge_result {
        Ok(bridge) => bridge,
        Err(error) => {
            let outcome = TerminalOutcome::failure(
                TerminalPhase::Startup,
                TerminalErrorCategory::Startup,
                error,
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
#[path = "tests/main.rs"]
mod tests;
