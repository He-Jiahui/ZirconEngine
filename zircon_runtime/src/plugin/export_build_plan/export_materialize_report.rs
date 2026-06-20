use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportMaterializeReport {
    pub archive_file: Option<PathBuf>,
    pub generated_files: Vec<PathBuf>,
    pub copied_packages: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

impl ExportMaterializeReport {
    pub fn extend(&mut self, other: Self) {
        if self.archive_file.is_none() {
            self.archive_file = other.archive_file;
        }
        self.generated_files.extend(other.generated_files);
        self.copied_packages.extend(other.copied_packages);
        self.diagnostics.extend(other.diagnostics);
        self.fatal_diagnostics.extend(other.fatal_diagnostics);
    }
}
