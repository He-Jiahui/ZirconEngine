# Editor core root contracts current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor12 owns authoring descriptor construction and frozen extension publication; Editor02 owns editor-event delivery/retention; Editor04 owns pending-edit payload retention; Editor10 owns project-open/startup I/O.
- Accounting: keep the four exact root files in `pending.md`; do not add them to `review.md` before current-source managed Cargo, construction/payload scale counters and F0/F4 product traces are GREEN.
- Code disposition: no Rust source was changed. Existing modifications in `editor_operation.rs` and `mod.rs` were preserved exactly; the construction-only observations do not justify a behavior edit without scale evidence.

## Exact scope

| module | files | physical lines | inline tests | ignored | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---:|---|
| editor core root contracts | 4/4 | 630 | 0 | 0 | `992f306b1cec27b5d1572a982f2c519c20e116b7c3a06207cf15a66eec6da709` |

| file | current SHA256 | current-source review |
|---|---|---|
| `zircon_editor/src/core/editor_authoring_extension.rs` | `7E851F20BAF75D251C465A473617F800FDD94D5CB8C5F965568380D060282BD7` | Borrowed getters and move-based builders are appropriate for immutable descriptors. `with_track_type` sorts/deduplicates the complete vector after every append; repeated `with_required_capabilities` chains do the same through `push_capabilities`. This is construction-time amplification, not a stable-frame path. |
| `zircon_editor/src/core/editor_operation.rs` | `5DB1BEB5253E74757CF2C402C8D0C364E3AE1FE043FD57537D92CCF1D3E91AD5` | Operation-path validation streams over segments/chars without a temporary segment vector and `Borrow<str>` permits borrowed map lookup. The DTO intentionally owns JSON; expensive clones occur in event delivery and pending-edit retention, outside this root contract. |
| `zircon_editor/src/core/gui_startup_request.rs` | `F3847A251C3F5BE6A8847F581A4A761CFD63532522F17AD22D5EE16E6FD2C44F` | A small owned startup enum moves through run configuration into one startup match. The builtin-view branch clones one descriptor string once; project open/create cost is filesystem and authority work owned by the project plans, not this DTO. |
| `zircon_editor/src/core/mod.rs` | `860C9EAD89612C2F392466B9778BB43BA29AB20E29D86C3F95159F05AEA298A5` | Module mounts only; no runtime algorithm, allocation, lock, queue, callback or thread boundary. |

All four files were read in full. Supporting reachability was followed through `zircon_plugins/editor_support`, authoring descriptor tests, extension registration, operation dispatch/event publication, pending edits, retained-host run configuration and editor startup. Supporting callers are evidence, not additional folder accounting.

## Current bottlenecks and ownership

### PERF-MVP-538 / PERF-MVP-079: build once, finalize once

- `TimelineEditorDescriptor::with_track_type` performs `push + sort + dedup` for each chained track type. Building `k` entries one at a time repeatedly sorts growing prefixes; capability builders have the same shape when chained more than once.
- Current workspace reachability constructs one track type in `zircon_plugins/editor_support`'s test fixture and one in the editor descriptor test. No per-frame production caller was found, so this is not a new MVP P0 hotspot by itself.
- The relevant scale risk is plugin/catalog bootstrap: many descriptors are then registered family by family into `EditorExtensionRegistry`, whose candidate rebuild and deep-copy costs already belong to PERF-MVP-538/079. Editor12 should accept a batch/finalize API that owns unsorted input during construction, validates/sorts/deduplicates once, builds direct indexes once and atomically publishes one frozen generation.

### PERF-MVP-067 / PERF-MVP-551: keep wide payload ownership downstream

- `EditorOperationInvocation::with_arguments` moves the `serde_json::Value` into the DTO. Normal dispatch also moves the invocation into execution; the root file does not independently clone the JSON.
- Event journal/listener delivery can clone `operation_arguments` and materialize JSON again; this remains PERF-MVP-067. Play pending-edit retention and snapshots can retain wide invocations for a long session; this remains PERF-MVP-551, whose current source already shares intent payloads through `Arc` at the decision boundary.
- Do not replace the typed DTO with a second serialized string or a global payload cache. Acceptance belongs at the owners that fan out, retain and page the payload.

### PERF-MVP-075 / PERF-MVP-100: startup request is not the I/O owner

- `EditorGuiStartupRequest` is consumed once by `resolve_editor_startup_session`. Open/create variants move their `PathBuf` or draft into the project manager; there is no polling loop, queue or background-thread handoff in the root contract.
- Canonicalization, manifest parsing, inventory scans, recent-project validation and welcome probes remain PERF-MVP-075/100. Optimizing the small enum cannot establish the required F0 project-open budget.

## Optimization and acceptance plan

- Descriptor matrix: track types/capabilities/descriptors `0/1/100/10K`, registration batches `1/100`, duplicates `0/1/50%`. Record builder sorts, compared/moved strings, allocations/bytes, registry candidate builds and publications. Require one normalize/finalize and one atomic publish per structural generation, with stable generation work equal to zero.
- Operation matrix: payload `64 B/2 MiB/64 MiB`, listeners/pending edits `0/1/100/10K`, play stall `0/60 min`. Record JSON clone/materialization bytes, retained bytes, queue age, dispatch p95 and exit apply budget under PERF-MVP-067/551.
- Startup matrix: recent projects/assets `0/1/100/1K`, unchanged/invalid/missing/link cases. Record canonical/manifest/inventory/probe counts and F0 wall under PERF-MVP-075/100; root request copies must stay constant and construction-only.
- Preserve deterministic ordering, duplicate diagnostics, serde compatibility, operation-path validation, rollback semantics and startup behavior. Do not create a private thread pool or move UI-affine work merely to hide synchronous cost.

## Cross-engine evidence and intentional divergence

- Bevy `dev/bevy/crates/bevy_app/src/plugin.rs` exposes explicit `Plugin::build`, `finish` and `cleanup` lifecycle phases. This supports doing descriptor normalization/publication once during structural construction rather than in stable editor ticks. Zircon additionally requires transactional rollback and generation identity across dynamic editor plugins.
- Godot `dev/godot/core/object/class_db.h` exposes explicit class and extension registration/unregistration boundaries. This supports a finalized registry generation rather than rebuilding descriptor indexes on ordinary reads; Zircon keeps typed Rust DTOs and immutable shared snapshots instead of adopting ClassDB's global mutable authority.
- The existing editor-extension evidence remains authoritative for Bevy changed-data, Fyrox explicit sync and Unreal structural refresh routing; this root review does not duplicate those tasks.

## Static gates executed

- Read all 4 exact Rust files plus the production/supporting construction, dispatch, pending-edit and startup chains at current source.
- `rustfmt --check --edition 2024 --config skip_children=true` passed for 3/4 files. `editor_operation.rs` differs only in the existing serde import ordering.
- The current descriptor test file contains 4 tests; `zircon_plugins/editor_support` contains the supporting all-family batch fixture. Neither was executed dynamically in this pass.
- No managed Cargo, allocation counter, F0/F4 product trace or RenderDoc capture ran. RenderDoc is not applicable to these non-rendering DTO/module-mount files. The scope remains pending.
