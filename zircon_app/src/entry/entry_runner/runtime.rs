use std::{env, error::Error};

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

const RUNTIME_EXIT_AFTER_FIRST_FRAME_ENV: &str = "ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME";

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
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_library_load_start");
        let runtime = LoadedRuntime::load_default()?;
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_library_load_done");
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_start");
        let session = RuntimeSession::create_with_profile_and_project(
            runtime,
            runtime_session_args.profile.as_bytes(),
            runtime_session_args.project_root.as_deref(),
        )?;
        zircon_runtime::diagnostic_log::write_log("runtime_app", "runtime_session_create_done");
        let host_config = runtime_entry_app_config_for_session_profile_with_first_frame_exit(
            runtime_session_args.profile,
            runtime_exit_after_first_frame_enabled(),
        );
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

#[cfg(test)]
fn runtime_entry_app_config_for_session_profile(
    profile: RuntimeSessionProfile,
) -> RuntimeEntryAppConfig {
    runtime_entry_app_config_for_session_profile_with_first_frame_exit(
        profile,
        runtime_exit_after_first_frame_enabled(),
    )
}

fn runtime_entry_app_config_for_session_profile_with_first_frame_exit(
    profile: RuntimeSessionProfile,
    exit_after_first_frame: bool,
) -> RuntimeEntryAppConfig {
    let config = match profile {
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
    };
    if exit_after_first_frame {
        config.with_exit_after_first_presented_frame(true)
    } else {
        config
    }
}

fn runtime_exit_after_first_frame_enabled() -> bool {
    env::var_os(RUNTIME_EXIT_AFTER_FIRST_FRAME_ENV).is_some()
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

    #[test]
    fn first_frame_exit_flag_projects_into_runtime_host_config() {
        let config = runtime_entry_app_config_for_session_profile_with_first_frame_exit(
            RuntimeSessionProfile::Runtime,
            true,
        );

        assert!(config.exit_after_first_presented_frame());
    }
}
