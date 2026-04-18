use std::error::Error;

use zircon_editor::run_editor;

use crate::entry::{EntryConfig, EntryProfile};

use super::EntryRunner;

impl EntryRunner {
    pub fn run_editor() -> Result<(), Box<dyn Error>> {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Editor))?;
        run_editor(core)?;
        Ok(())
    }
}
