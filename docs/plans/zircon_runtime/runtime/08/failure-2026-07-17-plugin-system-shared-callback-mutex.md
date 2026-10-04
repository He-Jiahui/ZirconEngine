---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: plugin-system-shared-callback-mutex
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration/tests.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_runtime/src/scene/ecs/system/native/function_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/runtime_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/into_scene_system.rs
  - zircon_runtime/src/scene/ecs/system/native/scene_system.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin/callback.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/callback_lease.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay/benchmarks.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_runner/tests/worker_dispatch.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_systems.rs
tests:
  - cargo test -p zircon_runtime --lib --locked plugin::extension_registry::register::system_registration::tests::typed_scene_system_callback_state_is_private_per_world -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::extension_registry::register::system_registration::tests::external_scene_system_callback_state_is_private_per_world -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::extension_registry::register::system_registration::tests::external_scene_system_callbacks_overlap_across_worlds -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::extension_registry::register::runtime_scene_system_registration::tests::runtime_scene_system_callback_state_is_private_per_instance -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::callback_lease::native_callback_stable_owner_source_has_no_per_call_state_mutex -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::callback_lease::native_callback_owner_uses_atomic_transition_and_reports_zero_state_mutex_acquires -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::callback_lease::native_callback_snapshot_defers_lease_until_foreign_call -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::callback_lease::native_callback_snapshot_keeps_generation_alive_after_loaded_plugin_releases -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::callback_lease::native_callback_atomic_transition_survives_64_thread_lease_races -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::registration_replay::native_registration_replay_keeps_old_binding_generation_alive_after_reinstall -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::registration_replay::native_registration_replay_and_reload_publish_both_consistent_generation_orders -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked tests::plugin_extensions::extension_registry_systems::plugin_runtime_scene_system_registrations_apply_to_world -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked -- --test-threads=1
  - cargo test -p zircon_runtime --lib --locked plugin::native_plugin_loader::native_plugin_live_host::tests::callback_lease::native_callback_atomic_lease_64_thread_benchmark -- --exact --ignored --test-threads=1 --nocapture
---

# Runtime08：plugin system instances 共享 callback mutex

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：plugin extension registry 与 ECS system runtime 静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 共同验收：Plugins01 native generation/lifetime、Runtime11 worker scheduling
- 交接原因：回调所有权属于 system-instance/factory 模型；只在 native plugin replay 层绕开 mutex 会让 Rust plugin 与 runtime-scene systems 继续漂移。

## 失败现象与复现证据

`SystemRegistrationBuilder::register` 将传入的 `S: FnMut` 包装为 `Arc<Mutex<S>>`。每次 build 创建的
`SharedCallbackSceneSystem` 都 clone 同一 Arc，`SceneSystem::run` 每帧获取 mutex 后调用回调。
`RuntimeSceneSystemRegistrationBuilder` 使用相同结构。

这意味着单 World 稳态每次 system run 都有不必要的互斥开销；同一 registration 被多个 World/preview/PIE
实例 build 后，共享同一个可变 callback 状态并跨 World 串行。Native registration replay 的 callback 仅持有稳定
bridge scope/slots、逻辑上无状态，却也被该通用 builder 强制放入 mutex。

## 最低共享层根因

Registration 同时被当作“一份可变 callback 实例”和“可重复 build 的 system factory”。为支持重复 build，代码用
共享 mutex 掩盖所有权矛盾，而不是区分 per-instance stateful factory 与可 `Sync` 共享的 stateless callable。

## 架构修复验收

- Registration 存可重复调用的 system factory；每次 build 产生独立 callback/state 与 `SystemState<P>`。
- stateful `FnMut` callback 归单个 system instance 独占，`run(&mut self)` 直接调用，不获取共享 mutex。
- 明确支持 stateless `Fn + Send + Sync` callable，共享时不使用独占 mutex；native replay 走该契约或 generation-owned factory。
- 同一 registration 构建到两个 World 时 callback state 隔离，并能在无 access conflict 时由不同 worker 并行。
- hot reload/unload 不提前释放在途 system generation；panic/poison recovery 不再依赖共享 callback mutex 语义。
- 单/双 World benchmark 记录 callback mutex acquire=0、overlap、p95 与 worker utilization。

## 参考引擎原则

- Bevy `dev/bevy/crates/bevy_ecs/src/system/function_system.rs` 的 `FunctionSystem` 直接拥有 `func: F` 与 per-system state；
  可 clone factory 的约束显式存在，不用所有运行实例共享一把 callback mutex。
- Zircon 可迁移“system instance owns mutable state”原则，但必须保留自身 plugin generation、owner revocation 与 native ABI lifetime。

## 禁止临时方案

- 不得用 `try_lock` 跳帧或在冲突时静默丢 system run。
- 不得为移除 mutex 而用 `unsafe impl Sync` 包装 `FnMut`。
- 不得只给 native callback 增加旁路而继续让通用 plugin registration 共享可变状态。

## 修复结果与回传

Open state: `前向复核中`; no pass is claimed.

- 当前 lowest-owner 源码复核：`SystemRegistrationBuilder`、`ExternalSystemRegistrationBuilder` 和 `RuntimeSceneSystemRegistrationBuilder` 都保存可 clone 的 build template；每次 `build` 创建独立 `CallbackSceneSystem` / `ExternalCallbackSceneSystem` / `FunctionRuntimeSceneSystem`，实例持有自己的 `FnMut` 与 `SystemState`，运行路径不再保存或获取 callback `Mutex`。native replay 当前通过 `register_external_native_system` 进入同一 factory path。
- 已存在的 source tests 覆盖 typed/external/runtime callback 的 per-instance state；external worker-safe callback 还覆盖两个 World 实例的 overlap。该结论目前仅来自 source review 与未执行的测试源码，native hot-reload/unload generation lifetime、panic 语义以及受管多 World 并行验证仍未获得运行证据。
- 因此本 artifact 保持 `open`，直到 declared tests 和 native generation lifecycle evidence 在后续受管 validation gate 中返回真实结果。

### 2026-08-13 current-source reconciliation

- native generation lifetime 也已具备生产 owner 与源码矩阵：callback snapshot 持有 generation，foreign call 前取得无 per-call state mutex 的原子 lease；lifecycle transition 在 active callbacks 归零前拒绝 unload/reload。测试覆盖 snapshot 延迟 lease、generation keepalive、failed reload reopen、64-thread lease race 和 zero state-mutex acquire diagnostics。
- 因此共享 callback mutex handoff 的 production 实现已闭包，未发现 `Arc<Mutex<FnMut>>` 或跨 World 共享 mutable callback state 的回流。剩余仅是 declared per-instance/multi-World/native-generation managed test 与 benchmark terminal evidence；本文件继续 `open`，不以静态 source tests 代替运行结果。

### 2026-09-25 current-source owner and acceptance reconciliation

- The current lowest-owner chain is now indexed at the concrete factory and instance owners:
  `extension_registry/register/{system_registration.rs,runtime_scene_system_registration.rs}`
  retain an `Arc<dyn Fn() -> S + Send + Sync>` build factory, while
  `scene/ecs/system/native/{function_scene_system.rs,runtime_scene_system.rs,
  into_scene_system.rs,scene_system.rs}` own the per-instance callback and `SystemState` run
  path. The typed/external factory tests are in
  `register/system_registration/tests.rs`; the runtime-scene factory test is colocated in
  `runtime_scene_system_registration.rs`.
- Native generation ownership is indexed through
  `loaded_native_plugin/callback.rs` (atomic callback lease and zero state-mutex diagnostics),
  `native_plugin_live_host/registration_replay.rs` (generation cache/invalidation), and
  `native_plugin_live_host/lifecycle.rs` (transition/reload publication). The direct runtime
  evidence is in `native_plugin_live_host/tests/callback_lease.rs` and
  `native_plugin_live_host/tests/registration_replay.rs`; replay scaling is isolated in its
  benchmark module. `schedule_runner/tests/worker_dispatch.rs` and
  `tests/plugin_extensions/extension_registry_systems.rs` cover the upward worker and plugin
  stage integration. All newly indexed paths exist at the current checkout.
- Exact focused commands are now declared for per-World state, cross-World overlap, runtime
  scene instances, zero state-mutex callback diagnostics, generation keepalive/replay, and the
  64-thread lease race. The broad lib test and the ignored 64-thread benchmark remain explicit
  upward/performance gates; no static source receipt is promoted to a dynamic pass.
- Current hashes were checked before this receipt: `system_registration.rs`
  `0584946eb51fb4bc92b37529039253993fc93600f38e85ca918187be8a5a8eb5`,
  `system_registration/tests.rs`
  `670795870a1a5ba7d93fb5467877af78b87032a5fa8df31abb097dc6be603f18`,
  `runtime_scene_system_registration.rs`
  `1f24378aa741a60b904c92c0845ea471f2363df0d18dec31577fb3fd74de67c3`,
  `loaded_native_plugin/callback.rs` (current foreign dirty source; hash intentionally not
  claimed by this session), `native_plugin_live_host/tests/callback_lease.rs`
  `37d450b7003bc182ff83995ef4c511998f4d0cd36abd009381c8f4308e7253fa`, and
  `native_plugin_live_host/tests/registration_replay.rs`
  `7bbf42e1bdbc0eeddd4fc39d0ec7903464324f4042cd446a7478cf73e0e10165`.
- Focused managed Cargo, the full Runtime08/Plugins01/Runtime11 upward gates, the one- and
  two-World performance measurements, canonical fixed return, and closeout remain pending.
  The failure therefore remains `open`.

### 2026-09-25 independent current-source review receipt

- An independent read-only review rechecked the current owner chain and this handoff after the
  2026-09-25 reconciliation. All 17 `related_code` paths exist. The review confirmed the
  factory-to-per-instance callback/SystemState chain, native generation lease/replay/lifecycle
  ownership, and the upward worker/plugin integration paths match the source currently checked
  out.
- Each of the 13 focused test filters named in the frontmatter resolves to a real test, including
  the per-World/per-instance, zero state-mutex, generation keepalive/replay, and 64-thread lease
  race cases. The 64-thread benchmark is an explicit `#[ignore]` test with a matching filter;
  the broad library run and benchmark remain pending managed gates rather than static passes.
- The five claimed source SHA256 values match the current files. The dirty
  `loaded_native_plugin/callback.rs` remains explicitly foreign/unattributed and has no claimed
  hash. No source or test files were edited and no Cargo command was run for this review.
- Independent review result: **Critical 0 / Important 0 / Moderate 0**. This receipt only clears
  the documentation/source-index review; focused Cargo, upward integration, performance,
  canonical return, and closeout evidence remain pending, so the failure stays `open`.

### 2026-09-29 exact-filter acceptance correction

The 2026-09-25 review confirmed that the 13 named test functions exist, but their
bare function names paired with `--exact` did not name the nested Rust test paths.
The frontmatter now uses the fully qualified module paths verified against the
current test declarations. The ignored 64-thread benchmark retains `--ignored`.
No test was executed by this metadata correction; each managed ticket must still
report a nonzero selected test count before its result can be used for acceptance.
