---
title: Editor root paint-frame and command ownership performance review
date: 2026-08-22
module: zircon_editor retained-host root composition, paint_frame and paint_recording
priority: MVP-P0 editor frame recording and root fallback composition
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate window element list, cached elements and invalidation root
---

# Goal

Give one retained window frame/context stable ownership of paint snapshots and reusable command
storage. Full and damage paints must not deep-copy frame state, recreate all command allocations or
duplicate text/image identifiers when the corresponding generations are unchanged.

## Reviewed source

- Rust files: 26/26
- lines: 1,074
- bytes: 35,171
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `bf8711dfd9041b0fb86c131a3e52f310b872273dd5f7757c99fdf637c4f5e501`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope:

- `paint_workbench_renderer/{host_window.rs,root_frames{.rs,/**},skeleton{.rs,/**},style.rs,text.rs}`
- `paint_frame.rs` and `paint_frame/**`
- `paint_recording.rs` and `paint_recording/**`

Supporting production paths traced/read: workbench command selection, full/damage CPU test frame,
chrome command-stream replay, GPU presenter, softbuffer presenter, presentation snapshot and Runtime
text recording bridge.

## Correct foundations to retain

1. Root-frame resolution is fixed O(1), uses typed layout data and only falls back when committed root
   frames are absent.
2. CPU damage repaint keeps the previous framebuffer, clips/clears only exact damage and restores the
   prior clip after repaint.
3. Rect, border, separator and Runtime text primitives reject disjoint active damage before command
   recording, text layout or pixel fill. Root skeleton calls therefore do not imply full-window writes.
4. Production GPU/softbuffer presentation records commands without allocating an RGBA framebuffer;
   `HostRgbaFrame::filled` is mainly snapshot/initial CPU/replay/test work, not the steady command path.
5. Recording uses monotonic z indices, and command replay avoids sorting when order is already valid.
6. Text layout is retained behind an `Arc` cache before recording; CPU/GPU image payloads already use
   `Arc<[u8]>` at the command boundary.

## Structural findings

### P0: frame interaction state breaks shared ownership with a deep clone

Presentation paint generation returns `Arc<HostPaneInteractionStateData>`. All three workbench entry
paths pass a reference to `HostRgbaFrame::set_pane_interaction_state`, which clones the complete state
into an owned `Option<HostPaneInteractionStateData>`. The frame only reads this snapshot during the
same paint and never mutates it.

M1 stores the existing `Arc` in the frame and returns `as_deref()` to painters. With active paint
overrides, underlying state clones per frame become `1 -> 0`; without the override scope they become
`2 -> 1` because the fallback generation still must snapshot presentation data once. Pointer-owner
operations remain O(1), and no lifetime is tied to the mutable presentation.

### P0: every command recording starts with fresh growable storage

`record_host_frame_commands` creates `HostRgbaFrame::recording_only`; `HostPaintRecording::record_only`
uses a default empty `Vec`, then pushes every accepted command. Stable frames therefore repeat Vec
growth/allocation even when the prior command count and most retained ranges are known.

M2 makes command storage a reusable per-window arena with generation/range ownership. Reset preserves
capacity and invalidation patches dirty ranges. A guessed fixed capacity is rejected because editor
documents, plugins and text density vary by orders of magnitude.

### P0: recorded text and resource identifiers duplicate owned strings

Runtime text layout is returned as `Arc<PaintTextLayout>` with owned display text, but recording passes
`&str` to `record_text`, which creates another `String` for every visible text command. Image and atlas
commands likewise own `String` resource keys. This remains O(total accepted text/resource bytes) per
recording even when layout/resource generations are stable.

M3 stores generation-qualified shared/interned text and resource handles in commands. The text layout
cache and resource registry own bytes; commands own compact handles. Hard cutover must remove duplicate
String command variants rather than wrap them in another facade.

### P0: command output remains one immediate flat Vec

Damage culling reduces emitted commands, but no stable root/chrome/dock/overlay range survives between
recordings. Every accepted leaf rebuilds command structs and assigns new z indices. The scene-layer
plan owns typed dirty ranges; M4 integrates those ranges into reusable paint storage and preserves
stable ordering without replay sorting.

### P1: CPU full-frame construction always allocates and initializes all pixels

`HostRgbaFrame::filled` allocates `width * height * 4` and writes every pixel. This is correct for
snapshots, first CPU frame and explicit raster tests, while region repaint correctly reuses storage.
M5 adds reusable/resize-aware snapshot buffers only where call-site ownership supports it; production
command recording must never regress to an RGBA allocation.

### P1: clip/intersection implementations are duplicated

`paint_frame::geometry`, `paint_recording::damage` and other paint geometry owners each implement
nearly identical finite rectangle intersection. This is small per call but risks semantic drift around
NaN/empty bounds. M5 converges on one typed geometry contract after tests prove identical behavior.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`

Unreal creates one `FSlateInvalidationContext` and one `FSlateWindowElementList` for window paint.
The element list is noncopyable, separates cached and uncached draw elements, supports cached-element
push/pop and resets its element list for reuse. The invalidation root selects cached fast-path updates
or a slow rebuild; widget proxies retain persistent update state.

The transferable constraints are stable window-owned context/storage, shared state ownership and
invalidation-owned cached ranges. Zircon should not reproduce Unreal's raw pointer/thread model or
layer-id repair; current Rust ownership and measured command/backend behavior remain authoritative.

## Target architecture

1. A retained per-window paint context owns shared presentation snapshots, damage, theme/metrics,
   command arena and previous generation/range table.
2. `HostRgbaFrame` stores immutable paint snapshots by `Arc`; leaf APIs borrow typed state.
3. Command ranges reset/reuse capacity and patch only dirty scene generations in stable z order.
4. Text/image commands carry interned or shared generation handles, not repeated owned strings/bytes.
5. CPU raster buffers are retained and resize-aware; command-only paths remain pixel-buffer free.
6. One geometry owner defines finite visibility/intersection semantics for recording, replay and raster.

## Instrumentation and acceptance

Matrix: commands `0/1/1k/10k/100k`, text bytes `0/1k/1M`, images `0/1/1k`, damage
`none/outside/one-range/full`, interaction `stable/hover/scroll`, backend `GPU/softbuffer/snapshot`,
window `steady/resize`, plugin surfaces `0/16/128`.

| Evidence | Acceptance |
| --- | --- |
| state owner/deep clone counts and bytes | no frame deep clone; one shared snapshot |
| Vec capacity/growth/reallocation and command bytes | capacity reused; rebuild proportional to dirty ranges |
| text/resource owned/shared bytes | no per-command duplicate identifier allocation |
| RGBA allocate/clear/write bytes | zero command-path pixels; retained region work only |
| replay fallback sorts | zero for accepted stable stream |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add state clone, arena growth, owned/shared bytes, range rebuild/reuse and raster counters; capture. | attributable baseline |
| M1 | Store pane interaction snapshot as `Arc` and transfer the existing owner into the frame. | underlying clones active `1 -> 0`, fallback `2 -> 1` |
| M2 | Add reusable per-window command arena and generation/range table. | no steady Vec growth |
| M3 | Replace command text/resource Strings with canonical shared generation handles. | zero duplicate identifier bytes |
| M4 | Integrate retained scene ranges and hard-cut immediate flat rebuild ownership. | rebuild proportional to dirty ranges |
| M5 | Reuse eligible CPU buffers and converge rectangle geometry semantics. | resize/damage proportional raster work |
| M6 | Run managed command/interaction/WPR/power and RenderDoc/pixel/text parity matrix. | quantified accepted milestone |

## M1 implementation result

`HostRgbaFrame` now stores `Option<Arc<HostPaneInteractionStateData>>`. Its setter takes the existing
owner by value and the read API uses `as_deref()`, so every leaf retains the original borrowed
`Option<&HostPaneInteractionStateData>` contract. Both host-window entries and the componentized
window entry transfer the `Arc` returned by presentation paint generation instead of borrowing it for
another structure clone.

| Static ownership work per frame | Before | After | Change |
| --- | ---: | ---: | ---: |
| underlying interaction clones with active paint overrides | 1 | 0 | eliminated |
| underlying interaction clones without active overrides | 2 | 1 | -50% |
| frame snapshot owner size | full state | one `Arc` | O(1) owner |
| leaf interaction API | borrowed state | borrowed state | unchanged |

The remaining dock/floating `Arc::clone` operations are O(1) paint views and are separately owned by
the scene-context milestone. M1 does not claim that command Vec/String churn is solved.

Post-M1 direct owner scope:

- Rust files: 26/26
- lines: 1,076
- bytes: 35,193
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `b43bb7399a52b6e46dcf1dfcc73047a4978030bf9791edf3d400fe314a111737`
- unchanged direct owner files: 24 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `paint_frame/frame.rs` | 103 | 3,213 | `b63ca401781e3f2854eb5f96b43e1adb3fa20aac4c40c536f702b3da21bf229a` |
| `paint_workbench_renderer/host_window.rs` | 41 | 1,604 | `e3687d1dd9fc06bb542b452db4f7997ca9ef6b319ae3a760270bec80129c5512` |

Supporting componentized entry:
`scene_layers/overlay/componentized.rs`, 407 lines, 13,518 bytes, SHA256
`43f4ae3a827961cf4e3bdf2f7a2a63319da2c904947f733ea2f9a1c109b9106a`.

Focused static contract:
`tools/tests/test_editor_paint_frame_shared_state_performance_contract.py`, 40 lines, 1,729 bytes,
SHA256 `72fae75c063e2a2a14b1db53dee63b56d6e36cffc7cd100713ee1e985aefb780`.

## Validation state

- Full direct owner review: passed, 26/26 Rust files.
- Production recording, GPU, softbuffer, snapshot, replay and text bridges: traced/read.
- Relevant Unreal sources above: read and mapped to stable context/storage/cached-range constraints.
- M1 focused contract: RED 2/2 before implementation, GREEN 2/2 after implementation.
- Current owned editor performance-contract set: GREEN 74/74.
- Broad editor performance-contract set: 101/106 passed; its five failures are the unchanged known
  missing `component_showcase_state.rs`, missing `workbench_projection.rs`, missing `available_slots`,
  preview resize `.roots.clone()` and UI asset root dirty-helper `.roots.clone()` findings.
- `rustfmt --check` for the three changed Rust files and scoped `git diff --check`: passed.
- Paint-frame, componentized and workbench Rust behavior tests remain present, but are not claimed
  passing until managed Cargo is executable.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

These modules remain in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
