use std::error::Error;

use winit::event_loop::EventLoop;

use super::super::runtime_entry_app::RuntimeEntryApp;
use super::super::runtime_library::{LoadedRuntime, RuntimeSession};
use super::diagnostic_log_args::parse_diagnostic_log_startup_args;
use super::EntryRunner;

impl EntryRunner {
    pub fn run_runtime() -> Result<(), Box<dyn Error>> {
        Self::run_runtime_with_args(std::iter::empty::<String>())
    }

    pub fn run_runtime_with_args<I, S>(args: I) -> Result<(), Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let diagnostic_args = parse_diagnostic_log_startup_args(args)?;
        if !diagnostic_args.remaining_args.is_empty() {
            return Err(format!(
                "unknown runtime argument `{}`",
                diagnostic_args.remaining_args[0]
            )
            .into());
        }
        zircon_runtime::diagnostic_log::initialize_unity_process_log_with_filter(
            "runtime",
            diagnostic_args.filter,
        );
        let runtime = LoadedRuntime::load_default()?;
        let session = RuntimeSession::create(runtime)?;
        let event_loop = EventLoop::new()?;
        let app = RuntimeEntryApp::new(session);
        event_loop.run_app(app)?;
        Ok(())
    }
}
