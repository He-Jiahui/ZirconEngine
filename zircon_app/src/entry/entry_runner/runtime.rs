use std::error::Error;

use winit::event_loop::EventLoop;

use super::super::runtime_library::{LoadedRuntime, RuntimeSession};
use super::super::runtime_entry_app::RuntimeEntryApp;
use super::EntryRunner;

impl EntryRunner {
    pub fn run_runtime() -> Result<(), Box<dyn Error>> {
        let runtime = LoadedRuntime::load_default()?;
        let session = RuntimeSession::create(runtime)?;
        let event_loop = EventLoop::new()?;
        let app = RuntimeEntryApp::new(session);
        event_loop.run_app(app)?;
        Ok(())
    }
}
