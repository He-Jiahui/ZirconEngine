use zircon_runtime::plugin::ExportPipelineStage;

use zircon_editor::ExportStageProgressKind;

use crate::{
    EXPORT_TEMPLATE_ID, LIBRARY_EMBED_REPORT_ID, NATIVE_DYNAMIC_REPORT_ID,
    SOURCE_TEMPLATE_REPORT_ID,
};

pub const BUILD_EXPORT_LAYOUT_REFERENCE: &str =
    "docs/ui-and-layout/ai-workbench-style/ai-build-export-layout.png";
pub const PIPELINE_REPORT_PATH: &str = "report.json";
pub const STAGE_REPORT_FILE: &str = "report.json";

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
    pub stage: ExportPipelineStage,
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
    pub required_stage: ExportPipelineStage,
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
    pub fn stage(&self, stage: ExportPipelineStage) -> Option<&ExportWizardStageDescriptor> {
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
            stage_descriptor(ExportPipelineStage::Validate, "validate", "Validate"),
            stage_descriptor(
                ExportPipelineStage::CompileHost,
                "compile_host",
                "Compile Host",
            ),
            stage_descriptor(
                ExportPipelineStage::SourceTemplate,
                "source_template",
                "Source Template",
            ),
            stage_descriptor(
                ExportPipelineStage::CookAssets,
                "cook_assets",
                "Cook Assets",
            ),
            stage_descriptor(ExportPipelineStage::Pack, "pack", "Pack"),
            stage_descriptor(
                ExportPipelineStage::PlatformBundle,
                "platform_bundle",
                "Platform Bundle",
            ),
            stage_descriptor(ExportPipelineStage::Report, "report", "Report"),
        ],
        report_views: vec![
            ExportWizardReportViewDescriptor {
                id: "source_template",
                label: "SourceTemplate Report",
                template_id: SOURCE_TEMPLATE_REPORT_ID,
                required_stage: ExportPipelineStage::SourceTemplate,
            },
            ExportWizardReportViewDescriptor {
                id: "library_embed",
                label: "LibraryEmbed Report",
                template_id: LIBRARY_EMBED_REPORT_ID,
                required_stage: ExportPipelineStage::CompileHost,
            },
            ExportWizardReportViewDescriptor {
                id: "native_dynamic",
                label: "NativeDynamic Report",
                template_id: NATIVE_DYNAMIC_REPORT_ID,
                required_stage: ExportPipelineStage::Pack,
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
    stage: ExportPipelineStage,
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
