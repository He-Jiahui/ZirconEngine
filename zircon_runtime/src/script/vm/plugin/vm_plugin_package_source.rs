use std::path::PathBuf;

/// 发现时保留包根、清单与载荷规范路径，供稍后的缓存物化和宿主上下文记录来源；裸内存包可以没有磁盘路径。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VmPluginPackageSource {
    pub package_root: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub bytecode_path: Option<PathBuf>,
    pub zr_vm_project_path: Option<PathBuf>,
}
