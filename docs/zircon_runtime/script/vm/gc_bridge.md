---
related_code:
  - zircon_runtime/src/script/vm/gc_bridge/mod.rs
  - zircon_runtime/src/script/vm/gc_bridge/host_handle.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/handles.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/script/vm/gc_bridge/mod.rs
  - zircon_runtime/src/script/vm/gc_bridge/host_handle.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_instance.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
plan_sources:
  - user: 2026-07-13 Plugins 08 ZrVM M3 GC contract
tests:
  - zircon_runtime/src/script/vm/gc_bridge/host_handle.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - milestone M3 Cargo and unit validation deferred to the controller testing stage
doc_type: module-detail
---

# Script VM GC Bridge

## Purpose

The GC bridge defines the ownership and scheduling contracts that may cross between Zircon's Rust host and a managed VM. M3 establishes stable host identities, RAII VM roots, budgeted cooperative collection, and bounded diagnostics without exposing a host or VM pointer. Real ZrVM collector integration remains an M4 backend responsibility.

## Host Handle Model

`HostHandle` is an opaque pair of 32-bit slot index and 32-bit generation. Its neutral script representation is one `u64`: generation occupies the high bits and index occupies the low bits. `from_raw` and `into_raw` are lossless for all bit patterns. When ZrVM transports that payload through a signed `i64`, conversion uses a bit-preserving cast; a negative transport value is not rejected merely because its high bit is set.

`HostRegistry` owns a slot table and free list. A new slot starts at generation one. Revocation removes the record, increments the generation, and puts the index on the free list. Reuse therefore produces the same index with a different generation, so stale handles cannot become valid again. Resolution and revocation return `HostRegistryError` variants for missing indexes, vacant slots, generation mismatches, index exhaustion, and generation exhaustion. A generation-exhaustion error leaves the current record live because the registry cannot safely transition it to another identity.

The registry recovers poisoned locks through `into_inner`. Capability registration is fallible, and host export registration performs descriptor/callback validation before allocating its capability handle. Builtin scene calls encode handles with `into_raw`, reconstruct them with `from_raw`, and resolve object access before returning a summary.

## VM Object Root Lease

`VmObjectId` and `VmGcRootToken` are opaque numeric identifiers. `VmGcRootRegistry` is a `Send + Sync` backend trait whose implementation owns all VM-specific root tables and pointers. `VmObjectRef::new` registers exactly one root. Successful construction stores the token and backend in an `Arc` lease; clones share that lease. Dropping the final clone invokes `unregister_gc_root` exactly once and then releases the backend reference.

Registration failure returns `VmObjectRefError::RegistrationFailed` and never constructs the lease, so no unregister or partially live reference follows. Public accessors expose only the object identifier and root token.

## Cooperative GC Scheduling

`VmGcBudget` defaults to `DEFAULT_VM_GC_MAX_MICROS_PER_FRAME` (1000 microseconds). `VmPluginInstance::gc_step` is a default no-op returning an empty `VmGcStepOutcome`, so backends opt into work without forcing M3 to implement ZrVM collection internals.

Each manager call advances a deterministic GC frame index. The coordinator selects active slots in ascending `PluginSlotId` order, skips `BackendManaged` and `Disabled` policies, and applies `interval_frames` only to `Cooperative` slots. Before each backend call it subtracts already reported pause time and passes the remaining budget. A backend may report a pause larger than the remaining budget; that real pause is retained, `overrun_micros` reports the excess, and no later slot is scheduled.

Reports aggregate pause time, root count, and cross-boundary reference count while retaining per-slot outcomes and the budget each slot received. `VmGcDiagnostics` stores a rolling `VecDeque` bounded by `VM_GC_DIAGNOSTICS_HISTORY_CAPACITY`; pushing past capacity evicts the oldest frame.

## Plugin Integration

The ZrVM language runtime registers default `VmGcBudget` and `VmGcDiagnostics` world resources. Its planned `script.gc_step` runtime scene system uses the package-owned ID `zr_vm_language.script.gc_step`, runs at `SystemStage::Last`, and is explicitly ordered after `zr_vm_language.systems.last`. The ordinary dispatcher still runs VM Last-stage systems once; the GC system resolves `VmPluginManager`, calls only `gc_step`, and appends the successful report to the diagnostics resource.

## Test Coverage and Follow-up

Unit tests cover raw generation roundtrip, dead/stale/vacant/generation-exhaustion registry boundaries, poisoned-lock recovery, exact-once root release, shared clone leases, failed root registration, backend lifetime bounds, default constants, rolling history eviction, real overrun reporting, policy/interval selection, deterministic ordering, and remaining-budget propagation. Plugin registration tests cover both resources, the Last-stage system, its ordering constraint, descriptor anchor, and conservative world access. The feature-gated ZrVM value test covers a packed handle whose signed transport is negative.

Windows M3 validation passed 81/81 `script::vm` runtime tests with `core-min,script,net-contracts` and 11/11 default ZrVM plugin tests under the fixed `1.94.1-x86_64-pc-windows-msvc` toolchain, `--locked --offline --jobs 1`. The real `backend-zr-vm` test remains an M4 boundary because `E:/Git/zr_vm/build` is currently absent. M4 must implement the real ZrVM root registry and cooperative collector step behind these contracts without widening them to raw pointers.
