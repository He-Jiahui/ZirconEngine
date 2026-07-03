use super::*;

const SLICE: &str = "Runtime 15 M2 Hub message raw text policy hard cutover";
const STATUS: &str =
    "runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_hub_message_raw_text_policy_uses_current_names";

#[test]
fn runtime_15_hub_message_raw_text_policy_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_root
        .parent()
        .expect("zircon_runtime manifest should live under repository root");
    let hub_message = read_repo_text(manifest_root, "zircon_hub/src/state/hub_message/message.rs");
    let build_actions = read_repo_text(
        manifest_root,
        "zircon_hub/src/tauri_app/runtime_state/build_actions.rs",
    );
    let hard_cutover_doc = read_repo_text(
        manifest_root,
        "docs/engine-architecture/hard-cutover-migration-smells-m1.md",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let status_rows = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );
    let expected_status = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
    );
    let expected_date = read_repo_text(
        manifest_root,
        "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
    );

    let legacy_hits = hub_source_files(&repo_root.join("zircon_hub/src"))
        .into_iter()
        .flat_map(|path| {
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!("failed to read Hub source {}: {error}", path.display())
            });
            let relative = path
                .strip_prefix(repo_root)
                .expect("Hub source should be under repository root")
                .to_string_lossy()
                .replace('\\', "/");
            source
                .lines()
                .enumerate()
                .filter_map(move |(line_index, line)| {
                    if has_legacy_term(line)
                        || line.contains("HubMessage::legacy")
                        || line.contains("Self::Legacy")
                        || line.contains("HubMessageRepr::Legacy")
                    {
                        Some(format!("{relative}:{}: {}", line_index + 1, line.trim()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert!(
        legacy_hits.is_empty(),
        "Hub raw-text policy should not keep old generic legacy wording:\n{}",
        legacy_hits.join("\n")
    );

    assert_contains_all(
        "HubMessage raw-text owner",
        &hub_message,
        &[
            "RawText(String)",
            "pub fn raw_text(text: impl Into<String>) -> Self",
            "Self::RawText(text.into())",
            "ArchivedRawText(String)",
            "HubMessageRepr::ArchivedRawText(text) => Self::RawText(text)",
            "None if params.is_empty() => Self::RawText(id)",
        ],
    );
    assert_contains_all(
        "Hub build actions raw-text consumers",
        &build_actions,
        &[
            "HubMessage::raw_text(detail.clone())",
            "HubMessage::raw_text(report.log_excerpt())",
            "HubMessage::raw_text(report.recovery_hint())",
        ],
    );
    assert_contains_all(
        "hard-cutover migration smells document",
        &hard_cutover_doc,
        &[
            "`legacy-hub-message-archived-text-debt` was cleared",
            "HubMessage::raw_text",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "hard-cutover migration smells doc",
            hard_cutover_doc.as_str(),
        ),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                "zircon_hub/src/state/hub_message/message.rs",
                "HubMessage::raw_text",
                GUARD,
            ],
        );
    }
}

fn hub_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_hub_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_hub_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("Hub source directory should be readable") {
        let entry = entry.expect("Hub source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_hub_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn has_legacy_term(line: &str) -> bool {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|token| token.eq_ignore_ascii_case("legacy"))
}
