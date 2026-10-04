//! 落盘与归档阶段共享的结果收据；编辑器根据 fatal、写入路径和复制包决定是否继续调用 Cargo。
use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 记录导出计划实际写入或预览的结果；fatal 非空时消费者应停止后续构建。
pub struct ExportMaterializeReport {
    pub archive_file: Option<PathBuf>,
    pub generated_files: Vec<PathBuf>,
    pub copied_packages: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

impl ExportMaterializeReport {
    /// 供多阶段收据汇总；已有 archive_file 优先，调用方需按预期阶段顺序合并。
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
