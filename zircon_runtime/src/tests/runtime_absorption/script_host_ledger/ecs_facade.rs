use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi() {
    let gameplay_source = include_str!("../../../script/vm/gameplay_host.rs");
    let runtime_context_source = include_str!("../../../script/vm/runtime_context.rs");
    let ledger =
        include_str!("../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");

    for required_ledger_anchor in [
        "The current script gameplay ECS path is `zr.zircon.gameplay` through `ScriptRuntimeCallContext`",
        "`ZrHostEcsApiV1` belongs to the native/plugin ABI layer",
        "A VM plugin that needs plugin-owned bridge behavior should route through `zr.zircon.bridge`",
    ] {
        assert!(
            ledger.contains(required_ledger_anchor),
            "function ledger should record ECS access path judgement `{required_ledger_anchor}`"
        );
    }

    for required_source_anchor in [
        "const GAMEPLAY_MODULE: &str = \"zr.zircon.gameplay\";",
        "pub fn register_gameplay_host_module(",
        "runtime_context_for_frame(frame)?",
    ] {
        assert!(
            gameplay_source.contains(required_source_anchor),
            "gameplay host source should keep ECS facade anchor `{required_source_anchor}`"
        );
    }
    assert!(
        runtime_context_source.contains("pub(crate) struct ScriptRuntimeCallContext")
            && runtime_context_source.contains("pub(crate) level: LevelSystem")
            && runtime_context_source.contains("pub(crate) entity: EntityId"),
        "script runtime call context should continue to carry the gameplay facade ECS scope"
    );

    for file in script_source_files() {
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("read script source `{}`: {error}", file.display()));
        for forbidden in ["ZrHostEcsApiV1", "ZrHostEcsApi", "HostEcsApi"] {
            assert!(
                !source.contains(forbidden),
                "script source `{}` must stay off native ECS ABI symbol `{forbidden}`",
                file.display()
            );
        }
    }
}

fn script_source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rs_files(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("script"),
        &mut files,
    );
    files.sort();
    files
}

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("read source directory `{}`: {error}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|error| panic!("read source directory entry: {error}"))
            .path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
