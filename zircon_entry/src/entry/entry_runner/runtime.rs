use std::error::Error;

use winit::event_loop::EventLoop;

use crate::entry::{EntryConfig, EntryProfile};

use super::super::runtime_entry_app::RuntimeEntryApp;
use super::EntryRunner;

impl EntryRunner {
    pub fn run_runtime() -> Result<(), Box<dyn Error>> {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Runtime))?;
        let event_loop = EventLoop::new()?;
        let app = RuntimeEntryApp::new(core)?;
        event_loop.run_app(app)?;
        Ok(())
    }
}
