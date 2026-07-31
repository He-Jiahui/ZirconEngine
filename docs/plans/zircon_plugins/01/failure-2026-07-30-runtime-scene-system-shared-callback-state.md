---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: runtime-scene-system-shared-callback-state
origin_plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_runtime/runtime/06
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/runtime_scene_system_registration.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
tests:
  - per-World typed scene-system callback state fixture
  - per-World runtime scene-system callback state fixture
  - concurrent independent-World callback overlap fixture
---

# Plugins01: runtime scene system shared callback state

## 来源执行者

- 来源计划: `docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md`
- 来源执行切片: WorldDriver immutable runtime-extension generation and callback lock release
- 修复责任计划: `docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因: extension registry owns the registration build contract that determines whether each World receives private callback state.

## 失败现象与复现证据

`SystemRegistrationBuilder`, `ExternalSystemRegistrationBuilder`, and
`RuntimeSceneSystemRegistrationBuilder` capture one `Arc<Mutex<FnMut>>` at registration time.
Every World-built system then clones that same mutex, so independent Worlds serialize callback execution even after
`WorldDriver` snapshots its immutable extension generation and releases its lock.

2026-07-30 managed RED attempt `dcb2c917623e424f984d08b43ff6abec` /
`9417ad118a4d4f139ab85fad711ca4ba` used the source-bound
`cargo +1.94.1 test -p zircon_runtime --lib callback_state_is_private --locked --jobs 1 -- --nocapture --test-threads=1`
command and terminated `exit 101` during lib-test compilation. No target test ran: the sole error was foreign
Graphics E0432 at `graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs:7`, whose
relative import stopped at `resources::construct` instead of the exported `resources::terminal_resource_cache` module.
This is compile-blocker evidence, not a callback-state RED result.

2026-07-30 fresh managed source-bound RED job `3312ca743309462a86debe4fa154b95d` /
`1e9b4ef77645403e8eaeb227503cdf4e` then compiled the lib test and executed exactly three target
tests. It terminated `exit 101` after `0 passed; 3 failed; 9270 filtered out`: typed, external,
and runtime registrations each observed `[1, 2]` where their per-World/per-instance fixture requires
`[1, 1]`. This is the valid callback-state contract RED that authorizes the registry and SDK factory
cutover; no GREEN result is claimed by this failure record.

2026-07-30 managed Runtime GREEN job `cd9164b34e224094b1abccbc87b1681e` /
`1556196d588c481d97adc7a0e70063e1` compiled the source-bound registry and SDK surface and executed
exactly four `scene_system_callback` tests: typed, external, runtime-instance, and independent-World
overlap all passed (`4 passed; 0 failed; 9273 filtered out`). The follow-up source-bound SDK command
`cargo +1.94.1 test -p zircon_plugin_sdk --lib runtime_registration_builder_hides_module_owner_sequence --locked --jobs 1 -- --nocapture --test-threads=1`
did not execute its target. Managed job `d30d4e49a2aa474f9143efbf547fbff6` /
`5dffe135b5f54b9aa1f3da35ef9a60c0` terminated `exit 101` while compiling the SDK's no-default-feature
`zircon_runtime` dependency: `scene/level_system.rs:6` and `scene/level_system/frame_state.rs:4`
unconditionally import `crate::animation` although `lib.rs` gates that module, and
`scene/level_system.rs:314` has an ambiguous `emitted_event_bytes` integer. This is a Frameworks01
scene-to-animation optional-feature compile blocker; it is not a callback-state or SDK target-test RED.

## 最低共享层根因

The registration builders describe a reusable system factory but retain mutable callback state outside the World-owned system instance.
The shared `Arc<Mutex<S>>` is therefore an execution-time global authority rather than a per-World factory product.
The public `RuntimePluginRuntimeSceneSystemBuilder` forwards the same callback type and must expose the resulting
per-instance factory contract rather than retaining the pre-cutover unconstrained callback bound.

## 架构修复验收

- Each typed, external, and runtime scene registration builds a fresh callback instance for each World or runtime scene instance.
- No callback executes behind a cross-World registry or registration mutex.
- Independent Worlds can overlap callbacks while same-World ordering and `SystemParam` access behavior remain unchanged.
- The plugin SDK forwarding builder carries the same factory-capable callback bound as the registry surface.
- Registration validation and duplicate-id errors remain transactional and unchanged.

## 禁止临时方案

- Do not retain `Arc<Mutex<S>>` as a fallback, compatibility path, or test-only bypass.
- Do not remove `FnMut` semantics by silently changing user callbacks to shared `Fn` callbacks.
- Do not move the shared lock to a schedule call site or weaken access-conflict rules to manufacture overlap.

## 修复结果与回传

Open state: `Runtime callback GREEN is recorded; SDK forwarding validation awaits the Frameworks01 optional-feature compile-boundary return`; no SDK target pass is claimed.
