#[test]
fn review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary() {
    let root = include_str!("../../../../../bin/zircon_host_reflection_docs.rs");
    let args = include_str!("../../../../../bin/zircon_host_reflection_docs/args.rs");
    let error_owner = include_str!("../../../../../bin/zircon_host_reflection_docs/error.rs");
    let run = include_str!("../../../../../bin/zircon_host_reflection_docs/run.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_reflection =
        include_str!("../../../../../../../docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let function_ledger =
        include_str!("../../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");
    let module_doc =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");

    for required in ["mod args;", "mod error;", "mod run;", "run::run("] {
        assert!(
            root.contains(required),
            "host reflection docs bin root should stay a thin entry shell with `{required}`"
        );
    }

    for required in [
        "pub struct HostReflectionDocsArgs",
        "pub fn parse(",
        ") -> HostReflectionDocsResult<HostReflectionDocsArgs>",
        "HostReflectionDocsError::Usage",
        "pub type HostReflectionDocsResult<T> = std::result::Result<T, HostReflectionDocsError>;",
        "pub enum HostReflectionDocsError",
        "CollectBuiltInHostModules",
        "source: VmError",
        "WriteHostInterfaceDocs",
        "source: io::Error",
        "pub fn run(args: impl IntoIterator<Item = OsString>) -> HostReflectionDocsResult<()>",
        "HostReflectionDocsError::CollectBuiltInHostModules",
        "HostReflectionDocsError::WriteHostInterfaceDocs",
    ] {
        assert!(
            args.contains(required) || error_owner.contains(required) || run.contains(required),
            "host reflection docs typed-error path should contain `{required}`"
        );
    }

    for (label, source) in [("root", root), ("args", args), ("run", run)] {
        for forbidden in [
            "fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String>",
            "Result<(), String>",
            "Err(format!(",
            ".map_err(|error| format!(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String-error branch `{forbidden}`"
            );
        }
    }
}
