use std::error::Error;

use winit::event_loop::EventLoop;
use zircon_editor::run_editor;

use crate::entry::{EntryConfig, EntryProfile};

use super::super::runtime_entry_app::RuntimeEntryApp;
use super::EntryRunner;

impl EntryRunner {
    pub fn run_editor() -> Result<(), Box<dyn Error>> {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Editor))?;
        run_editor(core)?;
        Ok(())
    }

    pub fn run_runtime() -> Result<(), Box<dyn Error>> {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Runtime))?;
        let event_loop = EventLoop::new()?;
        let mut app = RuntimeEntryApp::new(core)?;
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    pub fn run_headless() -> Result<(), Box<dyn Error>> {
        let _ = Self::bootstrap(EntryConfig::new(EntryProfile::Headless))?;
        Ok(())
    }
}
