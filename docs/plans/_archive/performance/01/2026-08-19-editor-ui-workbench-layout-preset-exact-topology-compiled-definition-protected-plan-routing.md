---
related_code:
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/page_layout_template.rs
  - zircon_editor/src/ui/workbench/preset
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs
plan_sources:
  - docs/plans/performance/01/2026-08-19-editor-ui-workbench-layout-preset-exact-topology-compiled-definition-architecture-review.md
owner_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/zircon_editor/editor_layout/04-layout-presets-and-persistence.md
  - docs/plans/zircon_editor/editor_layout/05-page-layout-templates.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/LayoutService.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorkflowOrientedApp/WorkflowTabManager.cpp
doc_type: protected-plan-routing
status: requested
created_at: 2026-08-19
---

# Protected plan routing: Exact layout profiles and compiled definitions

## Reason for routing

Performance01 and numbered editor plans are shared authorities. EditorLayout04/05 currently record
focused completion, while Optimize13 and several reviewed source/tests contain foreign in-progress
changes. This record routes current 9/9-file evidence without overwriting those owners. Detailed
evidence source:
`2026-08-19-editor-ui-workbench-layout-preset-exact-topology-compiled-definition-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-077

Expand page-switch evidence to the exact current chain: full layout clone, full preset-store
deserialize, drawer clone/document scan, linear scope update and full store serialize/write, first
metadata/native sync, second full store deserialize, destructive preset/fallback apply and second
metadata/native sync. Acceptance requires one staged transaction and delta, immutable indexed
profile generation, UI-thread store parse/serialize/write `=0`, and one coalesced durability request.

Add the semantic gate: A -> B -> A without user edits must preserve exact mixed-axis tree, ratios,
tab-to-leaf placement and active tabs. The current `{axis, leaf_count}` capture is not a layout
roundtrip and must not remain under the same schema name.

### PERF-MVP-097

Remove any implication that borrowed canonical reads close layout clone work. Preset capture still
deep-clones an activity-window drawer map; apply writes both canonical and legacy drawers; host
capture clones the full layout. Schema v2 must eliminate the legacy write authority and profile
consumers must use immutable generations/typed deltas.

### PERF-MVP-076 and PERF-MVP-106

Add the presentation-time definition cost: every main/native `build_host_window_shell_data()`
constructs the complete Material/Fyrox/JetBrains/Unreal design stack only to consume four fixed IDs.
M0 must record constructor calls, allocated objects/bytes and time inside
`apply_shell_presentation_from_state`. Stable presentation requires definition compile/build and
owned allocation `=0`; all presenters borrow one compiled definition generation.

### Pending/review accounting

Record `layout_preset.rs`, `page_layout_template.rs` and `preset/**` as `9/9 static reviewed,
dynamic pending`, linked to the detailed evidence. Do not move them to `review.md` until exact
roundtrip, product wiring, allocator/counter, current Cargo, F4 and WPR/ETW/power gates pass.

## Requested EditorLayout04 correction

The historical S2 `implemented-focused-passed` result only proves the deliberately reduced schema
can serialize and reconstruct drawer modes/sizes plus a coarse split shape. It does not prove layout
persistence. Reopen the plan and replace its target contract:

- exact versioned `LayoutProfile` preserves topology, ratios, leaf allocation and active state;
- `LayoutModePatch` owns Authoring/Review/Focus/Debug visibility/extent changes without implicit
  document topology rewrite;
- missing/stale profile yields a diagnostic and staged template choice, not immediate Authoring
  mutation;
- A-B-A exact roundtrip, atomic failure/cancel and UI-thread zero whole-store I/O are completion
  gates;
- persistence uses generation-indexed dirty records, coalesced background durability and receipts.

Do not preserve the old lossy document format as a current compatibility layer. Consume it through a
one-way bounded migration with explicit loss diagnostics or reject it in favor of the saved LKG.

## Requested EditorLayout05 correction

The current `implemented-focused-passed` state is source-only: `PageLayoutTemplate` has no product
caller, `page_templates.toml` is read only by a test, and `presets.toml` has no consumer. Reopen the
plan until one authored source is compiled into one immutable `EditorLayoutDefinitionGeneration`
used by startup, page/window/view registry, shell projection and tests.

Delete the parallel Rust/TOML declarations in the replacing milestone. Product instantiation and
generation tests replace string-presence tests; plugin-owned page definitions carry owner/version
and retire/rebind through the shared plugin lifecycle.

## Requested Optimize13 and EditorLayout07 updates

Keep Optimize13 as the authority for transaction, schema migration, LKG/quarantine, dirty decision,
unknown-plugin placeholders and native placement. Add these constraints to M1-M4:

- profile topology is exact; template and mode patch are distinct typed documents;
- page switch is one `LayoutTransaction` against the same placement index/delta as dock operations;
- current serialization contains one canonical drawer authority and one definition/profile
  generation identity;
- restore admission has depth/node/tab/window/string/file budgets before live mutation;
- persistent document payload remains factory/toolkit-owned and separate from dock topology.

EditorLayout07 owns the stable node/stack identities and delta needed by exact profiles; it must not
make the profile store rediscover topology through full scans.

## Requested EditorUI08 updates

- Receive the compiled definition and profile/layout generations as borrowed handles.
- Do not construct the design stack during shell projection or cache a second mutable copy.
- Apply page-switch/dock deltas once to affected presenters; unchanged shell/preset generation does
  zero definition, layout, metadata and native work.
- Keep profile serialization and config durability outside UI/shell locks and presentation scopes.

The four-ID direct-constant cleanup may be implemented after M0 captures a current baseline, but it
does not close this routing item until the duplicate definition authorities and lossy profile schema
are removed.

## Acceptance handoff

Owner completion must return one current-source bundle containing:

1. exact canonical A-B-A topology golden tests plus legacy/current/future migration fixtures;
2. atomic missing/stale/invalid/cancel/fault tests proving live state and generations unchanged;
3. scale counters for store parse/serialize/write, layout/tab clone bytes, flatten probes,
   transaction/delta/metadata/native sync and definition builds/allocations;
4. product proof that page/preset definitions have one compiled owner and no dead duplicate source;
5. managed Windows Cargo/F4 and WPR/ETW allocation, lock, latency, RSS and package-power evidence on
   D/E/F only;
6. RenderDoc parity only if the hard cut changes visible layout rendering.

Until this handoff, keep the module pending and do not issue a milestone commit or WeCom completion
message for this workstream.
