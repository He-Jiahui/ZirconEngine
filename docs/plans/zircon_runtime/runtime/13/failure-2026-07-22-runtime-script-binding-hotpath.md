---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: runtime-script-binding-hotpath
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/script/vm/scene_system.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
tests:
  - cargo test -p zircon_runtime --lib script --locked --jobs 1 -- --nocapture --test-threads=1
  - powershell -NoProfile -Command "Select-String -Path 'zircon_runtime/src/script/vm/scene_system.rs','zircon_runtime/src/script/vm/gameplay_host/*.rs' -Pattern 'node_records\(\)|from_value|current_script_runtime_call_context|expect_string'"
---

# Runtime13：script binding与gameplay host热路径交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Runtime script 96/96逐Rust文件性能审查，PERF-MVP-442/443
- 修复责任计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 交接原因：Runtime13拥有script binding、host call frame与gameplay capability语义；最低根因是这些契约仍以per-frame JSON/full-World projection和per-call owned context执行。
- 生命周期键：`runtime-script-binding-hotpath`

## 失败现象与复现证据

历史基线中，Fixed/Update scene hook各自取得全部node records，clone/deserialize `script.bindings`，再过滤成第二层Vec；每binding格式化String key、按package name扫描slots并同步call export。当前 `scene_system` 已以前向 hard cut 取代该 hook，并按dynamic-component generation发布dense projection，所以稳定generation不再执行那条全量scan/JSON decode路径。

交接仍保持open：当前每个export仍构造owned runtime context handle，缺少运行时指标；projection重建仍承担生命周期状态，导致Fixed/Update初始化顺序和错误fan-out不正确；复杂gameplay/property数据的typed ABI和单一World access scope也未完成。已完成的borrowed guest-to-host input、stable callback cache、World索引entity existence和host-handle validity局部修复不得被回退。

## 最低共享层根因

Runtime13已定义清册、capability、ScriptCallTable和generation-owned active callback projection，但生命周期状态仍从可重建的projection对象派生，调用上下文仍以per-export owned handle构造。PERF-MVP-331的runtime context与本交接是同一调用架构的两层，不得分别新增兼容路径。

## 架构修复验收

- 按script-binding component/world generation发布active dense projection；Fixed/Update共享一次frame snapshot，stable generation不再node scan/JSON decode/key format/package resolve。
- 每binding持generation-aware callback handle和独立activation state；start只在新增或重新启用的activation触发，reload/实体移除/实质binding变更的确切转换规则随同一原子发布，且错误/调用顺序等价。
- ScriptCallSite与gameplay host统一borrowed/arena `HostCallFrame`；module/function/capability/String/Bytes和runtime context稳定调用零深clone。
- 一个host call只进入一次显式World access scope；entity/component/property/nearest/count使用typed index/change projection，禁止用`node_records()`或JSON作为内部热路径。
- 以1/100/10k nodes/bindings和1/100/1M calls记录clone/alloc/visits/resolve/lock/main-thread p95，并通过current-source Runtime script Cargo与F2/F4产品trace。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止缓存完整World/node-record/JSON snapshot跨generation而没有精确失效；禁止用全局锁保护“优化缓存”。
- 禁止在未声明scheduler access与callback thread-affinity前直接parallel iterator执行脚本。
- 禁止保留owned HostCallContext与borrowed HostCallFrame双生产路径。

## 2026-08-15 架构复核：当前状态与前向修复边界

Open state: `generation-owned projection and borrowed guest-to-host input are present, but the runtime-context hot path and its required metrics are not yet converged`. This handoff is not fixed or returned. No Cargo or performance acceptance is claimed by this review.

The previous wording that described the source repair as complete was too broad. The following current-source facts remain material:

- `scene_system.rs::call_export_for_binding` keeps the generation-aware callback handle, but builds `ScriptRuntimeCallContext { core: core.downgrade(), level: level.clone(), .. }` for every binding export. The dynamic TLS scope is valid only for the synchronous call, but it does not meet the plan's no per-call context-handle clone requirement.
- The direct guest-to-host path is already hard-cut: `ScriptCallSite::call` and `ScriptCallTable::call` take `ScriptHostArguments<'call>`. `HostExportRegistry` still has an explicit runtime-originated owned `Vec<ScriptHostValue>` convenience entry point, which adapts through `ScriptHostOwnedArgumentSource`; it is not a guest-input transport or a compatibility overload in the call-table boundary.
- Plugin08's `ZrVmScriptHostArgumentSource` visits each `NativeCallContext::with_argument` view directly: scalar values are copied, strings are lent through `with_str`, and byte arrays use `ScriptHostByteView::Source` with checked length/index access. It does not materialize a generic argument vector or byte vector during callback dispatch.
- `zr_vm_rust_binding` now publishes HRTB-bound `NativeCallContext::with_argument`; its `NativeArgumentView` cannot escape the visitor. The former public value-producing guest-input accessor is not present in the current binding source.
- The remaining measurable gap is real: `scene_system.rs::call_export_for_binding` still creates the weak core handle and cloned level per export. `ScriptHostHotPathMetrics` now records `script_context_level_clones` and `script_context_weak_handles` at those exact construction sites, so a later borrowed-scope hard cut has a source-owned zero-count target. No scale baseline or managed product trace exists yet; the performance gate remains open.

### 2026-08-15 metric-boundary audit

The current counter surface is not yet a complete allocation profile and must
not be used to claim a measured hot-path improvement. `HOST_ARGUMENT_FRAME_ALLOCS`
and `HOST_ARGUMENT_DEEP_CLONE_BYTES` have no mutation path because the generic
guest-to-host transport is visitor-backed; their intended zero is a source
contract that still needs one central guard, not a sampled runtime fact.
`ScriptHostValueRef::copy_string_at_business_boundary` and its byte counterpart
correctly record explicit owned conversions, but gameplay host and builtin host
currently duplicate borrowed-string visitor helpers and add partial local
accounting. In particular, JSON decode ownership and the `component_string`
fallback copy do not have one declared metric boundary. Runtime07's
`runtime_diagnostics::collect_runtime_diagnostics` also does not yet project a
`ScriptHostHotPathMetrics::snapshot`, so these counters are not product-readable.

Before a performance claim or scale benchmark, the Runtime13 hard cut must
centralize the string visitor under the framework call-frame/argument-view
owner, route every true String/Bytes ownership conversion through the central
boundary, and classify JSON decode bytes/count separately until typed ABI
removes JSON from gameplay hot paths. Runtime07 then owns projection of the
stable cumulative fields using an actual frame index and static diagnostics
metadata. This is instrumentation-contract work; it does not authorize a
counter-only optimization or a synthetic zero-valued diagnostic.

### Hard-cut design and remaining completion work

1. `zr_vm_rust_binding` now publishes the required synchronous, non-escaping native argument-view API. It reads scalar values directly and lends string/byte-array views under the visitor's safepoint/reentry lifetime; it must retain that contract and must not regress to a public owning `Value` or deep-copy DTO. ZrVM arrays remain indexed object maps, so the byte case stays a borrowed length/index/visitor view rather than an unsound `&[u8]`; the Plugin08 adapter must not materialize `Vec<u8>` on the dispatch path.
2. The neutral runtime now uses `ScriptHostValueRef<'call>` and `ScriptHostArguments<'call>` on guest-to-host input. `ScriptHostValue` remains the owned return/error transport, and runtime-originated calls may retain owned input through their explicit adapter. No production compatibility overload may reintroduce `Vec<ScriptHostValue>` at the guest-to-host call-table boundary; test fixtures may use a private static argument source only.
3. Plugin08 remains the only backend-specific conversion point. Its registered native closure may retain exactly one `ScriptCallSite` and one capability set at registration, but it must borrow native arguments during the callback and must not perform callback-time registry lookup.
4. The scene path must install a borrowed `ScriptRuntimeCallScope<'call>` over the existing `CoreHandle` and `LevelSystem`, with the dynamic scope restoring its outer value on nested calls. It must eliminate per-export `CoreWeak` creation and `LevelSystem::clone`; it must not create a global cache or retain `World`.

### Lifecycle ordering and callback fault-isolation completion contract

The current projection/cache work must not be treated as a lifecycle state machine.  In current source, `call_script_binding` gates `onStart` on the `Update` branch only, so a scheduler that first selects `FixedUpdate` invokes `onFixedUpdate` before initialization.  `tick_script_bindings` also propagates the first `call_script_binding` error with `?`, so one binding prevents later eligible bindings from receiving the same phase.  Both behaviors are open correctness defects; neither is an acceptable implicit project policy.

The eventual hard cut must allocate a stable script-component identity and an explicit activation state instead of deriving lifecycle ownership from a rebuilt `Rc<ActiveScriptBinding>` projection.  For each enabled activation, the dispatcher must attempt `onStart` before whichever of `onFixedUpdate` or `onUpdate` is selected first, and it must record the successful transition so repeated stable frames do not duplicate `onStart`.  A dynamic-component generation rebuild may refresh callback resolution, but it must not reset lifecycle state for an unaffected active component.  Reload, disable, entity removal, and a materially changed component are the only transitions permitted to create a new activation; their ordering rules must be declared with the same atom.

Phase dispatch must also separate per-binding execution from the phase result.  It must visit every eligible binding in deterministic projection order, retain a structured failure record for each failed binding, and return the declared aggregate only after the fan-out finishes.  The selected product policy may quarantine a component/package or fail a world after the fan-out, but it may not silently discard errors or use the first error to decide whether later bindings run.  This is one lifecycle/scheduler atom: do not add a `FixedUpdate` special case, a second `started` flag, or an error-swallowing compatibility path.

Required focused coverage for that atom:

- a first `FixedUpdate` observes `onStart` before `onFixedUpdate`; a first `Update` observes `onStart` before `onUpdate`;
- repeated fixed/update frames on an unchanged enabled component invoke `onStart` exactly once;
- an unrelated component-generation rebuild preserves the activation/callback state of unaffected bindings, while disable/re-enable, removal, reload, and materially changed bindings follow their declared new-activation rule;
- a failing binding records its identity/export/error and does not prevent a later eligible binding from receiving the same phase; the aggregate result is deterministic;
- nested runtime-call scope restoration and the zero per-export weak/level-handle construction requirements above remain true while lifecycle and failure records are collected.

This design is deferred behind the MVP M0 current-source Runtime-to-Editor-to-App baseline.  It does not authorize a scheduler/lifecycle implementation or change any MVP status until that dependency is green and the focused product traces are scheduled through the coordinator.

### Binding ABI shape preserved by the hard cut

- The guest-to-host path uses lifetime-branded `NativeCallContext<'call>::with_argument(index, visitor)`. `FunctionBuilder` and its stored callback use a higher-ranked callback signature so a `NativeArgumentView<'argument>` cannot escape the native call. The legacy value-producing input accessor remains removed rather than retained as a compatibility API.
- The C binding implements that call by constructing a stack-local `ZrRustBindingNativeArgumentView` for the visitor. Scalar reads are direct `ZrLib_CallContext_Read*` operations. A string or array view creates a `ZrLibTempValueRoot`, copies the current frame value into that root, then applies `ZrCore_Gc_NativeCallPinValue`; access reacquires the rooted value and cleanup unpins before ending the root. It must allocate neither a `ZrRustBindingValue` nor a callback execution owner for argument reads.
- The Rust visitor bridge catches panics before they cross the C callback boundary, maps them to the existing internal-error status, and guarantees the C cleanup path runs for success, type error, backend error, and panic. It exposes no raw pointer. String access may lend `&str` only during the visitor; array access exposes length plus checked byte-at/iteration, because the VM array layout is not contiguous.
- Runtime uses a visitor-backed `ScriptHostArguments<'call>`/`ScriptHostArgumentSource` abstraction instead of a returned slice. Its `with_argument` callback may lend `ScriptHostValueRef<'argument>`, while scalar convenience methods return copied scalars. `ScriptHostFromValue` is replaced with explicit borrowed decoders; any conversion to `String` or `Vec<u8>` is named at the business boundary that needs ownership.

### Runtime scope shape required by the hard cut

- Replace owned `ScriptRuntimeCallContext { core: CoreWeak, level: LevelSystem, .. }` with `ScriptRuntimeCallScope<'call> { core: &'call CoreHandle, level: &'call LevelSystem, entity, delta_seconds }`. `scene_system::call_export_for_binding` installs this scope around only `manager.invoke_callback`; it no longer calls `core.downgrade()` or `level.clone()` per export.
- Keep the existing TLS dynamic-restore guard, but store a pointer to the borrowed scope and make `with_active_script_runtime_call_context` lend it only through a higher-ranked closure. `VmReflectionWorldOperation` stays scope-bound and obtains `World` only through `LevelSystem`; it must not publish a `World` reference, cache it globally, or allocate an owning context.
- `core_handle()` becomes an infallible borrowed `&CoreHandle` within this scope. All gameplay-host consumers migrate in the same atom; their previous weak-upgrade error branch is removed rather than silently retained. External test construction remains behind the existing test-only seam and does not expose a production owned scope.
- Add focused tests for nested scope restoration, restoration after callback panic/error, absence of `core.downgrade()`/`level.clone()` in the export call path, and preservation of the existing reflection-world non-escape rule.

Reference alignment: Unreal's local `CoreUObject/Private/UObject/ScriptCore.cpp` drives Kismet native dispatch through the active `FFrame` (`Code` and `Locals`) and makes parameter copy/writeback an explicit frame boundary. Godot's local `core/object/method_bind.h` accepts `const Variant **p_args` for `call` and `validated_call`, while `tests/core/object/test_method_bind.cpp` verifies registered methods through that call boundary. Fyrox's local `fyrox-impl/src/script/mod.rs` invokes script lifecycle methods with a lifetime-bound `&mut ScriptContext`. Zircon should likewise carry one authoritative synchronous call scope through native dispatch, rather than reconstructing owned input and runtime context for each host export.

### Required module allocation

This hard cut must also obey the current structure convention. `core/framework/script.rs` is already 805 lines, and the ZrVM binding's C and Rust `native` implementation files are already 2071 and 1878 lines. Do not append another conversion, FFI, and lifecycle family to any of those roots.

- Convert the framework script contract into a thin `script` facade with named leaves for value declarations, borrowed input views, host-call frame construction, descriptors, and conversions. The public `core::framework::script` surface may remain curated, but the old monolithic file cannot remain the behavioral owner.
- Split the binding implementation by ownership: native callback registration/trampoline, native call-context metadata, borrowed argument views, and owned return/value construction. The Rust binding must mirror that division rather than making one `native.rs` a second monolith.
- Keep the existing Plugin08 `real_backend/values.rs` as the backend conversion owner and `host_modules.rs` as registration/wiring only. Do not move neutral runtime policy into the plugin or make either a forwarding compatibility layer.

The change is cross-repository and must be integrated in dependency order: the binding API and its safety tests first, then the Runtime13 neutral host contract and Plugin08 adapter as one hard cut, followed by the scene runtime scope. `scene_system.rs` is currently an untracked shared worktree file and the Plugin08 host-module file has unrelated formatting work; neither may be absorbed opportunistically by a partial Runtime13 commit.

### Cross-repository atomic integration

`zircon_plugin_zr_vm_language_runtime` consumes `zr_vm_rust_binding` and its sys crate through direct `../../../../zr_vm/...` path dependencies. A normal one-repository commit order cannot keep both repository heads buildable once `NativeCallContext::argument` is removed: publishing the binding first breaks the current Plugin08 source, while publishing the Plugin08 call site first cannot compile against the current binding. Reintroducing the old accessor as a bridge is explicitly disallowed.

The owners must therefore prepare the binding atom and the Zircon Runtime13/Plugin08 atom in their existing repository checkouts, validate a coordinator-paired combined source snapshot, and ask the coordinator to integrate the two recorded revision hashes as one paired source snapshot. The paired receipt must list both repository roots, both revision hashes, exact changed paths, and the source fingerprint used by validation. No guard-only, binding-only, or Plugin08-only integration may be advertised as copy-stable. The current shared-tree source changes and the Git write hold mean this paired integration is not yet authorized.

### Minimum atomic path families

- Binding repository: `zr_vm_rust_binding/CMakeLists.txt`, its public header, native C callback/context implementation and its new named leaves, Rust `lib.rs` plus the split native facade/leaves, sys FFI declarations, and focused native argument-view tests. The current 2071-line C `native.c` and 1878-line Rust `native.rs` must be split as part of this atom; no new behavior may be appended to either monolith.
- Zircon neutral runtime: convert `core/framework/script.rs` into the curated directory facade and leaves; update `script/vm/host/script_call_table.rs`, `host_export_registry.rs`, `runtime_context.rs`, `gameplay_host/values.rs`, direct builtin-host consumers, script-host tests, and the runtime diagnostics collector. The scope slice includes `scene_system.rs` only after its existing shared owner has joined the paired snapshot; it must not be copied into an unrelated Runtime13 change.
- Plugin08: update `real_backend/values.rs`, `host_modules.rs`, `reflection_host.rs`, `extension_host.rs`, and their focused tests together. `values.rs` remains the ZrVM-to-neutral conversion owner; `host_modules.rs` remains registration wiring and must not accumulate conversion policy.

### Required evidence before a fixed return

- Binding-level tests cover scalar/string/byte inputs, nested native calls, error and panic unwinding, and prove that a borrowed value cannot escape its native callback scope. If reentry is supported, the test must prove the outer view remains valid by pin/reacquire semantics; otherwise the API must reject reentry while a view is live with a documented deterministic error. Runtime conversion tests must separately prove that `String` and bytes become owned only at an explicit business boundary, never in generic argument transport.
- Runtime tests prove the input hard cut has no `Vec<ScriptHostValue>` call path, callback-table lookup remains generation-owned, and nested runtime scopes restore the outer entity/world authority.
- Performance instrumentation records `host_argument_frame_allocs`, `host_argument_deep_clone_bytes`, `guest_string_copy_bytes`, `guest_byte_copy_bytes`, `script_context_level_clones`, `script_context_weak_handles`, world-scope count, lock wait/hold, and host-call latency.
- Those cumulative counters belong in a small script-owned diagnostics leaf, using allocation-free atomics on the hot path. `runtime_diagnostics::collect_runtime_diagnostics` may sample them into `DiagnosticStore::record_static`; `diagnostic_log` sink metrics and per-call log records are not valid substitutes. The existing profiling capture may add only static-name host-call scopes/counters, so its idle path remains capture-gated and dynamic scope-name allocation is excluded from this benchmark path.
- Runtime07 owns the required product-snapshot projection through its existing [mvp-performance-observability](../07/failure-2026-07-17-mvp-performance-observability.md) failure. It must snapshot `ScriptHostHotPathMetrics` during `runtime_diagnostics::collect_runtime_diagnostics`, use `core.real_time().frame_index()` for every static-series sample, and publish the fixed `script.host` counter family with static names, units, and tags. A cumulative host-call count is a measurement value, never a diagnostic frame index; a per-call log or dynamically constructed series is not an acceptable substitute. Until that lowest owner projects the counters, they remain test-readable only and cannot support the managed product comparison.
- The initial `script.host` projection is exactly `call_count`, `argument_frame_allocs`, `argument_deep_clone_bytes`, `guest_string_copy_bytes`, `guest_byte_copy_bytes`, `context_level_clones`, `context_weak_handles`, and `world_scope_entries`, mirroring `ScriptHostHotPathMetricsSnapshot`. Count fields use `count`, byte fields use `byte`, and all eight carry the static `script` and `host` tags. Lock wait/hold and host-call latency have no source counter yet; they must remain absent until the runtime-scope atom supplies a real measurement, never be published as synthetic zeroes.
- A Windows managed performance run measures the declared 1/100/10k binding and 1/100/1M host-call scales, including p50/p95/p99, allocation/copy counters, CPU time, and power trace. No quantitative target is accepted before that baseline and post-change comparison exist outside `C:`.

## 修复结果与回传

The dense projection and generation-aware callback work remains useful partial progress, but this record stays `open` until the input-view and runtime-scope hard cuts above have been implemented, independently reviewed, and validated on current source.

Implemented, not yet accepted:

- Scene bindings now build one active dense projection keyed by world handle
  and the `script.bindings` dynamic-component generation. Stable generations
  reuse callbacks rather than rescanning nodes, decoding JSON, formatting keys,
  or resolving packages per frame.
- Each active binding retains its generation-aware callback handle. Replacing a
  deserialized world, including a replacement that removes bindings, advances
  the relevant dynamic-component generations so stale projections cannot be
  reused. Focused coverage exercises direct and staged world replacement.
- Host exports use borrowed `ScriptHostCallFrame` arguments; the hot path no
  longer maintains a second owned host-call-context bridge. Runtime reflection
  access is issued by `VmPluginHostContext`, with cross-crate fixtures limited
  to the explicit non-default `test-support` feature.
- Production `ScriptRuntimeCallContext` construction and TLS installation are
  crate-private. Cross-crate ZrVM fixtures now use only
  `ScriptRuntimeTestContext` and `with_script_runtime_test_context` under that
  same feature, so a production backend cannot replace the active runtime
  context.
- `VmReflectionWorldAccess` no longer lends a raw `World` from its persistent
  token. It issues an HRTB-bound `VmReflectionWorldOperation` only while a
  runtime script scope is active; the ZrVM reflection dispatcher performs its
  dense-token read/write inside that non-retainable operation ticket.

Static evidence: the script-binding boundary audit and selected `rustfmt
--check` scopes pass; `cargo metadata --no-deps --locked --format-version 1`
succeeds. A second source review identified owned guest-input call-frame,
private test construction, production-context, persistent-world-borrow, and
per-export context-clone findings. Current source addresses the first four;
the last remains explicitly measured for the pending borrowed runtime-scope
hard cut. Final re-review and Cargo evidence remain pending. These checks do
not establish runtime performance or behavior.

Required before `failure return`: run the declared current-source Runtime13
managed Cargo gate and the originating performance acceptance/trace gates. The
1/100/10k binding and 1/100/1M host-call measurements remain acceptance
evidence, not a claim made by this record.
