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
fn bridge_performance_baseline_weak_hot_path_keeps_cached_generation_before_provider_lookup() {
    let weak_source = include_str!("../../plugin/bridge/weak.rs");
    let provider_fn = weak_source
        .find("fn provider_with_slot")
        .expect("weak bridge provider path should exist");
    let cache_read = weak_source[provider_fn..]
        .find("if let Some((cached_generation, cached))")
        .expect("weak bridge should read the generation cache");
    let cache_generation_check = weak_source[provider_fn..]
        .find("*cached_generation == generation && generation % 2 == 0")
        .expect("weak bridge should match cached even generation before provider lookup");
    let cache_return = weak_source[provider_fn..]
        .find("return Ok((slot, cached.clone()));")
        .expect("weak bridge cache hit should return the cached provider");
    let provider_lookup = weak_source[provider_fn..]
        .find(".provider::<T>(slot)")
        .expect("weak bridge should fall back to table provider lookup only on cache miss");

    assert!(cache_read < provider_lookup);
    assert!(cache_generation_check < provider_lookup);
    assert!(cache_return < provider_lookup);
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
    assert!(runtime_call.contains("context.table.interface_snapshot(slot)"));
    assert!(runtime_call.contains("context.methods.get(&(interface_slot, method_slot)).copied()"));
    assert!(runtime_call.contains("method.call(NativeBridgeCall"));
    assert_source_order(
        runtime_call,
        "context.table.interface_snapshot(slot)",
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
        "pub fn register_bridge_host_module(",
        "pub fn register_bridge_host_module_from_manifest(",
    );
    let function_callback = source_between(
        bridge_host_source,
        "fn function_callback(",
        "fn ensure_bridge_enabled(",
    );
    let enabled_check = source_after(bridge_host_source, "fn ensure_bridge_enabled(");

    assert!(module_registration.contains(".resolve_slot(method.interface_id())"));
    assert!(module_registration
        .contains("callbacks.push(function_callback(bridge_table.clone(), slot, method));"));
    assert!(function_callback
        .contains("ensure_bridge_enabled(&bridge_table, slot, &method.interface_id)?"));
    assert!(function_callback.contains("interface_slot: slot"));
    assert!(function_callback.contains("method_slot: method.method_slot"));
    assert!(!function_callback.contains(".resolve_slot("));
    assert!(enabled_check.contains("bridge_table.interface_snapshot(slot)"));
    assert!(!enabled_check.contains("interface_snapshot_by_id"));
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
fn bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites() {
    let backend_source =
        include_str!("../../script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs");
    let module_registration = source_between(
        backend_source,
        "pub(super) fn register_host_modules(",
        "pub(super) fn native_function_label(",
    );
    let native_callback = source_between(
        backend_source,
        "fn build_native_function(",
        "fn zr_prototype_type(",
    );

    assert_source_order(
        module_registration,
        "let call_table = host.host_exports.script_call_table()?;",
        ".resolve(&module.descriptor.name, &function.name)",
    );
    assert_source_order(
        module_registration,
        ".resolve(&module.descriptor.name, &function.name)",
        "build_native_function(",
    );
    assert!(native_callback.contains("call_site.call(arguments, &capabilities)"));
    assert!(!native_callback.contains("host_exports"));
    assert!(!native_callback.contains("call_with_capabilities"));
    assert!(!native_callback.contains(".resolve("));
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
