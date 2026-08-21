# Editor core export single-pipeline architecture revalidation

## Status

- Result: `static_complete / structural_cutover_required / dynamic_pending`.
- Review date: 2026-08-19.
- MVP priority: P0 for reproducible F4 Build/Export; P1 for scale and durability tuning after the
  ownership cutover is correct.
- Owners: Editor15 owns the one export graph and stage receipts; Runtime04 owns cooked and packed
  artifact identity; Runtime11 and Editor14 own bounded process, filesystem and persistence work;
  Plugins09 owns native-plugin package receipts; EditorUI08 only projects immutable progress.
- Accounting: retain `zircon_editor/src/core/export/**` in `pending.md`. Do not add it to `review.md`
  before the single-owner cutover, current managed tests and the scale/trace matrix below pass.
- Code disposition: no Rust source changed. Five files are foreign modified only by formatting, and
  an active MVP session currently owns overlapping export process source. This review preserves it.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/export/**` | 9/9 | 3,061 | 24 | `361a4a15d3a4254ddbe2c7f5518320d5242091791bbf6843641ed1cc519ac4c0` |

The fingerprint is SHA256 over sorted normalized path, NUL, raw file bytes, NUL. All nine current
Rust files and all tests were read in full. The fingerprint exactly matches the 2026-07-30 report;
the tracked differences are import/assert formatting, not a new implementation. This report does not
replace that file-level review. It adds the missing product orchestration review through:

- `wizard/plan.rs`, which already builds the authoritative selected eight-stage graph;
- `wizard/run.rs`, which executes that graph and owns cancellation/progress;
- `wizard/execution.rs`, which owns the cancellable shared process adapter;
- `wizard/execution/core_pipeline.rs`, which creates additional nested core pipelines for
  `CompileHost` and `PlatformBundle`.

## Per-file acceptance record

| file | current-source verdict |
|---|---|
| `inventory.rs` | Strong file identity, overlapping-path generation reuse and 64 KiB streaming BLAKE3 are positive. Stable directory inputs still enumerate, sort, canonicalize and query every file; every fresh inventory still probes Python/Cargo/rustc and optional Node; cancellation/deadline checkpoints are absent; `Drop` performs full-cache clone, pretty JSON encoding, write, `sync_all` and replace on the caller. |
| `mod.rs` | Export surface only; it currently exposes the nested executor used by the wizard. |
| `pipeline.rs` | Typed dependencies, deterministic order, explicit skipped records and output revalidation are useful. Eight-stage graph algorithms are bounded and not a hotspot. The abstraction becomes harmful because the product creates multiple pipeline owners instead of executing one graph with one receipt/inventory generation. |
| `preset.rs` | Atomic versioned preset persistence is correct control-plane behavior. Whole-file encode/read and durability are small per explicit save and are not an independent P0 hotspot. |
| `stages/compile_host.rs` | The fallback runner creates two private reader threads, uses per-byte `VecDeque` tail insertion/eviction, serially waits for two log `sync_all` operations and one manifest `sync_all`, and lacks deadline/cancellation/process-tree cleanup. The product adapter has cancellation, but this exported fallback still defines a second process/log authority. |
| `stages/executor.rs` | `CompileHost::prepare` fingerprints broad source roots and launches three or four tool probes before reuse can be known. Build output is then recursively fingerprinted. A new executor for `PlatformBundle` repeats the complete source/tool preparation before revalidating the same staged output. |
| `stages/mod.rs` | Stage surface only. |
| `stages/platform_bundle.rs` | Four fixed existence checks are cheap. It validates an engine-development layout, while the outer wizard separately builds the actual project bundle; these are different artifacts hidden behind the same stage name. |
| `tests.rs` | Tests cover graph/reuse/tamper/preset/layout semantics, but only the isolated two-stage core plan. They do not assert one product pipeline owner, one source inventory generation, one tool-probe generation, stage receipt identity, cancellation during fingerprinting, or Build/Cook/Stage/Package product equivalence. |

## Structural bottlenecks

### P0: the product has an outer eight-stage pipeline and nested core pipelines

`ExportWizardPipelinePlan` already owns stage selection and order. `run_export_wizard_job` executes
each selected stage, emits progress and applies cancellation. Despite that, `run_core_compile_host`
constructs a new one-stage `ExportPipelinePlan`, loads a private `.core.json` report, executes it and
writes another durable report. Later `run_core_platform_bundle` constructs a fresh executor and runs
the two-stage core plan over the same report.

The second pass cannot reach `PlatformBundle` without replaying `CompileHost::prepare`. It therefore
re-enumerates every declared source directory, canonicalizes/stats files, launches Python/Cargo/rustc
and optional Node version processes, and hashes/revalidates the staged engine root. The outer wizard
then continues its own Cook/Pack/PlatformBundle/Report receipts. One product export consequently has:

- two graph authorities and two report formats;
- at least two fresh `ExportGenerationInventory` instances for CompileHost plus PlatformBundle;
- repeated source discovery and tool probes even when the first CompileHost stage just succeeded;
- two meanings of PlatformBundle: validation of the staged engine-development tree and creation of
  the project distribution bundle;
- cancellation owned by the outer runner but no cancellation during inner preparation/fingerprint.

This is the primary current design defect. Improving the BTreeMap or adding parallel directory walks
would preserve duplicate work and make cancellation/order harder to reason about.

### P0: stage reuse is derived by rescanning broad roots instead of consuming owner receipts

`CompileHost` declares hand-maintained repository paths and hashes their complete directory trees.
That duplicates the Rust/Node build tools' own dependency/action state and misses a typed relationship
between the produced binary set and its exact build invocation. PlatformBundle then treats a recursive
digest of the entire staged tree as both build truth and bundle input truth.

The target contract is receipt-driven:

1. Editor15 creates one immutable `ExportRunGeneration` from normalized preset, target, toolchain
   selection and source/project generations.
2. CompileHost delegates incremental compilation to the build owner and receives a
   `BuildProductManifest` containing target/config/toolchain/action-generation plus content-addressed
   produced files. The editor does not rediscover Rust dependencies by walking all source roots.
3. Runtime04 Cook returns a cooked-artifact manifest keyed by source asset generation and cook key;
   Pack consumes that manifest/chunk store without rereading source assets.
4. Plugins09 returns one native package manifest consumed by Stage; no stage independently rescans
   package directories.
5. Stage creates one destination-to-source manifest, incrementally copies changed entries and emits a
   staged-bundle receipt. Package/Archive consume that receipt and never infer membership by walking
   the output directory.
6. Every receipt separates required product artifacts from diagnostics/logs. Missing logs never
   invalidate valid binaries; tampered required artifacts do.

Content hashing remains the correctness fallback for changed or untrusted entries. It is not the
first operation for every stable export.

### P0: cache persistence and fingerprinting are outside shared budget/cancellation authority

`ExportGenerationInventory::drop` silently performs potentially large JSON serialization and durable
I/O. Error and cancellation paths pay the same work and discard persistence errors. Directory walks,
file reads and tool probes do not consult the outer cancellation signal. The system fallback also
owns two ad-hoc reader threads and three durability barriers.

Runtime11 must own explicit inventory/receipt persistence and process output as bounded jobs with
entry, byte, age and deadline budgets. Persistence returns a receipt or observable error; `Drop` does
no I/O. Fingerprint fallback checks cancellation after bounded entries/bytes. The product uses the
existing cancellable process-tree adapter only; the private system runner is deleted or reduced to a
thin adapter over that same authority.

## Unreal source evidence

- `BuildCookRun.Automation.cs:251-266` has one top-level owner and calls Build, Cook,
  CopyBuildToStagingDirectory, Package and Archive once in explicit order. It reports the whole run
  time at `26-45`.
- `ProjectParams.cs:1562-1577`, `1611-1633`, `1714-1736`, `1849-1906` models Build, Cook, Pak,
  Stage and Archive as distinct switches, including explicit skip/reuse modes. Stages are not hidden
  nested reruns of preceding stages.
- `BuildProjectCommand.Automation.cs:60-67` exits if Build is not selected, then builds one agenda of
  requested targets and verifies produced build products at `243-258`.
- `UnrealBuildTool/Modes/BuildMode.cs:426-515` constructs one action graph and action history;
  `541-620` determines outdated prerequisite actions. Incremental build truth belongs to the build
  system, not a packaging-layer recursive source hash.
- `CookCommand.Automation.cs:276-303` independently gates Cook/SkipCook and requires the editor
  product, preserving a stage boundary.
- `CopyBuildToStagingDirectory.Automation.cs:1617-1642` creates one staging manifest;
  `2941-2966` copies its explicit mapping incrementally; `6528-6572` packages/copies from the same
  manifest and then runs platform/custom post-stage hooks.
- Godot `editor_export_platform.cpp:1013-1056` retains per-file export cache identity, and
  `1319-1744` drives file export through one traversal. Fyrox `editor/src/export/mod.rs:347-405`
  supplies a secondary cancellation/process-cleanup example.

The intended alignment is ownership and dataflow, not a line-for-line port: one orchestration owner,
separate stage commands, build-system action history, cooked/product manifests and incremental stage
copy. Zircon should retain its stronger file identity and BLAKE3 fallback.

## Required hard cutover

1. Make `ExportWizardPipelinePlan` or its headless core successor the sole graph owner shared by UI,
   commandlet and CI. UI is a projection; it may not own a second semantic pipeline.
2. Delete `ExportWizardCoreStageProjection`, per-stage nested plan creation and `.core.json` report.
   One `ExportPipelineReport`/receipt journal records all selected stages atomically.
3. Replace `ZirconBuildStageExecutor` broad-source fingerprinting with a build product receipt. Until
   the build tool exposes it, treat build invocation as a conservative stage, but do not maintain a
   second hand-written source dependency graph in the editor.
4. Rename/split current engine-tree validation from project `PlatformBundle`. Stage consumes explicit
   Build/Cook/Pack/Plugin manifests and produces one destination mapping plus bundle receipt.
5. Move cache/report/log persistence to Runtime11 jobs; `Drop` I/O and private output threads become
   zero. Required artifacts and optional diagnostics have separate validity.
6. Preserve current tamper detection, deterministic ordering, atomic report replacement, strong file
   identity and streaming hash as fallback invariants.

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| ownership | UI/commandlet/CI, client/server, selected/full stages, resume/cancel/failure | graph owner=1; report journal=1; stage execute count<=1/run; nested pipeline/report=0 |
| discovery | files `1/1k/100k`, unchanged/1%/rename/delete, tools `0/1/4` changed | source full-tree editor walks=0 after build receipt; tool probes<=1/toolchain generation; stable stage visits near changed manifest entries |
| artifacts | build/cook/plugin/pack/stage entries `1/1k/100k`, logs present/deleted, required artifact tamper | log deletion rebuilds=0; required tamper invalidates exact stage/downstream; unchanged read/write/copy bytes near 0 |
| scheduling | cancel during discovery/hash/build/cook/copy/persist, worker stall, output `1MiB/1GiB` | bounded cancel latency; private threads=0; caller durability wait=0; queue entries/bytes/age and RSS hard bounded |
| product | cold/warm/1% changed F4 export, 31 runs, launch exported client/server | WPR/ETW CPU/waits/wakeups/file/process I/O/RSS/package-power distributions; stage counters explain wall time; exported product launches and matches preset |

Tracy spans must cover plan, build receipt, cook, pack, stage manifest, incremental copy, persist and
archive separately. WPR/ETW supplies CPU, waits, process and filesystem evidence. RenderDoc is not
applicable to the export control path; it is only used after launching the produced rendering product
to validate render-output parity. No artifact may be written to C:.

## Static gates executed

- Read 9/9 current Rust files, all 24 tests and the named product call chain.
- Reproduced the exact stable fingerprint `361a4a15...`; current foreign diffs are formatting only.
- Read the cited Unreal AutomationTool, UnrealBuildTool, Godot and Fyrox primary sources.
- Confirmed current product construction of one outer eight-stage graph, one nested CompileHost graph
  and a later fresh two-stage CompileHost/PlatformBundle graph.
- `python -m unittest tools.tests.test_editor15_export_generation_inventory_contract -v` passed
  10/10 static contracts. These protect the positive inventory/output baselines but do not test one
  product pipeline owner.
- Isolated `rustfmt --edition 2021 --check --config skip_children=true` passed 9/9 files. Both new
  documents have zero owned convention violations, all 18 routed paths exist, scoped diff check and
  plan audit pass, and the source fingerprint remained stable after documentation writes.
- Managed Cargo, WPR/ETW, Tracy and exported-product launch remain pending. The coordinator session
  registration endpoint timed out repeatedly after the prior session was retention-archived; this
  non-validation report continued without modifying overlapping product source.
- This is not an accepted milestone, so no commit or WeCom notification is due.
