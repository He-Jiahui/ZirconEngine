---
title: Editor pane component typed-generation performance review
date: 2026-08-22
module: zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs and pane_component_projection/{mod.rs,host_template_node.rs,template_node_data/**}
priority: MVP-P0 shared editor pane component projection
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate typed attributes and invalidation root
---

# Goal

Compile pane component semantics once into typed retained node generations. Stable component,
attribute, binding and resource generations must reuse semantic rows; layout, interaction and visual
changes must patch only their own field groups. Final host conversion must move owned identity fields
and must not clone every node merely to recognize a small fallback set.

## Reviewed source

- owner files: generic template runtime projection, pane component entry, host node projection and
  all template-node data assembly modules
- Rust files: 15/15
- current lines: 659
- current bytes: 26,805
- joined current source-bytes SHA256:
  `483a9831ab57d26724ece0f2354eadfc6a95935c6b192d8cc1571cc62004c700`
- pre-M1 lines: 660
- pre-M1 bytes: 26,922
- joined pre-M1 source-bytes SHA256:
  `5f31b187cd23b0983116f6f090cdd2075d30c5967e2ac257a27d7956486a06f8`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `template_runtime_projection.rs` | 149 | 5,127 | `5bdf8872611b5ffc0aed08916127c3a89c8b5a202312a0f122afce246dc786d8` |
| `pane_component_projection/mod.rs` | 53 | 1,621 | `115878e67f589f3da37a3871431d0f6e907c6c8cf8d657ce23e9eb31a232e7d5` |
| `pane_component_projection/host_template_node.rs` | 99 | 3,901 | `293edff4c0f1b8e04d21a601979cd3ecbfd13d4e769f05149bc6ccb091b5456f` |
| `template_node_data/mod.rs` | 14 | 288 | `c6cd73378ea59969a77f39befb4108ae7b171c56910cabaf54505ba28793677f` |
| `template_node_data/assembly.rs` | 49 | 1,819 | `d7b06039f63fdbb997f14967fa7fb21c4f822e75f03fc39266b331c84ff88ace` |
| `template_node_data/content.rs` | 47 | 2,458 | `d9556ea5e245effe9c25d57d377bfb14d68396a9fca0999eec12f45abd41c900` |
| `template_node_data/identity.rs` | 14 | 389 | `6bb0d9d13f6edeb654d80bb3e20aabd3635dc006279e5da80de6940d103b143a` |
| `template_node_data/interaction.rs` | 50 | 2,639 | `e2b536e26cc649205cf5889c5dee22d78b681067cd254d8c56cff03b19f57b6a` |
| `template_node_data/options_collection.rs` | 35 | 1,915 | `e4afd0a1174f0eacc6ee7e639006775614ccf7b2d8399016904068ee14cd2368` |
| `template_node_data/parts.rs` | 35 | 1,890 | `0c409c4024f9e834d05bcca6441f6325c0bab797ad71bad663e818aa576a1172` |
| `template_node_data/sample_grid.rs` | 10 | 279 | `536c168f37d1205d57db15c1ccf4a12e338eb82c0d3073b57930ffeb5bd0da2a` |
| `template_node_data/spatial.rs` | 31 | 1,289 | `0e515609c9b2b206d11858b1eea73b66c38fbdd2d56dc92bd2dccd80d98bbf02` |
| `template_node_data/timeline_strip.rs` | 10 | 300 | `ccc3c95739a7a733f3980ca273385bb4d111164a787328dcfc006d2e978dddb6` |
| `template_node_data/visual.rs` | 53 | 2,590 | `8705ed5b4ffae493f896945738205d650b81b732ac45127ff9d3a3331e64a85e` |
| `template_node_data/weight_heatmap.rs` | 10 | 300 | `bbddd417ec43f2a8d330637c3eeff9899e5f9c22e6ae3a1f6595857e908cfbef` |

All fifteen files were read in full. Production ownership was followed through the complete generic
pane runtime/surface/layout/host-model path and component-showcase, console and inspector consumers.
The complete component projection directory was statically inventoried for attribute access and
parsing shape; those related modules are not counted in the 15/15 owner total.

## Existing foundations to retain

The node projection receives `RetainedUiHostNodeProjection` by value, and final assembly already
moves most projected strings, lists and typed payloads into `TemplatePaneNodeData`. Field assignment
is separated into identity, content, options/collection, interaction, spatial and visual groups.
Component descriptors are provided by a retained registry, and the builtin template runtime is
initialized once. These boundaries are suitable for typed generations and category-level patches.

## Structural findings

### P0: stable template panes execute the complete runtime-to-wide-node pipeline

Generic projection builds a shared surface, reapplies component patches, computes full layout, builds
a host model, binds actions and converts every host node. Console, inspector and component showcase
repeat equivalent runtime/surface/host conversion paths. The final node projector does not receive a
source/template/layout receipt, so stable component semantics and unrelated layout changes cannot
reuse converted rows.

Publish a retained component-node generation from the template runtime. Pane consumers must share
that generation and apply narrow layout/interaction patches. The shared converted-template artifact
planned by the outer host-conversion review must consume this generation rather than becoming a
second independent cache.

### P0: one untyped attribute map is interpreted by every component projector

`host_template_node` unconditionally invokes drag, value/media, validation, selection, sample-grid,
timeline, heatmap, collection, world-space, popup/action, text, visual-style, visual-state and clip
projectors. Individual projectors often return early by role, so exact executed work varies by node,
but the production component projection tree contains 124 `attributes.get(...)` lookup sites plus
string normalization/split/parse paths over the same `BTreeMap<String, toml::Value>` authority.
Stable templates repeat this interpretation on every accepted projection.

Compile attributes and bindings once into a typed `ComponentSemanticPayload` selected by the
component descriptor. Common content/interaction/visual state should be typed field groups; optional
component families should be an enum or sparse payload, not every node probing every feature family.

### P0: every component becomes one very wide all-feature DTO

Assembly creates `TemplatePaneNodeData::default()` and assigns fields for text, media, selection,
collection, sample grid, timeline, heatmap, world space, popup/drag and visual state regardless of
the component family. A simple label therefore crosses the same flat ABI shape as a table, popup,
world-space surface or timeline strip. This increases construction, clone and invalidation surface
and prevents semantic identity from surviving a narrow state change.

Split stable semantic identity from optional component payloads and from mutable layout/interaction/
visual patches. The external UI ABI may remain flat temporarily, but it must be materialized once at
the boundary from a shared typed generation and removed from internal steady-state ownership.

### P1: owned node identity is cloned before final assembly

`host_template_node` receives the host node by value but clones `component` and drops the original.
The generic content fallback separately clones every optional control ID before conversion so it can
match twelve special layout anchors; most nodes never match. Both are pure ownership mistakes and
are safe M1 candidates: move `component`, and match the already converted shared control ID.

### P2: grouped assignment modules are not themselves a hotspot

The `template_node_data` assignment functions perform direct moves and scalar stores. Keep their
category boundaries for the later patch protocol. Do not combine them into one large assignment
function; the bottleneck is repeated semantic derivation and full materialization, not function-call
count inside final assembly.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeMetaData.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Types/SlateAttributeDescriptor.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`

Unreal registers typed widget attributes with descriptor-owned invalidation reasons and updates the
registered attribute value instead of reparsing a generic property map on presentation
(`SlateAttributeMetaData.cpp:36-57`, `105-196`; `SlateAttributeDescriptor.cpp:11-30`). Its
invalidation root retains cached element data and runs a fast update only for explicitly invalidated
widgets, reserving full rebuild for the slow path (`SlateInvalidationRoot.cpp:356-424`).

The transferable invariant is typed attributes plus category-specific invalidation on persistent
widgets. It is not a monolithic flat DTO or a larger all-purpose projector.

## Target architecture

1. Compile template attributes/bindings into `ComponentSemanticPayload` and an exact component
   descriptor receipt. Invalid values fail or diagnose once per source generation.
2. Publish a stable `PaneComponentGeneration` keyed by template document, data, component registry,
   resource/text and ABI receipts. Stable generations preserve node identity.
3. Separate semantic content from layout, interaction, visual and resource patches. Each attribute
   descriptor maps to the smallest invalidation category.
4. Use typed optional payloads for collection, popup, drag, timeline, heatmap, world-space and other
   specialized families. Unrelated component families perform no attribute probes or allocations.
5. Make generic pane, console, inspector and showcase paths consume the same projection owner. Delete
   their parallel runtime/surface/host conversion paths after parity tests.
6. Materialize the flat host ABI once, or remove it when every consumer accepts typed generations.

Complexity targets:

- stable component projection: O(1), zero attribute lookups/parses and zero row reconstruction;
- one content/style/interaction attribute change: O(1) lookup plus addressed field-group patch;
- layout-only change: O(changed layout nodes), semantic payload retained;
- specialized family work on unrelated components: zero;
- duplicate runtime/host/flat component row authorities: zero.

## Instrumentation and acceptance

| Evidence | Target |
| --- | --- |
| surface/layout/host-model/component conversion calls | stable generation = 0 |
| attribute lookup/parse/normalization by component family | source generation only |
| semantic/layout/interaction/visual rows built or patched | changed category only |
| flat DTO fields/bytes materialized | one boundary owner; stable = 0 |
| component/control identity string clones | M1 = 0 in reviewed path |
| main-thread CPU and input-to-paint latency | report median/p95/max |

Matrix: nodes 0/1/1,000/10,000; attributes 0/1/16/128; bindings 0/1/16; component families label,
button, input, collection, popup, drag, timeline, heatmap and world-space; stable projections 1/1,000;
content/style/interaction/layout/resource/registry changes. Capture lookups, parses, row/field bytes,
allocations, category patches, CPU, latency, RSS and package energy on one source/executable
fingerprint.

Use managed Windows validation and WPR/ETW with artifacts only on D/E/F. RenderDoc is reserved for
current-source pixel/draw parity; it cannot validate attribute derivation or retained ownership.

## M1 result

`host_template_node` now moves the component string from its owned
`RetainedUiHostNodeProjection`; for N converted host nodes this removes N component `String` clones.
The generic content fallback now matches the already converted shared control ID, removing up to N
additional control-ID `String` clones while preserving the same eleven anchor IDs and zero-size frame
condition. Paths that call `host_template_node` directly receive the component result; generic pane
projection receives both results.

M1 does not change projector count, the untyped attribute map, wide node assembly or full surface/
layout/host-model rebuilding. Those remain M2-M4.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add projection, attribute-lookup/parse, row/field and category-patch counters; capture baseline. | component/attribute scale evidence |
| M1 | Move owned component/control identity without temporary cloning. | focused RED-to-GREEN contract and behavior parity |
| M2 | Compile typed semantic payloads and descriptor invalidation categories. | stable lookup/parse = 0 |
| M3 | Publish shared pane component generations and narrow patches. | stable rows = 0; changed categories only |
| M4 | Converge generic/console/inspector/showcase paths and remove parallel wide owners. | one projection authority |
| M5 | Run managed scale, WPR/power, interaction and RenderDoc parity matrix. | quantified before/after and product parity |

## Validation state

- Full owner source review: passed, 15/15 Rust files.
- Generic runtime/surface/layout/host path, console/inspector/showcase consumers, component attribute
  inventory and Unreal references: read.
- M1 source implementation: complete. Its focused ownership contract moved RED 2/2 to GREEN 2/2.
- Combined owned performance contracts: passed, 28/28. Related fixture/row-patch contracts: passed,
  5/5. Changed Rust `rustfmt` and scoped diff check: passed.
- Managed Rust behavior tests and M0-M5 remain pending.
- Managed Cargo is unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived`; the focused command
  was rejected before Cargo launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable fingerprint.
