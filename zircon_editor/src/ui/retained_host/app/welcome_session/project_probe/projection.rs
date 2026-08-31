use crate::core::project::{ProjectAuthorityError, ProjectProbe};

pub(super) fn project_probe_projection(
    result: Result<ProjectProbe, ProjectAuthorityError>,
) -> (bool, Option<String>) {
    match result {
        Ok(_) => (true, None),
        Err(
            ProjectAuthorityError::ProjectName { .. }
            | ProjectAuthorityError::EmptyProjectLocation
            | ProjectAuthorityError::ProjectMissing { .. }
            | ProjectAuthorityError::ManifestMissing { .. },
        ) => (false, None),
        Err(error) => (
            false,
            Some(format!("Existing project check failed: {error}")),
        ),
    }
}

pub(super) fn merge_probe_diagnostic(
    creation_validation: String,
    open_diagnostic: Option<String>,
) -> String {
    let Some(open_diagnostic) = open_diagnostic else {
        return creation_validation;
    };
    if creation_validation.is_empty() {
        open_diagnostic
    } else {
        format!("{creation_validation}; {open_diagnostic}")
    }
}
