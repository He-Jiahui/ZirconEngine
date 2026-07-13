---
related_code:
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_runtime/src/script/vm/gc_bridge/mod.rs
  - zircon_runtime/src/script/vm/gc_bridge/host_handle.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_runtime/src/script/vm/gc_bridge/mod.rs
  - zircon_runtime/src/script/vm/gc_bridge/host_handle.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
plan_sources:
  - user: 2026-07-13 implement the complete engine plugin architecture plan
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/script/vm/gc_bridge/host_handle.rs
  - zircon_runtime/src/script/vm/gc_bridge/vm_object_ref.rs
  - zircon_runtime/src/script/vm/gc_bridge/budget.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
doc_type: module-detail
---

# ZrVM GC Bridge Integration

## Purpose

Plugins 08 M3 connects the ZrVM language plugin to the runtime-neutral script GC contract. The plugin owns scheduling and resource registration; the shared script layer owns handle safety, root leases, budget accounting, and diagnostics. Real `zr_vm` collector calls remain the M4 backend task.

## Registration Surface

The runtime plugin registers `VmGcBudget::default` and `VmGcDiagnostics::default` as typed scene resources. It also registers the planned `script.gc_step` behavior under package-owned system ID `zr_vm_language.script.gc_step` at `SystemStage::Last`, explicitly after `zr_vm_language.systems.last`.

The ordinary Last dispatcher still invokes VM systems once. The separate GC system resolves `VmPluginManager`, calls only `gc_step`, and appends the successful frame report to the scene's rolling diagnostics resource. This keeps GC work out of VM gameplay dispatch and makes the budget observable.

The runtime module declares four system anchors:

- `zr_vm_language.systems.fixed_update`
- `zr_vm_language.systems.update`
- `zr_vm_language.systems.last`
- `zr_vm_language.script.gc_step` (the package-owned ID for the planned `script.gc_step` system)

Arbitrary backend collection work retains conservative world access. Registration tests independently check the three gameplay dispatchers, the GC anchor, its Last-stage ordering constraint, both resources, and conservative access.

## Public Backend Contract

The plugin facade re-exports the runtime-neutral M3 types required by a backend implementation: generational `HostHandle`, `VmObjectRef` and root registry identifiers, `VmGcBudget`, per-slot outcomes, frame reports, and `VmGcDiagnostics`.

Host handles continue to use the existing `u64` script ABI. The real-backend regression preserves a packed handle even when its high generation bit makes the transported ZrVM `i64` negative. No raw host or VM pointer crosses the facade.

M4 must implement the real ZrVM root table and cooperative collector step behind these types. It must not widen the public contract to raw pointers or introduce a second GC scheduling path.

## Validation Status

Windows M3 validation passed the runtime `script::vm` domain (81/81) with `core-min,script,net-contracts`, and the default ZrVM plugin package (11/11 plus empty doctest set), using fixed toolchain `1.94.1-x86_64-pc-windows-msvc`, `--locked --offline --jobs 1`. The package registration test also regenerates and validates the four manifest anchors, including `zr_vm_language.script.gc_step`.

The real `backend-zr-vm` feature was not rerun because `E:/Git/zr_vm/build` is absent. That collector/root integration remains explicitly owned by M4. Detailed commands and failure-to-fix evidence are recorded in [`../../plans/zircon_plugins/08/2026-07-13-zr-vm-m3-output-records.md`](../../plans/zircon_plugins/08/2026-07-13-zr-vm-m3-output-records.md).
