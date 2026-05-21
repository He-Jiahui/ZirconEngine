use std::error::Error;

use winit::event_loop::EventLoop;
use zircon_runtime::core::framework::window::{
    WindowDescriptor, WindowExitCondition, WindowLifecyclePolicy,
};
use zircon_runtime::platform::EventLoopPolicy;

use super::super::runtime_entry_app::{RuntimeEntryApp, RuntimeEntryAppConfig};
use super::super::runtime_library::{LoadedRuntime, RuntimeSession};
use super::diagnostic_log_args::parse_diagnostic_log_startup_args;
use super::runtime_session_args::{
    parse_runtime_session_startup_args, RuntimeSessionProfile, RUNTIME_SESSION_STARTUP_HELP,
};
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
        let runtime_session_args =
            parse_runtime_session_startup_args(diagnostic_args.remaining_args)?;
        if runtime_session_args.help_requested {
            println!("{RUNTIME_SESSION_STARTUP_HELP}");
            return Ok(());
        }
        if !runtime_session_args.remaining_args.is_empty() {
            return Err(format!(
                "unknown runtime argument `{}`",
                runtime_session_args.remaining_args[0]
            )
            .into());
        }
        zircon_runtime::diagnostic_log::initialize_unity_process_log_with_config(
            "runtime",
            diagnostic_args.filter,
        );
        #[cfg(feature = "profiling-tracy")]
        let _ = zircon_runtime::core::diagnostics::profiling::initialize_tracy_sink();
        #[cfg(feature = "profiling")]
        let profile_capture =
            zircon_runtime::core::diagnostics::profiling::start_capture_from_env("runtime");
        let runtime = LoadedRuntime::load_default()?;
        let session =
            RuntimeSession::create_with_profile(runtime, runtime_session_args.profile.as_bytes())?;
        let host_config =
            runtime_entry_app_config_for_session_profile(runtime_session_args.profile);
        let event_loop = EventLoop::new()?;
        let app = RuntimeEntryApp::new(session, host_config);
        let result = event_loop.run_app(app);
        #[cfg(feature = "profiling")]
        if profile_capture.is_some() {
            match zircon_runtime::core::diagnostics::profiling::stop_and_export_capture_from_env() {
                Some(Ok(report)) => eprintln!("profile report exported: {}", report.export_dir),
                Some(Err(error)) => eprintln!("profile report export failed: {error}"),
                None => {}
            }
        }
        result?;
        Ok(())
    }
}

fn runtime_entry_app_config_for_session_profile(
    profile: RuntimeSessionProfile,
) -> RuntimeEntryAppConfig {
    match profile {
        RuntimeSessionProfile::Runtime => RuntimeEntryAppConfig::default(),
        RuntimeSessionProfile::Editor | RuntimeSessionProfile::Dev => {
            RuntimeEntryAppConfig::default().with_event_loop_policy(EventLoopPolicy::DesktopApp)
        }
        RuntimeSessionProfile::Minimal | RuntimeSessionProfile::Headless => {
            RuntimeEntryAppConfig::default()
                .with_window_descriptor(WindowDescriptor::default().without_primary_window())
                .with_event_loop_policy(EventLoopPolicy::Headless)
                .with_window_lifecycle_policy(
                    WindowLifecyclePolicy::default()
                        .with_exit_condition(WindowExitCondition::DontExit),
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_session_profile_selects_default_game_host_config() {
        let config = runtime_entry_app_config_for_session_profile(RuntimeSessionProfile::Runtime);

        assert!(config.window_descriptor().primary_window.is_some());
        assert_eq!(config.event_loop_policy(), EventLoopPolicy::Game);
        assert!(config
            .window_lifecycle_policy()
            .should_exit_after_primary_close());
    }

    #[test]
    fn editor_and_dev_profiles_select_desktop_app_event_loop_policy() {
        for profile in [RuntimeSessionProfile::Editor, RuntimeSessionProfile::Dev] {
            let config = runtime_entry_app_config_for_session_profile(profile);

            assert!(config.window_descriptor().primary_window.is_some());
            assert_eq!(config.event_loop_policy(), EventLoopPolicy::DesktopApp);
        }
    }

    #[test]
    fn minimal_and_headless_profiles_disable_primary_window_creation() {
        for profile in [
            RuntimeSessionProfile::Minimal,
            RuntimeSessionProfile::Headless,
        ] {
            let config = runtime_entry_app_config_for_session_profile(profile);

            assert_eq!(config.window_descriptor().primary_window, None);
            assert_eq!(config.event_loop_policy(), EventLoopPolicy::Headless);
            assert_eq!(
                config.window_lifecycle_policy().exit_condition,
                WindowExitCondition::DontExit
            );
        }
    }
}
