---
title: Editor visual projection and transition activation performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{button_style.rs,surface_defaults/**,surface_metrics/**,visual_state/**,visual_style/**,transition_metadata/**}
priority: MVP-P0 editor retained UI presentation
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation and active animation timers
---

# Goal

Compile immutable visual descriptors once per template/component generation, keep interaction state as
small retained patches, and schedule only active transitions. Stable nodes must not repeatedly resolve
generic TOML aliases, reconstruct wide button/element styles, or carry timing strings for a transition
that does not exist.

## Reviewed source

- Rust files: 34/34
- lines: 1,816
- bytes: 58,703
- joined raw source-bytes SHA256:
  `a6a393747856f7daf0d8aa3caf56441e693bccd7425152f21f90557b5c1d3733`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `button_style.rs` | 331 | 10,828 | `fdd5f0e94c7b84a403d68a933a6938a47f1a50202d73e6ee4924de1f46937460` |
| `surface_defaults/alert_dialog.rs` | 94 | 2,982 | `9d1b0ae5698584ce18f94f0932a07dab58a7a1c3663d96a543bbdf030c481d68` |
| `surface_defaults/badge.rs` | 124 | 4,022 | `f10e456f536c19b7ae6b038cb50f84d466c21a73cc2c4695cfdd8589f6598bd2` |
| `surface_defaults/chip.rs` | 100 | 3,261 | `1da24422874f6791fc17b86b8d8d5f7442935c3923cf4a0fc589a4d8d24ca4a8` |
| `surface_defaults/component_variant.rs` | 104 | 3,470 | `4ff6927ed4684eb40b8f311f909397da9b93dc55543fd433b70c27973bb5138f` |
| `surface_defaults/mod.rs` | 48 | 1,197 | `7491462a747daba27d958629f5a55a98d4437778f27072d2afbe78e98c55a1c9` |
| `surface_defaults/shared.rs` | 87 | 2,694 | `78aead1b53c34f464531536bb5858c2f7f776348e3f06556af01199103683d00` |
| `surface_defaults/skeleton.rs` | 61 | 2,228 | `fd51ca139f1c2a0e0500aaa917b984cd63bdcad858005f671addd9666449ff30` |
| `surface_defaults/surface.rs` | 58 | 1,809 | `1ea6722a517abcc4ecc83d742e8b5f64d2f92f5a46daa91cbc019d5ee2b9458d` |
| `surface_defaults/text_tone.rs` | 29 | 1,020 | `0dd99771538834aa0b7a3987be8d92a7edd04aee0969bc6f32412bd3d9d2c6ca` |
| `surface_defaults/validation.rs` | 33 | 904 | `275f75b3b5777c6e26fa28f97a5d19c69caafb500b5517d64e5a35859036bbb9` |
| `surface_metrics/border.rs` | 30 | 960 | `51dba6573c256243c08ada50d1c499678562fd3d0c39bc84d047fc664855e09d` |
| `surface_metrics/corner.rs` | 29 | 925 | `16d6ccbe3b0365eeda26b879b3fdeaef3dbd18b733168adb2a698325219b7291` |
| `surface_metrics/elevation.rs` | 47 | 1,252 | `e110595f46c57e8c3f67c31a5bb2389e98193c1e735eef90a5fa1100dcb6aa01` |
| `surface_metrics/mod.rs` | 10 | 274 | `5bcad2b92eca374974c59a048937e42e351b88ce2bab783f217cac6f4374916e` |
| `surface_metrics/values.rs` | 14 | 529 | `07c0ec825b9126625d3e4011d97d2494877f5881bcf3985ad89c9ad2121d0449` |
| `surface_metrics/z_index.rs` | 43 | 1,593 | `9e15716b63cf77558d225e0099adbd09487eb6c3426a0e96cae04b81b2afadce` |
| `transition_metadata/direction.rs` | 22 | 546 | `f5abfb01ff0f54a94eea0b6f66bd5857cb7a22f360680aceaed425d26e13328e` |
| `transition_metadata/kind.rs` | 28 | 814 | `3a445279a75a5a031b012cf967b4c39c7051a89675faf0b80dba59bd00480a51` |
| `transition_metadata/mod.rs` | 38 | 1,287 | `ac0d421604755c1bc7f2dd82acf896ee0ea10f214217f9152e22c799c3394186` |
| `transition_metadata/model.rs` | 9 | 346 | `3d3b5eb356eb3faa9e793653456bb49060fd6c486da0d9cdfad1664671a2492d` |
| `transition_metadata/state.rs` | 79 | 2,126 | `2089b801b104ca957474edeababd5b0eff08c9200871e3f0a3873829d3fbc57d` |
| `transition_metadata/timing.rs` | 55 | 1,841 | `9ccb661fea526e2be36961c59abb499695bb3f429657f5d28596f999fadac5a1` |
| `visual_state/flags.rs` | 42 | 1,453 | `ca3c5d600a174f8ca9bcdf6734635b3566adae33cdbfa2b2a98be27177c7c3a1` |
| `visual_state/icon.rs` | 29 | 936 | `378b9a91d795bf1518a702f542d413a4bc8336c2b9ec61e17a0b1b9650685fcc` |
| `visual_state/mod.rs` | 38 | 1,228 | `006154c4f077babc76f131c781806729e1c398caa0c39b8aba306470f3f48c86` |
| `visual_state/model.rs` | 22 | 933 | `258c191fbfede26b218f39cbd9a88dc2c27615b68e51ebecace38d5792476475` |
| `visual_state/ripple.rs` | 38 | 1,149 | `6023ea235a64602f94fcebdaa7d56e3b56363a92af04b2a5a6982decc39a40a8` |
| `visual_state/state_layer.rs` | 29 | 944 | `98b902ec5169ae32f57c1c61ee3441eb8eff8ffc80d0296e96b501df200d9797` |
| `visual_style/button.rs` | 27 | 901 | `5a322b0e216ad3c624c8ec87cb5d802574772c49c67185c0ea71479d49378acd` |
| `visual_style/component.rs` | 24 | 774 | `98b10efc66f8743076f336c562eda566c91d5470c37eab6dc0c3f29e6d7902ec` |
| `visual_style/mod.rs` | 45 | 1,446 | `14baf2b0d96218b6ae2e6cada3660111562b28be0e39500e87b3fd64ab499652` |
| `visual_style/model.rs` | 18 | 796 | `3df959334a9254d3784bd0b9582b1d5754b4d589ce14fc1801d988139a24f3a0` |
| `visual_style/surface.rs` | 31 | 1,235 | `9afc81ed2cfbef839faaa8ca712d2f263eef398670835fbcf7efe34dcfc86073` |

Supporting paths traced: generic host-node projection/assembly, retained template conversion,
`TemplatePaneNodeData` equality and cache byte accounting, transition opacity painting, runtime
`resolve_button_style_from_values`, and the parallel layouts view-projection pipeline.

## Correct foundations to retain

1. Component roles provide deterministic surface, elevation, corner and transition defaults.
2. Alias projection uses `Cow`: the complete TOML map is cloned only when an alias must actually be
   inserted. It is not an unconditional whole-map clone.
3. Transition opacity is O(1) and is gated by transition kind in the painter.
4. Interaction state and popup/animation apply belong on the UI thread. Worker scheduling is not the
   remedy for repeated retained-state projection.

## Structural findings

### P0: generic nodes repeatedly materialize unrelated visual capabilities

`host_template_node` calls both visual-state and visual-style projection for every node. With no
matching attributes, visual-state alone attempts 29 ordered-map lookups: ten interaction flags, six
state-layer aliases, seven ripple aliases and six icon aliases. Visual style then resolves a wide
`ResolvedButtonStyle` for every node. That type is also the generic element-style carrier used by
alerts, avatars, badges, chips, dividers, paper, skeleton, text fields and timeline primitives, so a
role-only button gate would be incorrect. The architectural defect is the flat DTO/name, not merely
one slow function.

The target is a compiled `VisualDescriptorGeneration`: immutable role/default/element tokens shared
by stable nodes, with capability-specific optional records. Interaction flags, ripple coordinates and
active transition progress become small retained patches keyed by node generation.

### P0: inert nodes perform a complete transition projection

After resolving an empty transition kind, the current path still attempts 16 additional
`BTreeMap::get` calls for active/status/progress/entered/duration/easing/direction. It allocates the
default `"entered"` status only to calculate other fields, then allocates and retains the default
cubic-bezier easing even though the painter ignores all transition timing when kind is empty.

M1 will return the semantic no-transition record immediately after kind resolution: active/entered
true, progress one, duration zero, empty easing/direction. Attributes such as `transition_progress`
without a transition kind are inert by definition and must not dirty/cache a non-animation node.

### P0: metadata is rebuilt instead of owning an active transition session

Transition progress is modeled as strings/numbers inside generic node data and included in full-node
equality. The parallel layouts view-projection pipeline independently resolves the same transition
attributes and defaults. This makes animation advancement look like template reprojection rather than
a bounded active-set update, and duplicates semantic authority.

The target is one optional immutable `TransitionSpec` plus a retained `TransitionSession` containing
clock/direction/progress. Only active sessions wake the UI loop; completion removes them from the
active set. Paint/layout invalidation must be explicit and change-proportional.

### P1: variant assembly performs repeated token scans and transient strings

MUI-compatible badge/chip/alert/skeleton defaults repeatedly allocate default strings, PascalCase
tokens and formatted composite tokens, while `append_variant_token` rescans the growing whitespace
list. This is bounded for present components but should move into the static descriptor compiler,
not be recomputed on stable state ticks. Replacing it with another runtime hash set would add the
wrong hot-path authority.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Animation/CurveSequence.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Animation/CurveSequence.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/SlateAttributeDescriptor.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeDescriptor.cpp`

Slate associates attribute changes with specific invalidation reasons such as paint, layout and child
order. Its `FCurveSequence` registers an active timer only when required, avoids duplicate timer
registration, and returns `Stop` when playback ends; Slate can otherwise sleep to reduce power.

The transferable invariants are compiled attribute/capability metadata, reasoned invalidation and an
active animation set. Zircon should not copy Slate macros, widget ownership or numeric timing budgets,
and this source evidence does not establish a Zircon wall-time target before profiling.

## Target architecture

1. EditorUI06 compiles role/default/variant/element style into immutable descriptors at template or
   stylesheet generation changes; stable ticks reuse shared records.
2. EditorUI01 owns compact interaction receipts and an active transition scheduler on the UI thread.
   Pointer moves may be latest-wins; input edges and transition start/finish remain ordered.
3. Runtime UI09 defines typed paint/layout/accessibility invalidation reasons and one transition spec
   contract shared by layouts and retained host.
4. EditorUI08 consumes descriptor generations and patches retained nodes without rebuilding flat TOML
   projections.
5. After all consumers migrate, delete duplicate layouts/retained transition resolvers and flat
   generic transition/style fields.

## Instrumentation and acceptance

Matrix: nodes `100/1k/10k`, interactive share `0/1/10/100%`, transition share `0/1/10/100%`,
active transitions `0/1/16/256`, stable/1% style changes, display `30/60/120 Hz`.

| Evidence | Acceptance |
| --- | --- |
| TOML lookups and owned style/string bytes | stable generation: zero reprojection; inert transition: kind only |
| descriptor builds and interaction patches | one build per generation; work proportional to changed capabilities |
| active timer/session count and wakeups | zero animation wakeups when active set is empty |
| node equality/cache dirty reasons | transition progress dirties only required paint/layout consumers |
| CPU/allocation/RSS/input latency/context switches/power | same current-source executable before/after |
| RenderDoc | pixel/draw parity for opacity/slide/collapse only; not CPU style acceptance |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add descriptor/lookup/allocation/dirty/active-session counters and capture the matrix. | attributable baseline |
| M1 | Gate inert transition metadata immediately after empty kind resolution. | focused RED-to-GREEN contract and behavior parity |
| M2 | Compile and share typed static visual descriptor generations. | stable style projection/build = 0 |
| M3 | Introduce retained interaction patches and one active transition-session scheduler. | zero idle animation wakeups |
| M4 | Unify layouts/retained invalidation authority and hard-cut flat duplicate fields/resolvers. | one semantic owner |
| M5 | Run managed scale/input/WPR/power and RenderDoc parity matrix. | quantified acceptance |

## M1 implementation result

`projected_transition_metadata` now returns an explicit no-transition record immediately after an
empty kind is resolved. Fade, Grow, Slide, Zoom and Collapse retain the complete existing
state/timing/direction path. Orphan transition state/timing attributes on an ordinary component are
treated as inert and no longer change its retained node equality/cache payload.

Per ordinary no-transition node:

| Transition projection work | Before | After | Change |
| --- | ---: | ---: | ---: |
| BTreeMap transition-field lookups | 18 | 2 | -88.9% |
| transient/retained default timing strings | 2 | 0 | -100% |
| state/progress/entered/duration/direction projection calls | 6 | 0 | -100% |

The two remaining lookups resolve `transition_kind`/`transition`; role-based transition components
then continue through the original path. M1 does not eliminate the generic visual-state/style call,
the parallel layouts resolver or whole-node transition fields; M2-M4 own those boundaries.

Post-M1 owner scope:

- Rust files: 34/34
- lines: 1,835
- bytes: 59,174
- joined raw source-bytes SHA256:
  `2ff965c628b41b78818063540c049265cd02a10a7f585df6f4f3ae4da9bdbe84`
- unchanged owner files: 32 retain the pre-M1 fingerprints above

| Changed file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `transition_metadata/mod.rs` | 42 | 1,389 | `6fb81b6f1b5640184e7ca6e4a5c8d11c5ab85bccf804659c42dcf6935730532c` |
| `transition_metadata/model.rs` | 24 | 715 | `18db537152038053dd5e0515e3b9d028423904af8b87b6f810e1bd017dae066e` |

Related Rust regression: `pane_component_projection/tests/mui_variants.rs`, 285 lines, 11,251
bytes, SHA256 `cb5b82b76d4cb9641690cd6ed007b55ab710e46999c965a4246f503c6760ca40`.

Focused contract: `tools/tests/test_editor_inert_transition_projection_contract.py`, 44 lines,
1,807 bytes, SHA256
`1c97a0990f7a01126c7d6f3750554aa6618faad2cfaf2288705527485970e319`.

## Validation state

- Full owner source review: passed, 34/34 Rust files.
- Host/cache/painter/runtime-style/parallel-layout consumers and Unreal sources above: read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change.
- Current owned performance-contract set: GREEN 47/47.
- `rustfmt --check` for changed Rust files and scoped `git diff --check`: passed.
- A Rust regression verifies orphan transition fields remain inert on a non-transition component; it
  is present but not claimed passing until managed Cargo is executable.
- M0 and M2-M5 remain pending; no dynamic performance claim is made from static lookup counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
