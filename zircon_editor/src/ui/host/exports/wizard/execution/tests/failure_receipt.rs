use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use zircon_runtime_interface::export::{
    ExportPipelineReport, ExportPreset, ExportStageStatus, ExportTargetMode,
};

use super::*;

struct Fixture {
    root: PathBuf,
    command: ExportWizardPipelineStageCommand,
    report_path: String,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-astra-export-receipt-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let repo = root.join("repo");
        // Minimal server source inventory for the real CompileHost executor.
        for relative in [
            "Cargo.toml",
            "Cargo.lock",
            "templates/fixture",
            "tools/zircon_build.py",
            "zircon_app/Cargo.toml",
            "zircon_app/build.rs",
            "zircon_app/src/lib.rs",
            "zircon_runtime/Cargo.toml",
            "zircon_runtime/build.rs",
            "zircon_runtime/src/lib.rs",
            "zircon_runtime/assets/fixture",
            "zircon_runtime/reflection_macros/fixture",
            "zircon_runtime/runtime-feature-presets.toml",
            "zircon_runtime_interface/Cargo.toml",
            "zircon_runtime_interface/src/lib.rs",
        ] {
            let path = repo.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, relative).unwrap();
        }
        let output = root.join("output");
        let staged = output.join("ZirconEngine");
        std::fs::create_dir_all(&staged).unwrap();
        std::fs::write(
            staged.join("previous-build"),
            b"old output must not mask failed exit",
        )
        .unwrap();
        let report = root.join("report.json").display().to_string();
        let report_path = format!("{report}.core.json");
        let command = ExportWizardPipelineStageCommand {
            stage: ExportStage::CompileHost,
            program: "unused".into(),
            working_dir: None,
            args: vec![],
            consumed_artifacts: vec![],
            produced_artifacts: vec![],
            expected_stdout_keys: vec![],
            missing_inputs: vec![],
            native_program: None,
            native_args: None,
            native_working_dir: None,
            core_projection: Some(ExportWizardCoreStageProjection::CompileHost {
                report_path: report,
                profile: "server".into(),
                host_path: "unused".into(),
                preset: ExportPreset::new("server", ExportTargetMode::ServerRuntime),
                repo_root: repo.display().to_string(),
                build_output_root: output.display().to_string(),
                // Only version probes execute; the injected runner owns the build command.
                python: "rustc".into(),
                cargo: "rustc".into(),
                locked: true,
                dry_run: false,
            }),
        };
        Self {
            root,
            command,
            report_path,
        }
    }

    fn run(
        &self,
        runner: &mut impl ExportWizardCommandRunner,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        run_core_compile_host(runner, &self.command, &mut |_| {}, &mut || false)
    }

    fn report(&self) -> ExportPipelineReport {
        load_core_pipeline_report(&self.report_path)
            .unwrap()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct StubRunner {
    exit_code: Option<i32>,
    calls: usize,
}

impl ExportWizardCommandRunner for StubRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.calls += 1;
        assert!(
            command.core_projection.is_none(),
            "adapter must not recurse into core"
        );
        Ok(ExportWizardCommandExecution {
            exit_code: self.exit_code,
            stdout_lines: vec!["build output".into()],
            stderr_lines: vec!["build diagnostic".into()],
        })
    }

    fn run_with_output_and_cancel(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        _emit: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
        should_cancel: &mut (dyn FnMut() -> bool + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        let _ = should_cancel();
        self.run(command)
    }
}

#[test]
fn astra_m3_failed_compile_persists_failure_retries_and_then_resumes_success() {
    let fixture = Fixture::new();
    let mut runner = StubRunner {
        exit_code: Some(23),
        calls: 0,
    };
    assert!(matches!(
        fixture.run(&mut runner),
        Err(EditorExportBuildError::WizardStageFailed {
            stage: ExportStage::CompileHost,
            exit_code: Some(23)
        })
    ));
    assert_eq!(
        fixture
            .report()
            .record(ExportStage::CompileHost)
            .unwrap()
            .status,
        ExportStageStatus::Failed
    );
    assert_eq!(runner.calls, 1);

    runner.exit_code = None;
    assert!(matches!(
        fixture.run(&mut runner),
        Err(EditorExportBuildError::WizardStageFailed {
            exit_code: None,
            ..
        })
    ));
    assert_eq!(
        fixture
            .report()
            .record(ExportStage::CompileHost)
            .unwrap()
            .status,
        ExportStageStatus::Failed
    );
    assert_eq!(runner.calls, 2);

    runner.exit_code = Some(0);
    assert_eq!(
        fixture.run(&mut runner).unwrap().stdout_lines,
        ["build output"]
    );
    assert_eq!(
        fixture
            .report()
            .record(ExportStage::CompileHost)
            .unwrap()
            .status,
        ExportStageStatus::Passed
    );
    assert_eq!(runner.calls, 3);

    runner.exit_code = Some(47);
    assert_eq!(fixture.run(&mut runner).unwrap().exit_code, Some(0));
    assert_eq!(runner.calls, 3, "qualified unchanged success should resume");
    assert_eq!(
        fixture
            .report()
            .record(ExportStage::CompileHost)
            .unwrap()
            .status,
        ExportStageStatus::Skipped
    );

    // A legacy Passed/Skipped report may have been produced by the broken adapter.
    let legacy = fixture.report();
    std::fs::write(&fixture.report_path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(load_core_pipeline_report(&fixture.report_path)
        .unwrap()
        .is_none());
    assert!(matches!(
        fixture.run(&mut runner),
        Err(EditorExportBuildError::WizardStageFailed {
            exit_code: Some(47),
            ..
        })
    ));
    assert_eq!(runner.calls, 4);
    assert_eq!(
        fixture
            .report()
            .record(ExportStage::CompileHost)
            .unwrap()
            .status,
        ExportStageStatus::Failed
    );
}

#[test]
fn astra_m3_corrupt_future_and_unreadable_receipts_fail_before_build() {
    let fixture = Fixture::new();
    let mut runner = StubRunner {
        exit_code: Some(0),
        calls: 0,
    };
    for bytes in [
        b"{broken".as_slice(),
        br#"{"format_version":999,"stages":[]}"#.as_slice(),
        br#"{"format_version":1,"stages":"invalid"}"#.as_slice(),
    ] {
        std::fs::write(&fixture.report_path, bytes).unwrap();
        assert!(
            matches!(fixture.run(&mut runner), Err(EditorExportBuildError::CoreArtifactFingerprint { source }) if source.kind() == std::io::ErrorKind::InvalidData)
        );
        assert_eq!(std::fs::read(&fixture.report_path).unwrap(), bytes);
        assert_eq!(runner.calls, 0);
    }
    std::fs::remove_file(&fixture.report_path).unwrap();
    std::fs::create_dir(&fixture.report_path).unwrap();
    assert!(load_core_pipeline_report(&fixture.report_path).is_err());
    assert_eq!(runner.calls, 0);
}

#[test]
fn astra_m3_versioned_report_round_trip_and_replace_preserve_records() {
    let fixture = Fixture::new();
    assert!(load_core_pipeline_report(&fixture.report_path)
        .unwrap()
        .is_none());
    let report = ExportPipelineReport::default();
    write_core_pipeline_report(&fixture.report_path, &report).unwrap();
    assert_eq!(fixture.report(), report);
    write_core_pipeline_report(&fixture.report_path, &report).unwrap();
    assert_eq!(fixture.report(), report);
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&fixture.report_path).unwrap()).unwrap();
    assert_eq!(value["format_version"], CORE_PIPELINE_RECEIPT_VERSION);
}

#[test]
fn astra_m3_process_cancellation_retains_cancelled_error_and_cannot_resume() {
    let fixture = Fixture::new();
    let mut runner = StubRunner {
        exit_code: None,
        calls: 0,
    };
    let mut checks = 0;
    let error = run_core_compile_host(&mut runner, &fixture.command, &mut |_| {}, &mut || {
        checks += 1;
        checks > 1
    })
    .unwrap_err();
    assert!(matches!(error, EditorExportBuildError::Cancelled { .. }));
    assert_eq!(runner.calls, 1);
    assert_eq!(
        fixture
            .report()
            .record(ExportStage::CompileHost)
            .unwrap()
            .status,
        ExportStageStatus::Failed
    );
    runner.exit_code = Some(0);
    fixture.run(&mut runner).unwrap();
    assert_eq!(runner.calls, 2);
}

#[test]
fn astra_m3_platform_failure_is_persisted_with_compile_success() {
    let fixture = Fixture::new();
    let mut runner = StubRunner {
        exit_code: Some(0),
        calls: 0,
    };
    fixture.run(&mut runner).unwrap();
    let mut command = fixture.command.clone();
    let Some(ExportWizardCoreStageProjection::CompileHost {
        preset,
        repo_root,
        build_output_root,
        python,
        cargo,
        locked,
        report_path,
        ..
    }) = command.core_projection.take()
    else {
        unreachable!()
    };
    command.core_projection = Some(ExportWizardCoreStageProjection::PlatformBundle {
        target_mode: preset.target_mode,
        dry_run: false,
        preset,
        repo_root,
        build_output_root,
        python,
        cargo,
        locked,
        report_path,
    });
    command.stage = ExportStage::PlatformBundle;
    assert!(matches!(
        run_core_platform_bundle(&command),
        Err(EditorExportBuildError::PlatformBundleLayout(_))
    ));
    let report = fixture.report();
    assert_eq!(
        report.record(ExportStage::CompileHost).unwrap().status,
        ExportStageStatus::Skipped
    );
    assert_eq!(
        report.record(ExportStage::PlatformBundle).unwrap().status,
        ExportStageStatus::Failed
    );
}

#[test]
fn astra_m3_failure_receipt_io_error_preserves_cause_and_retires_prior_success() {
    struct FailReportWrite {
        staging: PathBuf,
    }
    impl ExportWizardCommandRunner for FailReportWrite {
        fn run(
            &mut self,
            _command: &ExportWizardPipelineStageCommand,
        ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
            std::fs::create_dir(&self.staging).unwrap();
            Ok(ExportWizardCommandExecution {
                exit_code: Some(19),
                stdout_lines: vec![],
                stderr_lines: vec![],
            })
        }
    }
    let fixture = Fixture::new();
    let mut runner = StubRunner {
        exit_code: Some(0),
        calls: 0,
    };
    fixture.run(&mut runner).unwrap();
    // Change an input so the prior Passed receipt requires a new execution.
    std::fs::write(fixture.root.join("repo/Cargo.toml"), b"changed manifest").unwrap();
    let staging = PathBuf::from(&fixture.report_path)
        .with_extension(format!("core.json.{}.staging", std::process::id()));
    let error = fixture
        .run(&mut FailReportWrite {
            staging: staging.clone(),
        })
        .unwrap_err();
    assert!(
        matches!(error, EditorExportBuildError::CoreFailureReceipt { failure, .. }
        if matches!(*failure, EditorExportBuildError::WizardStageFailed { exit_code: Some(19), .. }))
    );
    assert!(
        fixture.report().stages.is_empty(),
        "the previous success must be retired before execution"
    );
    std::fs::remove_dir(staging).unwrap();
    fixture.run(&mut runner).unwrap();
    assert_eq!(
        runner.calls, 2,
        "failed receipt publication must not skip the retry"
    );
}
