use std::path::Path;

// 识别元数据侧车与 ReplaceFileW 的 zmeta 临时兄弟文件，上层据此拒绝将其映射成资产 URI。
pub(super) fn is_meta_sidecar(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".zmeta") || name.ends_with(".meta.toml"))
        || crate::asset::replace_file_meta_temp::is_replace_file_meta_temporary(path)
}
