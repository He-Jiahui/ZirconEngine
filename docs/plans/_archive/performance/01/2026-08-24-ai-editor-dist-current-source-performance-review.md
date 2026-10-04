---
title: AI Editor Dist Current-Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/ai/editor/src
  - zircon_plugins/ai/dist/src
status: static_complete_build_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeDebugger.cpp
---

# AI Editor Dist Current-Source Performance Review

## 1. Coverage and execution truth

The Editor/Dist production scope is **7/7 Rust files**, **1,155 physical / 1,057 non-empty lines**, **40,486 bytes** and **2 inline tests**. At repository revision `f811b3bf474d70347199772a175422333dfb36f6`, its ordered fingerprint is `626b1f9d66d5a3a5a190a1f2a6e22c2aabccfa507749bd3c047eea1dca7f7610`.

| Boundary | Files | Static result |
|---|---:|---|
| Editor registration/capability/plugin | 4 | Commands, asset type, graph palette, event consumers and contribution assembly reviewed. |
| Runtime mirror/overlay | 2 | Snapshot replacement, lookup/pruning, geometry construction and extension registration reviewed. |
| Dist | 1 | Native registration carrier and executable behavior parity reviewed. |

## 2. Product and performance findings

### P0: the Editor contribution no longer matches its host API

`editor/src/overlay.rs:2` imports `ViewportToolModeDescriptor` and `:133` calls `register_viewport_tool_mode`. The current editor contract exposes `SceneModeDescriptor`/`register_scene_mode` and `ViewportOverlayProviderRegistration`/`register_viewport_overlay_provider`; the referenced legacy names do not exist. The file declares `AI_PERCEPTION_OVERLAY_PROVIDER_ID` but registers no provider factory.

This is a build/product blocker before performance tuning. A non-compiling or uninstantiable overlay path cannot provide valid RenderDoc or frame-time evidence.

### P0: authoring commands are descriptors without executable operations

`editor/src/plugin.rs:120-187` registers Import/Open/Validate/Compile commands, importer, toolkit and graph palette descriptors. No AI Editor file registers operation factories that load, validate, compile and install an artifact. Tests assert descriptor presence, not execution. The Dist crate is a stateless native registration carrier rather than an equivalent editor/runtime implementation.

Measuring palette construction or command dispatch would optimize a shell, not the product chain. The acceptance path must be source asset -> typed edit transaction -> background compile receipt -> exact runtime generation -> PIE/debug mirror.

### P0: runtime emits full snapshots and Editor replaces full mirrors

`runtime_mirror.rs:68-97` removes every entry for a world and reinserts the complete snapshot. `agents_in_world` filters the whole BTreeMap (`:129-138`) instead of using the ordered key range, while `agent` scans all worlds (`:108-115`). Node-result pruning builds a nested map/set of cloned active-node strings and then retains the full result map (`:247-290`).

This compounds the runtime's unconditional snapshot building. The correct transport is generation-qualified deltas plus reader/selection demand, not faster cloning of full snapshots.

### P1: overlay generation makes two full passes and has no product budget

`overlay.rs:158-220` first calls `overlay_capacity`, which scans all mirrored agents/stimuli, then scans them again to construct output. Each visible receiver generates trigonometric sight/hearing geometry with up to 24 segments (`:266-336`) plus pick shapes and stimulus lines. There is no selected-agent, bounds, distance, frequency, byte, primitive or upload budget.

The disabled controller correctly emits no overlay. The enabled path still needs retained generation reuse and demand filters; exact preallocation is not enough when the product algorithm visits every agent twice.

## 3. Unreal source constraints

The local Unreal source demonstrates a complete authoring/debug chain rather than registration-only UI:

- `BehaviorTreeEditor.cpp:587-646` binds graph editing and breakpoint commands to executable callbacks. `:1139-1195` controls editability during PIE and updates the behavior asset/abort visualization when properties change.
- `BehaviorTreeDebugger.cpp:140-292` advances debugger state from execution steps, updates views only through debugger state transitions and exposes tickability separately. `BehaviorTreeEditor.cpp:538-562` renders inactive, paused and historical-step state from that debugger.

Zircon should use the same product distinction: editor descriptors declare surfaces, while factories/jobs/receipts own real asset compilation and debugger consumers own retained, requested runtime state. It should not copy Slate or UObject layout.

## 4. Dependency-ordered optimization plan

### M0: restore current editor contract and fail closed

Migrate the overlay contribution to the current scene-mode/provider registrations, add the provider factory and verify capability negotiation. Until then, hide/reject the mode rather than advertise an unavailable overlay. This must be done with the owner of the active editor API migration, not as an isolated compatibility shim.

### M1: implement the authoring product chain

Provide typed Import/Open/Validate/Compile/Toggle factories. Compile on Editor09 background jobs with owner/session/generation identity, progress, cancellation and stale-result rejection. Install the exact artifact generation used by PIE; persist edit transactions and errors rather than mock descriptor success.

### M2: replace snapshots with retained deltas

Subscribe only while a debugger/overlay reader is active. Publish added/changed/removed agent and node-result deltas with sequence/generation receipts. Store per-world maps or ordered ranges and update only affected rows/nodes. Enforce backlog/overflow and resync semantics.

### M3: bound and retain overlay geometry

Filter selected agents and viewport bounds before geometry generation. Cache per-agent geometry by debug generation/settings, use a circle unit-table or backend primitive where appropriate, and rebuild only changed agents. Enforce primitive, byte, upload and frequency budgets with truncation telemetry.

### M4: close Dist parity

Either provide a versioned bridge to the same operation/compiler/debug behavior or truthfully describe Dist as a registration carrier whose source runtime/editor dependencies are required. Test source/native profile capability matrices.

### M5: qualify editor scenarios

Measure project open, asset import/open/edit/validate/compile, PIE attach/detach, agent/node-result delta rates, overlay disabled/enabled/selected/bounded and large debug backlogs. Report UI-thread p50/p95/p99, job queue/wait, allocations/bytes, mirror entries changed, geometry primitives/uploads, CPU, wakeups, RSS and WPR power.

## 5. Acceptance gates

1. AI Editor compiles against the current extension API and every visible mode/provider has an instantiable factory.
2. Import/Open/Validate/Compile/Toggle execute typed operations; descriptor-only success is rejected.
3. Runtime debug cost is zero without a reader and proportional to changed/selected agents with a reader.
4. Mirror updates are delta/generation based and do not replace or scan all worlds on every delivery.
5. Overlay work is retained and bounded by selection/bounds/frequency/bytes/primitives.
6. Source and Dist capability matrices state where execution is actually hosted and fail closed when absent.
7. Current-source executable WPR and RenderDoc captures pass before protected-ledger promotion.

## 6. Validation status

- Per-production-Rust-file static review: **7/7 complete** for the captured fingerprint.
- Current API contract check: failed statically on legacy viewport-tool symbols; this is recorded as a blocker, not normalized with a shim.
- Cargo/tests: pending because the managed Windows validation session is not executable.
- WPR/ETW and RenderDoc: pending because no launchable current-source editor exists; RenderDoc will cover overlay draw/upload/GPU behavior only, not CPU ownership.
- Protected ledgers, milestone commit and WeCom completion remain pending.
