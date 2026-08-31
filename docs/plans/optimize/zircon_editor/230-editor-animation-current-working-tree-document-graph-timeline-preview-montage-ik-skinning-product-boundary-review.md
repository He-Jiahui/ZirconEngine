---
title: Editor Animation 当前工作树 Document、Graph、Timeline、Preview、Montage、IK、Skinning 与 Product Boundary 复审及重构计划
category: zircon_editor
report_id: Editor230
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/136-editor-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/201-editor-animation-montage-section-slot-segment-notify-branching-point-sync-root-motion-runtime-playback-preview-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/170-runtime-animation-current-working-tree-source-compiled-pose-skinning-ik-root-motion-event-editor-boundary-review.md
related_code:
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/timeline
  - zircon_editor/src/ui/curve
  - zircon_plugins/animation/editor
  - zircon_plugins/animation_graph/editor
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation
  - zircon_runtime/src/core/framework/animation/compiler
plan_sources:
  - docs/plans/optimize/zircon_editor/136-editor-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/201-editor-animation-montage-section-slot-segment-notify-branching-point-sync-root-motion-runtime-playback-preview-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequenceBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimMontage.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimNotifyQueue.h
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimMontagePanel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimMontageSectionsPanel.cpp
  - dev/godot/scene/resources/animation.h
  - dev/godot/scene/resources/animation.cpp
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/godot/scene/animation/animation_player.cpp
  - dev/godot/editor/animation/animation_track_editor.cpp
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/signal.rs
  - dev/Fyrox/editor/src/plugins/inspector/editors/animation.rs
  - dev/bevy/crates/bevy_animation/src/animation_curves.rs
  - dev/bevy/crates/bevy_animation/src/animation_event.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionBakeCurve.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionSampleCurve.cs
---

# 结论

当前 Editor 已经从纯占位向可审计的 animation document 迈进：`AnimationAuthoringDocument` 是独立 source owner，带 revision、typed mutation、transaction/history、compile diagnostics 和 last-good product；session 也开始拆出 sequence、graph、state-machine、timeline、curve foundation。插件注册了 Animation/Animation Graph authoring contribution、graph palette、validate/compile/open operation，host 具备 document 路由和 canonical save/autosave 入口。

然而这仍是“文档与面板底座”，不是可交付的动画编辑器。`AnimationDocumentCompilation` 每次 mutation 同步重编译，并把 `AnimationCompileProduct` 直接放进内存 last-good；没有 artifact id、compiler fingerprint、dependency closure、async job 或 runtime install receipt。sequence/graph body ZUI 只有 header 和 slot，真实 timeline/canvas 的 binding、selection、curve editing、graph interaction 和 preview consumer 不在资产里。Workbench 的 Montage、Control Rig、Motion Matching、Retarget、Pose Library、Compression 页面是真实可解析的 ZUI，却用 `visibility = "collapsed"` 和固定文本/route，没有 document kind、operation factory 或 runtime artifact。编辑器无法证明一个字段已经影响 runtime、cook 或 renderer。

## 当前工作树证据

| 选择集 | files / lines / bytes / test attrs / ignored | fingerprint |
|---|---:|---|
| Animation document/session/timeline/curve | **47 / 4,779 / 181,269 / 23 / 0** | `2982876f5b0358bd150a25df17534d08582267effb51f85f3ccec7177eec7f9a` |
| Animation/Animation Graph editor plugin | **10 / 1,115 / 45,155 / 13 / 0** | `189fd013c957a589a071cfdd5e32672f478c70b1b205f7878218267d7a5c4cd2` |
| Runtime/compiler owner evidence | **166 / 18,901 / 727,120 / 165 / 1** | `81ed726fe992f714786f9fb6d40d701385ae505b50c723e1d937f66e61ebf043` |
| Animation ZUI host/workbench assets | **13 / 156,363 bytes** | current working-tree asset snapshot |

The editor animation path is dirty and partly untracked: document store, curve/timeline foundations, history, route-loading tests and session modules are new; host editing/lifecycle/save files and plugin graph/runtime files are modified. This report does not discard or normalize those changes.

## Findings

### ED-AN-01 — Document source and executable artifact are conflated (P0, Open)

`zircon_editor/src/core/editing/animation_document/compilation.rs:7-73` compiles on construction and on every `swap_asset_if_revision`, stores the current product and an in-memory last-good product. `is_successful()` only means the source-only compiler returned an artifact-shaped enum; it does not identify a content-addressed artifact, compiler version, dependency generation, skeleton binding or runtime installation. A broken edit can remain visible while a different runtime cache continues executing a previous plugin artifact with no explicit status contract.

Introduce `AnimationDocumentSnapshot` and `AnimationArtifactReceipt`: source revision, semantic digest, compiler fingerprint, dependency snapshot, diagnostics, last-good generation and runtime install status. Compilation must be an editor job with cancellation and stale-result rejection. Save, preview, cook and runtime install must consume the same receipt, not a cloned enum.

### ED-AN-02 — Mutation/history is typed but still a single-document local transaction (P1, Open)

Host editing correctly prepares a replacement before opening `HistoryContextId::Document` and commits through `AnimationEditCommand`, but session playback state (`current_frame`, range, playing, looping, speed, selected span) is separate transient state. There is no document participant for external asset reload, no collaboration/merge identity, no multi-document graph/clip dependency transaction and no artifact rollback receipt. A graph node edit can therefore be undoable in source while preview/runtime remains on an unrelated compiled generation.

Make history entries carry document key, source revision, artifact generation, dependency set and preview/runtime effects. Add conflict-aware external reload and a dependency transaction for graph -> state machine -> clip/skeleton edits. Keep transport transient but publish its cursor/generation to preview so stale frames cannot commit.

### ED-AN-03 — Sequence/timeline UI foundations have no authoritative data projection (P0, Open)

`animation_sequence_body.zui` is only a header plus `animation_timeline_slot`; `AnimationEditorSession` initializes a frame/range/selection model but does not own a projected track tree, key identity, curve data, event markers or virtualized row source. Existing timeline/curve foundations use `AnimationTrackPath` and source indices, while the runtime compiler has separate channel semantics. Exact-key, equal-time, quaternion tangent, morph curve and event id policy is not shared with runtime.

Add a read-only `AnimationTimelineSnapshot` generated from the document/artifact: stable track/key/event ids, typed channel metadata, frame/time mapping, diagnostics and viewport generation. Commands must address ids, not vector indices. The UI should virtualize rows and keys from this snapshot and route every edit through the document mutation engine.

### ED-AN-04 — Graph editor registration is not graph product execution (P0, Open)

`zircon_plugins/animation_graph/editor/src/plugin.rs` registers palette, graph editor descriptors and validate/compile operation paths. Its `lib.rs` validator checks node ids, references, output count and cycles, while `compile_animation_graph` returns only the selected output source string; state-machine compile returns counts and entry state. No operation factory in the shown editor path opens a graph document, produces a cooked bytecode artifact, installs it in PreviewWorld or reports runtime evaluator compatibility. The runtime plugin has a second compile/evaluate implementation.

Make graph editor operations resolve the canonical document store, invoke the shared compiler job, and return a typed operation receipt with artifact id and diagnostics. Graph canvas must edit stable node/edge ids and preserve layout metadata. Validation, compile, preview and save must all use the same dependency snapshot and generation.

### ED-AN-05 — Preview is a slot and a route, not a PreviewWorld (P0, Open)

Host routing/lifecycle can open animation sessions and save canonical bytes, but the sequence and graph body ZUI only provide empty slots. There is no documented preview world/entity, skeleton binding, camera, playback clock, pose generation, event journal, render submission or stale-frame rejection in the animation editor path. A successful `Preview` route in the workbench cannot prove that the edited graph drives a skinned mesh.

Create a per-session `AnimationPreviewWorld` with explicit source/artifact/world generations, fixed or scrub clock, target skeleton, camera, render surface and event sink. Preview commands must return `Preparing/Ready/Failed/Cancelled` receipts and only present frames matching the current document/artifact generation. Add import -> edit -> compile -> preview -> reload tests with IBM and morph fixtures.

### ED-AN-06 — Advanced animation workspaces are presentation fixtures (P1, Open)

The workbench assets for Montage, Control Rig, Motion Matching, Retarget, Pose Library, Compression and Sequencer contain polished sections, tables and routes. They are collapsed components with hard-coded names, values and statuses; they do not appear in the Animation plugin contribution list, and the current runtime has no corresponding artifact owner for montage slots, root motion, retarget profiles, motion databases, pose assets or compression settings. This is a high-risk false-success surface because the UI implies product maturity.

Either hide these workspaces behind an unavailable capability with a diagnostic, or register each only after a typed asset kind, document, operation factory, compiler artifact, preview consumer and save/reload path exists. Do not accept field events into a control-local feedback loop.

### ED-AN-07 — Asset routing is incomplete for clip/skeleton and dependency graphs (P1, Open)

Current host route logic covers sequence, graph and state-machine document kinds. There is no equivalent first-class animation clip/skeleton/skin binding editor route in the new document store, even though runtime asset import creates labeled skeleton/clip and detached IBM Data assets. A graph or sequence can refer to a clip whose skeleton binding is missing or stale without an editor-level dependency panel, repair action or artifact closure indicator.

Add clip/skeleton/binding document kinds or a read-only dependency inspector with typed references, source/import status, artifact generations and repair candidates. Opening a graph must resolve dependencies through the asset graph and surface missing/mismatched skeleton binding before preview.

### ED-AN-08 — Editor/runtime skinning and IK diagnostics have no visible contract (P1, Open)

The runtime plugin currently exposes IK math jobs but no production evaluation stage, and the renderer has a separate palette computation. The editor has no skeleton binding remap view, IBM inspection, palette generation status, IK solve diagnostic stream or CPU/GPU deformation fallback report. A preview can look correct for an identity skeleton while failing with reordered joints or non-uniform scale.

Add binding and deformation diagnostics to the document/preview snapshot: remap table, IBM source, pose generation, solver status, current/previous palette upload and device capability. Display errors as typed diagnostics attached to the relevant asset/key/node, not as a generic Ready label.

### ED-AN-09 — Transport and event editing are not runtime playback semantics (P1, Open)

The session owns `current_frame`, `playing`, `looping` and `speed`, but has no direction/discontinuity/loop occurrence cursor and no terminal contract for reverse, seek or end-of-clip. Event tracks have no stable ids in the asset schema; timeline selection uses path/frame rather than key/event identity. Scrubbing therefore cannot guarantee notify dedupe or the same pose/event result as runtime.

Share the runtime `PlaybackCursor` and event journal schema with the editor. Scrub/preview should explicitly choose evaluate-only, fire-events or suppress-events mode and return a receipt. Add tests for reverse, loop boundary, same-time markers, seek and stale preview cancellation.

### ED-AN-10 — Save/autosave is canonical bytes but not artifact-aware publication (P1, Open)

The host save path now has atomic/durability stages and autosave payloads, which is useful. It writes document bytes, but does not publish the compiler artifact receipt, dependency generation, preview generation or runtime install status. A save can succeed while compile is stale, and a reload can reopen source without the last-good artifact provenance.

Make save a two-phase source/artifact publication: source commit with expected revision, compiler receipt publication, dependency closure and recovery record. On reopen, show current/last-good artifact and diagnostics; never infer Ready from successful byte I/O.

## Refactor order

1. **M0 capability truth:** hide or fail-closed all fixture workspaces; register only real document kinds and operation factories.
2. **M1 canonical document/artifact:** add source snapshot, dependency graph, compiler job, artifact receipt and runtime install status to history/save.
3. **M2 timeline/graph projection:** stable ids, typed channel/event metadata, virtualization and ID-based mutations shared with runtime compiler.
4. **M3 PreviewWorld:** per-session clock/world/render/event ownership, generation-qualified frames and cancellation/terminal receipts.
5. **M4 skin/IK diagnostics:** binding remap/IBM inspection, IK post-process diagnostics and CPU/GPU deformation status.
6. **M5 advanced products:** implement montage, retarget, pose library, compression, motion matching, control rig and cinematic integrations only after their runtime artifacts and cook paths exist.

## Qualification gates

| Gate | Required evidence | Current |
|---|---|---|
| ED-AN-1 document/artifact identity | Source revision, artifact id, compiler fingerprint and dependency generation are visible and persisted | Fail |
| ED-AN-2 mutation/history | Undo/redo/reload/recompile/preview effects are one qualified transaction | Partial |
| ED-AN-3 timeline truth | Stable track/key/event ids and typed curves drive both UI and compiler | Fail |
| ED-AN-4 graph truth | Node/edge edits produce the canonical compiled graph and receipt | Fail |
| ED-AN-5 PreviewWorld | Edited clip/graph drives a real skeleton/mesh with generation-safe frame output | Fail |
| ED-AN-6 transport/events | Reverse/loop/seek/scrub and notify modes match runtime | Fail |
| ED-AN-7 skin/IK diagnostics | IBM/remap/palette/IK status and failures are inspectable | Fail |
| ED-AN-8 save/reopen | Source and artifact publication recover atomically after interruption | Partial |
| ED-AN-9 advanced workspace truth | Montage/retarget/pose/control-rig/motion matching each have owner/artifact/consumer | Fail |
| ED-AN-10 scale/soak | Large timeline/graph, preview replacement and long playback remain bounded | Fail |

Current total: **8 Fail / 2 Partial / 0 Pass**. Editor, PreviewWorld, render-device and long playback validation were not run in this review-only pass.

## Reference comparison

- Unreal Persona/Sequencer separates animation data model, compiled derived data, per-instance playback, sync groups, notify queue and montage sections. Zircon currently has document/session foundations but no equivalent artifact/PreviewWorld/notify receipt boundary.
- Godot's animation resource/player/mixer and track editor show that editing, playback and skeleton application need explicit owners; Fyrox track/signal/machine types show stable signal and track concepts; Bevy curves/events/transitions show typed curve and transition data. These are useful contract references, not proof that a single engine feature is complete.
- Unity Graphics curve bake/sample files are relevant only for source-versus-baked curve separation and typed curve slots; the local checkout is not a full Unity animation editor.

This report is review-only and deliberately does not change production code, tests, Cargo manifests, ABI or ZUI assets.
