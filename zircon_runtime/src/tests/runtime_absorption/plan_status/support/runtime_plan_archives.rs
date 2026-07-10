use super::runtime_plan_sources::runtime_plan_dir;

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_numbered_archive_source(
    plan_id: &str,
) -> String {
    let archive_dir = runtime_plan_dir().join(plan_id);
    let mut archive_paths = std::fs::read_dir(&archive_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", archive_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read runtime archive entry: {error}"))
                .path()
        })
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    archive_paths.sort();
    archive_paths
        .into_iter()
        .map(|path| {
            std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_numbered_archive_sources() -> String
{
    (1..=15)
        .map(|plan_id| runtime_numbered_archive_source(&format!("{plan_id:02}")))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_plan_source_with_archive(
    plan_id: &str,
    plan_source: &str,
) -> String {
    format!(
        "{plan_source}\n{}",
        runtime_numbered_archive_source(plan_id)
    )
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_index_with_numbered_archives(
    index_source: &str,
) -> String {
    format!("{}\n{index_source}", runtime_numbered_archive_sources())
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_subplan_sources_with_archives(
) -> Vec<(String, String)> {
    super::runtime_plan_sources::runtime_subplan_sources()
        .into_iter()
        .map(|(filename, source)| {
            let source = runtime_plan_source_with_archive(&filename[..2], &source);
            (filename, source)
        })
        .collect()
}
