use super::*;

#[test]
fn runtime_15_net_http_hyper_http1_client_policy_is_isolated() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let backend_root = "zircon_plugins/net/features/http/runtime/src/backend";
    let backend = read_repo_text(manifest_root, &format!("{backend_root}.rs"));
    let client = read_repo_text(manifest_root, &format!("{backend_root}/client.rs"));
    let policy = read_repo_text(
        manifest_root,
        &format!("{backend_root}/http1_client_policy.rs"),
    );
    let hard_cutover_audit = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let net_doc = read_repo_text(manifest_root, "docs/zircon_plugins/net/runtime.md");
    let hard_cutover_doc = read_repo_text(
        manifest_root,
        "docs/engine-architecture/hard-cutover-migration-smells-m1.md",
    );
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let expected_status = read_runtime_15_naming_status_map(manifest_root);
    let expected_date = read_runtime_15_naming_date_map(manifest_root);

    assert_contains_all(
        "Net HTTP backend module wiring",
        &backend,
        &["mod http1_client_policy;"],
    );
    assert_contains_all(
        "Net HTTP backend client consumes policy owner",
        &client,
        &["http1_client_policy::plain_http_client()"],
    );
    for retired in [
        "hyper_util::client::legacy::Client",
        "hyper_util::client::legacy::{",
    ] {
        assert!(
            !client.contains(retired),
            "Net HTTP backend client should not directly expose the third-party Hyper policy path {retired}"
        );
    }
    assert_contains_all(
        "Net HTTP/1 policy owner",
        &policy,
        &[
            "type PlainHttpClient",
            "pub(super) fn plain_http_client() -> PlainHttpClient",
            "hyper_util::client::legacy::{connect::HttpConnector, Client}",
            "TokioExecutor::new()",
        ],
    );
    assert_contains_all(
        "hard-cutover audit recognizes isolated Net HTTP policy",
        &hard_cutover_audit,
        &[
            "external-hyper-http1-client-policy",
            "HARD_CUTOVER_ALLOWED_CLASSIFICATIONS",
            "zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("Net runtime doc", net_doc.as_str()),
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
                "Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover",
                "runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred",
                "zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs",
                "external-hyper-http1-client-policy",
                "runtime_15_net_http_hyper_http1_client_policy_is_isolated",
            ],
        );
    }
}
