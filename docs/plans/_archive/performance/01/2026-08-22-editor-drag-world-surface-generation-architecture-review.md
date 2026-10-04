---
title: Editor drag-session and world-surface generation performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{drag_overlay,world_space}/**
priority: MVP-P0 editor direct manipulation and viewport UI
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine stateful DragDropOperation and gated WidgetComponent redraw
---

# Goal

Split drag-session static payload from high-frequency cursor/drop-target updates, and publish enabled
world-space UI surfaces as one typed immutable generation. Ordinary UI nodes must not parse world
transform/render fields, pointer moves must not rebuild static drag strings, and viewport submission
must not rediscover world surfaces by repeatedly scanning the complete host scene.

## Reviewed source

- Rust files: 13/13
- lines: 402
- bytes: 12,895
- joined raw source-bytes SHA256:
  `f5f25880055b06e8c8cf735c7b41239220152b3e9471172db23050b6c7136fb5`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `drag_overlay/attributes.rs` | 34 | 1,009 | `b4614aa9af02e7f5d67d19261311e70baf11d541e5ddd2b9872fc22c9f895461` |
| `drag_overlay/cursor.rs` | 30 | 1,039 | `d15d8e7af7d18a28c1e657963aad7e6a2f7b945c893a33b975a2775ac051d2b2` |
| `drag_overlay/drop_target.rs` | 30 | 989 | `1d0a244323372e36809acb15758e093ef956e59308ddda9de7da923b01a5c554` |
| `drag_overlay/indicator.rs` | 20 | 551 | `be9b7594bc16f21786999a20b7192c701148542ea518f88eb37330fec846965d` |
| `drag_overlay/mod.rs` | 59 | 2,056 | `e6c5bed24664b0bbc59a352aefca2adfd406aee6cc3e7afd857165a24fd65794` |
| `drag_overlay/model.rs` | 25 | 752 | `bc34426df8cadcf88ddf5d369f2f793b7ebf859671ecbb5fcd30ad633c2122c6` |
| `drag_overlay/payload.rs` | 23 | 776 | `5f447099456dc0a3d6a16f74ad8585d0834619680343a9be63dda2ce4c614c07` |
| `world_space/activation.rs` | 13 | 362 | `4527f7fe20eef768f9bbe290f612574881c13c9031303964711655b520774c78` |
| `world_space/mod.rs` | 38 | 1,259 | `4adee972ea2118f5757e1f3c080d189dd6c90fe778c8268a5b42d9e86eb9066a` |
| `world_space/model.rs` | 19 | 761 | `6f06e3529acd33f97a07a3a27d732c4b686a5b57ce2d5f314d02d2afac08d3d2` |
| `world_space/rendering.rs` | 34 | 1,036 | `beeb14ba42197c82c35f4eaedfd7c64d61facc261f4e92811438850cfda6780a` |
| `world_space/surface.rs` | 28 | 822 | `4d33e0dcbe5ce10c80950c3956f9874ff28018629a8ed23beab245ba5b5e5354` |
| `world_space/transform.rs` | 49 | 1,483 | `3357ce417ba5a4e5d4f5e53684257b58c702e85a1be75d633cba2cb944273b22` |

Supporting paths traced: generic `host_template_node`, spatial/interaction host DTO application,
world-space submission builder/filter, viewport submission/pointer capture and component tests.

## Correct foundations to retain

1. Drag cursor and drop-target values are fixed-size numeric records; their projection is O(1).
2. Non-drag nodes return before parsing the complete drag payload/cursor/target/indicator. The shared
   `drop_source_summary` exception is deliberate current cross-component data.
3. World transform parsing caps each vector at three components and preserves default scale one.
4. Downstream submission filters `world_space_enabled` before reading transform/size/render fields.
   Disabled metadata is not a render or pointer-hit candidate.

## Structural findings

### P0: all UI nodes parse world-only fields before activation

`projected_world_space` parses transform, surface and rendering before evaluating activation. An
ordinary disabled node therefore performs ten BTreeMap field lookups: three transform arrays, world
size, pixels-per-meter, billboard, depth-test, render-order, camera-target and finally enabled. If
authored values exist it may also allocate a float Vec and camera `String` despite never being
submitted.

Because the downstream builder first filters `world_space_enabled`, activation can safely precede
all other reads. M1 reduces an ordinary node from ten world-field lookups to one, a structural 90%
lookup reduction for this category. It does not remove generic wide DTO construction.

### P0: pointer updates rebuild static drag payload presentation

For a drag-overlay node, every projection rereads payload kind/label/reference, clones label and
reference again into generic text/value text, rebuilds indicator strings, and reprojects static
preview dimensions along with dynamic cursor/target coordinates. At 125/500/1000 Hz pointer input,
this architecture couples static string ownership to high-frequency motion.

The target is one `DragSessionGeneration` with shared payload/preview/accepted-kinds plus a compact
latest-wins pointer/target patch. Down/up/cancel/capture edges remain ordered and cannot be coalesced;
only move position/delta can be coalesced before the UI-thread apply.

### P0: world-surface candidates are rediscovered downstream

The host scene builder filters every pane/window node for `world_space_enabled`, then constructs owned
submissions including a camera-target string. Viewport state stores and can clone the submission Vec.
This review does not change that owner, but the correct architecture is a world-surface generation
indexed at component projection/change time and shared through viewport/render extract.

### P1: generic node projection mixes unrelated capability categories

Drag and world projections are called unconditionally from the generic host-node converter. Early
role/activation gates are safe M1 reductions, but M2 must replace the flat all-feature DTO path with
typed capability generations; continuing to stack early returns will not make refresh ownership
clear or change-proportional.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/DragAndDrop.h`
- `dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Components/WidgetComponent.h`
- `dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/WidgetComponent.cpp`

Unreal models drag/drop as a stateful `FDragDropOperation` that owns its decorator/cursor state and
receives `OnDragged` updates, rather than reconstructing the operation payload on every pointer
sample. `UWidgetComponent` distinguishes world from screen space, gates drawing through
`ShouldDrawWidget`, supports explicit redraw requests/manual redraw, and can stop ticking when no
redraw is requested.

The transferable invariants are an explicit drag-session lifetime, compact motion updates, world
capability gating and dirty/redraw receipts. Zircon should not copy Unreal's decorator-window or UMG
component implementation, and no Unreal numeric timing budget is inferred.

## Target architecture

1. EditorUI01 owns `DragSessionGeneration`: immutable shared payload/preview/drop policy and ordered
   begin/drop/end/cancel receipts, plus a latest-wins UI-thread pointer/target patch.
2. Component generation publishes typed `WorldSpaceSurfaceGeneration` only for enabled components,
   keyed by transform/content/render/camera revisions and stable control/surface identity.
3. EditorUI08 and Runtime UI09 share that generation through host scene, viewport hit/capture and
   render extract. Stable generations do not rescan host nodes or clone submission strings.
4. Render09/14 consume one camera/order/depth/billboard record; CPU UI projection does not create a
   second rendering authority.
5. Generic flat drag/world fields are deleted after paint, hit, input, accessibility and diagnostics
   migrate to typed generations.

## Instrumentation and acceptance

Matrix: UI nodes `100/1k/10k`, world surfaces `0/1/16/256`, drag payload `64 B/2 KiB/256 KiB`,
pointer `125/500/1000 Hz`, display `30/60/120 Hz`, stable and 1% transform/content/target changes.

| Evidence | Acceptance |
| --- | --- |
| world capability/field lookups and parse allocations | disabled ordinary node: one activation read, zero world value materialization |
| host-node visits and submissions cloned | stable world generation: zero rebuild/copy |
| drag static strings/bytes and pointer patches | static payload built once/session; move applies at most once/display frame |
| ordered drag edges and pointer capture | no lost/duplicated begin/down/up/drop/end/cancel; capture parity |
| CPU/allocation/RSS/input latency/context switches/power | same current-source executable before/after |
| RenderDoc | world UI draw/order/depth/billboard and drag overlay pixel/draw parity only |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add world lookup/build/copy and drag static/dynamic/edge counters; capture matrix. | attributable baseline |
| M1 | Gate world transform/surface/render parsing on activation. | focused RED-to-GREEN contract and behavior parity |
| M2 | Publish typed drag-session and world-surface generations. | generic static drag/world DTO rebuild removed |
| M3 | Coalesce move patches and share indexed world submissions through viewport/render. | stable rebuild/copy = 0 |
| M4 | Hard-cut flat legacy fields and unify paint/hit/input/accessibility consumers. | one authority per generation |
| M5 | Run managed scale/input, WPR/power and RenderDoc parity matrix. | quantified acceptance |

## M1 implementation result

World activation is now resolved before transform, surface or rendering projection. A disabled node
returns an explicit default record whose scale remains `(1,1,1)`, matching the prior no-attribute
host DTO baseline. Enabled `WorldSpaceSurface` and explicit `world_space_enabled=true` nodes retain
the original field parsing path.

Per ordinary disabled node:

| World projection work | Before | After | Change |
| --- | ---: | ---: | ---: |
| BTreeMap world-field lookups | 10 | 1 | -90% |
| transform array parses | up to 3 | 0 | -100% |
| world-size Vec materialization | up to 1 | 0 | -100% |
| camera-target String ownership | up to 1 | 0 | -100% |

The conditional rows depend on mistakenly/authored world values being present; missing ordinary
attributes performed lookups but no Vec/String allocation before M1. M1 does not remove the generic
host call or downstream full-scene world-surface discovery; M2-M3 own those boundaries.

Post-M1 scope:

- Rust files: 13/13
- lines: 431
- bytes: 13,581
- joined raw source-bytes SHA256:
  `b33eacd6c1261be7a065b3220128863e8de8bd1b64164b591a5d93cceac73689`
- unchanged owner files: 11 retain the pre-M1 fingerprints above

| Changed file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `world_space/mod.rs` | 43 | 1,349 | `4449e61c1ab4f004094d55bb16712e053edf779a8955572a6dc8ee3d814765c2` |
| `world_space/model.rs` | 43 | 1,357 | `6e7f357267ad258df3c7bb8c83114ae97212448cef2f1eb79ca3b0475128ae07` |

Related Rust regression: `pane_component_projection/tests/world_space.rs`, 58 lines, 2,450 bytes,
SHA256 `87a87b7f3a0eabb27ddf57d5695355ab6b3a5ed34187208eaa8d9ed2d1b8baae`.

Focused contract: `tools/tests/test_editor_world_space_activation_gate_contract.py`, 40 lines,
1,545 bytes, SHA256
`c26a8dc11bd6c92ef8c98de00fb09942eb10a80d169e017da99fa8e35ee4310f`.

## Validation state

- Full owner source review: passed, 13/13 Rust files.
- Host/submission/viewport consumers and Unreal drag/world-widget sources: read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change and after `rustfmt`.
- Current owned performance-contract set: GREEN 42/42.
- `rustfmt --check` for 13/13 owner files and scoped `git diff --check`: passed.
- A Rust regression verifies that a disabled ordinary component ignores authored world-only fields;
  it is present but not claimed passing until managed Cargo is executable.
- Managed Rust behavior tests and M0 plus M2-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
