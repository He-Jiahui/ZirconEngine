#[path = "asset_manifest_template.rs"]
mod asset_manifest_template;
#[path = "cargo_manifest_template.rs"]
mod cargo_manifest_template;
#[path = "default_profile.rs"]
mod default_profile;
#[path = "error.rs"]
mod error;
#[path = "export_build_plan.rs"]
mod export_build_plan;
#[path = "export_generated_file.rs"]
mod export_generated_file;
#[path = "export_materialize_report.rs"]
mod export_materialize_report;
#[path = "export_profile_validation.rs"]
mod export_profile_validation;
#[path = "export_validate_report.rs"]
mod export_validate_report;
#[path = "from_project_manifest.rs"]
mod from_project_manifest;
#[path = "generated_files.rs"]
mod generated_files;
#[path = "library_embed_compile_plan.rs"]
mod library_embed_compile_plan;
#[path = "linked_feature_source.rs"]
mod linked_feature_source;
#[path = "main_template.rs"]
mod main_template;
#[path = "materialize/mod.rs"]
mod materialize;
#[path = "native_dynamic_package_plan.rs"]
mod native_dynamic_package_plan;
#[path = "../native_plugin_load_manifest_template.rs"]
mod native_plugin_load_manifest_template;
#[path = "platform_host_files.rs"]
mod platform_host_files;
#[path = "plugin_selection_template.rs"]
mod plugin_selection_template;
#[path = "project_manifest_validation.rs"]
mod project_manifest_validation;
#[path = "source_template_build_plan.rs"]
mod source_template_build_plan;

pub use error::ExportBuildPlanError;
pub use export_build_plan::{ExportBuildPlan, ExportPlanAdmissionProof};
pub(crate) use export_build_plan::{ExportLinkedRuntimeCrate, ExportRuntimeCrateRegistrationKind};
pub use export_generated_file::ExportGeneratedFile;
pub use export_materialize_report::ExportMaterializeReport;
pub use export_validate_report::{
    ExportValidateGeneratedFileSummary, ExportValidatePlanSummary, ExportValidateProfileSummary,
    ExportValidateReport,
};
pub use library_embed_compile_plan::{
    LibraryEmbedCompileHostPlan, LibraryEmbedCompileHostTarget, LibraryEmbedLinkedRuntimeCrate,
};
use linked_feature_source::admit_linked_feature_sources;
pub use linked_feature_source::ExportLinkedFeatureSourceReceipt;
pub use native_dynamic_package_plan::{
    NativeDynamicPackageAbiV3Contract, NativeDynamicPackageExportPlan,
};
pub use source_template_build_plan::SourceTemplateBuildValidationPlan;
