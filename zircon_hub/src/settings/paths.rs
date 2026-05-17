use std::path::PathBuf;

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
