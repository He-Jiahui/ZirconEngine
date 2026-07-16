---
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/blackboard.rs
  - zircon_plugins/ai/runtime/src/blackboard/layout.rs
  - zircon_plugins/ai/runtime/src/blackboard/observer.rs
  - zircon_plugins/ai/runtime/src/blackboard/store.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/execution_gate.rs
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/selector.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/integration.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/condition.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/perception.rs
  - zircon_plugins/ai/runtime/src/perception/adapter.rs
  - zircon_plugins/ai/runtime/src/perception/components.rs
  - zircon_plugins/ai/runtime/src/perception/scan.rs
  - zircon_plugins/ai/runtime/src/perception/stimuli.rs
  - zircon_plugins/ai/runtime/src/manager/service.rs
  - zircon_plugins/ai/runtime/src/manager/snapshot.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/manager/validation/integration.rs
  - zircon_plugins/ai/runtime/src/manager/validation/runtime_inputs.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/manager_validation.rs
  - zircon_plugins/ai/runtime/src/tests/integration_tasks.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_compile.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_node_catalog.rs
  - zircon_plugins/ai/runtime/src/tests/module.rs
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
  - zircon_plugins/ai/runtime/src/tests/perception_runtime.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick.rs
  - zircon_plugins/ai/runtime/src/tests/blackboard_condition_abort.rs
  - zircon_plugins/ai/runtime/src/tests/blackboard_store.rs
  - zircon_plugins/ai/runtime/src/tests/observer_abort.rs
  - zircon_plugins/ai/runtime/src/tests/observer_binding_lifecycle.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/mod.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase/fixtures.rs
  - zircon_runtime/src/core/framework/ai/mod.rs
  - zircon_runtime/src/core/framework/ai/perception.rs
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/plugin/bridge/import.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - docs/zircon_runtime/core/framework/navigation/agent_outcomes.md
  - zircon_runtime/src/core/framework/ai/behavior_tree.rs
  - zircon_runtime/src/core/framework/ai/error.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/owner_revocation.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/systems.rs
implementation_files:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/blackboard.rs
  - zircon_plugins/ai/runtime/src/blackboard/layout.rs
  - zircon_plugins/ai/runtime/src/blackboard/observer.rs
  - zircon_plugins/ai/runtime/src/blackboard/store.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/selector.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/integration.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/condition.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/perception.rs
  - zircon_plugins/ai/runtime/src/perception/adapter.rs
  - zircon_plugins/ai/runtime/src/perception/components.rs
  - zircon_plugins/ai/runtime/src/perception/scan.rs
  - zircon_plugins/ai/runtime/src/perception/stimuli.rs
  - zircon_plugins/ai/runtime/src/manager/service.rs
  - zircon_plugins/ai/runtime/src/manager/snapshot.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/manager/validation/integration.rs
  - zircon_plugins/ai/runtime/src/manager/validation/runtime_inputs.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
tests:
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/manager_validation.rs
  - zircon_plugins/ai/runtime/src/tests/integration_tasks.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_compile.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_node_catalog.rs
  - zircon_plugins/ai/runtime/src/tests/module.rs
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
  - zircon_plugins/ai/runtime/src/tests/perception_runtime.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick.rs
  - zircon_plugins/ai/runtime/src/tests/blackboard_condition_abort.rs
  - zircon_plugins/ai/runtime/src/tests/blackboard_store.rs
  - zircon_plugins/ai/runtime/src/tests/observer_abort.rs
  - zircon_plugins/ai/runtime/src/tests/observer_binding_lifecycle.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/mod.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase/fixtures.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --jobs 1 --message-format short --color never
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
doc_type: module-detail
---

# AI Runtime Plugin

## Purpose

`zircon_plugins/ai/runtime` is the first-party optional AI runtime plugin. It embeds the canonical `ai.runtime` module descriptor, exposes `DefaultAiManager` through the stable manager service surface, and publishes package metadata for behavior-tree, blackboard, and perception capabilities. Plugins 06 M1 supplies compiled behavior-tree assets, a typed node implementation catalog, the standard v1 node families, per-agent execution state, and a real `ai.behavior_tick` Update system with LOD throttling. M2 adds schema-compiled dense Blackboard storage, per-slot generations and observer bindings, plus deterministic `None`/`Self`/`LowerPriority`/`Both` branch aborts. M3 adds real MoveTo, PlayAnimation, and ScriptTask handoffs through an AI-local integration port and closes with a scene-driven patrol/detect/chase scenario. M4 adds typed source/receiver ECS components, a fair frame-budgeted sight/hearing scan, stimulus aging and forgetting, optional physics occlusion, and bus-driven hearing stimuli. The package remains Experimental / Partial because M5 editor/debug tooling remains open.

## Related Files

`src/behavior_tree.rs` is the behavior-tree domain entry. `behavior_tree/compile.rs` parses `.btree.toml` into the neutral descriptor and compiles a preorder dense node array plus a separate dense direct-child index table. `catalog.rs` owns the `TypedExtensionPoint`-backed implementation catalog and the typed `ai.behavior_node_registry.v1` runtime interface; `nodes/{composite,decorator,service,task}.rs` own standard declarations, while `nodes/integration.rs` owns cross-subsystem task requests, result mapping, abort cleanup, World mutation, and neutral script-bridge calls. `executor.rs` owns traversal and per-agent state, `executor/selector.rs` owns selector resume, reactive branch rechecks, and lower-priority preemption, `executor/integration.rs` owns integration-task dispatch, `executor/abort.rs` owns sorted abort queues and active-branch cleanup, `executor/condition.rs` isolates blackboard/perception predicates, and `executor/support.rs` holds shared tree traversal support. `blackboard/layout.rs` compiles schemas to stable typed slots, `store.rs` owns dense arrays/generations/atomic synchronization, and `observer.rs` binds descriptor keys to slot-indexed observer buckets. The old `manager/execution.rs` was deleted in the planned hard cutover. `manager/behavior_tree.rs` validates and compiles registrations against the manager's live shared catalog; `manager/tick.rs` keeps lock scope outside execution and persists instance state. `manager/validation/runtime_inputs.rs` owns runtime blackboard and perception input checks, keeping production owners under the structure budget.

`perception/components.rs` owns the sparse ECS source/receiver components and authoring descriptors. `perception/scan.rs` owns the bounded pair cursor, cone/range tests, optional `physics.query.v1` raycast adapter, and tick report. `perception/stimuli.rs` owns per-receiver age, refresh, deterministic snapshot projection, and forgetting. `perception/adapter.rs` converts neutral sound/animation/gameplay hearing events into stimuli without coupling their producers to the AI manager. The 15-line `perception.rs` and the crate root only expose the curated surface. `plugin/registration.rs` owns both real Update-stage systems and their ordering, while `tick_lod.rs` owns deterministic Full/Half/Quarter behavior scheduling.

## Behavior Model

The plugin registers these capabilities:

- `runtime.plugin.ai`
- `runtime.feature.ai.behavior_tree`
- `runtime.feature.ai.blackboard`
- `runtime.feature.ai.perception`

`runtime.feature.ai.perception` is `Complete` at the M4 runtime boundary. M3-T4 now also closes the patrol/detect/chase runtime product scenario. The package, behavior-tree, and blackboard rows remain `Partial`; project/profile/export tooling therefore exposes the production runtime without pretending that EQS-style querying or M5 editor authoring/debug tooling is complete.

`DefaultAiManager` stores registered behavior-tree descriptors and blackboard schema descriptors behind stable numeric handles. It stores blackboard entries and perception snapshots per `(WorldHandle, EntityId)` and produces `AiRuntimeSnapshot` records for diagnostics or editor tooling.

## Control Flow

At registration time, `AiRuntimePlugin` constructs one shared `Arc<DefaultAiManager>` for the module service descriptor, runtime scene system, and behavior-node registry service. The plugin exports that service through `RuntimeExtensionRegistry` as the typed `ai.behavior_node_registry.v1` interface declared by both its linked descriptor and `plugin.toml`. Rust plugins can resolve the interface, contribute `BehaviorNodeDescriptor` values, and immediately register trees against the same live `RwLock`-backed catalog owned by the AI manager. Bootstrap standard-node rows are rebound in place to the actual interned AI owner, preserving their stable slots even when callers compiled trees through the public manager before plugin registration.

The same SDK registration owner declares `AiTickBudget`, `PerceivedStimuli`, the source/receiver component descriptors, and the neutral `AiHearingStimulusEvent`. It imports `physics.query.v1` as an optional weak lifecycle bridge. No physics provider yields deterministic range+cone sight; an enabled provider adds a closest-hit raycast and suppresses refresh when another entity blocks the source. The plugin never resolves a concrete physics manager or retains a direct provider object.

The central registry's owner-revocation listener first seals the owner against new execution leases, waits for in-flight node code and runtime objects, then removes that owner's private catalog slots and retires compiled trees, active instances, and reports before the contributor can unload. Next-generation registration waits for old cleanup and is linearized by the same owner lease. Composition code may also seed the catalog through `AiRuntimePlugin::with_behavior_node_catalog` or `DefaultAiManager::with_behavior_node_catalog`. An external node contributes a factory for a per-agent `BehaviorNodeRuntime` object whose tick context exposes parameters, blackboard, perception and elapsed delta. Standard nodes use the same catalog path. ScriptTask accepts only provider-qualified `<package>::<node-id>` callback refs and calls a plugin-SDK `BridgeImport<dyn ScriptBehaviorBridge>`. AI declares ZrVM as an optional interface dependency and never enables or resolves the concrete script subsystem. The import binds only after the merged registry finalizes its shared bridge table; disable, reload, diagnostics and provider absence are therefore observed through the normal plugin lifecycle. ZrVM's exported provider weakly binds its manager and refreshes callback handles by slot generation. Tree registration snapshots the current catalog, validates the versioned DTO, and compiles implementation names into stable dense catalog slots, semantics, optional factories, parameters, and direct-child ranges. Missing implementations and malformed topology return typed errors rather than flattened strings.

At tick time, `manager/tick.rs` snapshots the compiled tree set, schema and perception under a short poison-recovering lock, removes the agent instance state, executes without holding the manager lock, then writes the instance and report back. A schema-backed agent keeps its dense `BlackboardStore` across ticks; conditions read resolved slots directly, while snapshots clone DTO entries only at the external boundary. Manager writes preserve pending slot changes until the executor consumes them. Switching or disabling the root tree aborts active runtime objects before retiring prior node state. Sequence retains its active-child cursor, Parallel retains terminal branch results, and Selector resumes its Running child while evaluating explicit observer policies. When a previously failed higher-priority blackboard or perception decorator becomes eligible, the selector aborts the active lower branch before the replacement task mutates its integration host. A side-effect-free tri-state probe carries this ordering through nested Sequence, Selector, RandomSelector, and Parallel owners. Sequence proves preceding Blackboard guards in execution order, RandomSelector inspects only its persisted or deterministic current-tick child, and Parallel combines known-false reactive children with cached or explicit non-reactive outcomes under success-before-failure policy order. A fixed failed aggregate reuses the cached higher branch; a provably eligible aggregate aborts before MoveTo mutation; a condition that cannot be inspected remains deferred until its own tick. Unrelated nested fallbacks and unselected random children therefore cannot manufacture a reactive preemption. Both paths use the same subtree abort owner, so an active MoveTo receives `abort` and clears its old destination exactly once. Changed slots enqueue affected nodes in stable order: `Self` cleans the active observed branch, `LowerPriority` removes lower selector work and re-enters the high-priority branch, and `Both` performs both actions. `None` remains non-reactive. Aborting an active `RunSubtree` recursively cleans the target tree; terminal residual state is dropped without a duplicate callback, while active external runtimes receive `on_abort`. Cooldown state and terminal Parallel siblings survive unrelated branch aborts. Perception-only conditions may still opt into per-tick reactive evaluation without fabricating a Blackboard observer. RandomSelector uses non-negative `weight.<child_id>` or `weight_<index>` parameters and retains a selected Running branch. Cooldown, TimeLimit, Loop and Wait retain timers/counters per agent; Inverter and ForceResult transform terminal status; BlackboardCondition evaluates typed blackboard/perception predicates; UpdateBlackboardDistance exposes service callback status; RunSubtree enters a registered compiled tree with recursion protection. MoveTo writes the neutral `NavMeshAgent.destination` on its first tick, then consumes production `NavAgentTickReport.arrived_agents` / `no_path_agents` outcomes only when their destination matches the active request. It never enables or depends on navigation debug capture, and stale same-target feedback cannot complete a newly started task. Abort resets the destination to the entity position. PlayAnimation writes an explicit typed parameter value or a dedicated trigger to a state-machine player with graph-player fallback, sets `playing`, and completes. ScriptTask passes the agent as a VM `HostHandle` plus delta time and maps null/bool/string returns to node status. Public ticks without an integration host return `Blocked` instead of simulating an endless Running task. Explicit legacy `result` parameters remain supported for the semantics matrix. SetBlackboard and EmitEvent retain their staged dispatch.

The registered `ai.perception_tick` and `ai.behavior_tick` systems both run at `SystemStage::Update`, with an explicit `perception -> behavior` edge. Perception collects sorted receiver/source components, advances a persisted pair cursor by at most `AiTickBudget.max_pairs_per_frame`, refreshes visible or audible stimuli, ages every retained stimulus, and removes it when age reaches the receiver timeout. Event stimuli first enter a World-owned, 1024-entry bounded ingress backlog; hearing and animation readers share one per-frame ingest limit, so a burst cannot allocate an unbounded duplicate buffer or monopolize animation conversion. A sound-provider read failure leaves the backlog intact across the next `UpdateEvents` rotation, while activation changes and frame gaps clear both ECS cursors, the backlog, and its drop diagnostics. Sound playback uses a separate per-World sequence cursor over the optional Sound manager's non-destructive journal. A missing optional service leaves that cursor uninitialized; the first available read starts at the journal tail, while resolution or provider errors propagate instead of consuming unrelated bus input. The system replaces the manager's snapshots for that world before behavior evaluation, so perception decorators see the current frame. Behavior then reads the active camera and agent world transforms, maps distance to Full/Half/Quarter LOD, deterministically staggers half/quarter agents, accumulates skipped elapsed time per agent so timed nodes retain wall-clock duration, ticks the same shared manager, and emits `AiAgentTickReport` through the scene event store. Missing transforms conservatively use Full rate.

Decorator blackboard comparisons are intentionally parameter-driven. `blackboard_key` selects one runtime entry, `exists` checks presence or absence, `equals_bool` / `equals_string` / `equals_integer` / `equals_scalar` / `equals_vec3` / `equals_entity` compare exact typed values, and numeric threshold parameters `greater_than_integer`, `greater_or_equal_integer`, `less_than_integer`, `less_or_equal_integer`, `greater_than_scalar`, `greater_or_equal_scalar`, `less_than_scalar`, and `less_or_equal_scalar` compare integer or scalar blackboard attributes. Multiple value comparison parameters on the same decorator are combined with deterministic AND semantics. The optional bool `invert` parameter flips the final condition gate before child selection, but it does not invert the child task or subtree result. This follows Unreal's raw decorator condition plus inverse flag, vector blackboard equality, and arithmetic operation shape while keeping the concrete interpretation in the AI runtime plugin rather than adding an expression evaluator to the shared framework.

Decorator perception comparisons use the same deterministic condition gate. `perception_sense` filters snapshot stimuli by normalized sense (`sight`, `hearing`, `damage`, `touch`, or `custom`, with common aliases accepted at runtime), `perception_source` filters by source entity, `perception_min_strength` filters weak stimuli, `perception_max_age_seconds` filters stale stimuli, and `perception_exists = false` turns the filter into an absence check. Perception filters AND-compose with blackboard comparisons before `invert` is applied. This follows Unreal's `FAIStimulus` shape of sense, strength, location, and age plus active stimulus queries, while keeping Zircon's current implementation as a pure descriptor tick instead of adding a perception aging system or latent service scheduler.

## Edge Cases and Constraints

Behavior trees are rejected when the explicit format version is not `1`, ids are empty, node ids duplicate, the root node is missing, a child edge points to an unknown node, the DTO kind disagrees with the implementation catalog category, a node kind has an invalid child count, or the descriptor graph is not a root-owned tree. `.btree.toml` parsing preserves the TOML source error and runs the same structural/parameter validation before dense compilation; registered subtree existence is deferred to manager registration, where the complete registered-tree set is available. Topology validation walks from the root, rejects cycles before the recursive executor can see them, requires the root to have no incoming edge, requires every non-root node to have exactly one incoming edge, rejects duplicate or shared child edges, and rejects unreachable nodes. Selector, sequence, and parallel nodes may own child lists; decorators must own exactly one child; task, service, and subtree descriptors must be childless in this staged runtime. Behavior-node parameters are rejected when keys duplicate, values are non-finite, built-in parameter owners/types/values violate their contracts, or timing/count/weight/result values are invalid. Blackboard layouts reject duplicate keys and unknown value types. Observer-bound root or reachable subtree trees require a schema; missing keys, missing schemas, and unknown keys return distinct typed manager errors. Full entry synchronization validates every entry before mutation, so one invalid value cannot leave a partially updated store.

Invalid/non-positive source strength, invalid event position/strength, non-positive hearing range, and non-finite receiver ranges do not create stimuli. A zero pair budget still ages and forgets old stimuli and accepts event-driven hearing input. Source/receiver ordering and the persisted flattened cursor prevent a small budget from starving later pairs. Same-entity static pairs are not charged. Missing physics is an intended capability fallback; provider errors or disable transitions are observed as bridge absence, not converted into a stale strong reference.

M3-T4 closes only the runtime patrol/detect/chase product scenario. Graph authoring, runtime-node highlighting, Blackboard inspection, and perception overlays remain owned by M5; no editor completion is inferred from the runtime scenario.

## Test Coverage

The M1–M3-T3 suite covers the established compilation, catalog, executor, Blackboard, observer-abort, LOD, and lifecycle contracts plus MoveTo arrival/failure/abort/unavailable/stale-feedback mapping, typed animation trigger/scalar writes, and VM host-handle callback roundtrip. Windows managed focused job `4dc79d53e3af4391979ce9e819477920` passed all 9 integration-task behavior tests. The refreshed current-source package job `f0565138c2174de39104a459b1496cb8` passed AI 67/67, Navigation 64/64, plugin SDK 10/10, and all three doctest sets. ZrVM package job `d1762caf2d724be29c698a3f5b35d966` passed its full package tests and doctests.

The runtime manifest contribution test keeps static `plugin.toml`, linked descriptor metadata, and the built-in catalog in sync. The planned scoped verification commands are listed in the document header and should be rerun during the milestone testing stage.

M3-T4 behavior coverage is owned by `src/tests/scenarios/patrol_detect_chase.rs`. One test drives a real scene level through plugin registration, starts patrol with the production integration host, enables a sight source, observes the same-frame perception-before-behavior switch to chase, injects a target-qualified navigation arrival, and observes the emitted success report. The focused preemption test fixes the integration-host order as patrol move, patrol abort, then chase move, preventing stale lower-priority state from surviving a reactive selector transition.

The accepted Windows managed M3-T4 focused job `ceae105af06b4885b8852dc51a6ff695` passed the initial two scenario tests (2/2, exit 0), and focused regression job `c0619de2d83a46c48a39569f70b593a3` preserved the `None`/`Self` observer-policy contract. Independent review then expanded coverage for nested execution order, Parallel success/failure aggregation, RandomSelector selection, and host mutation ordering. The first refreshed full-package job `bdcd816e50d7425ba6fdd655ff9c9181` compiled current source and passed 95/96 tests; its only failure exposed an invalid zero-child Decorator fixture. That fixture now owns the required succeeded child, and the 884-line test owner is split into a 405-line behavior owner plus a 506-line folder-backed fixture owner. The final post-split managed package gate is pending.

M4 behavior coverage is owned by `src/tests/perception_runtime.rs`: pair-budget capping, cursor fairness, timeout forgetting, raycast occlusion, no-physics cone fallback, sound-event hearing registration, stable receiver snapshots, bounded/aged ingress, activation and frame-gap resets, sound-error recovery across a real event-store rotation, SDK resource/component/manifest projection, schedule ordering, and real scene-system-to-manager snapshot propagation. Sound playback tests additionally cover independent readers, per-World capacity/sequence isolation, and overwritten-history reporting.

The accepted current-source M4 gates are Windows managed jobs `14793e415ec1442c8de52545b1d59eed` for the full AI package, `0d9f89ecd3b04a95acb77adfe2ec87cd` for the full Sound package, `839e57989b4842ccab52fb635cc71548` for the full plugin SDK package, and `62e2f5f56994496e984a1f3eb4142d1b` for the upward `zircon_runtime` build. All four exited successfully. The final independent review reported Critical 0 / Important 0, and the plugin structure audit reported zero manifest, free-function registration, compatibility-shim, SDK-mirror, and distribution-boundary violations.
