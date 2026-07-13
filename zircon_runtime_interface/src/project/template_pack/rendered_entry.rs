use crate::project::RelPath;

/// One project-relative file rendered from the packaged template truth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedProjectTemplateEntry {
    pub path: RelPath,
    pub bytes: Vec<u8>,
}
