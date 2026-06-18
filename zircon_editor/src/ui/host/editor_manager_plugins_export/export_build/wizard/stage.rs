use zircon_runtime::plugin::ExportPipelineStage;

pub fn export_pipeline_stage_cli_id(stage: ExportPipelineStage) -> &'static str {
    match stage {
        ExportPipelineStage::Validate => "validate",
        ExportPipelineStage::SourceTemplate => "source_template",
        ExportPipelineStage::NativeDynamic => "native_dynamic",
        ExportPipelineStage::CompileHost => "compile_host",
        ExportPipelineStage::CookAssets => "cook_assets",
        ExportPipelineStage::Pack => "pack",
        ExportPipelineStage::PlatformBundle => "platform_bundle",
        ExportPipelineStage::Report => "report",
    }
}

pub fn export_pipeline_stage_report_name(stage: ExportPipelineStage) -> &'static str {
    match stage {
        ExportPipelineStage::Validate => "Validate",
        ExportPipelineStage::SourceTemplate => "SourceTemplate",
        ExportPipelineStage::NativeDynamic => "NativeDynamic",
        ExportPipelineStage::CompileHost => "CompileHost",
        ExportPipelineStage::CookAssets => "CookAssets",
        ExportPipelineStage::Pack => "Pack",
        ExportPipelineStage::PlatformBundle => "PlatformBundle",
        ExportPipelineStage::Report => "Report",
    }
}
