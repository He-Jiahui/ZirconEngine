# Sound Dynamic Event Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Execute registered sound dynamic event handlers through a deterministic runtime-local executor registry.

**Architecture:** Neutral report DTOs live in `zircon_runtime::core::framework::sound`. Concrete callback storage and execution live in `zircon_plugins/sound/runtime`; execution reuses the existing descriptor validation and deterministic delivery ordering.

**Tech Stack:** Rust, Cargo, `Arc<dyn Fn(...)>`, existing sound dynamic event registry/tests.

---

## Source Map

- Modify `zircon_runtime/src/core/framework/sound/events.rs`: add execution status/result/report DTOs.
- Modify `zircon_runtime/src/core/framework/sound/manager.rs`: add execute API. Runtime-local executor registration will be an inherent `DefaultSoundManager` API because callback trait objects are not neutral DTOs.
- Modify `zircon_runtime/src/core/framework/sound/mod.rs`: export new DTOs.
- Modify `zircon_plugins/sound/runtime/src/dynamic_events.rs`: add helper for deterministic deliveries without duplicating ordering logic if needed.
- Modify `zircon_plugins/sound/runtime/src/engine/state.rs`: store executor callbacks.
- Modify `zircon_plugins/sound/runtime/src/service_types.rs`: implement executor registration/unregistration and `SoundManager::execute_dynamic_events`.
- Modify `zircon_plugins/sound/runtime/src/tests/dynamic_events.rs`: add execution tests.
- Update `docs/engine-architecture/runtime-sound-extension.md` and `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md`.

## Milestone 1: Neutral Execution Reports

- [x] Add DTOs in `events.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundDynamicEventExecutionStatus {
    Succeeded,
    Failed,
    SkippedMissingExecutor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundDynamicEventHandlerExecution {
    pub delivery: SoundDynamicEventDelivery,
    pub status: SoundDynamicEventExecutionStatus,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundDynamicEventExecutionReport {
    pub executions: Vec<SoundDynamicEventHandlerExecution>,
}
```

- [x] Export the DTOs from `mod.rs`.
- [x] Add `fn execute_dynamic_events(&self) -> Result<SoundDynamicEventExecutionReport, SoundError>;` to `SoundManager`.

## Milestone 2: Runtime Executor Registry

- [x] In `engine/state.rs`, add executor key/type and state storage.

```rust
pub(crate) type SoundDynamicEventExecutor = Arc<dyn Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SoundDynamicEventExecutorKey {
    pub(crate) plugin_id: String,
    pub(crate) handler_id: String,
}
```

Add `dynamic_event_executors: HashMap<SoundDynamicEventExecutorKey, SoundDynamicEventExecutor>` to `SoundEngineState`.

- [x] In `service_types.rs`, add inherent methods on `DefaultSoundManager`:

```rust
pub fn register_dynamic_event_executor<F>(
    &self,
    plugin_id: impl Into<String>,
    handler_id: impl Into<String>,
    executor: F,
) -> Result<(), SoundError>
where
    F: Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static;

pub fn unregister_dynamic_event_executor(
    &self,
    plugin_id: &str,
    handler_id: &str,
) -> Result<(), SoundError>;
```

Registration must require an existing handler descriptor. Duplicate registration replaces the executor.

- [x] Clean up executors when `unregister_dynamic_event` or `unregister_dynamic_event_handler` removes descriptors.

## Milestone 3: Execution Flow

- [x] Implement `SoundManager::execute_dynamic_events` in `DefaultSoundManager`.
- [x] Reuse existing deterministic delivery computation.
- [x] For every delivery, look up the executor key. Missing executor becomes `SkippedMissingExecutor`.
- [x] Executor `Ok(())` becomes `Succeeded`.
- [x] Executor `Err(detail)` becomes `Failed` with detail.
- [x] Do not abort on failure; keep executing later deliveries.

## Milestone 4: Tests And Docs

- [x] Add focused dynamic event tests for registration validation, ordering, skipped missing executors, failure continuation, and cleanup.
- [x] Update docs/session notes with behavior, tests, and remaining ABI callback gap.

## Milestone 5: Testing Stage

- [x] Run formatting:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
rustfmt --check "zircon_runtime\src\core\framework\sound\events.rs" "zircon_runtime\src\core\framework\sound\manager.rs" "zircon_runtime\src\core\framework\sound\mod.rs"
```

- [x] Run focused dynamic event tests:

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" dynamic_events --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-dynamic-event-execution" --message-format short --color never
```

- [x] Run whitespace check:

```powershell
git diff --check -- "zircon_runtime\src\core\framework\sound" "zircon_plugins\sound" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md" "docs\superpowers\specs\2026-05-04-sound-dynamic-event-execution-design.md" "docs\superpowers\plans\2026-05-04-sound-dynamic-event-execution.md"
```

## Acceptance Criteria

- Dynamic events can execute registered runtime-local handlers.
- Execution order matches delivery order.
- Missing executors and failing executors are reported per handler.
- Descriptor cleanup removes executors.
- Docs narrow the remaining gap to ABI/dynamic-library plugin callback execution and editor routing.
