use std::path::PathBuf;

use zircon_runtime::core::framework::project::ExportProfile;
use zircon_runtime::plugin::LibraryEmbedCompileHostPlan;
use zircon_runtime_interface::export::ExportStage;

use crate::core::export::{
    CompileHostStage, ExportPipelinePlan, ExportStageNode, PlatformBundleLayout,
};

use super::{
    export_pipeline_stages_for_strategies, ExportWizardCoreStageProjection,
    ExportWizardPipelineOptions, ExportWizardPipelineStageCommand, ExportWizardStageArtifactPath,
};

const ZIRCON_EXPORT_MODULE: &str = "tools.zircon_export";
const STAGES_DIR: &str = "stages";
const STDOUT_LOG_FILE_NAME: &str = "stdout.log";
const STDERR_LOG_FILE_NAME: &str = "stderr.log";
const OUTPUT_LOG_MANIFEST_FILE_NAME: &str = "output-log.json";
const COMPILE_HOST_STAGE_DIR: &str = "compile_host";
const COMPILE_HOST_TARGET_DIR: &str = "target";
const COMPILE_HOST_STAGED_DIR: &str = "staged";
const BUNDLE_DIR: &str = "bundle";
const REPORT_FILE_NAME: &str = "report.json";
const COOKED_ASSET_MANIFEST_NAME: &str = "assets.json";
const PACK_FILE_NAME: &str = "assets.zrpack";
const SOURCE_TEMPLATE_PROJECT_DIR: &str = "project";
const NATIVE_DYNAMIC_PLUGINS_DIR: &str = "plugins";
const NATIVE_DYNAMIC_LOADER_MANIFEST_NAME: &str = "native_plugins.toml";
const SOURCE_ASSET_MANIFEST_INPUT: &str = "source_asset_manifest";
const HOST_EXECUTABLE_INPUT: &str = "host_executable";
const DELTA_PACK_PAIR_INPUT: &str = "previous_pack+delta_pack";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPipelinePlan {
    pub profile: String,
    pub out: String,
    pub stages: Vec<ExportWizardPipelineStageCommand>,
    pub diagnostics: Vec<String>,
    core_plan: ExportPipelinePlan,
}

impl ExportWizardPipelinePlan {
    pub fn command(&self, stage: ExportStage) -> Option<&ExportWizardPipelineStageCommand> {
        self.stages.iter().find(|command| command.stage == stage)
    }

    pub fn is_ready(&self) -> bool {
        self.diagnostics.is_empty()
            && self
                .stages
                .iter()
                .all(|command| command.missing_inputs.is_empty())
    }

    pub fn ordered_commands(&self) -> impl Iterator<Item = &ExportWizardPipelineStageCommand> {
        self.core_plan
            .ordered_nodes()
            .iter()
            .filter_map(|node| self.command(node.stage))
    }

    pub fn unavailable(
        profile: impl Into<String>,
        out: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            profile: profile.into(),
            out: out.into(),
            stages: Vec::new(),
            diagnostics: vec![diagnostic.into()],
            core_plan: ExportPipelinePlan::new([])
                .expect("the unavailable export presentation has an empty valid graph"),
        }
    }
}

pub fn export_wizard_pipeline_plan(
    options: ExportWizardPipelineOptions,
) -> ExportWizardPipelinePlan {
    let diagnostics = pipeline_diagnostics(&options);
    let core_plan = planned_pipeline(&options);
    let planned_stages = core_plan
        .ordered_nodes()
        .iter()
        .map(|node| node.stage)
        .collect::<Vec<_>>();
    let stages = core_plan
        .ordered_nodes()
        .iter()
        .map(|node| stage_command(&options, node.stage, &planned_stages))
        .collect();
    ExportWizardPipelinePlan {
        profile: options.preset.profile_ref.clone(),
        out: options.out,
        stages,
        diagnostics,
        core_plan,
    }
}

pub fn export_wizard_compile_host_target_dir(out: &str) -> String {
    join_path(
        out,
        &[STAGES_DIR, COMPILE_HOST_STAGE_DIR, COMPILE_HOST_TARGET_DIR],
    )
}

pub fn export_wizard_compile_host_executable_path(
    out: &str,
    profile: &ExportProfile,
    target_dir: Option<&str>,
) -> String {
    let binary = LibraryEmbedCompileHostPlan::binary_for_target_mode(profile.target_mode);
    let cargo_profile =
        LibraryEmbedCompileHostPlan::cargo_profile_for_build_mode(profile.build_mode);
    let executable = format!("{binary}{}", std::env::consts::EXE_SUFFIX);
    let target_dir = target_dir.map_or_else(
        || export_wizard_compile_host_target_dir(out),
        ToOwned::to_owned,
    );
    join_path(&target_dir, &[cargo_profile, executable.as_str()])
}

fn stage_command(
    options: &ExportWizardPipelineOptions,
    stage: ExportStage,
    planned_stages: &[ExportStage],
) -> ExportWizardPipelineStageCommand {
    let mut args = base_args(options, stage);
    let mut consumed_artifacts = Vec::new();
    let mut produced_artifacts = vec![
        artifact("report", stage_report_path(&options.out, stage)),
        artifact(
            "stdout_log",
            stage_output_artifact_path(&options.out, stage, STDOUT_LOG_FILE_NAME),
        ),
        artifact(
            "stderr_log",
            stage_output_artifact_path(&options.out, stage, STDERR_LOG_FILE_NAME),
        ),
        artifact(
            "output_log_manifest",
            stage_output_artifact_path(&options.out, stage, OUTPUT_LOG_MANIFEST_FILE_NAME),
        ),
    ];
    let mut expected_stdout_keys =
        vec!["report", "stdout_log", "stderr_log", "output_log_manifest"];
    let mut missing_inputs = Vec::new();
    let mut program = options.python.clone();
    let mut working_dir = options.repo_root.clone();
    let mut core_projection = None;
    let mut native_program = None;
    let mut native_args = None;
    let mut native_working_dir = None;

    match stage {
        ExportStage::Validate => {
            push_option(&mut args, "--validator", options.validator.as_deref());
        }
        ExportStage::SourceTemplate => {
            let validate_report = validate_report_path(&options.out);
            push_required_option(&mut args, "--validate-report", &validate_report);
            if options.source_template_build {
                args.push("--source-template-build".to_string());
            }
            consumed_artifacts.push(artifact("validate_report", validate_report));
            produced_artifacts.push(artifact(
                "project",
                join_path(
                    &options.out,
                    &[STAGES_DIR, stage.cli_id(), SOURCE_TEMPLATE_PROJECT_DIR],
                ),
            ));
            expected_stdout_keys.extend(["validate_report", "project"]);
        }
        ExportStage::NativeDynamic => {
            let validate_report = validate_report_path(&options.out);
            push_required_option(&mut args, "--validate-report", &validate_report);
            consumed_artifacts.push(artifact("validate_report", validate_report));
            produced_artifacts.push(artifact(
                "plugins_dir",
                native_dynamic_plugins_dir_path(&options.out),
            ));
            produced_artifacts.push(artifact(
                "loader_manifest",
                native_dynamic_loader_manifest_path(&options.out),
            ));
            expected_stdout_keys.extend([
                "validate_report",
                "native_plugin_root",
                "stage_output",
                "loader_manifest",
            ]);
        }
        ExportStage::CompileHost => {
            let repo_root = options.repo_root.clone().unwrap_or_else(|| ".".to_string());
            let build_output_root = compile_host_build_output_root(&options.out);
            let mut compile_host = CompileHostStage::new(&repo_root, &build_output_root)
                .with_python(&options.python)
                .with_cargo(options.cargo.as_deref().unwrap_or("cargo"));
            if options.no_locked {
                compile_host = compile_host.without_lock();
            }
            if options.dry_run {
                compile_host = compile_host.with_dry_run();
            }
            let command = compile_host.command(&options.preset);
            program = command.program.to_string_lossy().into_owned();
            working_dir = Some(command.working_directory.display().to_string());
            native_program = Some(command.program.clone());
            native_args = Some(command.args.clone());
            native_working_dir = Some(command.working_directory.clone());
            args = command
                .args
                .into_iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect();
            let layout =
                PlatformBundleLayout::expected(&build_output_root, options.preset.target_mode);
            let report_path = stage_report_path(&options.out, stage);
            produced_artifacts.push(artifact(
                "staged_engine_root",
                layout.engine_root.display().to_string(),
            ));
            produced_artifacts.push(artifact("host", layout.launcher.display().to_string()));
            expected_stdout_keys.push("host");
            core_projection = Some(ExportWizardCoreStageProjection::CompileHost {
                report_path,
                profile: options.preset.profile_ref.clone(),
                host_path: layout.launcher.display().to_string(),
                preset: options.preset.clone(),
                repo_root,
                build_output_root,
                python: options.python.clone(),
                cargo: options.cargo.clone().unwrap_or_else(|| "cargo".to_string()),
                locked: !options.no_locked,
                dry_run: options.dry_run,
            });
        }
        ExportStage::CookAssets => {
            if let Some(source_asset_manifest) = &options.source_asset_manifest {
                push_required_option(&mut args, "--asset-manifest", source_asset_manifest);
                consumed_artifacts.push(artifact(
                    "source_asset_manifest",
                    source_asset_manifest.clone(),
                ));
            } else {
                missing_inputs.push(SOURCE_ASSET_MANIFEST_INPUT);
            }
            produced_artifacts.push(artifact(
                "cooked_asset_manifest",
                cooked_asset_manifest_path(&options.out),
            ));
            expected_stdout_keys.extend(["source_asset_manifest", "cooked_asset_manifest"]);
        }
        ExportStage::Pack => {
            let cooked_asset_manifest = cooked_asset_manifest_path(&options.out);
            let pack_file = pack_file_path(options);
            push_option(&mut args, "--packer", options.packer.as_deref());
            push_required_option(&mut args, "--asset-manifest", &cooked_asset_manifest);
            push_required_option(&mut args, "--pack-file", &pack_file);
            push_option(
                &mut args,
                "--previous-pack",
                options.previous_pack.as_deref(),
            );
            push_option(&mut args, "--delta-pack", options.delta_pack.as_deref());
            if options.determinism_check {
                args.push("--determinism-check".to_string());
            }
            consumed_artifacts.push(artifact("asset_manifest", cooked_asset_manifest));
            if let Some(previous_pack) = &options.previous_pack {
                consumed_artifacts.push(artifact("previous_pack", previous_pack.clone()));
            }
            produced_artifacts.push(artifact("pack", pack_file));
            if let Some(delta_pack) = &options.delta_pack {
                produced_artifacts.push(artifact("delta_pack", delta_pack.clone()));
            }
            expected_stdout_keys.extend(["asset_manifest", "pack", "previous_pack", "delta_pack"]);
        }
        ExportStage::PlatformBundle => {
            let pack_file = pack_file_path(options);
            push_required_option(&mut args, "--pack-file", &pack_file);
            push_option(
                &mut args,
                "--host-executable",
                options.host_executable.as_deref(),
            );
            push_option(&mut args, "--template-dir", options.template_dir.as_deref());
            push_option(
                &mut args,
                "--engine-version",
                options.engine_version.as_deref(),
            );
            push_option(
                &mut args,
                "--target-platform",
                options.target_platform.as_deref(),
            );
            consumed_artifacts.push(artifact("pack", pack_file));
            if let Some(host_executable) = &options.host_executable {
                consumed_artifacts.push(artifact("host", host_executable.clone()));
            }
            if let Some(template_dir) = &options.template_dir {
                consumed_artifacts.push(artifact("template", template_dir.clone()));
            }
            if options.host_executable.is_none() && options.template_dir.is_none() {
                missing_inputs.push(HOST_EXECUTABLE_INPUT);
            }
            produced_artifacts.push(artifact(
                "bundle",
                join_path(&options.out, &[BUNDLE_DIR, &options.preset.profile_ref]),
            ));
            expected_stdout_keys.extend(["bundle", "host", "native_plugins", "pack", "template"]);
            core_projection = Some(ExportWizardCoreStageProjection::PlatformBundle {
                build_output_root: compile_host_build_output_root(&options.out),
                target_mode: options.preset.target_mode,
                dry_run: options.dry_run,
                preset: options.preset.clone(),
                repo_root: options.repo_root.clone().unwrap_or_else(|| ".".to_string()),
                python: options.python.clone(),
                cargo: options.cargo.clone().unwrap_or_else(|| "cargo".to_string()),
                locked: !options.no_locked,
                report_path: stage_report_path(&options.out, ExportStage::CompileHost),
            });
        }
        ExportStage::Report => {
            for dependency in planned_stages
                .iter()
                .copied()
                .filter(|dependency| *dependency != ExportStage::Report)
            {
                consumed_artifacts.push(artifact(
                    "report",
                    stage_report_path(&options.out, dependency),
                ));
            }
            produced_artifacts.push(artifact(
                "pipeline_report",
                pipeline_report_path(&options.out),
            ));
            expected_stdout_keys.push("pipeline_report");
        }
    }

    if stage != ExportStage::CompileHost {
        append_common_options(&mut args, options);
    }

    if stage == ExportStage::Pack
        && (options.previous_pack.is_some() ^ options.delta_pack.is_some())
    {
        missing_inputs.push(DELTA_PACK_PAIR_INPUT);
    }

    ExportWizardPipelineStageCommand {
        stage,
        program,
        working_dir,
        args,
        consumed_artifacts,
        produced_artifacts,
        expected_stdout_keys,
        missing_inputs,
        core_projection,
        native_program,
        native_args,
        native_working_dir,
    }
}

fn stage_output_artifact_path(out: &str, stage: ExportStage, file_name: &str) -> String {
    join_path(out, &[STAGES_DIR, stage.cli_id(), file_name])
}

fn planned_pipeline(options: &ExportWizardPipelineOptions) -> ExportPipelinePlan {
    let stages = planned_pipeline_stages(options);
    let nodes = stages.iter().enumerate().map(|(index, stage)| {
        ExportStageNode::new(
            *stage,
            index.checked_sub(1).map(|previous| stages[previous]),
        )
    });
    ExportPipelinePlan::new(nodes).expect("wizard stage selection preserves an acyclic order")
}

fn planned_pipeline_stages(options: &ExportWizardPipelineOptions) -> Vec<ExportStage> {
    match options.strategies.as_deref() {
        Some(strategies) => export_pipeline_stages_for_strategies(strategies),
        None => ExportStage::ALL.to_vec(),
    }
}

fn base_args(options: &ExportWizardPipelineOptions, stage: ExportStage) -> Vec<String> {
    vec![
        "-m".to_string(),
        ZIRCON_EXPORT_MODULE.to_string(),
        "--profile".to_string(),
        options.preset.profile_ref.clone(),
        "--preset".to_string(),
        options.preset_path.clone(),
        "--project".to_string(),
        options.project.clone(),
        "--out".to_string(),
        options.out.clone(),
        "--stage".to_string(),
        stage.cli_id().to_string(),
    ]
}

fn compile_host_build_output_root(out: &str) -> String {
    join_path(
        out,
        &[STAGES_DIR, COMPILE_HOST_STAGE_DIR, COMPILE_HOST_STAGED_DIR],
    )
}

fn append_common_options(args: &mut Vec<String>, options: &ExportWizardPipelineOptions) {
    push_option(args, "--repo-root", options.repo_root.as_deref());
    push_option(args, "--cargo", options.cargo.as_deref());
    push_option(args, "--target-dir", options.target_dir.as_deref());
    if options.offline {
        args.push("--offline".to_string());
    }
    if options.no_locked {
        args.push("--no-locked".to_string());
    }
    if options.pretty {
        args.push("--pretty".to_string());
    }
    if options.dry_run {
        args.push("--dry-run".to_string());
    }
}

fn push_option(args: &mut Vec<String>, option: &str, value: Option<&str>) {
    if let Some(value) = value {
        push_required_option(args, option, value);
    }
}

fn push_required_option(args: &mut Vec<String>, option: &str, value: impl AsRef<str>) {
    args.push(option.to_string());
    args.push(value.as_ref().to_string());
}

fn pipeline_diagnostics(options: &ExportWizardPipelineOptions) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if options.previous_pack.is_some() ^ options.delta_pack.is_some() {
        diagnostics.push(
            "Pack delta execution requires both previous_pack and delta_pack paths".to_string(),
        );
    }
    diagnostics
}

fn artifact(key: impl Into<String>, path: impl Into<String>) -> ExportWizardStageArtifactPath {
    ExportWizardStageArtifactPath {
        key: key.into(),
        path: path.into(),
    }
}

fn validate_report_path(out: &str) -> String {
    stage_report_path(out, ExportStage::Validate)
}

fn stage_report_path(out: &str, stage: ExportStage) -> String {
    join_path(out, &[STAGES_DIR, stage.cli_id(), REPORT_FILE_NAME])
}

fn cooked_asset_manifest_path(out: &str) -> String {
    join_path(
        out,
        &[STAGES_DIR, "cook_assets", COOKED_ASSET_MANIFEST_NAME],
    )
}

fn native_dynamic_plugins_dir_path(out: &str) -> String {
    join_path(
        out,
        &[
            STAGES_DIR,
            ExportStage::NativeDynamic.cli_id(),
            NATIVE_DYNAMIC_PLUGINS_DIR,
        ],
    )
}

fn native_dynamic_loader_manifest_path(out: &str) -> String {
    join_path(
        out,
        &[
            STAGES_DIR,
            ExportStage::NativeDynamic.cli_id(),
            NATIVE_DYNAMIC_PLUGINS_DIR,
            NATIVE_DYNAMIC_LOADER_MANIFEST_NAME,
        ],
    )
}

fn pack_file_path(options: &ExportWizardPipelineOptions) -> String {
    options
        .pack_file
        .clone()
        .unwrap_or_else(|| join_path(&options.out, &[STAGES_DIR, "pack", PACK_FILE_NAME]))
}

fn pipeline_report_path(out: &str) -> String {
    join_path(out, &[REPORT_FILE_NAME])
}

fn join_path(root: &str, parts: &[&str]) -> String {
    let mut path = PathBuf::from(root);
    for part in parts {
        path.push(part);
    }
    path.to_string_lossy().into_owned()
}
