use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ExportBuildPlanError {
    #[error("missing export profile {profile_name}")]
    MissingProfile { profile_name: String },
}
