mod app;
mod args;
mod background_load;
mod camera;
mod frame_io;
mod gpu_timing_evidence;
mod hdri;
mod presenter;
mod project_assets;
mod renderdoc;
mod scene;
mod work_paths;

use std::error::Error;

use app::PbrMirrorViewerApp;
use args::{print_help, ViewerConfig};
use winit::event_loop::EventLoop;
use work_paths::ViewerWorkPaths;

fn main() -> Result<(), Box<dyn Error>> {
    let config = ViewerConfig::from_args(std::env::args().skip(1))?;
    if config.help_requested {
        print_help();
        return Ok(());
    }

    let work_paths = ViewerWorkPaths::new(&config.work_dir, config.ibl_cache_dir.as_deref());
    let renderdoc_capture_path = config
        .renderdoc_capture_path
        .as_deref()
        .unwrap_or_else(|| work_paths.renderdoc_capture_template());
    let renderdoc_bridge = renderdoc::preload_renderdoc_dll(
        config.renderdoc_dll.as_deref(),
        Some(renderdoc_capture_path),
    )?;
    let event_loop = EventLoop::new()?;
    let event_loop_proxy = event_loop.create_proxy();
    event_loop.run_app(PbrMirrorViewerApp::new(
        config,
        event_loop_proxy,
        renderdoc_bridge,
    ))?;
    Ok(())
}
