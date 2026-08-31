---
title: Editor retained template-node shared conversion artifact performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/{component_contract_metadata.rs,floating_pane_geometry.rs,root_template_overlay.rs,template_layout_context.rs,template_node_conversion.rs}
priority: MVP-P0 editor template nodes, root overlays and host conversion
status: source_reviewed_structural_implementation_pending
reference_engine: Unreal Engine Slate persistent attributes and invalidation roots
---

# Goal

Convert each immutable template-node generation to the retained host contract once. Menu, page, dock,
pane, floating, Welcome and root-overlay consumers must reuse shared converted rows; stable options,
images, component metadata and layout variants must not be reconstructed during every host apply.

## Reviewed source

- owner files: component metadata, floating-pane geometry, root overlay, table layout context and
  generic template-node conversion
- Rust files: 5/5
- current lines: 896
- current bytes: 31,686
- joined current source-bytes SHA256:
  `ff1854348f31af91261befd62bd9a1968511d3bef7dac5112f2e8dda3bc79af0`
- owning commit before review: `4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `component_contract_metadata.rs` | 92 | `7246bc4efd83a6022bb7e45f3c23dfcaaf17dd22cc8f236e46952da96b0a5419` |
| `floating_pane_geometry.rs` | 95 | `0b9505de53972b552cef239482c10b5eaf0ca61408b90dcad40694586920c9d6` |
| `root_template_overlay.rs` | 307 | `0d3eea2dde39e26b6fc71f6298ed54e766d36634ab0ec50db737ade730913fee` |
| `template_layout_context.rs` | 90 | `9b251f5e4b7c6bbb7540663ec2c456d156c541f25a43ff1b160f231ccc645c3e` |
| `template_node_conversion.rs` | 312 | `5f23664cd91937d5eb0ca7910b8f046bd77d869ae248c24ef95cd49ea3bda481` |

All five files were read in full. Production callers were followed through full scene conversion,
floating/Welcome conversion, UI-asset details and root-template apply. `ModelRc` persistent ownership,
the workbench compiled-property review and component registry were inspected as related boundaries.

## Existing foundations to retain

Generic model mapping borrows source rows and has a clone-probe test proving no intermediate source
row clone. `SharedString`, `ModelRc` and preview images are reference-counted. Floating-pane geometry
is fixed O(1) arithmetic with one shared formula and behavior tests. Component descriptors come from
a static retained registry with bounded fallback match tables. Table context only modifies table rows.

## Structural findings

### P0: every full scene conversion remaps every template-node model

`to_host_contract_template_nodes` allocates a new model and constructs one very wide
`TemplatePaneNodeData` per source row. Full apply calls it independently for menu templates/popups,
page chrome, status, every dock rail/header, floating headers, Welcome and pane bodies. A shared source
`ModelRc` generation has no converted-host artifact, so stable node identity is discarded at the ABI
boundary.

The conversion owner needs a cache keyed by exact source model identity plus host-contract ABI,
resource/text and layout-context generations. Consumers must clone one converted `ModelRc` owner.

### P0: the nominal owned converter still clones the complete source row

`to_host_contract_template_node_owned` accepts `ViewTemplateNodeData` by value but immediately borrows
it and calls the cloning converter. UI-asset center/panel/detail rows therefore clone every retained
string, options owner, button style and image handle even when the caller already transferred
ownership. The hard cut needs a true move conversion or, preferably, direct shared converted row.

Rewriting the 200-field mapping without executable Rust behavior tests is not a safe simple edit. It
must be implemented with generated/typed field mapping and parity tests under managed validation.

### P0: options presentation is recomputed per converted row

Every conversion joins the entire options model into a comma-separated `String`, then also clones the
options `ModelRc`. For O options this is O(total option bytes) allocation per conversion, even if the
source options owner is unchanged. Options text is derived presentation data and belongs in the
source/converted node artifact with the options generation.

### P1: root overlay rescans the full template and resolves media on every apply

Root overlay projection scans all N root-template nodes to find a boolean property, clones selected
metadata and calls preview-image resolution for each overlay. The current builtin has zero explicit
overlay rows, but the cost scales with unrelated template nodes and repeats for every full/native
window apply. The template compiler should publish an overlay index and converted media artifacts.

### P1: layout context is applied after wide row construction

Workbench projection builds a complete host row and then appends one width-tier token only for table
nodes. Width-only changes therefore remain coupled to semantic node construction. The layout variant
must be a narrow geometry/layout patch keyed by tier, while semantic/style owners remain shared.

### P2: component metadata and floating geometry are not hotspots

Descriptor lookup uses the retained registry and fallback tokens are static match results. Floating
content size/frame performs bounded scalar arithmetic. Keep these simple; do not add caches without
counter evidence. Their results should still live in the consuming generation artifact to preserve
one authority.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeMetaData.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeDescriptor.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal retains typed attributes on widget metadata and maps changes to descriptor-owned invalidation
reasons instead of reconstructing a generic transfer row (`SlateAttributeMetaData.cpp:36-57`,
`105-196`; `SlateAttributeDescriptor.cpp:11-30`). Its invalidation root retains cached element data,
rebuilds only on the slow path and updates only explicitly invalid fast widgets otherwise
(`SlateInvalidationRoot.cpp:356-424`).

The transferable invariant is a persistent typed node/widget owner carried through invalidation, not
full source-to-host DTO remapping on every apply.

## Target architecture

1. Publish `ConvertedTemplateNodeArtifact { rows, receipt }` from the template/projection owner. The
   receipt includes source model identity, host ABI, resource/text and relevant style generations.
2. Make menu/page/status/dock/pane/floating/Welcome consumers share converted row owners. Delete
   per-consumer whole-model mapping.
3. Store derived options text, structured options and media handles with their exact option/resource
   generations. Stable conversion does zero joins or image resolution.
4. Publish a root-overlay row index and overlay artifacts during template compilation. Full apply does
   not scan unrelated nodes.
5. Split semantic converted rows from layout/mount/tier patches. Width-only changes replace table
   layout fields without reconstructing the row.
6. Replace the false owned converter with true ownership transfer or remove it after all UI-asset
   consumers use shared artifacts.

Complexity targets:

- unchanged model conversion: O(1), zero rows/strings/options/images built;
- changed source rows: O(changed rows and bytes), unrelated rows retain identity;
- root overlay: O(changed overlay rows), no full-template scan on apply;
- width-tier change: O(table rows), semantic/resource rows retained;
- duplicate source/host wide rows: zero outside required final ABI owner.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| conversion calls/rows/bytes by source identity | unchanged = 0 |
| option joins and bytes | unchanged options = 0 |
| preview media resolutions | unchanged resource = 0 |
| root nodes visited for overlay | apply = 0; compiler O(N) once |
| true owned-conversion field clones | 0 or entry removed |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: models 0/1/16; nodes 0/1/1,000/10,000; options 0/1/1,000/10,000; overlays 0/1/64;
stable conversions 1/1,000; source-row, options, resource, text, width tier, mount and render-only
changes. Capture row/byte conversions, joins, image work, allocations, CPU, latency, RSS and package
energy on one source/executable fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source pixel/draw parity; it cannot validate DTO conversion or string-join cost.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add conversion row/byte, options join, media and overlay-visit counters; capture baseline. | source-bound node/options scale evidence |
| M1 | Publish shared converted node artifacts and exact receipts. | stable conversion = 0; shared row identity |
| M2 | Move options/media derivation to their generation owners and publish overlay index. | stable join/media/scan = 0 |
| M3 | Split layout-tier/mount patches from semantic rows. | width-only changes touch table/layout rows only |
| M4 | Remove false owned conversion and per-consumer mapping paths. | one converted-node authority |
| M5 | Run scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 5/5 Rust files.
- Production apply/scene/UI-asset callers, registry/model ownership and Unreal references: read.
- No source edit was made: the safe fix is a cross-consumer ownership cut requiring managed Rust
  behavior validation, not a local loop rewrite.
- M0-M5 implementation and dynamic acceptance: pending.
- Managed Cargo remains unavailable because the current validation Session is terminal `archived`.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
