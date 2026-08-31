use semver::VersionReq;

use super::directional_range::classify_incompatible_requirement;
use super::{
    ProjectEngineCompatibility, ProjectEngineCompatibilityDisposition,
    ProjectEngineCompatibilityError, ProjectEngineVersion,
};

/// Evaluates the manifest's declared engine range without loading any project-derived code.
pub fn assess_project_engine_compatibility(
    requirement: Option<&str>,
    running_engine: &ProjectEngineVersion,
) -> Result<ProjectEngineCompatibility, ProjectEngineCompatibilityError> {
    let Some(requirement) = requirement else {
        return Ok(ProjectEngineCompatibility::new(
            None,
            running_engine.clone(),
            ProjectEngineCompatibilityDisposition::Compatible,
        ));
    };
    let parsed = VersionReq::parse(requirement).map_err(|source| {
        ProjectEngineCompatibilityError::InvalidRequirement {
            requirement: requirement.to_string(),
            source,
        }
    })?;
    let disposition = parsed
        .matches(running_engine.as_semver())
        .then_some(ProjectEngineCompatibilityDisposition::Compatible)
        .unwrap_or_else(|| classify_incompatible_requirement(&parsed, running_engine.as_semver()));
    Ok(ProjectEngineCompatibility::new(
        Some(requirement.to_string()),
        running_engine.clone(),
        disposition,
    ))
}
