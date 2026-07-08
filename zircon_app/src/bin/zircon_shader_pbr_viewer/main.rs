mod app;
mod args;
mod camera;
mod hdri;
mod presenter;
mod project_assets;
mod scene;

use std::error::Error;

use app::PbrMirrorViewerApp;
use args::{print_help, ViewerConfig};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let config = ViewerConfig::from_args(std::env::args().skip(1))?;
    if config.help_requested {
        print_help();
        return Ok(());
    }

    let event_loop = EventLoop::new()?;
    event_loop.run_app(PbrMirrorViewerApp::new(config))?;
    Ok(())
}
