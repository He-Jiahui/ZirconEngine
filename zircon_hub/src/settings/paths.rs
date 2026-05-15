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

fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}
