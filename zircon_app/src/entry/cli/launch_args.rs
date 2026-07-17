use zircon_editor::core::commandlet::{parse_commandlet_args, CommandletReport, CommandletRequest};

/// The first routing decision in the editor executable. Commandlets are recognized before GUI
/// startup parsing so a headless task never instantiates an editor host or workbench.
#[derive(Clone, Debug)]
pub(crate) enum EditorLaunchArgs {
    Commandlet(CommandletRequest),
    CommandletRejected(CommandletReport),
    Standard(Vec<String>),
}

impl EditorLaunchArgs {
    pub(crate) fn parse<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        match parse_commandlet_args(args.iter().cloned()) {
            Ok(Some(request)) => Self::Commandlet(request),
            Ok(None) => Self::Standard(args),
            Err(report) => Self::CommandletRejected(report),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EditorLaunchArgs;
    use zircon_editor::core::commandlet::{CommandletExitCode, CommandletStatus};

    #[test]
    fn unified_launch_args_route_run_to_the_editor_core_commandlet() {
        let args = EditorLaunchArgs::parse([
            "--run",
            "migrate-assets",
            "--project",
            "fixture",
            "--dry-run",
        ]);

        let EditorLaunchArgs::Commandlet(request) = args else {
            panic!("--run should route to the editor core commandlet");
        };
        assert_eq!(request.command(), "migrate-assets");
    }

    #[test]
    fn unified_launch_args_preserve_json_parameter_errors_for_commandlets() {
        let args =
            EditorLaunchArgs::parse(["--run", "unknown", "--project", "fixture", "--dry-run"]);

        let EditorLaunchArgs::CommandletRejected(report) = args else {
            panic!("unknown --run target should return the commandlet JSON report");
        };
        assert_eq!(report.exit_code(), CommandletExitCode::InvalidArguments);
        assert_eq!(report.status(), CommandletStatus::InvalidArguments);
    }
}
