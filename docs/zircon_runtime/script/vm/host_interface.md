---
related_code:
  - zircon_runtime/src/script/vm/host_interface/mod.rs
  - zircon_runtime/src/script/vm/host_interface/callback.rs
  - zircon_runtime/src/script/vm/host_interface/descriptor.rs
  - zircon_runtime/src/script/vm/host_interface/error.rs
  - zircon_runtime/src/script/vm/host_interface/registry.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_interfaces.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
implementation_files:
  - zircon_runtime/src/script/vm/host_interface/mod.rs
  - zircon_runtime/src/script/vm/host_interface/callback.rs
  - zircon_runtime/src/script/vm/host_interface/descriptor.rs
  - zircon_runtime/src/script/vm/host_interface/error.rs
  - zircon_runtime/src/script/vm/host_interface/registry.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_interfaces.rs
plan_sources:
  - user: 2026-07-13 implement the complete engine plugin architecture plan
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/tests/host_interfaces.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/host_interface.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
doc_type: module-detail
---

# VM Host Interface Registry

## Purpose

`VmHostInterfaceRegistry` is the runtime-neutral ownership boundary for VM packages that implement engine extension points. It lets a loaded package publish systems, behavior-tree nodes, RPC handlers, and editor operations without making `zircon_runtime::script` depend on the AI, networking, or editor plugin crates.

The registry solves two separate problems: it authenticates every registration against the package manifest's capability set, and it compiles symbolic module/function names into stable dense callback slots. Consumer plugins receive typed descriptors and invoke the callback through `VmPluginManager`; they never retain a VM pointer or call the ZrVM binding directly.

## Public Model

Four capability IDs gate the four descriptor families:

| Capability | Descriptor | Intended consumer |
|---|---|---|
| `runtime.script.extension.system` | `VmSystemRegistration` | fixed runtime scene dispatcher |
| `runtime.script.extension.bt_node` | `VmBehaviorNodeRegistration` | AI behavior-node adapter |
| `runtime.script.extension.rpc_handler` | `VmRpcHandlerRegistration` | networking RPC adapter |
| `runtime.script.extension.editor_operation` | `VmEditorOperationRegistration` | editor operation adapter |

`VmCallbackHandle { slot, module, function, generation }` is the stable callback identity. `slot` prevents cross-package aliasing; `module` and `function` are dense registration-time indices; `generation` records the last successful resolution. The callback table retains the symbolic target behind those indices so a stale handle can be refreshed after hot reload without a string lookup in the consumer hot path.

`VmInterfaceCaller` carries coordinator-assigned slot/generation identity plus the manifest capability set. A package cannot choose its own owner slot: `HotReloadCoordinator` assigns it before backend load and activation, and `VmPluginHostContext::interface_caller` fails with `MissingCaller` outside that owned lifecycle.

## Registration and Dispatch Flow

1. The coordinator allocates a package slot and installs `(slot, generation)` in the host context.
2. The VM package calls one function from the native `zr.zircon.extensions` module, or an embedded Rust caller uses the matching `host_interface` helper.
3. The registry checks the channel capability before accepting any identifier or compiling a callback.
4. Module and function names are interned into the slot-owned dense callback table. A generation-keyed descriptor is then published.
5. Consumers query descriptors against the manager's active slot records. Results are deterministic and select the newest registration not newer than the active generation.
6. `VmPluginManager::invoke_callback` resolves the dense target, refreshes a stale generation, and delegates the final export call to the active backend instance.

VM systems use three fixed host dispatchers (`FixedUpdate`, `Update`, and `Last`). Each dispatcher declares conservative world access because arbitrary VM code cannot provide a sound static ECS access set. The dispatcher passes only `delta_seconds` into each registered callback and executes contributions in deterministic registry order.

## Hot Reload and Failure Invariants

- A failed initial load discards the complete slot, including callbacks and descriptors.
- A failed replacement load, activation, or state restore discards every descriptor written by the attempted generation before the old slot can be retried.
- A successful reload changes the slot generation. Existing handles resolve through their dense indices and are refreshed before invocation.
- Unload always removes the slot's host-interface state, even when backend deactivation returns an error after the coordinator has removed the runtime slot.
- Registrations from inactive or failed slots are never returned to consumers.
- Registry locks recover poisoned state through the owner helper; production code does not use `.lock().unwrap()`.

These rules are important because registration can happen while a backend is loading or activating. Treating registration as side-effect-free until activation succeeds would leak duplicate generation entries and make a later retry fail incorrectly.

## Validation and Errors

All public failures use `VmHostInterfaceError`. Capability denial is reported before callback-table mutation. Empty or whitespace-padded identifiers are rejected, editor operation names must have exactly three non-empty dot-separated segments, and unsupported system stages return `InvalidSystemStage`. The real backend also validates native callback arity and value types without indexing unchecked argument positions.

RPC registrations store the existing `RpcPayloadSchema`, including its `ReflectSchemaRequest`; the native string argument is interpreted as the reflection type path at the script boundary. No second ZrVM-only schema model is introduced.

Duplicate IDs are rejected only within the same owner slot and generation. Different packages may use the same local ID because ownership is part of the registry key.

## Reference-Engine Alignment

- Godot's extension registration and instance-binding flow motivated the split between stable owner identity and generation-specific activation. Zircon adds explicit capability checks and typed errors at every registration entry.
- Bevy's boxed-system registry and conservative scheduler access informed the fixed VM dispatcher design. Zircon groups VM contributions behind three deterministic dispatchers because a script cannot declare a sound Rust ECS access set.
- Piccolo's name-registration followed by accessor construction informed the load-time interning step. Zircon keeps symbolic names only behind dense callback slots and resolves them at the final backend boundary.

The extra `slot` field in `VmCallbackHandle` is an intentional divergence from the abbreviated plan sketch: a module/function pair is not globally unique across independently loaded packages.

## Test Coverage

The M2 test owners cover stale-generation refresh, capability denial for every channel, descriptor publication, callback execution through the mock backend, conservative system access, failed-generation cleanup before retry, real native module registration, reload, and unload cleanup. `CapabilitySet::contains` deliberately accepts manifest order rather than assuming its public deserialized vector was sorted; capability checks happen at registration time, so the small linear scan is outside the callback hot path. On 2026-07-13 the managed Windows runtime binary passed 4/4 host-interface tests and 6/6 hot-reload coordinator tests; the root `backend-zr-vm,script` feature check also passed. The current plugin source then passed 9/9 default tests and 12/12 `backend-zr-vm` tests with an isolated, offline-generated lock and `--locked --offline`, leaving the foreign-modified main plugin lock untouched. The milestone acceptance record under `docs/plans/zircon_plugins/08/` owns the exact command results.

## Follow-up Boundaries

The neutral registry deliberately does not import concrete AI, networking, or editor managers. Their adapters consume the typed descriptors in their owning milestones. ZrVM M3 owns GC/host-handle lifetime semantics, M4 owns the full real-backend feature matrix, and M5 owns schema-aware state migration and rollback productionization.
