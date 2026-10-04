---
title: Editor selection option source generation performance review
date: 2026-08-22
module: zircon_editor retained-host selection_options and pane_option_projection
priority: MVP-P0 editor dropdowns menus segmented controls and option popups
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate typed list item source and visible row reuse
---

# Goal

Publish one immutable parsed option source per component generation and apply selection, focus, hover,
press, loading and query as compact retained state. Nodes without options must do no option-state
projection, and large option sets must not be reparsed or fully materialized for every interaction.

## Reviewed source

- Rust files: 8/8
- lines: 376
- bytes: 12,692
- joined raw source-bytes SHA256:
  `409aad3db17d17790dd34e7d6c5c1de5bc208c68196bc947ad6e6b263b9487de`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `selection_options/mod.rs` | 57 | 2,047 | `71a2c5158439851c1821aaaa46dc73c1bd168212078a5a06116d605271ff387d` |
| `selection_options/model.rs` | 12 | 515 | `2c31aa8d7d4964744c5dd98252a6d2d4639ac74e3649e315d5bf9a97225c04cb` |
| `selection_options/options.rs` | 9 | 298 | `71c4360aefc291baac04256c612e473e988682b3ff1b48ccd255bc01943ba301` |
| `selection_options/search.rs` | 10 | 295 | `04af4940a86d0a9a5dd276efcc6a0c28e8e9008dd43c2f1a5eeae2523fe52142` |
| `selection_options/selection.rs` | 23 | 731 | `6a781c26ec9abf8db47f07d352a15390a9a1e9f1b01f06464032c16226c38805` |
| `selection_options/structured.rs` | 11 | 380 | `77517e825b65058e85c61494c1f8c5f71fdf0390e86d317ff3bcd8495c71ae1a` |
| `selection_options/tree.rs` | 24 | 666 | `3d9999f5b4d72b8a350d233a80b1963cee0ee902298ec21cec81650d870ece16` |
| `pane_option_projection.rs` | 230 | 7,760 | `390e505ee8c280268a6c0602b986bedafa02a4031fadef4f2ed7fa923a7acfa8` |

Supporting paths traced: command-palette/notification combined option projection, generic host-node
assembly, option keyboard/popup/hit/paint consumers, workbench notification cache, collection source
window review and component/showcase tests.

## Correct foundations to retain

1. Command palette and notification center use one combined `(options, structured_options)` parse,
   avoiding duplicate specialized parsing per node.
2. Query matching is ASCII case-insensitive without allocating a lowercase copy per option row.
3. Structured option rows expose stable id/label and explicit selected/disabled/special/focused/
   hovered/pressed/loading flags shared by paint, hit and keyboard consumers.
4. `options_text` is still a production fallback for labels and several material primitives; it is
   not dead data and cannot be removed independently.

## Structural findings

### P0: nodes without options still project option state

The generic path builds an empty options Vec and still calls `structured_options_for_node`. Before its
empty row iterator is reached, that function attempts twelve map reads: three selected-value aliases,
six option-state sets, focused index, hovered option id and query. It constructs seven empty set
values and then returns an empty Vec.

M1 will return before this work when the source options slice is empty. Selection/search/tree fields
remain independent and retain their current behavior.

### P0: one option source is materialized into several owned representations

Generic `options` clones every TOML string. Each row is then parsed into temporary owned raw/id/label/
flag strings before a second id/label host row is created. The source Vec is converted again to shared
strings and `options.join(", ")` creates a third user-facing representation. This is O(N) owned work
per projection and scales with total options even when only a popup window is visible.

### P0: interaction changes rebuild static option parsing

Query, selected, disabled, focus, hover, press and loading state are all read together with static
id/label/flag parsing. A pointer hover or query edit rebuilds all sets and all structured rows. Large
menus/selectors therefore couple high-frequency dynamic state to static source ownership.

The target is an immutable `OptionSourceGeneration` with parsed shared ids/labels/static flags plus a
compact `OptionStatePatch` keyed by generation. Query results and popup rows are visible-window
bounded; hover/focus/press patch addressed indices without reconstructing the source.

### P1: selected values recursively convert generic TOML

`selected_option_ids_from_value` converts through `UiValue::from_toml`; nested arrays/maps can clone
their complete content even though selection needs scalar ids. M2 should compile schema-typed option
identity and reject/diagnose incompatible values before the interaction path, rather than adding
another runtime string-normalization cache.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/STableViewBase.cpp`

Slate `SListView` observes a typed item source. During a generation pass, its widget generator reuses
an existing row when the item is already visible, creates a row only when needed and releases unseen
widgets at pass completion. Keyboard navigation consumes item identity/source state instead of
reparsing display strings.

The transferable invariants are one typed source, visible row generation/reuse and identity-based
interaction state. Zircon should not copy Slate templates/pointers or infer an option row size/timing
budget without current-source measurement.

## Target architecture

1. EditorUI06 compiles raw option declarations once into shared `OptionSourceGeneration` records with
   stable identity, label, accessibility text and static flags.
2. EditorUI01 owns ordered selection/focus/press changes and latest-wins hover/query patches on the UI
   thread; no worker may reorder interaction edges.
3. EditorUI08 retains the source generation and materializes only visible popup/menu/select rows with
   bounded overscan and row reuse.
4. Runtime UI09 shares the same identity/index between paint, hit, keyboard, accessibility and event
   payloads; no consumer reparses option strings.
5. Hard-cut raw/structured/options-text duplication after all fallback painters consume typed source
   summaries directly.

## Instrumentation and acceptance

Matrix: nodes `100/1k/10k`, options `0/1/16/256/10k`, visible rows `4/16/64`, selected `0/1/50%`,
query `0/4/32 chars`, hover `125/500/1000 Hz`, stable/1% source changes.

| Evidence | Acceptance |
| --- | --- |
| option field reads and temporary/retained bytes | no-options node: zero structured reads/ownership |
| static option parses and source generations | once per declaration generation |
| visible rows/materialized strings | O(V), independent of total N |
| hover/query/selection patches | addressed/latest-wins state only; no source rebuild |
| paint/hit/keyboard/accessibility identity | one shared index/id authority |
| CPU/allocation/RSS/input latency/context switches/power | same current-source executable before/after |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add option read/parse/owned-byte/row/patch counters and capture matrix. | attributable baseline |
| M1 | Gate structured option projection on a non-empty source. | focused RED-to-GREEN contract |
| M2 | Compile immutable typed option source and compact dynamic state patches. | static parse once/generation |
| M3 | Virtualize/reuse visible popup/menu/select rows and unify interaction identity. | O(V) materialization |
| M4 | Hard-cut duplicate raw/structured/text representations and string parsers. | one source authority |
| M5 | Run managed scale/input/WPR/power and visual/accessibility parity matrix. | quantified acceptance |

## M1 implementation result

Generic structured-option projection now returns immediately for an empty source slice. Orphan
query/hover/selected attributes cannot create rows and no longer enter the structured state parser.
Non-empty option sources retain the exact existing parsing and interaction semantics.

Per no-options node:

| Structured option work | Before | After | Change |
| --- | ---: | ---: | ---: |
| option-state/query BTreeMap reads | 12 | 0 | -100% |
| empty selected/state set values | 7 | 0 | -100% |
| structured option parser calls | 1 | 0 | -100% |

M1 does not remove independent selection/search/tree reads or the O(N) multi-representation work for
non-empty options. M2-M4 own those boundaries.

Post-M1 owner scope:

- Rust files: 8/8
- lines: 402
- bytes: 13,384
- joined raw source-bytes SHA256:
  `c2b570ac6e34f0524da40c80080ba9ab1f8c72addb70b99e83b24cb1828c3c87`
- unchanged owner files: 7 retain the pre-M1 fingerprints above

| Changed file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `selection_options/structured.rs` | 37 | 1,072 | `dbff79252d3f258625655f11fbc41473cc617eeb2031eb16279f8b91d06cc0dc` |

Focused contract: `tools/tests/test_editor_selection_option_source_performance_contract.py`, 27
lines, 895 bytes, SHA256
`95de20d4ac3bf3386a9f91d47d6328df1aaf564f35c3af35e266e747df99a84b`.

## Validation state

- Full owner source review: passed, 8/8 Rust files.
- Host/painter/hit/keyboard/workbench consumers and Unreal list sources above: read.
- M1 focused contract: RED 1/1 before the change, GREEN 1/1 after the change.
- Current owned performance-contract set: GREEN 50/50.
- `rustfmt --check` for the changed Rust file and scoped `git diff --check`: passed.
- A Rust regression verifies an empty source ignores orphan option-state attributes; it is present but
  not claimed passing until managed Cargo is executable.
- M0 and M2-M5 remain pending; no dynamic performance claim is made from static field-read counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR remains pending a current-source launchable editor; RenderDoc is not an acceptance tool for
  this CPU/interaction-source module.

The module remains in `pending.md` until M0-M5 pass on one source/executable/options fingerprint.
