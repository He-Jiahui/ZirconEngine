use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VmPluginPackageSource {
    pub package_root: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
    pub bytecode_path: Option<PathBuf>,
}
