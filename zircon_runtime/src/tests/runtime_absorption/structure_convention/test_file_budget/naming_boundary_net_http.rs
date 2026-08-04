use super::*;

const STATUS: &str =
    "runtime_15_net_http_policy_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 Net HTTP policy guard child-owner split";
const GUARD: &str = "runtime_15_net_http_policy_guard_is_child_owner";

#[test]
fn runtime_15_net_http_policy_guard_is_child_owner() {
    let parent = read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/net/http1_client_policy.rs",
    );

    assert_contains_all(
        "Runtime 15 Net naming parent mounts HTTP/1 policy child",
        &parent,
        &[
            "#[path = \"net/http1_client_policy.rs\"]",
            "mod http1_client_policy;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_net_http_hyper_http1_client_policy_is_isolated"),
        "runtime_15_m2/net.rs should mount the HTTP/1 policy child instead of defining the naming guard"
    );

    assert_contains_all(
        "Runtime 15 Net HTTP child owns policy guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_net_http_hyper_http1_client_policy_is_isolated",
            "external-hyper-http1-client-policy",
            "http1_client_policy::plain_http_client()",
            "runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/net/http1_client_policy.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
}
