use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime_interface::ui::template::UiAssetKind;

use crate::core::jobs::{EditorJob, JobContext, JobError};
use crate::ui::host::asset_editor_sessions::imports::{
    collect_ui_asset_import_document, UiAssetImportResolution, UiAssetImportTraversal,
};
use crate::ui::host::asset_editor_sessions::{
    build_ui_asset_editor_session_from_source, preview_size_for_preset, ui_asset_source_hash,
    UiAssetStaleImportDiagnostic,
};
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::project_access::{
    normalize_ui_asset_asset_id, resolve_existing_project_asset_path,
};

use super::plan::{UiAssetDirectRefreshPlan, UiAssetImportRefreshPlan, UiAssetRefreshPlan};
use super::result::{
    UiAssetDirectRefreshOutcome, UiAssetDirectRefreshResult, UiAssetImportRefreshResult,
    UiAssetRefreshBatch,
};

pub(super) struct UiAssetRefreshJob {
    pub(super) plan: UiAssetRefreshPlan,
}

impl EditorJob for UiAssetRefreshJob {
    type Output = UiAssetRefreshBatch;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let total = self
            .plan
            .direct_instances
            .len()
            .saturating_add(self.plan.import_instances.len());
        let mut traversal = UiAssetImportTraversal::default();
        let mut direct_results = Vec::with_capacity(self.plan.direct_instances.len());
        let mut import_results = Vec::with_capacity(self.plan.import_instances.len());
        let mut completed = 0usize;

        for plan in self.plan.direct_instances {
            context.check_cancelled()?;
            let outcome =
                run_direct_refresh(self.plan.project.as_ref(), &mut traversal, &context, &plan);
            direct_results.push(UiAssetDirectRefreshResult { plan, outcome });
            completed += 1;
            context.report_progress(completed as u32, total as u32, "Refreshing UI asset source");
        }
        for plan in self.plan.import_instances {
            context.check_cancelled()?;
            let (imports, errors) = collect_imports(
                self.plan.project.as_ref(),
                &mut traversal,
                &plan.widget_refs,
                &plan.style_refs,
            );
            import_results.push(UiAssetImportRefreshResult {
                plan,
                imports,
                errors,
            });
            completed += 1;
            context.report_progress(
                completed as u32,
                total as u32,
                "Refreshing UI asset imports",
            );
        }
        context.check_cancelled()?;
        Ok(UiAssetRefreshBatch {
            generation: self.plan.generation,
            dependency_generation: self.plan.dependency_generation,
            changed_asset_ids: self.plan.changed_asset_ids,
            project_root: self.plan.project_root,
            direct_results,
            import_results,
        })
    }
}

fn run_direct_refresh(
    project: Option<&ProjectManager>,
    traversal: &mut UiAssetImportTraversal,
    context: &JobContext,
    plan: &UiAssetDirectRefreshPlan,
) -> UiAssetDirectRefreshOutcome {
    let external_source = match fs::read_to_string(&plan.source_path) {
        Ok(source) => source,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return UiAssetDirectRefreshOutcome::Missing;
        }
        Err(error) => {
            return UiAssetDirectRefreshOutcome::Failed {
                message: error.to_string(),
            };
        }
    };
    if ui_asset_source_hash(&external_source) == plan.disk_source_hash {
        return UiAssetDirectRefreshOutcome::Unchanged;
    }
    if plan.source_dirty {
        return UiAssetDirectRefreshOutcome::Conflict { external_source };
    }
    if context.check_cancelled().is_err() {
        return UiAssetDirectRefreshOutcome::Failed {
            message: JobError::Cancelled.to_string(),
        };
    }
    let preview_size = preview_size_for_preset(plan.route.preview_preset);
    let session = match build_ui_asset_editor_session_from_source(
        plan.route.clone(),
        external_source.clone(),
        preview_size,
    ) {
        Ok(session) => session,
        Err(error) => {
            return UiAssetDirectRefreshOutcome::Invalid {
                external_source,
                message: error.to_string(),
            };
        }
    };
    let (widget_refs, style_refs) = session.import_references();
    let (imports, import_errors) = collect_imports(project, traversal, &widget_refs, &style_refs);
    UiAssetDirectRefreshOutcome::Reloaded {
        external_source,
        session,
        imports,
        import_errors,
    }
}

fn collect_imports(
    project: Option<&ProjectManager>,
    traversal: &mut UiAssetImportTraversal,
    widget_refs: &[String],
    style_refs: &[String],
) -> (UiAssetImportResolution, Vec<UiAssetStaleImportDiagnostic>) {
    let resolver = |reference: &str| resolve_import_path(project, reference);
    let mut errors = Vec::new();
    for reference in widget_refs {
        if let Err(error) =
            collect_ui_asset_import_document(&resolver, reference, UiAssetKind::Widget, traversal)
        {
            errors.push(UiAssetStaleImportDiagnostic {
                reference: normalize_ui_asset_asset_id(reference).to_string(),
                message: error.to_string(),
            });
        }
    }
    for reference in style_refs {
        if let Err(error) =
            collect_ui_asset_import_document(&resolver, reference, UiAssetKind::Style, traversal)
        {
            errors.push(UiAssetStaleImportDiagnostic {
                reference: normalize_ui_asset_asset_id(reference).to_string(),
                message: error.to_string(),
            });
        }
    }
    (traversal.finish_resolution(), errors)
}

fn resolve_import_path(
    project: Option<&ProjectManager>,
    reference: &str,
) -> Result<PathBuf, EditorError> {
    let reference = normalize_ui_asset_asset_id(reference);
    if reference.starts_with("res://") {
        let project = project.ok_or_else(|| {
            EditorError::UiAsset(format!(
                "cannot resolve {reference} without an open project generation"
            ))
        })?;
        return resolve_existing_project_asset_path(project, reference);
    }
    Ok(PathBuf::from(reference))
}
