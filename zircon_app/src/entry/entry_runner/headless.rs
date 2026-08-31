use std::error::Error;

use crate::entry::{EntryConfig, EntryProfile};

use super::EntryRunner;

impl EntryRunner {
    pub fn run_headless() -> Result<(), Box<dyn Error>> {
        let _composition = Self::compose(EntryConfig::new(EntryProfile::Headless))?;
        Ok(())
    }
}
