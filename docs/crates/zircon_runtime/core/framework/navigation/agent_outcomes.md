---
related_code:
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
tests:
  - zircon_plugins/navigation/runtime/src/tests/crowd.rs
  - zircon_plugins/ai/runtime/src/tests/integration_tasks.rs
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-review-findings-2026-06.md
doc_type: module-detail
---

# Navigation agent production outcomes

## Contract

`NavAgentTickReport` publishes lightweight production outcomes independently of `NavigationDebugCapture`:

- `arrived_agents`: `(entity, destination)` pairs whose evaluated destination reached the stopping threshold.
- `no_path_agents`: `(entity, destination)` pairs whose current request could not obtain a path.

The destination is part of each outcome so a consumer can reject feedback from an older request. `debug_agents` remains optional diagnostic data and is populated only when debug capture is enabled; gameplay and AI correctness must not depend on it.

Both the crowd-backed path and the legacy manager tick path emit the production outcomes. The AI MoveTo adapter detects navigation availability through the registered `NavAgentTickReport` event channel, writes the requested destination on the first task tick, and does not consume feedback until the task has established that target. Later ticks accept an arrival or no-path result only when its destination matches the active request. Abort resets the destination to the entity position.

## Validation

Navigation tests prove that an arrival is emitted while debug capture is disabled. AI behavior tests prove Running-to-Success, Running-to-Failure, abort cleanup, unavailable navigation, and rejection of stale feedback for the same target before a new task has started.
