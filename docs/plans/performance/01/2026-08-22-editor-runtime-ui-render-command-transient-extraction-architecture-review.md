---
title: Editor runtime UI render command transient extraction performance review
date: 2026-08-22
module: zircon_editor retained-host render_commands and render_command_conversion
priority: MVP-P0 shared editor control paint pipeline
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation root cached element list and typed draw storage
---

# Goal

Make one generation-qualified retained paint/display-list artifact the authority between Runtime UI
and the Editor host. Immediate consumers must not compute cache/debug metadata they discard; cached
consumers must compare against a prior artifact rather than infer reuse from the presence of a newly
computed hash. Text, image and shape payloads must not be repeatedly cloned through command, paint
element, host command and raster stages on every frame.

## Reviewed source

- Rust files: 40/40
- lines: 2,127
- bytes: 72,777
- joined raw source-bytes SHA256:
  `a24147e9aca4d2e4e4e65154cc282093e70b767ac410f851695be76191dad0a7`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| Module/folder | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `paint_template_nodes/render_commands{.rs,/**}` | 19 | 722 | 24,696 |
| `paint_template_nodes/render_command_conversion{.rs,/**}` | 21 | 1,405 | 48,081 |

Supporting production paths traced/read where relevant:

- `zircon_runtime_interface/src/ui/surface/render/{command,list,paint,cache}.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`
- retained-host image raster/recording pipeline and recorded/chrome image command conversion

## Correct foundations to retain

1. Invalid/zero-opacity frames are rejected before host draw dispatch.
2. Ordered z-index input takes an allocation-free draw path; fallback stable sorting is instrumented.
3. Opaque identity-mapped images use row copies, while scaled images use premultiplied bilinear
   sampling.
4. Text run/cluster paint style and decorations remain explicit; uniform shaped clusters are already
   collapsed to one host text command.
5. Shared image bytes use `Arc`; atlas production handles avoid embedding RGBA in each command.

## Structural findings

### P0: an immediate host consumer computes cache/debug metadata and then drops it

For every `UiRenderCommand`, editor conversion calls `to_paint_elements(0)`. That method serializes
the complete command through `serde_json` into an FNV writer to compute `cache_generation`, then
allocates `format!("{:?}", kind)` for every emitted paint element. The host conversion reads neither
field and discards the temporary elements immediately.

An image-bearing command can emit up to four elements, so one command performs one full DTO
serialization plus one to four debug-label allocations before useful conversion begins. M1 will add
an explicit transient extraction method that preserves payload/geometry/effects but leaves
`cache_generation` and `debug_label` empty. Existing cache, parity, visualizer and GPU planner paths
retain the metadata-bearing method.

### P0: each frame materializes consecutive owned DTO layers

The current path is:

`UiRenderCommand -> Vec<UiPaintElement> -> Vec<HostPaintCommand> -> recorded/chrome command or CPU raster`

Paint-element construction clones brushes, resource ids, source/shaped text, runs, decorations and
font keys. Host conversion then clones run/line/cluster strings and clip frames again. The outer host
Vec starts empty without a command-derived capacity. Stable presentation therefore pays O(total
payload bytes) ownership work even when no node is dirty.

M2 must publish a retained prepared paint artifact from the source/view generation and expose borrowed
or shared payloads to both editor/native GPU consumers. A capacity hint alone is not acceptance.

### P0: current cache status is not a prior-generation comparison

`UiRenderCachePlan::from_paint_elements_and_batches` marks a paint entry reused when the current
element has any generation and the external invalidation reason is `Unchanged`. It does not compare
that generation with a previous retained element. Batch reuse similarly checks that current
generations are present. The plan therefore depends entirely on another owner making the invalidation
reason correct; freshly recomputing JSON hashes does not itself prove reuse.

M2 must retain previous `{node/source, geometry, clip, resource, text-shape, style}` generations and
compare the relevant domains before reusing prepared paint/batches. Cache counters must reflect actual
artifact reuse, not only a status label.

### P0: text is prepared, cloned and potentially laid out across several stages

`UiRenderCommand::text_paint` constructs owned shaped text, runs, decorations and resource keys.
Editor conversion selects runs, shaped lines/clusters or fallback and clones text into host commands.
Fallback alignment invokes runtime text measurement; host CPU drawing then invokes its text
layout/draw path again. Text decorations also clone clip frames per command.

M3 must make shaping/layout output a shared immutable artifact keyed by text/font/locale/width/style
generation. Native/GPU/CPU backends consume glyph/runs and decorations from that artifact without
recreating source strings or measuring an already resolved layout.

### P0: semi-transparent image commands copy and rewrite the full source image

For every image command with opacity below one, editor draw clones the full RGBA `Arc` into a Vec and
multiplies every alpha byte before atlas/shared-image drawing. Recorded image commands have no opacity
field, so opacity is baked into resource bytes before recording. This is O(source pixels) allocation
and CPU work per draw, independent of destination visibility; atlas fast-path is disabled for partial
opacity.

M4 must carry opacity/tint as draw-instance state through recorded/chrome/native commands and apply it
at blend time. CPU raster sampling should multiply sampled alpha without copying source pixels. The
resource key continues to identify immutable source bytes, not an opacity variant.

### P1: fallback ordering materializes an index vector and stable-sorts it

Every host list first scans z-index order. An inversion allocates `(index, &command)` for all commands
and stable-sorts O(N log N). This is a valid correctness fallback and already counted, but M5 should
make canonical paint order part of retained artifact generation and reject/instrument unexpected
inversions before the hot draw path.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElements.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElementTypes.cpp`

Slate updates/repaints a widget from explicit update/invalidation flags; paint, layout and render
transform have distinct paths. `FSlateWindowElementList` writes typed elements directly into managed
containers, preserves widget-owned cached element handles and cached render batches, and reuses its
memory manager. Repaint clears/replaces only the affected widget cache. Draw creation does not JSON
serialize the complete widget DTO to decide whether its immediately created element is reusable.

The transferable invariants are explicit dirty domains, previous retained artifact ownership,
typed/direct element storage, reusable allocation arenas and batch retention. Zircon should not copy
Unreal pointer lifetimes or assume Unreal timings; the current-source Zircon capture remains the gate.

## Target architecture

1. Runtime UI09 assigns monotonic source/style/layout/resource generations when those domains change;
   it does not derive hot-path identity by serializing the whole DTO.
2. Runtime `UiSurface` is the prepared render-list authority for shared brush/text/image payloads,
   canonical paint order and per-node command ranges.
3. EditorUI08 retains only generation-qualified bridge/range references and backend products for
   native panes not yet expressed as Runtime UI. It must not retain a parallel command payload cache.
4. Cache plans compare current domain generations with previous retained entries and report actual
   reused/rebuilt bytes, elements and batches.
5. Text shaping/layout and image source bytes remain immutable shared resources; per-draw opacity,
   transform, clip and selection are compact instance state.
6. Transient extraction remains an explicit diagnostic/fallback route, not a second product
   presentation authority.

## Instrumentation and acceptance

Matrix: commands `0/1/1k/10k/100k`, payload `quad/text/image/mixed`, text bytes
`0/16/1k/64k`, runs/clusters `0/1/32/1k`, images `16px/256px/4K`, opacity
`0/0.5/1`, ordered/1% inverted z, stable/1%/100% dirty, one/eight surfaces.

| Evidence | Acceptance |
| --- | --- |
| JSON generation bytes/calls and debug-label allocations | zero for transient extraction |
| paint/host DTO allocations and cloned payload bytes | zero for unchanged retained nodes |
| previous/current domain-generation comparisons | every reuse decision attributable |
| command/element/batch rebuild and reused bytes | proportional to dirty nodes/batches |
| text shape/measure/layout counts | once per changed text-layout generation |
| image source copies/alpha rewrites | zero per draw; opacity is instance state |
| fallback sort count/time | zero in accepted canonical workloads |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before/after |
| RenderDoc draw/batch/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add extraction/hash/debug/cloned-byte/reuse/image/sort counters and capture matrix. | attributable baseline |
| M1 | Add transient element extraction and use it in editor immediate conversion. | no cache hash/debug metadata on host route |
| M2 | Build retained prepared render-list generations with prior-domain comparison. | unchanged nodes allocate/rebuild zero |
| M3 | Share immutable shaped text/layout/decorations across consumers. | once per changed text generation |
| M4 | Carry image opacity as instance state through recording/native/CPU blend. | zero full-image opacity copies |
| M5 | Retain canonical ordered batches and hard-cut duplicate DTO/materialization routes. | no fallback sort in product matrix |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel/text parity matrix. | quantified accepted milestone |

## M1 implementation result

Runtime Interface now exposes explicit transient paint-element extraction with and without layout
metrics. It builds the same geometry, payload and effects as the existing cache-aware method, but
sets both `cache_generation` and `debug_label` to `None`. Editor host runtime-command conversion is
the first and only production consumer changed to this route. Existing cache-aware extraction and the
Runtime GPU planner retain their current metadata behavior.

Per runtime command on the editor immediate host route:

| Static extraction work | Before | After | Change |
| --- | ---: | ---: | ---: |
| complete-command JSON/FNV generation calls | 1 | 0 | -100% |
| bytes serialized only for cache generation | command JSON bytes | 0 | -100% |
| debug-label string allocations | one per paint element | 0 | -100% |
| cache/debug metadata fields | populated | `None` | explicit transient contract |

M1 does not remove the temporary paint-element Vec, payload/text clones, host command Vec, image
opacity copies or fallback sorting. M2-M5 own those boundaries.

Post-M1 direct owner scope:

- Rust files: 40/40
- lines: 2,127
- bytes: 72,787
- joined raw source-bytes SHA256:
  `db2fda9abfd277f432c8f98541c4d938c93652558fcf556fbae2cd9f2a75fe24`
- unchanged direct owner files: 39 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `render_command_conversion/commands/command.rs` | 22 | 815 | `0d26fe605e2329948862fefbd9a9b6211f3cfa080fcf34e931332a251b80aa9e` |

Changed supporting public contract:

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `zircon_runtime_interface/src/ui/surface/render/command.rs` | 594 | 19,142 | `274fd4e1490f781474345be392a259fe3dbb6896e854440d48c4228a02d64494` |

Focused contract:
`tools/tests/test_editor_runtime_render_command_transient_extraction_contract.py`, 34 lines, 1,375
bytes, SHA256
`f5b871f06f65a852a345109cd426e6748a8e9dfc167a913d8de9b5012925605c`.

## Validation state

- Full direct owner review: passed, 40/40 Rust files.
- Runtime Interface command/list/paint/cache, Runtime GPU planner, image raster/recording and Unreal
  sources above: traced and relevant production paths read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change.
- Current owned performance-contract set: GREEN 54/54.
- `rustfmt --check` for the two changed Rust files and scoped `git diff --check`: passed.
- A Rust regression verifies cached extraction keeps metadata while transient extraction omits it; it
  is present but not claimed passing until managed Cargo is executable.
- M0 and M2-M6 remain pending; no elapsed-time, GPU or power claim is made from static loop/allocation
  counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a launchable current-source editor; RenderDoc cannot validate JSON
  hashing, string cloning or CPU image-opacity copies.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
