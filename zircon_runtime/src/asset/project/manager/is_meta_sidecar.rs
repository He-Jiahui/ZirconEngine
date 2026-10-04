use std::path::Path;

// .zmeta 与 .meta.toml 保存资源元数据；临时兄弟文件仅按已观察到的 ReplaceFileW 命名形状识别。
// collect_files 在源遍历时排除它们，避免把控制文件重新当作独立资产。
pub(super) fn is_meta_sidecar(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".zmeta") || name.ends_with(".meta.toml"))
        || crate::asset::replace_file_meta_temp::is_replace_file_meta_temporary(path)
}
