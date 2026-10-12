use std::path::PathBuf;

/// 新建项目和源码引擎设置的初始建议路径；用户配置可覆盖这些默认值。
pub fn default_project_dir() -> PathBuf {
    user_home_dir()
        .map(|home| home.join("ZirconProjects"))
        .unwrap_or_else(|| PathBuf::from("ZirconProjects"))
}

pub fn default_source_dir() -> PathBuf {
    user_home_dir()
        .map(|home| home.join("ZirconEngine"))
        .unwrap_or_else(|| PathBuf::from("ZirconEngine"))
}

/// 源码构建的发布输出初始位置；必须与受管理构建根策略一致，调用方可在设置中覆盖。
#[cfg(windows)]
pub fn default_build_output_dir() -> PathBuf {
    PathBuf::from("E:/cargo-targets/zircon-local/hub-build")
}

#[cfg(not(windows))]
pub fn default_build_output_dir() -> PathBuf {
    user_home_dir()
        .map(|home| home.join("ZirconBuilds"))
        .unwrap_or_else(|| PathBuf::from("ZirconBuilds"))
}

pub fn default_device_install_dir() -> PathBuf {
    user_home_dir()
        .map(|home| home.join("ZirconDevices").join("LocalDevice"))
        .unwrap_or_else(|| PathBuf::from("ZirconDevices").join("LocalDevice"))
}

fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}
