use zircon_runtime_interface::export::ExportStage;

use zircon_editor::ExportStageProgressKind;

use crate::{
    EXPORT_TEMPLATE_ID, LIBRARY_EMBED_REPORT_DOCUMENT, LIBRARY_EMBED_REPORT_ID,
    NATIVE_DYNAMIC_REPORT_DOCUMENT, NATIVE_DYNAMIC_REPORT_ID, SOURCE_TEMPLATE_REPORT_DOCUMENT,
    SOURCE_TEMPLATE_REPORT_ID,
};

pub const BUILD_EXPORT_LAYOUT_REFERENCE: &str =
    "docs/ui-and-layout/ai-workbench-style/ai-build-export-layout.png";
pub const PIPELINE_REPORT_PATH: &str = "report.json";
pub const STAGE_REPORT_FILE: &str = "report.json";
pub const REPORT_PIPELINE_REPORT_ENTRY_KEY: &str = "report.pipeline_report";
pub const REPORT_EXPORT_PLAN_STRATEGIES_ENTRY_KEY: &str = "report.export_plan.strategies";
pub const REPORT_EXPORT_PLAN_REQUIRED_STAGES_ENTRY_KEY: &str = "report.export_plan.required_stages";
pub const REPORT_EXPORT_PLAN_COMPLETED_STAGES_ENTRY_KEY: &str =
    "report.export_plan.completed_stages";
pub const REPORT_EXPORT_PLAN_UNSUPPORTED_STRATEGIES_ENTRY_KEY: &str =
    "report.export_plan.unsupported_strategies";
pub const REPORT_NATIVE_PLUGINS_PAYLOAD_BUNDLE_PATH_ENTRY_KEY: &str =
    "report.native_plugins_payload.bundle_path";
pub const REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_COUNT_ENTRY_KEY: &str =
    "report.native_plugins_payload.package_count";
pub const REPORT_NATIVE_PLUGINS_PAYLOAD_FILE_COUNT_ENTRY_KEY: &str =
    "report.native_plugins_payload.file_count";
pub const REPORT_NATIVE_PLUGINS_PAYLOAD_CONTENT_HASH_ENTRY_KEY: &str =
    "report.native_plugins_payload.content_hash";
pub const REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_IDS_ENTRY_KEY: &str =
    "report.native_plugins_payload.package_ids";
pub const EXPORT_PLAN_REPORT_SUMMARY_ENTRY_KEYS: &[&str] = &[
    REPORT_PIPELINE_REPORT_ENTRY_KEY,
    REPORT_EXPORT_PLAN_STRATEGIES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_REQUIRED_STAGES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_COMPLETED_STAGES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_UNSUPPORTED_STRATEGIES_ENTRY_KEY,
];
pub const SOURCE_TEMPLATE_REPORT_SUMMARY_ENTRY_KEYS: &[&str] =
    EXPORT_PLAN_REPORT_SUMMARY_ENTRY_KEYS;
pub const LIBRARY_EMBED_REPORT_SUMMARY_ENTRY_KEYS: &[&str] = EXPORT_PLAN_REPORT_SUMMARY_ENTRY_KEYS;
pub const NATIVE_DYNAMIC_REPORT_SUMMARY_ENTRY_KEYS: &[&str] = &[
    REPORT_PIPELINE_REPORT_ENTRY_KEY,
    REPORT_EXPORT_PLAN_STRATEGIES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_REQUIRED_STAGES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_COMPLETED_STAGES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_UNSUPPORTED_STRATEGIES_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_BUNDLE_PATH_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_COUNT_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_FILE_COUNT_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_CONTENT_HASH_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_IDS_ENTRY_KEY,
];
pub const SOURCE_TEMPLATE_REPORT_TEMPLATE_CONTROL_IDS: &[&str] = &[
    "SourceTemplateReportRoot",
    "SourceTemplateReportSummary",
    "SourceTemplateProjectPath",
    "SourceTemplateCargoCommand",
    "SourceTemplateGeneratedFiles",
    "SourceTemplateDiagnostics",
];
pub const LIBRARY_EMBED_REPORT_TEMPLATE_CONTROL_IDS: &[&str] = &[
    "LibraryEmbedReportRoot",
    "LibraryEmbedReportSummary",
    "LibraryEmbedCompileHost",
    "LibraryEmbedFeatureMatrix",
    "LibraryEmbedBundleStatus",
    "LibraryEmbedDiagnostics",
];
pub const NATIVE_DYNAMIC_REPORT_TEMPLATE_CONTROL_IDS: &[&str] = &[
    "NativeDynamicReportRoot",
    "NativeDynamicReportSummary",
    "NativeDynamicAbiSummary",
    "NativeDynamicPackageList",
    "NativeDynamicLoaderManifest",
    "NativeDynamicDiagnostics",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardRegion {
    ProfileTree,
    StageTimeline,
    ReportInspector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardAction {
    GeneratePlan,
    RunStage,
    OpenReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportWizardRegionDescriptor {
    pub region: ExportWizardRegion,
    pub label: &'static str,
    pub role: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportWizardStageDescriptor {
    pub stage: ExportStage,
    pub stage_id: &'static str,
    pub label: &'static str,
    pub report_path: &'static str,
    pub primary_action: ExportWizardAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportWizardReportViewDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub template_id: &'static str,
    pub template_document: &'static str,
    pub required_stage: ExportStage,
    pub summary_entry_keys: &'static [&'static str],
    pub template_control_ids: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardDescriptor {
    pub template_id: &'static str,
    pub layout_reference: &'static str,
    pub regions: Vec<ExportWizardRegionDescriptor>,
    pub stages: Vec<ExportWizardStageDescriptor>,
    pub report_views: Vec<ExportWizardReportViewDescriptor>,
}

impl ExportWizardDescriptor {
    pub fn stage(&self, stage: ExportStage) -> Option<&ExportWizardStageDescriptor> {
        self.stages
            .iter()
            .find(|descriptor| descriptor.stage == stage)
    }

    pub fn report_view(&self, id: &str) -> Option<&ExportWizardReportViewDescriptor> {
        self.report_views.iter().find(|view| view.id == id)
    }
}

pub fn export_wizard_descriptor() -> ExportWizardDescriptor {
    ExportWizardDescriptor {
        template_id: EXPORT_TEMPLATE_ID,
        layout_reference: BUILD_EXPORT_LAYOUT_REFERENCE,
        regions: vec![
            ExportWizardRegionDescriptor {
                region: ExportWizardRegion::ProfileTree,
                label: "Profiles",
                role: "profile and target selection",
            },
            ExportWizardRegionDescriptor {
                region: ExportWizardRegion::StageTimeline,
                label: "Pipeline",
                role: "Validate-to-Report stage progress",
            },
            ExportWizardRegionDescriptor {
                region: ExportWizardRegion::ReportInspector,
                label: "Report",
                role: "selected stage diagnostics and generated artifacts",
            },
        ],
        stages: vec![
            stage_descriptor(ExportStage::Validate, "validate", "Validate"),
            stage_descriptor(
                ExportStage::SourceTemplate,
                "source_template",
                "Source Template",
            ),
            stage_descriptor(
                ExportStage::NativeDynamic,
                "native_dynamic",
                "Native Dynamic",
            ),
            stage_descriptor(ExportStage::CompileHost, "compile_host", "Compile Host"),
            stage_descriptor(ExportStage::CookAssets, "cook_assets", "Cook Assets"),
            stage_descriptor(ExportStage::Pack, "pack", "Pack"),
            stage_descriptor(
                ExportStage::PlatformBundle,
                "platform_bundle",
                "Platform Bundle",
            ),
            stage_descriptor(ExportStage::Report, "report", "Report"),
        ],
        report_views: vec![
            ExportWizardReportViewDescriptor {
                id: "source_template",
                label: "SourceTemplate Report",
                template_id: SOURCE_TEMPLATE_REPORT_ID,
                template_document: SOURCE_TEMPLATE_REPORT_DOCUMENT,
                required_stage: ExportStage::SourceTemplate,
                summary_entry_keys: SOURCE_TEMPLATE_REPORT_SUMMARY_ENTRY_KEYS,
                template_control_ids: SOURCE_TEMPLATE_REPORT_TEMPLATE_CONTROL_IDS,
            },
            ExportWizardReportViewDescriptor {
                id: "library_embed",
                label: "LibraryEmbed Report",
                template_id: LIBRARY_EMBED_REPORT_ID,
                template_document: LIBRARY_EMBED_REPORT_DOCUMENT,
                required_stage: ExportStage::CompileHost,
                summary_entry_keys: LIBRARY_EMBED_REPORT_SUMMARY_ENTRY_KEYS,
                template_control_ids: LIBRARY_EMBED_REPORT_TEMPLATE_CONTROL_IDS,
            },
            ExportWizardReportViewDescriptor {
                id: "native_dynamic",
                label: "NativeDynamic Report",
                template_id: NATIVE_DYNAMIC_REPORT_ID,
                template_document: NATIVE_DYNAMIC_REPORT_DOCUMENT,
                required_stage: ExportStage::NativeDynamic,
                summary_entry_keys: NATIVE_DYNAMIC_REPORT_SUMMARY_ENTRY_KEYS,
                template_control_ids: NATIVE_DYNAMIC_REPORT_TEMPLATE_CONTROL_IDS,
            },
        ],
    }
}

pub fn stage_progress_kinds() -> [ExportStageProgressKind; 4] {
    [
        ExportStageProgressKind::Pending,
        ExportStageProgressKind::Running,
        ExportStageProgressKind::Passed,
        ExportStageProgressKind::Fatal,
    ]
}

fn stage_descriptor(
    stage: ExportStage,
    stage_id: &'static str,
    label: &'static str,
) -> ExportWizardStageDescriptor {
    ExportWizardStageDescriptor {
        stage,
        stage_id,
        label,
        report_path: STAGE_REPORT_FILE,
        primary_action: ExportWizardAction::RunStage,
    }
}
