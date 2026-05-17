use std::path::{Path, PathBuf};

use crate::settings::BuildProfile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildCommandOptions {
    pub python_path: PathBuf,
    pub cargo_path: PathBuf,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub profile: BuildProfile,
    pub jobs: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

impl BuildCommand {
    pub fn for_editor_runtime(options: &BuildCommandOptions) -> Self {
        let mut args = vec![
            options
                .source_dir
                .join("tools")
                .join("zircon_build.py")
                .to_string_lossy()
                .into_owned(),
            "--targets".to_string(),
            "editor,runtime".to_string(),
            "--out".to_string(),
            options.output_dir.to_string_lossy().into_owned(),
            "--mode".to_string(),
            options.profile.as_mode().to_string(),
            "--cargo".to_string(),
            options.cargo_path.to_string_lossy().into_owned(),
        ];
        if let Some(jobs) = options.jobs {
            args.push("--jobs".to_string());
            args.push(jobs.to_string());
        }
        Self {
            program: options.python_path.clone(),
            args,
            cwd: options.source_dir.clone(),
        }
    }

    pub fn command_line(&self) -> Vec<String> {
        std::iter::once(self.program.to_string_lossy().into_owned())
            .chain(self.args.iter().cloned())
            .collect()
    }
}

impl BuildCommandOptions {
    pub fn new(
        python_path: impl Into<PathBuf>,
        cargo_path: impl Into<PathBuf>,
        source_dir: impl Into<PathBuf>,
        output_dir: impl Into<PathBuf>,
        profile: BuildProfile,
        jobs: Option<u16>,
    ) -> Self {
        Self {
            python_path: python_path.into(),
            cargo_path: cargo_path.into(),
            source_dir: source_dir.into(),
            output_dir: output_dir.into(),
            profile,
            jobs,
        }
    }

    pub fn with_source_output(
        python_path: impl Into<PathBuf>,
        cargo_path: impl Into<PathBuf>,
        source_dir: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        profile: BuildProfile,
        jobs: Option<u16>,
    ) -> Self {
        Self::new(
            python_path,
            cargo_path,
            source_dir.as_ref().to_path_buf(),
            output_dir.as_ref().to_path_buf(),
            profile,
            jobs,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_matches_staged_tool_contract() {
        let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
            "python",
            "cargo-nextest",
            "E:/Git/ZirconEngine",
            "E:/build out",
            BuildProfile::Debug,
            Some(4),
        ));

        assert_eq!(command.program, PathBuf::from("python"));
        assert_eq!(command.cwd, PathBuf::from("E:/Git/ZirconEngine"));
        assert_eq!(
            PathBuf::from(&command.args[0]),
            PathBuf::from("E:/Git/ZirconEngine")
                .join("tools")
                .join("zircon_build.py")
        );
        assert_eq!(
            &command.args[1..],
            [
                "--targets",
                "editor,runtime",
                "--out",
                "E:/build out",
                "--mode",
                "debug",
                "--cargo",
                "cargo-nextest",
                "--jobs",
                "4",
            ]
        );
    }
}
