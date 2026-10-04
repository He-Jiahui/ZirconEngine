---
title: Plugin Timeline Sequence Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/timeline_sequence
  - zircon_runtime/src/animation/sequence
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
status: static_complete_shared_source_preserved_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/DragOperations/CurveEditorDragOperation_MoveKeys.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/ToolableTimeline/Caches/MultiChannelKeyCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/Compilation/MovieSceneCompiledDataManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntitySystemRunner.cpp
---

# Plugin Timeline Sequence Current Source Performance Review

## 1. Coverage and execution truth

The package review covers **6/6 Rust files**, **1,046 physical / 959 non-empty lines**, **38,641 bytes**, **15 test markers** and **1 ignored performance test**. The package-relative `path + NUL + LF-normalized bytes + NUL` SHA-256 is `2bcddda1b9800b982f55beaed611c75ffdfc2fa664dc485025063a57b84ec02e`.

| Module/folder | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| `editor` | 5 | 944 | Registers timeline metadata and exposes standalone validation/key-move helpers; no document/controller/compiler/operation handler. |
| `dist` | 1 | 102 | Publishes an editor registration manifest; no commands, bridge methods, state or unload behavior. |

Three package files already contain shared uncommitted work and were preserved. Per-file `rustfmt --check --edition 2021 --config skip_children=true` passes **4/6** files; shared `editor/src/lib.rs` and `editor/src/tests.rs` retain formatting differences. Related Animation Runtime tick and asset load/acquire files are also shared-modified and were inspected read-only.

Rust tests and dynamic tools were not run because the managed Windows validator is unavailable and no launchable current-source Timeline Sequence product exists.

## 2. Structural performance findings

### P0: the authoring package is unreachable and non-executable

The linked first-party Editor catalog does not select Timeline Sequence. Its only template resource, `plugins://timeline_sequence/editor/authoring.zui`, does not exist. Open, create track, delete track, move keyframe and validate operations have no production handlers outside descriptors/tests. Three declared track types and the timeline editor descriptor are metadata only. Native dist is stateless and exposes zero commands/bridge methods while reporting that authoring remains hosted by the Editor module.

The package is correctly Experimental, but descriptors cannot qualify a product. Readiness requires selected provider/resource closure, an Animation authoring document, stable typed operations, transaction receipts, compiler publication and runtime-backed preview.

### P0: one key move still scales with the complete Sequence

`move_timeline_keyframe` first calls `validate_timeline_sequence`, which traverses every binding, track and key and may allocate/sort/deduplicate diagnostic strings. It then binary-searches the target track but rotates the entire crossed slice. Its cost is therefore:

`O(total_sequence_keys + log(track_keys) + moved_distance)`

not `O(log(track_keys))`. During pointer drag this repeats for every input event. The request identifies binding/track/key by array index, so filtering, concurrent insertion or structural reordering can retarget a late event. The accepted move has no document revision, stable key ID, transaction, undo delta, coalescing key, compile generation or preview acknowledgement.

Preserve preflight-before-publish and stable equal-time ordering, but move them into a document transaction. Begin-drag snapshots selected stable IDs once; updates operate on a bounded scratch projection; commit validates affected channels plus cross-document invariants once, applies one reversible delta and publishes one compiler generation.

### P0: Runtime deep-clones every active Sequence every frame

Animation Runtime's tick maps every pending sequence player through `load_animation_sequence_asset`. Generic `load_typed` returns `asset.as_ref().clone()`, so each active player can deep-clone all bindings, entity/property paths, channels, keys and values every frame. `LoadedSequenceSample` owns that clone. The pipeline caches only `CompiledAnimationSequence` property writers by asset revision; it does not retain the immutable sequence lease used for sampling.

The existing `acquire_animation_sequence_asset` already returns a shared `ResourceLease<AnimationSequenceAsset>`, but adopting it is not just a call rename: lease lifetime, asset generation, replacement epoch, requested-player batching and cache retirement must share one owner. The frame path must retain one prepared lease/artifact per `(asset, revision/profile)` and let all players borrow it.

### P0: key sampling performs a full finite scan before binary search

`AnimationChannelSampleExt::sample` begins with `self.keys.iter().any(|key| !key.time_seconds.is_finite())` on every track, player and frame. Only after that `O(K)` pass does it use `partition_point` for interval lookup. `apply_compiled_sequence_to_world` samples every compiled track, so effective key lookup is still:

`O(active_players x compiled_tracks x keys_per_track)`

plus cloned output values and world property writes. The check cannot simply be deleted because binary asset import has no shared semantic compiler proving finite, ordered, type-compatible channels. Move finite/order/type/tangent validation to import/edit compile, publish a generation-qualified prepared channel, and make runtime sampling use validated arrays/cursors without per-frame whole-channel checks.

### P0: source, Editor model and Runtime prepared artifact do not share identity

`AnimationSequenceAsset` gives a binding an optional target ID, but tracks and keys have no stable IDs. Editor operation requests use vector indices. `CompiledAnimationSequenceTrack` also stores binding/track indices back into the mutable source asset; a structural edit with stale compiled projection can address different content unless asset revision/currentness invalidates it perfectly. The plugin's `TimelineEventMarker` is a separate DTO and is not stored in `AnimationSequenceAsset`; the declared event-marker track therefore has no source persistence or sequence runtime evaluation path. Runtime clip events are a different asset/domain.

Create stable binding/track/channel/key/event identities in one versioned source schema. Semantic compile produces immutable prepared channels, target writers, event cursors and a source-to-runtime debug map. Editor, preview and game Runtime must consume the same artifact/generation rather than three approximate models.

### P0: no workload scheduling or atomic evaluation transaction

Runtime walks loaded sequences synchronously inside the tick and writes sampled properties track by track. The cache removes non-requested assets every frame; there is no prepared page budget, multi-player batch, job DAG, deadline/cancellation, worker scratch, property-claim arbitration or atomic all-track commit. A late failure can increment missing counters after earlier properties were already written.

The target uses the shared Runtime task owner for `Update -> Sample -> Resolve Claims -> Commit`, batches by prepared artifact/channel layout, keeps deterministic owner-thread commit and reports deferred/stale/failure outcomes. Editor scrub uses the same evaluator in an isolated preview world, with a policy for events and pre-animated state.

### P1: auxiliary helpers allocate complete projections

`sorted_timeline_track_paths` formats a new `String` for every track and sorts the complete vector each call. This is acceptable at an explicit compiler/report boundary, not for repeated pane refresh. `validate_timeline_sequence` creates free-form strings and global sort/dedup even when a move only affects one channel. Projection pages should carry stable rows and typed diagnostics keyed by source IDs; view filtering/sorting must reuse revisioned indexes.

`validate_event_marker_payload` also fails to reject non-finite marker time/duration. Because the DTO is not persisted or executed, patching this helper alone would improve a test-only island while preserving the missing event schema.

### P1: the ignored benchmark hides ownership cost and compares two linear paths

The ignored gate uses one 16,384-key track and performs 32 independent moves per sample. It clones 32 complete sequences before starting the timer, excluding the largest current ownership cost. The optimized side still scans all 16,384 keys and rotates almost the full track on every move; it only replaces `sort_by` with binary position plus rotation. A 20% win over `O(n log n)` does not demonstrate interactive drag, multi-track validation, compiler publication, preview or Runtime performance.

Future evidence must include 100K/1M visible and off-screen keys, selected-key count, pointer-event coalescing, affected channels, validation/compile/publish time, allocations, undo bytes, preview latency, active players/tracks and per-frame clone/sample/write costs.

## 3. Unreal source evidence and adopted boundaries

Unreal Sequencer/Curve Editor provides the primary architecture evidence:

- `CurveEditorDragOperation_MoveKeys.cpp:34-67` snapshots selected stable key handles and starting positions once at drag begin.
- Lines 70-85 accumulate pointer movement and apply it at the input boundary; lines 89-109 restore the snapshot on cancel; lines 113-127 publish final positions and close the scoped change.
- Lines 141-217 reuse scratch storage, derive snapped deltas from the starting snapshot, batch `SetKeyPositions` as Interactive and retain last applied positions rather than revalidating every unrelated key.
- `MultiChannelKeyCache.h:44-88` builds caches by channel/range or selected handles; lines 194-240 use a parallel threshold for multi-channel recomputation; lines 258-299 prepare and batch key-time application.
- `MovieSceneCompiledDataManager.cpp:34-38` versions compiler logic; lines 432-458 invalidate/reset compiled data; lines 573-610 publish hierarchy, evaluation template/field, entity field, signature and compiler version.
- `MovieSceneEntitySystemRunner.cpp:20-43` exposes evaluation phases/counters; lines 148-220 queue updates rather than treating every edit/request as immediate whole-sequence work.

The transferable rules are stable handles, begin/update/commit/cancel drag state, channel/range caches, batched/parallelizable preparation, versioned compiled data and phased evaluation. Zircon should not copy Unreal's UObject/ECS class count, but it must preserve these complexity and ownership boundaries.

## 4. Required optimization sequence

| Milestone | Owner result | Acceptance gate |
|---|---|---|
| M0 Product truth | Select provider, add real resource/toolkit/document/handlers, keep unsupported tracks hidden or typed-unavailable. | Production bootstrap opens a real Sequence and every visible command returns a document/transaction receipt. |
| M1 Stable source schema | Versioned binding/track/channel/key/event IDs, rational time domain and shared semantic validation. | Reorder/filter/late events cannot retarget keys; malformed/non-finite/type-invalid source fails before publication. |
| M2 Prepared artifact/lease | Compile validated channels, target writers, event cursors and debug map into an immutable generation-qualified artifact. | Multiple players share one lease; frame path performs zero owned Sequence clones and zero full-key validation scans. |
| M3 Editor transaction | Stable selected-key snapshot, coalesced drag updates, bounded scratch projection, cancel restore and one commit/undo delta. | Drag cost scales with selected/affected keys; unrelated tracks are not scanned per pointer event; cancel restores exact source. |
| M4 Runtime DAG | Batch active instances by artifact, sample/decode on shared workers, resolve property claims and commit deterministically. | Counters explain queued/deferred/sample/commit work; no partial property publication on failure or stale generation. |
| M5 Preview/product projection | Runtime-backed isolated preview, virtualized time/row/key pages, visible-range query and density LOD. | 100K/1M-key documents keep bounded UI work; preview/game use identical artifact and sampling semantics. |
| M6 Dynamic acceptance | WPR/ETW CPU/allocation/power captures on current source and named scale corpus. | Editor input P50/P95/P99 and Runtime frame budgets pass; clone bytes and per-sample full-key scans are zero. |

## 5. Instrumentation contract

Record document/source/artifact generations, bindings/tracks/keys/events, selected/visible/affected keys, pointer events received/coalesced/committed, validation/compile/projection microseconds, allocations/undo bytes, active players, shared-lease hits, compiled cache hit/miss/reason, sampled tracks/keys searched, deferred jobs, property claims/writes/failures and preview/game generation parity.

WPR/ETW owns CPU scheduling, allocation, main-thread stalls and power evidence. RenderDoc is not a primary Timeline CPU tool; use it only when a real preview/render product needs pixel/pass verification.

## 6. This review's implementation decision

No production source was changed. The Editor helper, Runtime tick and asset load/acquire paths are shared-modified. More importantly, switching one call to a lease without defining cache/generation retirement, or deleting the per-sample finite scan without a semantic compiler, would trade measurable cost for lifetime/correctness risk. The first safe code milestone is M1/M2 with tests, followed by the frame-path hard cut.

Static review is complete for `zircon_plugins/timeline_sequence`; Runtime/Editor dynamic acceptance remains pending and this is not a milestone-completion claim.
