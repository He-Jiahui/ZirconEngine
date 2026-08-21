---
related_code:
  - zircon_editor/src/ui/workbench/event
  - zircon_editor/src/core/editor_event/workbench
  - zircon_editor/src/ui/workbench/layout
  - zircon_editor/src/ui/workbench/view
  - zircon_editor/src/ui/host/editor_event_execution/layout_command.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch
  - zircon_editor/src/ui/retained_host/menu_pointer
  - zircon_editor/src/ui/workbench/reflection/model_build.rs
tests:
  - zircon_editor/src/tests/workbench/host_events
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/49-editor-event-runtime-envelope-listener-registry-journal-replay-snapshot-dirty-lifecycle-product-integration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenuEntry.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenuEntry.cpp
doc_type: current-architecture-performance-review
status: static_complete_structural_cutover_required_dynamic_pending
created_at: 2026-08-19
---

# Editor UI Workbench event canonical layout command generation architecture review

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- MVP priority: P0 for duplicate Layout authority, no-op invalidation and stable menu generation; P1
  for legacy menu codecs and the one-variant host wrapper.
- Accounting: retain `zircon_editor/src/ui/workbench/event/**` in `pending.md`. Do not add it to
  `review.md` before one canonical Layout command model, typed commit receipts, compiled menu identity,
  scale counters and F4 product traces pass.
- Code disposition: no Rust source changed. `zircon_editor/src` is held by the active MVP00 session,
  and the correct fix removes cross-module authorities rather than optimizing one conversion helper.

## Exact scope

| scope | files | physical lines | tests | raw bytes | ordered path-and-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/event/**` | 13/13 | 768 | 0 in-module | 33,989 | `e48bdd7ea8b8d72c145df6682d4d0e626048dafe2dc81a329a96b07bd84e0cd1` |
| external Workbench host-event tests | 2/2 | 165 | 7 | 5,391 | `9f741f20f7e401fb57fa98e7d3f23964dd85f1dc5d4598f7beaa74a82f3af97a` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw bytes, NUL. All 13 production files
and both external test files were read in full. The scoped source and tests are clean in the current
worktree.

## Module acceptance record

| module | files/lines | current-source performance verdict |
|---|---:|---|
| core/UI conversion | 1 / 330 | P0: converts all 15 Layout variants and related IDs/hosts in both directions. The product callback moves UI data into Core, then execution clones Core strings and path vectors back into the identical UI model while holding the Workbench shell lock. |
| host dispatch/error/event | 3 / 30 | `EditorHostEvent` has one `Menu` variant and every production caller immediately destructures it. Unknown action allocates another owned String only for the error path. |
| operation/menu binding | 3 / 133 | A typed `EditorOperationPath` is copied into both control ID and payload String. Menu actions separately allocate canonical action ID and control ID before binding construction. |
| menu codecs | 2 / 191 | Canonical ID, control ID, operation path and legacy aliases are independent exhaustive maps. Static actions use 41 `.to_string()` sites plus six formatting sites; decode accepts multiple legacy prefix/name forms. |
| menu item binding | 1 / 104 | Every projection rebuilds an owned binding. Reflection calls it twice for one leaf. A leaf with neither action nor operation receives an empty clickable binding instead of an explicit non-action kind. |
| node-kind codecs | 2 / 60 | Duplicates lower/upper naming maps for nine node kinds. It is part of the same generated identity problem, not an independent hotspot. |
| constants/export facade | 2 / 24 | Structural only. |

## Structural bottlenecks

### P0: one layout interaction crosses two identical domain models

`crate::core::editor_event::workbench` and `crate::ui::workbench::{layout,view}` each define the same
15-variant `LayoutCommand`, `MainPageId`, `ViewInstanceId`, `ViewHost`, `WorkspaceTarget`, drawer,
split and tab-anchor types. The retained callback consumes the UI command and moves it into the Core
copy. `execute_layout_command()` receives the Core copy by reference, calls
`ui_layout_command_from_core()`, deep-copies every owned ID/name/path, then gives the recreated UI
command to `LayoutManager`.

This is not an ABI boundary: both models are inside `zircon_editor`. Focus, drawer mode, page and
extent events pay the conversion while the global shell mutex is held. Split and host changes also
copy path vectors and nested IDs. A type alias would conceal rather than solve ownership. Optimize13's
`LayoutAuthority` must own the only command/ID schema; callbacks, transactions, persistence and the
Workbench manager consume it directly.

### P0: `changed=false` still publishes full layout and reflection invalidation

`execute_layout_command()` always allocates an effects vector containing `LayoutChanged`,
`PresentationChanged` and `ReflectionChanged`, even when `LayoutManager` returns `changed=false`.
`editor_event_dispatch.rs` copies those effects into the record and calls
`refresh_workbench_for_event_record()` unconditionally. The retained bridge requests layout and
presentation recompute, while reflection publishes Workbench presentation invalidation.

Therefore PERF-MVP-097's accurate no-op detection does not currently stop downstream work. A repeated
focus/mode/extent/page command can still rebuild chrome, view model, menu bindings, routes and
reflection. The fix is a typed `LayoutCommitReceipt { changed, generation, diff, effects }`, not a
blanket `if changed` around every side effect: preset persistence and diagnostics may succeed without
changing topology and need their own explicit receipt domains.

### P0: stable menu identity is rebuilt separately for presentation, pointer and reflection

Current command-registry menu items already carry typed `EditorOperationPath`, but
`editor_operation_binding()` serializes the path into two owned Strings per binding. The live pointer
layout calls `menu_item_binding()` for each leaf. Workbench reflection calls it twice for the same
leaf, first to clone the control ID and then to store a second complete binding. A separate host-scene
projection contains a third builder, but current caller search found only its crate-private re-export,
so this review does not claim it executes in the product path.

No-op Layout effects and other full reflection invalidations amplify this cost. The target is one
immutable `MenuGraphGeneration` produced from the command registry generation, with stable typed
command handle, entry kind, shared static label/path/shortcut metadata and dynamic enabled/checked
deltas. Pointer, visual and reflection consumers borrow or share that generation; they do not each
recreate binding strings.

### P0: a menu click can traverse binding, host event and operation identity repeatedly

Retained menu fallback constructs an `EditorUiBinding` from the incoming action string, copying the
same value into control ID and payload. It decodes that binding into `MenuAction`, wraps it in the
one-variant `EditorHostEvent`, immediately unwraps it, maps it through a separate operation-path table,
and parses a new owned `EditorOperationPath` before invoking the command service. Common binding
dispatch performs the same host-event and operation-path conversion.

Optimize08 already owns the correct shape: one versioned command/menu identity catalog and one
InvocationGateway. A compiled local menu entry carries its command handle directly. Only a bounded
external migration adapter accepts legacy strings, records their use and is deleted at the declared
cutoff.

### P1: current tests preserve text aliases, not the product algorithm

The seven external tests assert native string roundtrips and explicitly require old forms such as
`CreateNode.Cube`, `OpenView.editor.scene`, `menu_action.*`, `SaveProject` and `CloseProject`. They do
not cover Layout conversion, no-op effects, binding construction count, menu generation reuse,
owner revoke, stale handles or large menus. Migrate golden coverage to the versioned external codec
and product InvocationGateway, then delete the legacy aliases and `EditorHostEvent` wrapper. Tests
must not keep a second internal execution route alive.

## Reference-engine evidence

- Unreal `ToolMenuEntry.h:178-193` creates menu entries from shared `FUICommandInfo`, an optional
  `FUICommandList`, explicit submenu or explicit separator kinds. `ToolMenuEntry.cpp:188-276` resolves
  the action from the shared command identity and derives name, label, tooltip and icon through
  `SetCommand()`. This supports one command/menu catalog and rejects Zircon's hand-maintained
  action-ID, control-ID and operation-path maps.
- Unreal `TabManager.h:852-890` gives `FTabManager::FLayout` one shared recursive layout model;
  `PersistLayout()`, `RestoreFrom()` and relocation operate on that model. Zircon should keep Rust
  transaction and validation semantics, but not copy a command between two identical in-crate enums.
- Unreal `TabManager.cpp:1101-1180` gathers one persistent layout and debounces save requests by five
  seconds specifically to avoid resize hitches. `OnTabRelocated()` updates the live manager and
  requests deferred persistence. This supports separating an in-memory layout commit/diff from
  persistence work rather than issuing broad synchronous effects per event.
- Unreal `UICommandInfo.h`/`UICommandList.h` retain shared command identity and direct action lookup.
  Zircon should adopt the identity/ownership shape without reproducing Unreal globals or delegate
  lifetime rules.

These sources establish ownership and flow, not timing parity. Same-hardware product traces remain
mandatory.

## Required architecture cutover

1. Optimize13/EditorUI08 moves the canonical Layout schema, IDs, command and validation into one
   `LayoutAuthority`. Workbench state consumes the canonical command directly. Delete the UI/Core
   duplicate enums and all conversion helpers in the same hard cut.
2. Every layout mutation stages and validates a candidate, commits one generation and returns a typed
   receipt containing exact diff and affected domains. Failure and no-op retain the previous authority;
   no-op topology emits no layout/presentation/reflection invalidation.
3. Optimize08/Editor08 publishes a versioned command/menu identity catalog and immutable
   `MenuGraphGeneration`. Menu entries use explicit action, submenu, separator/header and widget
   kinds. Static binding/route metadata is compiled once per generation; dynamic command state is a
   narrow delta.
4. Local menu, keyboard, palette, template and pointer actions carry a typed command handle to the
   InvocationGateway. External strings are parsed once under budgets and policy. Remove
   `EditorHostEvent`, parallel menu codecs and operation reparsing after migration telemetry reaches
   zero.
5. Reflection, pointer and visual menu projections share the same menu generation and route handles.
   Reflection builds a binding at most once per changed entry and never twice in one row constructor.
6. Optimize49 receives semantic command receipts only. No-op layout requests do not masquerade as
   document commits; audit may record an explicitly sampled no-op without triggering UI rebuild.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| layout authority | every 15-variant command; tabs `1/1k/10k`; depth `1/64/max+1` | one production Layout command/ID definition; conversion helpers and duplicate serde schemas `=0`; target validation before mutation; failure leaves authority byte-equivalent |
| no-op | focus/mode/extent/page repeated `1/1k/1M` | changed `=false`; layout/presentation/reflection effects, menu/reflection/chrome build and persistent write `=0`; shell-lock work near O(1) lookup |
| changed layout | move/attach/split/drawer/page/preset; affected nodes `1/100/10k` | one commit generation and one typed diff; work proportional to affected subtree; one publication; persistence outside UI lock and debounced where allowed |
| menu generation | entries `1/100/10k/100k`; owners `1/1k`; stable/context/reload | static graph/binding/label/path build once per definition generation; stable build and String bytes `=0`; context change touches affected state; owner revoke atomic |
| invocation | UI/menu/keyboard/template/remote plus every legacy alias | local parse/String/binding/host-wrapper conversions `=0`; one gateway policy decision and receipt; external codec bounded; legacy use metered then zero at cutoff |
| product | F4 cold/warm/idle/menu storm/layout storm, 31 runs | WPR/ETW CPU, shell/command lock wait+hold, wakeups, allocation bytes, invalidation/build counts, input-to-effect p95 and package power on identical hardware/assets/settings; artifacts remain on D/E/F |

RenderDoc is conditional. Require it if layout/menu cutover changes submitted UI geometry, clipping,
resources or visible output; then capture draw/event/resource and pixel parity. WPR/ETW and allocator
counters own the command/invalidation proof.

## Static gates executed

- Read 13/13 production files and both external test files; reproduced 768 production lines, 33,989
  bytes, seven external tests and production fingerprint `e48bdd7e...`.
- Traced both Layout conversion directions through retained callback, shell-locked execution, event
  record, retained invalidation and full reflection. Confirmed no changed gate removes effects.
- Traced menu generation through live pointer layout and reflection; confirmed reflection constructs
  two complete bindings per leaf. The separate host-scene builder has no current production caller.
- Read the cited Unreal ToolMenus, command and TabManager primary sources and current Optimize08,
  Optimize13 and Optimize49 owner reports.
- `rustfmt --edition 2021 --check` passed for all 13 production and 2 external test files. Scoped
  `git diff --check`, 43/43 routed-path existence and
  `python -m tools.session_coordinator --repo-root . --json plan audit` passed. The production
  fingerprint remains `e48bdd7e...` after the documentation write.
- The documentation convention gate reports 0 violations owned by these two records. The unrelated
  repository baseline remains 692 violations across 242 documents; concurrent work increased the
  scanned document count to 2,718.
- Dynamic Cargo, layout/menu allocation counters, no-op and scale matrices, F4 launch, WPR/ETW,
  package power and conditional RenderDoc evidence remain pending. This is not an accepted milestone,
  so no commit or WeCom notification is due.
