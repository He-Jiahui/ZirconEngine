use crate::project::ProjectManifestSummary;

use super::{ProjectTemplateId, RenderedProjectTemplateEntry};

/// Fully rendered, filesystem-independent template payload consumed by Editor and Hub.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedProjectTemplate {
    pub id: ProjectTemplateId,
    pub summary: ProjectManifestSummary,
    pub entries: Vec<RenderedProjectTemplateEntry>,
}
