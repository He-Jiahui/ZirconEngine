use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportMaterializeReport {
    pub generated_files: Vec<PathBuf>,
    pub copied_packages: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

impl ExportMaterializeReport {
    pub fn extend(&mut self, other: Self) {
        self.generated_files.extend(other.generated_files);
        self.copied_packages.extend(other.copied_packages);
        self.diagnostics.extend(other.diagnostics);
        self.fatal_diagnostics.extend(other.fatal_diagnostics);
    }
}
