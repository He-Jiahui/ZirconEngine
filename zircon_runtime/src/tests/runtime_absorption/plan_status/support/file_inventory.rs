pub(in crate::tests::runtime_absorption::plan_status) fn runtime_absorption_guard_modules(
) -> Vec<&'static str> {
    include_str!("../../mod.rs")
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("mod ")
                .and_then(|module| module.strip_suffix(';'))
        })
        .collect()
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_absorption_plan_status_support_files(
) -> Vec<String> {
    let tests_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests");
    let plan_status_dir = tests_root.join("runtime_absorption").join("plan_status");
    let mut files = Vec::new();

    collect_rust_files_relative_to(&plan_status_dir, &tests_root, &mut files);
    files.sort();
    files
}

fn collect_rust_files_relative_to(
    directory: &std::path::Path,
    relative_root: &std::path::Path,
    files: &mut Vec<String>,
) {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.unwrap_or_else(|error| panic!("failed to read source entry: {error}")))
        .collect();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files_relative_to(&path, relative_root, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let relative_path = path.strip_prefix(relative_root).unwrap_or_else(|error| {
                panic!(
                    "failed to make {} relative to {}: {error}",
                    path.display(),
                    relative_root.display()
                )
            });
            let anchor = relative_path
                .components()
                .map(|component| {
                    component.as_os_str().to_str().unwrap_or_else(|| {
                        panic!("source path should be utf-8: {}", path.display())
                    })
                })
                .collect::<Vec<_>>()
                .join("/");
            files.push(anchor);
        }
    }
}
