fn main() {
    let profiling_enabled = std::env::var_os("CARGO_FEATURE_PROFILING").is_some()
        || std::env::var_os("CARGO_FEATURE_PROFILING_CHROME").is_some()
        || std::env::var_os("CARGO_FEATURE_PROFILING_TRACY").is_some()
        || std::env::var_os("CARGO_FEATURE_PROFILING_MEMORY").is_some();
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let profile_dir = std::env::var_os("OUT_DIR")
        .and_then(|out_dir| active_profile_dir(std::path::Path::new(&out_dir)));
    let using_profiling_profile =
        profile == "profiling" || profile_dir.as_deref() == Some("profiling");

    if profiling_enabled && profile == "release" && !using_profiling_profile {
        panic!(
            "Zircon profiling features are disabled for ordinary release builds; use `cargo build --profile profiling --features profiling ...` instead"
        );
    }
}

fn active_profile_dir(out_dir: &std::path::Path) -> Option<String> {
    let components = out_dir
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();
    let build_index = components
        .iter()
        .rposition(|component| *component == "build")?;
    build_index
        .checked_sub(1)
        .map(|index| components[index].to_string())
}
