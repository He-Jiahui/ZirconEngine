use std::collections::BTreeMap;
use std::fmt;

use zircon_runtime_interface::export::{
    ExportArtifactRef, ExportDigest, ExportStage, ExportStageStatus,
};

use super::{
    zircon_build_stage_plan, CompileHostStage, ExportPipelinePlan, ExportPresetStore,
    ExportStageExecutor, ExportStageNode, ExportStageOutput, ExportStagePreparation,
    PlatformBundleLayout, ZirconBuildCommand, ZirconBuildCommandExecution,
    ZirconBuildCommandRunner, ZirconBuildStageExecutor,
};

#[test]
fn server_compile_host_inventory_excludes_client_only_hub_tooling() {
    let server = super::stages::compile_host_source_paths_for_target(
        zircon_runtime_interface::export::ExportTargetMode::ServerRuntime,
    );

    assert!(server.contains(&"zircon_runtime/src"));
    assert!(!server.iter().any(|path| path.starts_with("zircon_hub/")));
    assert!(!server.iter().any(|path| path.starts_with("zircon_editor/")));
    assert!(!super::stages::target_requires_node_toolchain(
        zircon_runtime_interface::export::ExportTargetMode::ServerRuntime,
    ));
}

#[test]
fn pipeline_rejects_missing_dependencies_and_cycles() {
    let missing = ExportPipelinePlan::new([ExportStageNode::new(
        ExportStage::Pack,
        [ExportStage::CookAssets],
    )]);
    assert!(matches!(
        missing,
        Err(super::ExportPipelinePlanError::MissingDependency {
            stage: ExportStage::Pack,
            dependency: ExportStage::CookAssets,
        })
    ));

    let cycle = ExportPipelinePlan::new([
        ExportStageNode::new(ExportStage::Validate, [ExportStage::Pack]),
        ExportStageNode::new(ExportStage::Pack, [ExportStage::Validate]),
    ]);
    assert!(matches!(
        cycle,
        Err(super::ExportPipelinePlanError::DependencyCycle { .. })
    ));
}

#[test]
fn identical_fingerprints_skip_every_completed_stage() {
    let plan = two_stage_plan();
    let mut initial = FixtureExecutor::default();
    let first = plan.run(&mut initial, None).unwrap();
    assert_eq!(
        initial.executions,
        vec![ExportStage::Validate, ExportStage::Pack]
    );

    let mut resumed = FixtureExecutor::default();
    let second = plan.run(&mut resumed, Some(&first)).unwrap();
    assert!(resumed.executions.is_empty());
    assert!(second
        .stages
        .iter()
        .all(|record| record.status == ExportStageStatus::Skipped));
}

#[test]
fn expected_output_identity_prevents_cross_directory_resume() {
    let plan = two_stage_plan();
    let mut initial = FixtureExecutor::default();
    let first = plan.run(&mut initial, None).unwrap();

    let mut moved = FixtureExecutor {
        output_root: "different-dist".to_string(),
        ..FixtureExecutor::default()
    };
    let second = plan.run(&mut moved, Some(&first)).unwrap();

    assert_eq!(
        moved.executions,
        vec![ExportStage::Validate, ExportStage::Pack]
    );
    assert!(second
        .stages
        .iter()
        .all(|record| record.status == ExportStageStatus::Passed));
}

#[test]
fn changed_upstream_output_reruns_that_stage_and_its_consumer() {
    let plan = two_stage_plan();
    let mut initial = FixtureExecutor::default();
    let first = plan.run(&mut initial, None).unwrap();

    let mut changed = FixtureExecutor {
        validate_parameter: digest(9),
        ..FixtureExecutor::default()
    };
    let second = plan.run(&mut changed, Some(&first)).unwrap();

    assert_eq!(
        changed.executions,
        vec![ExportStage::Validate, ExportStage::Pack]
    );
    assert!(second
        .stages
        .iter()
        .all(|record| record.status == ExportStageStatus::Passed));
}

#[test]
fn failed_stage_is_recorded_and_resume_restarts_from_that_stage() {
    let plan = two_stage_plan();
    let mut failing = FixtureExecutor {
        fail_stage: Some(ExportStage::Pack),
        ..FixtureExecutor::default()
    };
    let failure = plan.run(&mut failing, None).unwrap_err();
    assert_eq!(failure.stage(), ExportStage::Pack);
    assert_eq!(
        failure
            .report()
            .record(ExportStage::Validate)
            .unwrap()
            .status,
        ExportStageStatus::Passed
    );
    assert_eq!(
        failure.report().record(ExportStage::Pack).unwrap().status,
        ExportStageStatus::Failed
    );

    let mut resumed = FixtureExecutor::default();
    let report = plan.run(&mut resumed, Some(failure.report())).unwrap();
    assert_eq!(resumed.executions, vec![ExportStage::Pack]);
    assert_eq!(
        report.record(ExportStage::Validate).unwrap().status,
        ExportStageStatus::Skipped
    );
    assert_eq!(
        report.record(ExportStage::Pack).unwrap().status,
        ExportStageStatus::Passed
    );
}

#[test]
fn preset_store_round_trips_and_replaces_versioned_zpreset_files() {
    let fixture = PresetStoreFixture::new();
    let store = ExportPresetStore::new(&fixture.root);
    let mut preset = zircon_runtime_interface::export::ExportPreset::new(
        "desktop_windows",
        zircon_runtime_interface::export::ExportTargetMode::ClientRuntime,
    );
    let path = store.save("shipping", &preset).unwrap();
    assert_eq!(path, fixture.root.join("export/shipping.zpreset"));
    assert_eq!(store.load("shipping").unwrap(), preset);

    preset.debug = true;
    store.save("shipping", &preset).unwrap();
    assert!(store.load("shipping").unwrap().debug);
}

#[test]
fn preset_store_rejects_names_that_can_escape_the_export_directory() {
    let fixture = PresetStoreFixture::new();
    let store = ExportPresetStore::new(&fixture.root);
    assert!(matches!(
        store.preset_path("../shipping"),
        Err(super::ExportPresetStoreError::InvalidName { .. })
    ));
}

#[test]
fn compile_host_command_wraps_zircon_build_with_preset_target_mode() {
    let mut preset = zircon_runtime_interface::export::ExportPreset::new(
        "desktop_windows",
        zircon_runtime_interface::export::ExportTargetMode::ClientRuntime,
    );
    preset.debug = true;
    let stage = CompileHostStage::new("E:/Git/ZirconEngine", "D:/export/compile_host")
        .with_python("py")
        .with_cargo("cargo-nextest");
    let command = stage.command(&preset);
    let args = command
        .args
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(command.program, std::ffi::OsString::from("py"));
    assert_eq!(
        argument_value(&args, "--targets"),
        Some("hub,editor,runtime")
    );
    assert_eq!(argument_value(&args, "--mode"), Some("debug"));
    assert_eq!(
        argument_value(&args, "--runtime-features"),
        Some("target-client")
    );
    assert_eq!(argument_value(&args, "--cargo"), Some("cargo-nextest"));
}

#[test]
fn platform_bundle_requires_merged_assets_runtime_library_editor_and_hub() {
    let fixture = PresetStoreFixture::new();
    let engine = fixture.root.join("ZirconEngine");
    std::fs::create_dir_all(engine.join("assets")).unwrap();
    for file in [
        runtime_library_name_for_test().to_owned(),
        executable_name_for_test("zircon_editor"),
        executable_name_for_test("zircon_hub"),
    ] {
        std::fs::write(engine.join(file), b"fixture").unwrap();
    }

    let layout = PlatformBundleLayout::validate(
        &fixture.root,
        zircon_runtime_interface::export::ExportTargetMode::ClientRuntime,
    )
    .unwrap();
    assert_eq!(layout.assets_root, engine.join("assets"));
    assert_eq!(layout.runtime_library.parent(), Some(engine.as_path()));
    assert_eq!(
        layout.launcher,
        engine.join(executable_name_for_test("zircon_hub"))
    );
}

#[test]
fn zircon_build_executor_runs_compile_host_then_validates_platform_bundle() {
    let fixture = PresetStoreFixture::new();
    let engine = fixture.root.join("ZirconEngine");
    std::fs::create_dir_all(engine.join("assets")).unwrap();
    for file in [
        runtime_library_name_for_test().to_owned(),
        executable_name_for_test("zircon_editor"),
        executable_name_for_test("zircon_hub"),
    ] {
        std::fs::write(engine.join(file), b"fixture").unwrap();
    }
    let preset = zircon_runtime_interface::export::ExportPreset::new(
        "desktop_windows",
        zircon_runtime_interface::export::ExportTargetMode::ClientRuntime,
    );
    let compile_host = CompileHostStage::new("E:/Git/ZirconEngine", &fixture.root);
    let runner = RecordingBuildRunner::default();
    let mut executor = ZirconBuildStageExecutor::new(preset, compile_host, runner);

    let report = zircon_build_stage_plan().run(&mut executor, None).unwrap();
    let runner = executor.into_runner();

    assert_eq!(runner.commands.len(), 1);
    assert_eq!(
        report.record(ExportStage::CompileHost).unwrap().status,
        ExportStageStatus::Passed
    );
    assert_eq!(
        report.record(ExportStage::PlatformBundle).unwrap().status,
        ExportStageStatus::Passed
    );
    assert!(report
        .record(ExportStage::PlatformBundle)
        .unwrap()
        .io
        .outputs
        .iter()
        .any(|artifact| artifact.key == "launcher"));
}

#[test]
fn zircon_build_resume_reexecutes_when_staged_output_is_deleted_or_tampered() {
    let fixture = PresetStoreFixture::new();
    write_client_layout(&fixture.root, b"fixture");
    let preset = zircon_runtime_interface::export::ExportPreset::new(
        "desktop_windows",
        zircon_runtime_interface::export::ExportTargetMode::ClientRuntime,
    );
    let compile_host = CompileHostStage::new("E:/Git/ZirconEngine", &fixture.root);
    let mut initial = ZirconBuildStageExecutor::new(
        preset.clone(),
        compile_host.clone(),
        RecordingBuildRunner::default(),
    );
    let first = zircon_build_stage_plan().run(&mut initial, None).unwrap();

    let runtime = fixture
        .root
        .join("ZirconEngine")
        .join(runtime_library_name_for_test());
    std::fs::write(&runtime, b"tampered").unwrap();
    let mut resumed = ZirconBuildStageExecutor::new(
        preset.clone(),
        compile_host.clone(),
        RecordingBuildRunner::default(),
    );
    let report = zircon_build_stage_plan()
        .run(&mut resumed, Some(&first))
        .unwrap();
    assert_eq!(resumed.into_runner().commands.len(), 1);
    assert_eq!(
        report.record(ExportStage::CompileHost).unwrap().status,
        ExportStageStatus::Passed
    );

    std::fs::remove_file(&runtime).unwrap();
    let mut missing =
        ZirconBuildStageExecutor::new(preset, compile_host, RecordingBuildRunner::default());
    assert!(zircon_build_stage_plan()
        .run(&mut missing, Some(&report))
        .is_err());
    assert_eq!(missing.into_runner().commands.len(), 1);
}

#[test]
fn zircon_build_resume_cannot_reuse_a_different_output_root() {
    let first_fixture = PresetStoreFixture::new();
    let second_fixture = PresetStoreFixture::new();
    write_client_layout(&first_fixture.root, b"first");
    write_client_layout(&second_fixture.root, b"second");
    let preset = zircon_runtime_interface::export::ExportPreset::new(
        "desktop_windows",
        zircon_runtime_interface::export::ExportTargetMode::ClientRuntime,
    );
    let mut first = ZirconBuildStageExecutor::new(
        preset.clone(),
        CompileHostStage::new("E:/Git/ZirconEngine", &first_fixture.root),
        RecordingBuildRunner::default(),
    );
    let report = zircon_build_stage_plan().run(&mut first, None).unwrap();

    let mut second = ZirconBuildStageExecutor::new(
        preset,
        CompileHostStage::new("E:/Git/ZirconEngine", &second_fixture.root),
        RecordingBuildRunner::default(),
    );
    let resumed = zircon_build_stage_plan()
        .run(&mut second, Some(&report))
        .unwrap();

    assert_eq!(second.into_runner().commands.len(), 1);
    assert_eq!(
        resumed.record(ExportStage::CompileHost).unwrap().status,
        ExportStageStatus::Passed
    );
}

fn two_stage_plan() -> ExportPipelinePlan {
    ExportPipelinePlan::new([
        ExportStageNode::new(ExportStage::Validate, []),
        ExportStageNode::new(ExportStage::Pack, [ExportStage::Validate]),
    ])
    .unwrap()
}

#[derive(Default)]
struct FixtureExecutor {
    validate_parameter: ExportDigest,
    fail_stage: Option<ExportStage>,
    executions: Vec<ExportStage>,
    outputs: BTreeMap<ExportStage, ExportArtifactRef>,
    output_root: String,
}

impl ExportStageExecutor for FixtureExecutor {
    type Error = FixtureError;

    fn prepare(
        &mut self,
        stage: ExportStage,
        completed: &[zircon_runtime_interface::export::ExportStageRecord],
    ) -> Result<ExportStagePreparation, Self::Error> {
        let inputs = completed
            .last()
            .and_then(|record| record.io.outputs.first())
            .cloned()
            .into_iter()
            .collect();
        let parameter_digest = if stage == ExportStage::Validate {
            self.validate_parameter
        } else {
            digest(2)
        };
        Ok(ExportStagePreparation {
            inputs,
            expected_outputs: vec![ExportArtifactRef::new(
                stage.cli_id(),
                format!("{}/{}", self.output_root_or_default(), stage.cli_id()),
            )],
            parameter_digest,
        })
    }

    fn execute(
        &mut self,
        stage: ExportStage,
        preparation: &ExportStagePreparation,
    ) -> Result<ExportStageOutput, Self::Error> {
        self.executions.push(stage);
        if self.fail_stage == Some(stage) {
            return Err(FixtureError(stage));
        }
        let output = preparation.expected_outputs[0].clone().with_digest(
            if stage == ExportStage::Validate {
                self.validate_parameter
            } else {
                digest(3)
            },
        );
        self.outputs.insert(stage, output.clone());
        Ok(ExportStageOutput {
            outputs: vec![output],
            diagnostics: Vec::new(),
        })
    }
}

impl FixtureExecutor {
    fn output_root_or_default(&self) -> &str {
        if self.output_root.is_empty() {
            "dist"
        } else {
            &self.output_root
        }
    }
}

#[derive(Debug)]
struct FixtureError(ExportStage);

impl fmt::Display for FixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fixture stage {} failed", self.0.cli_id())
    }
}

impl std::error::Error for FixtureError {}

fn digest(byte: u8) -> ExportDigest {
    ExportDigest::from_bytes([byte; 32])
}

fn argument_value<'a>(args: &'a [String], option: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == option)
        .map(|window| window[1].as_str())
}

fn executable_name_for_test(stem: &str) -> String {
    format!("{stem}{}", std::env::consts::EXE_SUFFIX)
}

#[cfg(target_os = "windows")]
fn runtime_library_name_for_test() -> &'static str {
    "zircon_runtime.dll"
}

#[derive(Default)]
struct RecordingBuildRunner {
    commands: Vec<ZirconBuildCommand>,
}

impl ZirconBuildCommandRunner for RecordingBuildRunner {
    type Error = std::convert::Infallible;

    fn run(
        &mut self,
        command: &ZirconBuildCommand,
    ) -> Result<ZirconBuildCommandExecution, Self::Error> {
        self.commands.push(command.clone());
        Ok(ZirconBuildCommandExecution {
            stdout: b"staged build ready".to_vec(),
            stderr: Vec::new(),
        })
    }
}

#[cfg(target_os = "macos")]
fn runtime_library_name_for_test() -> &'static str {
    "libzircon_runtime.dylib"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn runtime_library_name_for_test() -> &'static str {
    "libzircon_runtime.so"
}

struct PresetStoreFixture {
    root: std::path::PathBuf,
}

impl PresetStoreFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-export-preset-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }
}

fn fixture_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}

impl Drop for PresetStoreFixture {
    fn drop(&mut self) {
        let temp = std::env::temp_dir();
        if self.root.starts_with(&temp)
            && self
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("zircon-editor-export-preset-"))
        {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

fn write_client_layout(root: &std::path::Path, contents: &[u8]) {
    let engine = root.join("ZirconEngine");
    std::fs::create_dir_all(engine.join("assets")).unwrap();
    for file in [
        runtime_library_name_for_test().to_owned(),
        executable_name_for_test("zircon_editor"),
        executable_name_for_test("zircon_hub"),
    ] {
        std::fs::write(engine.join(file), contents).unwrap();
    }
}
