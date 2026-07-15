---
related_code:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_runtime/src/plugin/bridge/import.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/interfaces.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/prelude.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/integration.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/manager/validation/integration.rs
  - zircon_plugins/ai/runtime/src/manager/validation/runtime_inputs.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tests/integration_tasks.rs
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - docs/zircon_runtime/script/vm/behavior_bridge.md
tests:
  - managed job 4dc79d53e3af4391979ce9e819477920: cargo test -p zircon_plugin_ai_runtime integration_tasks (9 passed)
  - managed job f01ba36253a940cdb01ab32062b532e2: cargo test -p zircon_plugin_ai_runtime --locked
  - managed job 994e3cdbaaa54ea586a18a731d1d78c6: cargo test -p zircon_plugin_sdk --locked
  - managed job c72f831ac23045b68d685230d1613177: cargo test -p zircon_plugin_navigation_runtime --locked
  - managed job d1762caf2d724be29c698a3f5b35d966: cargo test -p zircon_plugin_zr_vm_language_runtime --locked
  - managed job f0565138c2174de39104a459b1496cb8: cargo test -p zircon_plugin_ai_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_sdk --locked --jobs 1
  - managed job a7eadca606ea4796a41a92abc25796da: cargo test -p zircon_runtime --features script,ai-contracts,net-contracts --lib interface_import --locked
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
doc_type: plan-output-record
---

# Plugins06 M3-T1–T3 integration task output record

Plan: docs/plans/zircon_plugins/06-ai.md
Milestone: M3.1
Status: completed
Files: ["docs/plans/zircon_plugins/06/2026-07-15-ai-m3-integration-task-output-records.md", "docs/zircon_plugins/ai/runtime.md", "docs/zircon_runtime/core/framework/navigation/agent_outcomes.md", "docs/zircon_runtime/script/vm/behavior_bridge.md", "zircon_plugins/ai/plugin.toml", "zircon_plugins/ai/runtime/Cargo.toml", "zircon_plugins/ai/runtime/src/behavior_tree.rs", "zircon_plugins/ai/runtime/src/behavior_tree/executor.rs", "zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs", "zircon_plugins/ai/runtime/src/behavior_tree/executor/integration.rs", "zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs", "zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs", "zircon_plugins/ai/runtime/src/manager.rs", "zircon_plugins/ai/runtime/src/manager/parameters.rs", "zircon_plugins/ai/runtime/src/manager/tick.rs", "zircon_plugins/ai/runtime/src/manager/validation.rs", "zircon_plugins/ai/runtime/src/manager/validation/integration.rs", "zircon_plugins/ai/runtime/src/manager/validation/runtime_inputs.rs", "zircon_plugins/ai/runtime/src/plugin.rs", "zircon_plugins/ai/runtime/src/plugin/registration.rs", "zircon_plugins/ai/runtime/src/tests/integration_tasks.rs", "zircon_plugins/ai/runtime/src/tests/mod.rs", "zircon_plugins/ai/runtime/src/tests/registration.rs", "zircon_plugins/navigation/runtime/src/agent.rs", "zircon_plugins/navigation/runtime/src/manager/tick.rs", "zircon_plugins/navigation/runtime/src/tests/crowd.rs", "zircon_plugins/plugin_sdk/src/lib.rs", "zircon_plugins/plugin_sdk/src/prelude.rs", "zircon_plugins/plugin_sdk/src/registration.rs", "zircon_plugins/zr_vm_language/plugin.toml", "zircon_plugins/zr_vm_language/runtime/src/lib.rs", "zircon_plugins/zr_vm_language/runtime/src/plugin.rs", "zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs", "zircon_runtime/src/core/framework/navigation/agent.rs", "zircon_runtime/src/core/framework/script.rs", "zircon_runtime/src/core/framework/script/behavior_bridge.rs", "zircon_runtime/src/plugin/bridge.rs", "zircon_runtime/src/plugin/bridge/import.rs", "zircon_runtime/src/plugin/bridge/weak.rs", "zircon_runtime/src/plugin/extension_registry_error.rs", "zircon_runtime/src/plugin/extension_registry/ownership.rs", "zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs", "zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs", "zircon_runtime/src/plugin/mod.rs", "zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/interfaces.rs", "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs", "zircon_runtime/src/script/mod.rs", "zircon_runtime/src/script/vm/behavior_bridge.rs", "zircon_runtime/src/script/vm/mod.rs", "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs"]
Date: 2026-07-16

| Date | Slice | Status | Evidence |
|---|---|---|---|
| 2026-07-15 | M3-T1 MoveTo, M3-T2 PlayAnimation, M3-T3 ScriptTask | Implemented; M3-T4 deferred until M4 | Focused AI job `4dc79d53e3af4391979ce9e819477920`: 9/9 integration tests. Refreshed package job `f0565138c2174de39104a459b1496cb8`: AI 67/67, Navigation 64/64, SDK 10/10 plus doctests. ZrVM job `d1762caf2d724be29c698a3f5b35d966`: full package plus doctests green. Current shared-source runtime job `a7eadca606ea4796a41a92abc25796da` compiled with `script,ai-contracts,net-contracts` and passed the final-merge/lifecycle `interface_import` test. |

## Scope Delivered

Architecture outcome: `behavior_tree/nodes/integration.rs` owns one focused AI-local port. The executor remains the state-machine owner and supplies per-agent context; the runtime adapter mutates only neutral World components/events. MoveTo consumes target-qualified production navigation outcomes without enabling debug capture, rejects stale feedback on task start, and resets the neutral destination on abort. ScriptTask uses a plugin-SDK `BridgeImport` declared in the AI manifest and resolved from the merged/finalized bridge table; ZrVM exports the real interface through a weak manager-backed provider. Import declarations are checked against manifest dependencies. Navigation, animation and script implementation crates are not imported by AI. `executor/integration.rs` and `validation/runtime_inputs.rs` keep production owners below the structure budget. Legacy explicit `result` parameters remain a deliberate compatibility path for the existing semantics matrix.

Reference outcome: Unreal task lifecycle and abort behavior lead the three-state semantics; Godot NavigationAgent feedback informs target completion/no-path mapping; Fyrox parameter containers inform typed animation writes. M3-T4 is not marked complete because the plan explicitly depends on M4 perception.

## Fresh Testing Evidence

- `4dc79d53e3af4391979ce9e819477920`: 9/9 focused AI integration behavior tests.
- `f0565138c2174de39104a459b1496cb8`: AI 67/67、Navigation 64/64、SDK 10/10 与 doctests。
- `d1762caf2d724be29c698a3f5b35d966`: ZrVM full package 与 doctests。
- `a7eadca606ea4796a41a92abc25796da`: 当前共享源码 Runtime 向上编译及 final-merge/lifecycle interface import 测试通过。
- Coordinator validation copy `6c113d3e76f04b8fb40274e7e8ad773d` / run `f6730a4c4fa64f198dc4b9c67e7888cc`: 24/24 coordinator-action tests，exit 0。

## Review

Independent review outcome: final read-only review reported Critical 0 / Important 0 after verifying owner-revoke import rebinding, linked-only ZrVM interface metadata, production navigation outcomes, first-tick stale guards, SDK registration paths, bidirectional manifest checks, weak VM ownership, provider-qualified generation handling, and the 790/706-line structure split.
