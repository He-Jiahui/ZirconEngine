use super::*;
use crate::core::framework::script::ScriptHostValueKind;

fn test_site(id: u32, module_name: &str, function_name: &str) -> ScriptCallSite {
    let callback: HostExportCallback = Arc::new(|_| Ok(ScriptHostValue::Null));
    ScriptCallSite::new(
        ScriptCallSiteId(id),
        Arc::from(module_name),
        ScriptHostFunctionDescriptor::new(function_name, 0, 0, ScriptHostValueKind::Null),
        callback,
    )
}

#[test]
fn from_entries_preserves_non_contiguous_module_groups() {
    let table = ScriptCallTable::from_entries(
        7,
        vec![
            test_site(0, "runtime.time", "now"),
            test_site(1, "runtime.input", "poll"),
            test_site(2, "runtime.time", "delta"),
        ],
    );

    assert!(table.resolve("runtime.time", "now").is_some());
    assert!(table.resolve("runtime.input", "poll").is_some());
    assert!(table.resolve("runtime.time", "delta").is_some());
}
