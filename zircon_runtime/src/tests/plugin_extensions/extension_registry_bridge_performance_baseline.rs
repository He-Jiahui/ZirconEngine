use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::framework::bridge::PluginInterface;
use crate::plugin::RuntimeExtensionRegistry;

const BRIDGE_PERFORMANCE_SAMPLE_CALLS: u32 = 65_536;
const STRONG_BRIDGE_DIRECT_ARC_RATIO_BUDGET: u128 = 32;
const PINNED_BRIDGE_DIRECT_ARC_RATIO_BUDGET: u128 = 32;
const WEAK_BRIDGE_DIRECT_ARC_RATIO_BUDGET: u128 = 512;
const BRIDGE_PERFORMANCE_MIN_BUDGET_NANOS: u128 = 5_000_000;

trait WeatherQueryInterface: Send + Sync {
    fn sample_temperature(&self) -> i32;
}

impl PluginInterface for dyn WeatherQueryInterface {
    const INTERFACE_ID: &'static str = "weather.query.v1";
}

#[derive(Debug)]
struct WeatherQueryProvider {
    temperature: i32,
}

impl WeatherQueryInterface for WeatherQueryProvider {
    fn sample_temperature(&self) -> i32 {
        self.temperature
    }
}

#[test]
fn bridge_performance_baseline_strong_layout_matches_direct_arc() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 19 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();

    let bridge = table
        .resolve_strong::<dyn WeatherQueryInterface>()
        .expect("strong bridge");

    assert_eq!(
        std::mem::size_of_val(&bridge),
        std::mem::size_of::<Arc<dyn WeatherQueryInterface>>()
    );
    assert_eq!(bridge.sample_temperature(), 19);
}

#[test]
fn bridge_performance_baseline_weak_hot_path_uses_generation_matched_snapshot_before_slow_lookup() {
    let weak_source = include_str!("../../plugin/bridge/weak.rs");
    let call = source_between(weak_source, "    pub fn call<R>", "    pub fn pin(");

    assert_source_order(
        call,
        "self.current_generation(slot)",
        "let cached = self.cached.load();",
    );
    assert_source_order(
        call,
        "snapshot.generation == generation",
        "snapshot.provider.upgrade()",
    );
    assert_source_order(
        call,
        "snapshot.provider.upgrade()",
        "return Ok(f(provider.as_ref()));",
    );
    assert_source_order(
        call,
        "return Ok(f(provider.as_ref()));",
        "self.refresh_provider(slot)",
    );
    assert!(!call.contains("Mutex"));
    assert!(!call.contains("RwLock"));
}

#[test]
fn bridge_performance_baseline_pin_guard_keeps_only_resolved_target_for_batch_calls() {
    let weak_source = include_str!("../../plugin/bridge/weak.rs");
    let guard_struct = weak_source
        .find("pub struct BridgeGuard<T: ?Sized>")
        .expect("bridge guard should exist");
    let guard_impl = weak_source[guard_struct..]
        .find("impl<T: ?Sized> Deref for BridgeGuard<T>")
        .expect("bridge guard should dereference to the resolved target");
    let guard_fields = &weak_source[guard_struct..guard_struct + guard_impl];

    assert!(guard_fields.contains("target: Arc<T>"));
    assert!(!guard_fields.contains("table:"));
    assert!(!guard_fields.contains("slot:"));
    assert!(!guard_fields.contains("generation:"));
}

#[test]
fn bridge_performance_baseline_native_bridge_calls_use_pre_resolved_slots() {
    let native_source = include_str!("../../plugin/native_plugin_loader/host_api_adapter.rs");
    let descriptor_resolution = source_between(
        native_source,
        "    pub fn from_method_descriptors(",
        "    pub const fn handle(&self)",
    );
    let runtime_call = source_between(
        native_source,
        "unsafe extern \"C\" fn native_host_bridge_call_v1(",
        "unsafe extern \"C\" fn native_host_diagnostics_emit_v1(",
    );

    assert!(descriptor_resolution.contains(".resolve_slot(descriptor.interface_id())"));
    assert!(
        descriptor_resolution.contains("Ok((slot, descriptor.method_slot(), descriptor.method()))")
    );
    assert!(runtime_call.contains("InterfaceSlot::from_raw(interface_slot)"));
    assert!(runtime_call.contains("context.table.entry(slot)"));
    assert!(runtime_call.contains("context.methods.get(&(interface_slot, method_slot)).copied()"));
    assert!(runtime_call.contains("method.call(NativeBridgeCall"));
    assert_source_order(
        runtime_call,
        "context.table.entry(slot)",
        "context.methods.get(&(interface_slot, method_slot)).copied()",
    );
    assert_source_order(
        runtime_call,
        "context.methods.get(&(interface_slot, method_slot)).copied()",
        "method.call(NativeBridgeCall",
    );
    assert!(!runtime_call.contains(".resolve_slot("));
    assert!(!runtime_call.contains("descriptor.interface_id"));
}

#[test]
fn bridge_performance_baseline_vm_bridge_callbacks_capture_resolved_slot() {
    let bridge_host_source = include_str!("../../script/vm/host/bridge_host_module.rs");
    let module_registration = source_between(
        bridge_host_source,
        "pub fn register_bridge_host_module<Table>(",
        "fn function_descriptor(",
    );
    let function_callback = source_between(
        bridge_host_source,
        "fn function_callback<Table>(",
        "fn ensure_bridge_enabled<Table>(",
    );
    let enabled_check = source_after(bridge_host_source, "fn ensure_bridge_enabled<Table>(");

    assert!(module_registration.contains(".resolve_interface_slot(method.interface_id())"));
    assert!(module_registration
        .contains("callbacks.push(function_callback(bridge_table.clone(), slot, method));"));
    assert!(function_callback
        .contains("ensure_bridge_enabled(&bridge_table, slot, &method.interface_id)?"));
    assert!(function_callback.contains("interface_slot: slot"));
    assert!(function_callback.contains("method_slot: method.method_slot"));
    assert!(!function_callback.contains(".resolve_slot("));
    assert!(enabled_check.contains("bridge_table.interface_status_at(slot)"));
    assert!(!enabled_check.contains("interface_snapshot_by_id"));
}

#[test]
fn bridge_status_and_summary_do_not_materialize_owned_interface_snapshots() {
    let table_source = include_str!("../../plugin/bridge/table.rs");
    let summarize = source_between(
        table_source,
        "    fn summarize_entries<'a>(",
        "    fn owner_transition_report(",
    );
    assert!(summarize.contains("entry.snapshot_state()"));
    assert!(!summarize.contains("snapshot_for_entry"));

    let status = source_after(table_source, "    fn interface_status_at(");
    assert!(status.contains("self.entry(slot)"));
    assert!(!status.contains("self.interface_snapshot(slot)"));
}

#[test]
fn extension_owner_queries_use_membership_index_and_revocation_notifies_without_cloning_listeners()
{
    let access = include_str!("../../plugin/extension_registry/access.rs");
    assert!(access.contains("collect::<HashSet<_>>()"));
    assert!(access.contains("module_names.contains(module_name)"));
    assert!(!access.contains("module_names.iter().any"));

    let registry = include_str!("../../plugin/extension_registry/runtime_extension_registry.rs");
    assert!(registry.contains("for listener in &self.owner_revocation_listeners"));
    assert!(!registry.contains("self.owner_revocation_listeners.clone()"));
}

#[test]
fn bridge_owner_reload_borrows_registry_exports_without_cloning_the_replacement_batch() {
    let access = include_str!("../../plugin/extension_registry/access.rs");
    let owned_exports = source_between(
        access,
        "    pub(crate) fn interface_exports_owned_by(",
        "    pub fn interface_owners_for_runtime_modules",
    );
    assert!(owned_exports.contains("impl Iterator<Item = (&str, &InterfaceExport)>"));
    assert!(!owned_exports.contains("Vec<(String, InterfaceExport)>"));
    assert!(!owned_exports.contains("interface_id.clone()"));
    assert!(!owned_exports.contains("export.clone()"));

    let lifecycle =
        include_str!("../../plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs");
    let reload = source_between(
        lifecycle,
        "    pub fn reload_bridge_provider_at_frame_boundary(",
        "    fn reject_strong_dependents(",
    );
    assert!(reload.contains("replacement_exports.iter().copied()"));
    assert!(!reload.contains("replacement_exports.clone()"));
}

#[test]
fn bridge_table_build_consumes_extension_owner_rows_without_key_reresolution() {
    let source = include_str!("../../plugin/extension_registry/register/bridge_registration.rs");
    let build = source_after(source, "    fn build_bridge_table(&self)");
    assert!(build.contains(".iter()"));
    assert!(!build.contains(".values().iter()"));
    assert!(!build.contains("self.plugin_interfaces.resolve("));
    assert!(!build.contains("self.plugin_interfaces.owner_for_slot("));
}

#[test]
fn bridge_performance_baseline_script_call_table_calls_dense_id_without_name_lookup() {
    let call_table_source = include_str!("../../script/vm/host/script_call_table.rs");
    let table_impl = source_after(call_table_source, "impl ScriptCallTable {");
    let call_body = source_between(
        table_impl,
        "    pub fn call(",
        "pub(crate) struct ScriptCallTableBuilder",
    );

    assert!(call_body.contains("let Some(site) = self.get(id) else"));
    assert!(call_body.contains("site.call(arguments, granted_capabilities)"));
    assert!(!call_body.contains("by_name"));
    assert!(!call_body.contains("to_string()"));
    assert!(!call_body.contains("resolve("));
}

#[test]
#[cfg(debug_assertions)]
fn bridge_performance_baseline_pin_guard_records_single_resolution_for_batch_calls() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            owner,
            Arc::new(WeatherQueryProvider { temperature: 5 }),
        )
        .unwrap();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    let bridge = table.resolve_weak::<dyn WeatherQueryInterface>();

    let guard = bridge.pin().expect("weak bridge guard");
    assert_eq!(guard.sample_temperature(), 5);
    assert_eq!(guard.sample_temperature(), 5);
    assert_eq!(guard.sample_temperature(), 5);

    let diagnostics = table.diagnostics(slot).unwrap();
    assert_eq!(diagnostics.enabled_calls, 1);
}

#[test]
fn bridge_performance_baseline_samples_wall_clock_hot_paths() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let direct: Arc<dyn WeatherQueryInterface> = Arc::new(WeatherQueryProvider { temperature: 29 });
    registry
        .export_interface::<dyn WeatherQueryInterface>(owner, direct.clone())
        .unwrap();
    let table = registry.frozen_bridge_table();
    let strong = table
        .resolve_strong::<dyn WeatherQueryInterface>()
        .expect("strong bridge");
    let weak = table.resolve_weak::<dyn WeatherQueryInterface>();
    let guard = weak.pin().expect("weak bridge guard");

    let direct_sample = sample_bridge_hot_path("direct_arc", || direct.sample_temperature());
    let strong_sample = sample_bridge_hot_path("strong_bridge", || strong.sample_temperature());
    let weak_sample = sample_bridge_hot_path("weak_bridge_cached", || {
        weak.call(|provider| provider.sample_temperature())
            .expect("weak bridge call")
    });
    let pinned_sample = sample_bridge_hot_path("weak_bridge_pinned", || guard.sample_temperature());

    assert_equivalent_bridge_sample(&direct_sample, &strong_sample);
    assert_equivalent_bridge_sample(&direct_sample, &weak_sample);
    assert_equivalent_bridge_sample(&direct_sample, &pinned_sample);
    assert_sample_within_ratio_budget(
        &direct_sample,
        &strong_sample,
        STRONG_BRIDGE_DIRECT_ARC_RATIO_BUDGET,
    );
    assert_sample_within_ratio_budget(
        &direct_sample,
        &pinned_sample,
        PINNED_BRIDGE_DIRECT_ARC_RATIO_BUDGET,
    );
    assert_sample_within_ratio_budget(
        &direct_sample,
        &weak_sample,
        WEAK_BRIDGE_DIRECT_ARC_RATIO_BUDGET,
    );

    eprintln!("{}", direct_sample.diagnostic());
    eprintln!("{}", strong_sample.diagnostic());
    eprintln!("{}", weak_sample.diagnostic());
    eprintln!("{}", pinned_sample.diagnostic());
}

#[derive(Clone, Debug)]
struct BridgePerformanceSample {
    label: &'static str,
    iterations: u32,
    elapsed: Duration,
    checksum: i64,
}

impl BridgePerformanceSample {
    fn diagnostic(&self) -> String {
        format!(
            "bridge.performance_sample: label={} iterations={} elapsed_ns={} ns_per_call={:.2} checksum={}",
            self.label,
            self.iterations,
            self.elapsed.as_nanos(),
            self.nanos_per_call(),
            self.checksum
        )
    }

    fn nanos_per_call(&self) -> f64 {
        self.elapsed.as_nanos() as f64 / f64::from(self.iterations)
    }
}

fn source_after<'source>(source: &'source str, needle: &str) -> &'source str {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("source should contain `{needle}`"));
    &source[start..]
}

fn source_between<'source>(source: &'source str, start: &str, end: &str) -> &'source str {
    let remainder = source_after(source, start);
    let end = remainder
        .find(end)
        .unwrap_or_else(|| panic!("source range starting at `{start}` should end at `{end}`"));
    &remainder[..end]
}

fn assert_source_order(source: &str, first: &str, second: &str) {
    let first_index = source
        .find(first)
        .unwrap_or_else(|| panic!("source should contain `{first}`"));
    let second_index = source
        .find(second)
        .unwrap_or_else(|| panic!("source should contain `{second}`"));
    assert!(
        first_index < second_index,
        "`{first}` should appear before `{second}` in source"
    );
}

fn sample_bridge_hot_path(
    label: &'static str,
    mut sample_call: impl FnMut() -> i32,
) -> BridgePerformanceSample {
    let mut checksum = 0_i64;
    let started = Instant::now();
    for _ in 0..BRIDGE_PERFORMANCE_SAMPLE_CALLS {
        checksum = checksum.wrapping_add(i64::from(std::hint::black_box(sample_call())));
    }
    let elapsed = started.elapsed();

    assert!(
        elapsed.as_nanos() > 0,
        "bridge performance sample `{label}` should exercise measurable work"
    );

    BridgePerformanceSample {
        label,
        iterations: BRIDGE_PERFORMANCE_SAMPLE_CALLS,
        elapsed,
        checksum,
    }
}

fn assert_equivalent_bridge_sample(
    expected: &BridgePerformanceSample,
    actual: &BridgePerformanceSample,
) {
    assert_eq!(actual.iterations, expected.iterations);
    assert_eq!(
        actual.checksum, expected.checksum,
        "bridge performance sample `{}` returned different results than `{}`",
        actual.label, expected.label
    );
}

fn assert_sample_within_ratio_budget(
    baseline: &BridgePerformanceSample,
    sample: &BridgePerformanceSample,
    max_ratio: u128,
) {
    let budget = baseline
        .elapsed
        .as_nanos()
        .saturating_mul(max_ratio)
        .max(BRIDGE_PERFORMANCE_MIN_BUDGET_NANOS);
    assert!(
        sample.elapsed.as_nanos() <= budget,
        "bridge performance sample `{}` exceeded ratio budget: {} ns > {} ns (baseline `{}` {} ns, ratio budget {}x)",
        sample.label,
        sample.elapsed.as_nanos(),
        budget,
        baseline.label,
        baseline.elapsed.as_nanos(),
        max_ratio
    );
}
