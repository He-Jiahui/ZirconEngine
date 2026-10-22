//! A short-lived native loading surface, independent of runtime/GPU initialization.
//!
//! The GUI entry owns its child and stdin lease. Stage updates do not drive engine
//! work; the retained host's first successful presentation is the readiness authority.

#[path = "startup_splash/child.rs"]
mod child;
#[path = "startup_splash/content.rs"]
pub(crate) mod content;
#[path = "startup_splash/paint.rs"]
mod paint;
#[path = "startup_splash/process.rs"]
mod process;
#[path = "startup_splash/stage.rs"]
mod stage;

pub use process::EditorStartupSplash;
pub use stage::EditorStartupStage;

/// Intercept only the private child route, before any runtime or diagnostic startup.
pub fn run_startup_splash_child(args: &[String]) -> Option<std::process::ExitCode> {
    (args.first().map(String::as_str) == Some(process::CHILD_ARGUMENT)).then(|| {
        match child::run(args.get(1).is_some_and(|value| value == "project")) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("[zircon_editor] startup splash unavailable: {error}");
                std::process::ExitCode::FAILURE
            }
        }
    })
}
