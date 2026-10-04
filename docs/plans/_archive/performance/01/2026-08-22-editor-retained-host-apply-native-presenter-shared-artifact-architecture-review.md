---
title: Editor retained-host apply and native-presenter shared-artifact performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/{apply_presentation.rs,scoped_presentation.rs,shell_content_presentation.rs,workbench_window_projection.rs}
priority: MVP-P0 editor retained apply, scoped invalidation and native windows
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate persistent windows and invalidation roots
---

# Goal

Make main and native editor windows consume the same immutable presentation artifacts and exact dirty
receipts. A changed generation must project each source segment once, regardless of window count;
scoped work must patch only the addressed rows/dock/pane, and unchanged nodes must retain row identity.

## Reviewed source

- owner files: `apply_presentation.rs`, `scoped_presentation.rs`,
  `shell_content_presentation.rs` and root `workbench_window_projection.rs`
- Rust files: 4/4
- current lines: 2,722
- current bytes: 102,366
- joined current source-bytes SHA256:
  `0d349ef8ef1d92347c741a961ede5ed6cd041d17a560f4c8921fb131d2905209`
- joined pre-M1 source-bytes SHA256:
  `c8d2d8eb3feca28cf15de93d6ebb601d470d7d83955b56065b603804f23e731f`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `apply_presentation.rs` | 773 | 28,847 | `0aac3fb2aebffae6447c2ecf81d5d14d1911d03b507c96d5ceeac9c27138835f` |
| `scoped_presentation.rs` | 535 | 17,774 | `664e163034045dd6f2f1fbc2d835c24f9a36af5002d5983245a1a7457e0f4766` |
| `shell_content_presentation.rs` | 842 | 29,631 | `ca7c18a0bd290ae07e70ad0d80eab67a6db7b48c4886660c78c226b3446aa71a` |
| `workbench_window_projection.rs` | 572 | 26,114 | `1871ae7ff02f1cffdbdf12eb808fe2453e6a7ac2a3ff9d9aad8ebb2a7af2fada` |

All four files were read in full. The workbench projection root declares ten child modules; those
children are separate pending Rust-file review units and are not counted here. Production ownership
was followed through recompute invalidation decisions, native-window presenter synchronization/store,
pane/scene conversion and `ModelRc` persistent row patches. Those related files are not counted as
fully reviewed in this record.

## Existing foundations to retain

Workbench projection can apply a sparse node workset with persistent row patches and exact changed
rows. Scoped UI-asset updates clone only matching floating rows, reuse all other rows and request
bounded damage. Shell-content patches validate structure generation, layout, shell ownership groups,
document identity and hit-model cardinality before committing one dock. Native presenters skip apply
when both source generation and target are unchanged. These are strong retained foundations.

## Structural findings

### P0: every native window repeats a complete presentation projection

The main host owns a persistent `HostChromeProjectionCache`, but native presenters call the wrapper
`apply_presentation`, which creates a fresh cache. On every changed source generation, each native
window rebuilds `ShellPresentation`, pane/floating data, scene data, host-contract scene data,
workbench nodes and welcome data before a later configuration step selects its floating target.

For W native windows, total changed-generation work is approximately W times the full workbench
projection/conversion, even though all windows share the same source generation. Store-level source
generation suppression avoids stable repeats but cannot share work across windows or reuse unchanged
subsegments between changed generations. Native windows need per-window persistent caches at minimum;
the target design is one shared source artifact plus a small target-window overlay.

### P0: full apply owns two complete scene shapes and maps every row

`build_host_scene_data_with_cache` constructs the workbench-window scene DTO, then
`to_host_contract_host_scene_data_with_runtime` constructs a second complete retained-host scene.
`map_model_rc` materializes new tab, frame, menu, node and floating models; pane conversion clones
the intermediate pane before consuming it. Both scene trees coexist during apply. An unchanged
segment can have cached inputs and still pay complete output-tree conversion.

The workbench/host boundary needs shared typed artifacts or a single host scene authority. A generic
whole-model mapper is not an incremental retained pipeline.

### P0: the fallback shell-content path discovers scope after full construction

The committed single-scope path correctly calls `patch_shell_content_presentation_from_state` before
full presentation construction. If shell-content invalidation cannot retain a single scope, the Full
decision can keep `reuse_shell_layout`; `apply_presentation_with_template_v2_data` then constructs a
complete `ShellPresentation` before comparing three pane IDs and attempting one-dock patch. A hit
saves scene/host conversion but not model/shell/pane/floating projection.

This path is not dead: it preserves a narrow patch when coalesced/legacy invalidation loses explicit
scope. M2 must infer a target from committed receipts/model IDs before payload construction, not
delete the path or continue guessing after full work.

### P0: scoped UI-asset lookup repeats floating-window traversals

Before patching one UI asset, the root predicate scans scene and native floating rows, native presenter
discovery scans native rows again, and mutation scans both collections again. Each addressed native
presenter runs a similar search over its own presentation. With W windows and duplicated all-window
collections, scoped work can approach quadratic visits even though only matching rows are cloned.

Publish `instance_id -> {root row, native presenter/window row}` indices with the committed floating
generation. A scoped update should visit and patch exact rows, not prove presence by repeated scans.

### P1: Welcome dispatch clones every projected node to modify at most four

`welcome_nodes_with_native_dispatch` maps the complete pane-node model, clones every wide node and
changes only project-name, location, create and open-existing controls. `ModelRc::with_row_patches`
already preserves base rows and metadata, and is used elsewhere in this owner group. This is a safe
M1: clone/replace only matching rows and retain identity for all others.

### P1: full workbench-node projection repeats expensive per-node resolution

The projection root builds a node index once, but every visible node reparses host properties, builds
TOML/style maps, resolves component roles/tokens/options/menu structures, joins text lists and calls
preview-image resolution. Previous-node reuse is limited mainly to notification rows. Sparse patches
are sound; full rebuild needs exact template/property/resource/text/style generations and reusable
node artifacts. The ten child modules require their own subsequent per-file review before design.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabWell.cpp`

Every Unreal `SWindow` owns a persistent hit-test grid and invalidation root and routes window paint
through `PaintInvalidationRoot` (`SWindow.cpp:2069-2149`). The root retains cached element data,
rebuilds the widget list only on the slow path, updates only the invalid fast list otherwise and does
no widget update when that list is empty (`SlateInvalidationRoot.cpp:356-424`). Docking installs the
existing foreground content owner into the parent stack rather than rebuilding all tab content
(`SDockingTabWell.cpp:842-866`).

The transferable invariant is persistent per-window invalidation state over shared live content.
Independent windows may have target geometry/damage, but they must not independently reconstruct the
same source presentation tree.

## Target architecture

1. Publish one immutable `HostPresentationArtifactSet` per accepted source receipt, split into shell,
   chrome, panes, docks/floating, workbench nodes and resources.
2. Give each main/native window a persistent apply cache holding only target, mount, scale, geometry,
   damage and native surface overlay generations. Shared source segments are never rebuilt per window.
3. Replace duplicate workbench-scene and host-scene value trees with shared typed artifacts or one
   authoritative host form. Final foreign ownership happens once per changed segment.
4. Move shell-content target inference before `ShellPresentation`. Coalesced scopes compare committed
   pane identities/generations and build exactly one candidate dock.
5. Publish floating pane instance and native presenter indices with the floating generation. Scoped
   UI-asset updates use exact rows and presenter IDs in O(matches).
6. Give workbench nodes exact property/style/resource/text/mount receipts. Full projection reuses
   unchanged node artifacts; sparse work remains persistent row patches.
7. Delete fresh-cache full apply wrappers, generic whole-scene mapping and repeated presence-scan
   fallbacks after every caller uses the shared artifact protocol.

Complexity targets:

- changed shared source with W native windows: O(changed source + W overlays), not W*source;
- unchanged window apply: O(1), zero projection/conversion;
- one scoped pane update: O(matches), zero unrelated floating scans/clones;
- one-dock shell content: O(changed dock), no full shell construction;
- full workbench projection: O(changed nodes/properties), stable nodes retained;
- duplicate complete scene bytes: zero.

## M1 result

`welcome_nodes_with_native_dispatch` now inspects borrowed rows, clones only the four recognized
controls, and applies them through `ModelRc::with_row_patches`. Nonmatching rows and model metadata
retain identity. All current role, action, value, text and disabled semantics remain in the matching
branches.

This changes each Welcome conversion from N wide node clones and N-row model reconstruction to N
borrowed visits plus at most four row clones. The source contract moved RED 1/1 to GREEN 1/1 and
forbids `model_rc`/`row_data` reconstruction in this function. It does not solve full scene conversion,
native-window multiplication or scoped floating scans; those remain M2-M5.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| source projection builds by generation across W windows | each changed source segment = 1 |
| per-window overlay/apply builds | changed target only |
| workbench/host duplicate scene bytes | 0 |
| pane/floating/node rows visited and cloned | exact changed rows |
| fresh cache creations | 0 in production presenters |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: native windows 0/1/4/16/1,000; workbench/pane/floating nodes 0/1/1,000/10,000; scoped matches
0/1/W; stable applies 1/1,000; source, target, mount, scale, geometry, resource, style, shell-content,
UI-asset and render-only changes. Capture projection/conversion/cache builds, visits/clones/bytes,
allocations, damage, main-thread CPU, latency, RSS and package energy on one source/executable
fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source multi-window draw/pixel parity; it cannot validate CPU projection multiplication.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add per-window/source artifact, projection/conversion, row visit/clone/byte and cache counters; capture baseline. | source-bound W/row scale evidence |
| M1 | Patch only Welcome native-dispatch rows. | RED-to-GREEN contract, unchanged row identity |
| M2 | Publish shared source artifacts and persistent per-window overlay caches. | source builds independent of W |
| M3 | Converge duplicate scene forms and move shell target inference before construction. | one scene authority; changed dock only |
| M4 | Publish pane/presenter indices and node property/resource receipts. | scoped O(matches); stable nodes retained |
| M5 | Delete fresh-cache/full-map/repeated-scan compatibility paths. | one retained apply protocol |
| M6 | Run scale, WPR/power, interaction and multi-window RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 4/4 Rust files.
- Recompute invalidation, native presenter/store, pane/scene conversion, persistent-row implementation
  and Unreal references: read.
- M1 source implementation: complete. Its static contract moved RED 1/1 to GREEN 1/1.
- Related pane/scene/chrome/shell/apply source contracts: passed, 15/15.
- Changed Rust `rustfmt`, scoped diff check and plan-record audit self-test: passed.
- Workbench projection child modules and M0/M2-M6 dynamic acceptance: pending.
- Managed Cargo remains unavailable because the current validation Session is terminal `archived`.

The module remains in `pending.md` until M0-M6 pass on one source/executable fingerprint.
