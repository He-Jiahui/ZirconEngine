fn runtime_plan_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root")
        .join("docs")
        .join("plans")
        .join("zircon_runtime")
        .join("runtime")
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_subplan_sources(
) -> Vec<(String, String)> {
    let plan_dir = runtime_plan_dir();
    let mut sources = Vec::new();

    for entry in std::fs::read_dir(&plan_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", plan_dir.display()))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read runtime plan entry: {error}"));
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("index.md")
        {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| panic!("runtime plan path should be utf-8: {}", path.display()))
            .to_owned();
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        sources.push((filename, source));
    }

    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources
}

pub(in crate::tests::runtime_absorption::plan_status) fn max_iso_date(
    source: &str,
) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut max_date = None;

    for start in 0..bytes.len().saturating_sub(9) {
        let candidate_bytes = &bytes[start..start + 10];
        let is_iso_date = candidate_bytes[0..4].iter().all(u8::is_ascii_digit)
            && candidate_bytes[4] == b'-'
            && candidate_bytes[5..7].iter().all(u8::is_ascii_digit)
            && candidate_bytes[7] == b'-'
            && candidate_bytes[8..10].iter().all(u8::is_ascii_digit);
        if is_iso_date {
            let candidate = std::str::from_utf8(candidate_bytes)
                .expect("ASCII date candidate should be valid utf-8");
            if match max_date {
                Some(date) => candidate > date,
                None => true,
            } {
                max_date = Some(candidate);
            }
        }
    }

    max_date
}
