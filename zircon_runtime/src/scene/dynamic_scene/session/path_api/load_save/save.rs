use std::path::Path;

use super::super::super::{io, RuntimeSessionArchive, RuntimeSessionArchiveError};

impl RuntimeSessionArchive {
    /// 保存完整归档；此入口同样使用原子替换，封存失败时不会覆盖现有目标。
    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), RuntimeSessionArchiveError> {
        io::save_to_path(self, path)
    }
}
