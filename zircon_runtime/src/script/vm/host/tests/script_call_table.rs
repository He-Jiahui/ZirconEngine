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

fn fault_context(entity: u64) -> crate::script::ScriptRuntimeCallContext {
    use crate::core::{framework::scene::WorldHandle, CoreRuntime};
    use crate::scene::{LevelMetadata, LevelSystem, World};
    use std::sync::Mutex;
    crate::script::ScriptRuntimeCallContext {
        core: CoreRuntime::new().weak(),
        level: LevelSystem::new(
            WorldHandle::new(130_040),
            Arc::new(Mutex::new(World::empty())),
            LevelMetadata::default(),
        ),
        entity,
        delta_seconds: 0.016,
        host_faults: Default::default(),
    }
}

#[test]
fn script_host_fault_callback_failure_keeps_first_fault_and_counts_each_call_once() {
    use crate::core::framework::script::{ScriptHostError, ScriptHostOwnedArgumentSource};
    use crate::script::vm::runtime_context::with_observed_script_runtime_call_context;
    let site = ScriptCallSite::new(
        ScriptCallSiteId(0),
        Arc::from("probe.host"),
        ScriptHostFunctionDescriptor::new("fail", 0, 0, ScriptHostValueKind::Null),
        Arc::new(|_| Err(ScriptHostError::new("Rust JSON error"))),
    );
    let source = ScriptHostOwnedArgumentSource::new(&[]);
    let (result, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        let error = site
            .call(ScriptHostArguments::new(&source), &CapabilitySet::default())
            .unwrap_err();
        crate::script::record_script_host_call_fault("later", "ignored", "secondary host failure");
        error
    });
    assert_eq!(
        result.to_string(),
        "plugin operation failed: host export call failed: probe.host.fail: Rust JSON error"
    );
    assert_eq!(summary.count, 2);
    let first = summary.first.unwrap();
    assert_eq!(
        (
            first.module.as_str(),
            first.function.as_str(),
            first.message.as_str()
        ),
        ("probe.host", "fail", "Rust JSON error")
    );
}

#[test]
fn script_host_fault_arity_capability_and_unknown_id_errors_are_recorded() {
    use crate::core::framework::script::ScriptHostOwnedArgumentSource;
    use crate::script::vm::runtime_context::with_observed_script_runtime_call_context;
    let source = ScriptHostOwnedArgumentSource::new(&[]);
    let mut site = test_site(0, "probe.host", "arity");
    site.descriptor.min_argument_count = 1;
    site.descriptor.max_argument_count = 1;
    let (_, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        site.call(ScriptHostArguments::new(&source), &CapabilitySet::default())
            .unwrap_err()
    });
    assert_eq!(summary.count, 1);
    assert!(summary
        .first
        .unwrap()
        .message
        .contains("expected 1..=1 arguments"));
    site.descriptor.min_argument_count = 0;
    site.descriptor
        .required_capabilities
        .push("probe.required".into());
    let (_, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        site.call(ScriptHostArguments::new(&source), &CapabilitySet::default())
            .unwrap_err()
    });
    assert_eq!(summary.count, 1);
    assert!(summary
        .first
        .unwrap()
        .message
        .contains("missing capability probe.required"));
    let (_, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        ScriptCallTable::default()
            .call(
                ScriptCallSiteId(99),
                ScriptHostArguments::new(&source),
                &CapabilitySet::default(),
            )
            .unwrap_err()
    });
    assert_eq!(summary.count, 1);
    assert!(summary
        .first
        .unwrap()
        .message
        .contains("script call site 99 is not registered"));
}

#[test]
fn script_host_fault_unregistered_module_and_function_are_recorded() {
    use crate::core::framework::script::ScriptHostModuleDescriptor;
    use crate::script::vm::runtime_context::with_observed_script_runtime_call_context;
    use crate::script::{HostExportFunction, HostExportRegistry};
    let exports = HostExportRegistry::default();
    let (_, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        exports.call("probe.host", "missing", vec![]).unwrap_err()
    });
    assert_eq!(summary.count, 1);
    assert!(summary
        .first
        .unwrap()
        .message
        .contains("module not registered: probe.host"));
    exports
        .register_module(
            ScriptHostModuleDescriptor::new("probe.host", "1.0.0").with_function(
                ScriptHostFunctionDescriptor::new("ok", 0, 0, ScriptHostValueKind::Null),
            ),
            [HostExportFunction::new("ok", |_| Ok(ScriptHostValue::Null))],
        )
        .unwrap();
    let (_, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        exports.call("probe.host", "missing", vec![]).unwrap_err()
    });
    assert_eq!(summary.count, 1);
    let first = summary.first.unwrap();
    assert_eq!(first.module, "probe.host");
    assert_eq!(first.function, "missing");
    assert!(first
        .message
        .contains("function not registered: probe.host.missing"));
}

#[test]
fn script_host_fault_outside_context_is_ignored_and_nested_scopes_restore_on_unwind() {
    use crate::script::record_script_host_call_fault;
    use crate::script::vm::runtime_context::{
        with_observed_script_runtime_call_context, with_script_runtime_call_context,
    };
    record_script_host_call_fault("outside", "ignored", "ignored");
    let (returned, summary) = with_observed_script_runtime_call_context(fault_context(1), || {
        let (value, inner) = with_observed_script_runtime_call_context(fault_context(2), || {
            record_script_host_call_fault("inner", "first", "é".repeat(600));
            42
        });
        assert_eq!(value, 42);
        assert_eq!(inner.count, 1);
        assert_eq!(inner.first.unwrap().message.len(), 1024);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_script_runtime_call_context(fault_context(3), || {
                record_script_host_call_fault("panic", "local", "isolated");
                panic!("unwind probe");
            });
        }));
        assert!(panic.is_err());
        record_script_host_call_fault("outer", "restored", "outer only");
        with_script_runtime_call_context(fault_context(4), || 17)
    });
    assert_eq!(returned, 17);
    assert_eq!(summary.count, 1);
    assert_eq!(summary.first.unwrap().module, "outer");
    let (_, empty) = with_observed_script_runtime_call_context(fault_context(5), || ());
    assert_eq!(empty.count, 0);
    assert!(empty.first.is_none());
}
