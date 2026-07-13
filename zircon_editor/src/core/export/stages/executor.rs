use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use zircon_runtime_interface::export::{
    ExportArtifactRef, ExportDigest, ExportPreset, ExportStage, ExportStageRecord, ExportTargetMode,
};
use zircon_runtime_interface::serialization::write_versioned_text;

use super::super::{
    ExportPipelinePlan, ExportStageExecutor, ExportStageNode, ExportStageOutput,
    ExportStagePreparation,
};
use super::{
    CompileHostStage, PlatformBundleLayout, PlatformBundleLayoutError, ZirconBuildCommandRunner,
};

pub struct ZirconBuildStageExecutor<R> {
    preset: ExportPreset,
    compile_host: CompileHostStage,
    runner: R,
}

impl<R> ZirconBuildStageExecutor<R> {
    pub fn new(preset: ExportPreset, compile_host: CompileHostStage, runner: R) -> Self {
        Self {
            preset,
            compile_host,
            runner,
        }
    }

    pub fn into_runner(self) -> R {
        self.runner
    }
}

impl<R> ExportStageExecutor for ZirconBuildStageExecutor<R>
where
    R: ZirconBuildCommandRunner,
{
    type Error = ZirconBuildStageExecutorError<R::Error>;

    fn prepare(
        &mut self,
        stage: ExportStage,
        completed: &[ExportStageRecord],
    ) -> Result<ExportStagePreparation, Self::Error> {
        let parameter_digest = stage_parameter_digest(&self.preset, &self.compile_host)
            .map_err(ZirconBuildStageExecutorError::EncodePreset)?;
        match stage {
            ExportStage::CompileHost => Ok(ExportStagePreparation {
                inputs: compile_host_inputs(&self.compile_host, self.preset.target_mode)
                    .map_err(ZirconBuildStageExecutorError::Fingerprint)?,
                expected_outputs: vec![ExportArtifactRef::new(
                    "staged_engine_root",
                    self.compile_host.staged_engine_root().display().to_string(),
                )],
                parameter_digest,
            }),
            ExportStage::PlatformBundle => {
                let compile_host = completed
                    .iter()
                    .find(|record| record.stage == ExportStage::CompileHost)
                    .ok_or(ZirconBuildStageExecutorError::MissingCompileHostRecord)?;
                Ok(ExportStagePreparation {
                    inputs: compile_host.io.outputs.clone(),
                    expected_outputs: vec![ExportArtifactRef::new(
                        "bundle",
                        self.compile_host.staged_engine_root().display().to_string(),
                    )],
                    parameter_digest,
                })
            }
            unsupported => {
                Err(ZirconBuildStageExecutorError::UnsupportedStage { stage: unsupported })
            }
        }
    }

    fn execute(
        &mut self,
        stage: ExportStage,
        _preparation: &ExportStagePreparation,
    ) -> Result<ExportStageOutput, Self::Error> {
        match stage {
            ExportStage::CompileHost => {
                let execution = self
                    .compile_host
                    .execute(&self.preset, &mut self.runner)
                    .map_err(ZirconBuildStageExecutorError::Build)?;
                let output = ExportArtifactRef::new(
                    "staged_engine_root",
                    self.compile_host.staged_engine_root().display().to_string(),
                );
                let output = if self.compile_host.is_dry_run() {
                    output
                } else {
                    output.with_digest(
                        digest_path(&self.compile_host.staged_engine_root())
                            .map_err(ZirconBuildStageExecutorError::Fingerprint)?,
                    )
                };
                Ok(ExportStageOutput {
                    outputs: vec![output],
                    diagnostics: command_diagnostics(execution.stdout, execution.stderr),
                })
            }
            ExportStage::PlatformBundle => {
                let layout = PlatformBundleLayout::validate(
                    self.compile_host.output_root(),
                    self.preset.target_mode,
                )
                .map_err(ZirconBuildStageExecutorError::BundleLayout)?;
                let outputs = [
                    ("bundle", layout.engine_root.as_path()),
                    ("launcher", layout.launcher.as_path()),
                    ("runtime_library", layout.runtime_library.as_path()),
                    ("assets", layout.assets_root.as_path()),
                ]
                .into_iter()
                .map(|(key, path)| artifact_with_current_digest(key, path))
                .collect::<Result<Vec<_>, _>>()
                .map_err(ZirconBuildStageExecutorError::Fingerprint)?;
                Ok(ExportStageOutput {
                    outputs,
                    diagnostics: Vec::new(),
                })
            }
            unsupported => {
                Err(ZirconBuildStageExecutorError::UnsupportedStage { stage: unsupported })
            }
        }
    }

    fn can_reuse(
        &mut self,
        stage: ExportStage,
        previous: &ExportStageRecord,
        preparation: &ExportStagePreparation,
    ) -> bool {
        let expected_locator = preparation
            .expected_outputs
            .first()
            .map(|value| &value.locator);
        if previous.io.outputs.is_empty()
            || expected_locator.is_some_and(|expected| {
                previous.io.outputs.first().map(|value| &value.locator) != Some(expected)
            })
        {
            return false;
        }
        if stage == ExportStage::PlatformBundle
            && PlatformBundleLayout::validate(
                self.compile_host.output_root(),
                self.preset.target_mode,
            )
            .is_err()
        {
            return false;
        }
        previous.io.outputs.iter().all(artifact_matches_disk)
    }
}

#[derive(Debug)]
pub enum ZirconBuildStageExecutorError<E> {
    UnsupportedStage { stage: ExportStage },
    MissingCompileHostRecord,
    EncodePreset(zircon_runtime_interface::serialization::WriteError),
    Fingerprint(std::io::Error),
    Build(E),
    BundleLayout(PlatformBundleLayoutError),
}

impl<E: fmt::Display> fmt::Display for ZirconBuildStageExecutorError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedStage { stage } => write!(
                formatter,
                "zircon build executor does not own stage `{}`",
                stage.cli_id()
            ),
            Self::MissingCompileHostRecord => {
                formatter.write_str("PlatformBundle requires a completed CompileHost stage record")
            }
            Self::EncodePreset(source) => {
                write!(formatter, "failed to fingerprint export preset: {source}")
            }
            Self::Fingerprint(source) => {
                write!(
                    formatter,
                    "failed to fingerprint export inputs or outputs: {source}"
                )
            }
            Self::Build(source) => write!(formatter, "CompileHost failed: {source}"),
            Self::BundleLayout(source) => write!(formatter, "PlatformBundle failed: {source}"),
        }
    }
}

impl<E: Error + 'static> Error for ZirconBuildStageExecutorError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EncodePreset(source) => Some(source),
            Self::Fingerprint(source) => Some(source),
            Self::Build(source) => Some(source),
            Self::BundleLayout(source) => Some(source),
            Self::UnsupportedStage { .. } | Self::MissingCompileHostRecord => None,
        }
    }
}

fn stage_parameter_digest(
    preset: &ExportPreset,
    stage: &CompileHostStage,
) -> Result<ExportDigest, zircon_runtime_interface::serialization::WriteError> {
    let encoded = write_versioned_text(preset)?;
    let command = stage.command(preset);
    let mut hasher = blake3::Hasher::new();
    hasher.update(encoded.as_bytes());
    hash_os_string(&mut hasher, &command.program);
    for arg in &command.args {
        hash_os_string(&mut hasher, arg);
    }
    hash_os_string(&mut hasher, command.working_directory.as_os_str());
    Ok(ExportDigest::from_bytes(*hasher.finalize().as_bytes()))
}

#[cfg(windows)]
fn hash_os_string(hasher: &mut blake3::Hasher, value: &std::ffi::OsStr) {
    use std::os::windows::ffi::OsStrExt;
    for unit in value.encode_wide() {
        hasher.update(&unit.to_le_bytes());
    }
    hasher.update(&[0, 0]);
}

#[cfg(not(windows))]
fn hash_os_string(hasher: &mut blake3::Hasher, value: &std::ffi::OsStr) {
    use std::os::unix::ffi::OsStrExt;
    hasher.update(value.as_bytes());
    hasher.update(&[0]);
}

fn command_diagnostics(stdout: Vec<u8>, stderr: Vec<u8>) -> Vec<String> {
    [stdout, stderr]
        .into_iter()
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .collect()
}

fn compile_host_inputs(
    stage: &CompileHostStage,
    target_mode: ExportTargetMode,
) -> std::io::Result<Vec<ExportArtifactRef>> {
    let root = stage.repo_root();
    let mut inputs = compile_host_source_paths_for_target(target_mode)
        .into_iter()
        .map(|relative| artifact_with_current_digest(relative, &root.join(relative)))
        .collect::<Result<Vec<_>, _>>()?;
    for relative in [".cargo", "rust-toolchain.toml", "rust-toolchain"] {
        inputs.push(artifact_with_optional_digest(
            relative,
            &root.join(relative),
        )?);
    }
    let mut helpers = fs::read_dir(root.join("tools"))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("zircon_build") && name.ends_with(".py"))
        })
        .collect::<Vec<_>>();
    helpers.sort();
    for helper in helpers {
        let key = helper
            .strip_prefix(root)
            .unwrap_or(&helper)
            .display()
            .to_string();
        inputs.push(artifact_with_current_digest(key, &helper)?);
    }
    inputs.push(tool_identity(
        "python",
        stage.python_program(),
        &["--version"],
    )?);
    inputs.push(tool_identity(
        "cargo",
        stage.cargo_program(),
        &["--version"],
    )?);
    inputs.push(tool_identity(
        "rustc",
        std::ffi::OsStr::new("rustc"),
        &["--version", "--verbose"],
    )?);
    if target_requires_node_toolchain(target_mode) {
        inputs.push(tool_identity(
            "node",
            std::ffi::OsStr::new("node"),
            &["--version"],
        )?);
    }
    Ok(inputs)
}

pub(in crate::core::export) fn compile_host_source_paths_for_target(
    target_mode: ExportTargetMode,
) -> Vec<&'static str> {
    let mut paths = vec![
        "Cargo.toml",
        "Cargo.lock",
        "templates",
        "zircon_app/Cargo.toml",
        "zircon_app/build.rs",
        "zircon_app/src",
        "zircon_runtime/Cargo.toml",
        "zircon_runtime/build.rs",
        "zircon_runtime/src",
        "zircon_runtime/assets",
        "zircon_runtime/reflection_macros",
        "zircon_runtime/runtime-feature-presets.toml",
        "zircon_runtime_interface/Cargo.toml",
        "zircon_runtime_interface/src",
    ];
    if target_requires_node_toolchain(target_mode) {
        paths.extend([
            "zircon_editor/Cargo.toml",
            "zircon_editor/build.rs",
            "zircon_editor/src",
            "zircon_editor/assets",
            "zircon_editor/ui",
            "zircon_hub/Cargo.toml",
            "zircon_hub/build.rs",
            "zircon_hub/src",
            "zircon_hub/assets",
            "zircon_hub/capabilities",
            "zircon_hub/icons",
            "zircon_hub/web",
            "zircon_hub/gen",
            "zircon_hub/hub.toml",
            "zircon_hub/package-lock.json",
            "zircon_hub/package.json",
            "zircon_hub/tauri.conf.json",
            "zircon_hub/node_modules/@tauri-apps/cli/tauri.js",
            "zircon_hub/node_modules/@tauri-apps/cli/package.json",
        ]);
    }
    paths
}

pub(in crate::core::export) const fn target_requires_node_toolchain(
    target_mode: ExportTargetMode,
) -> bool {
    matches!(target_mode, ExportTargetMode::ClientRuntime)
}

fn tool_identity(
    key: &str,
    program: &std::ffi::OsStr,
    version_args: &[&str],
) -> std::io::Result<ExportArtifactRef> {
    let output = Command::new(program).args(version_args).output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "tool {:?} {:?} exited with {:?}",
            program,
            version_args,
            output.status.code()
        )));
    }
    let mut bytes = output.stdout;
    bytes.extend(output.stderr);
    Ok(
        ExportArtifactRef::new(format!("{key}_toolchain"), program.to_string_lossy())
            .with_digest(ExportDigest::from_bytes(*blake3::hash(&bytes).as_bytes())),
    )
}

fn artifact_with_current_digest(
    key: impl Into<String>,
    path: &Path,
) -> std::io::Result<ExportArtifactRef> {
    Ok(ExportArtifactRef::new(key, path.display().to_string()).with_digest(digest_path(path)?))
}

fn artifact_with_optional_digest(
    key: impl Into<String>,
    path: &Path,
) -> std::io::Result<ExportArtifactRef> {
    let digest = if path.exists() {
        digest_path(path)?
    } else {
        ExportDigest::from_bytes(*blake3::hash(b"<missing>").as_bytes())
    };
    Ok(ExportArtifactRef::new(key, path.display().to_string()).with_digest(digest))
}

fn artifact_matches_disk(artifact: &ExportArtifactRef) -> bool {
    artifact
        .digest
        .and_then(|expected| {
            digest_path(Path::new(&artifact.locator))
                .ok()
                .map(|actual| (expected, actual))
        })
        .is_some_and(|(expected, actual)| expected == actual)
}

fn digest_path(path: &Path) -> std::io::Result<ExportDigest> {
    let mut hasher = blake3::Hasher::new();
    digest_path_into(path, path, &mut hasher)?;
    Ok(ExportDigest::from_bytes(*hasher.finalize().as_bytes()))
}

fn digest_path_into(root: &Path, path: &Path, hasher: &mut blake3::Hasher) -> std::io::Result<()> {
    let metadata = fs::metadata(path)?;
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    hasher.update(&(relative.len() as u64).to_le_bytes());
    hasher.update(relative.as_bytes());
    if metadata.is_file() {
        hasher.update(&[1]);
        let bytes = fs::read(path)?;
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
        return Ok(());
    }
    hasher.update(&[2]);
    let mut children = fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<PathBuf>, _>>()?;
    children.sort();
    for child in children {
        digest_path_into(root, &child, hasher)?;
    }
    Ok(())
}

pub fn zircon_build_stage_plan() -> ExportPipelinePlan {
    ExportPipelinePlan::new([
        ExportStageNode::new(ExportStage::CompileHost, []),
        ExportStageNode::new(ExportStage::PlatformBundle, [ExportStage::CompileHost]),
    ])
    .expect("the fixed CompileHost to PlatformBundle graph is acyclic")
}
