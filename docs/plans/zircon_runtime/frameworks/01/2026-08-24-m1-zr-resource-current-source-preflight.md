# Frameworks01 M1 `zr_resource` current-source preflight (2026-08-24)

## Status

- `preflight_complete`
- `physical_hard_cut_applied_current_source_156_write_paths`
- `current_source_schema3_r5_frozen_snapshot_779_inputs_938e3cd6`
- `latest_consumer_manifest_snapshot_555_consumers_550ab519`
- `latest_move_manifest_snapshot_71_owners_87_operations_52deb199`
- `sealed_write_set_156_paths_12377714`
- `hard_cut_patch_applied_155_changes_plus_1_coordinator_preapplied_path_4ee6898d`
- `read_set_write_set_admission_model_repaired`
- `all_156_write_path_leases_acquired_zero_conflicts`
- `old_resource_implementation_deleted_69_paths_two_curated_facades_remain`
- `zr_resource_unique_implementation_owner_71_files`
- `private_old_owner_path_references_zero`
- `atomic_write_facade_preserved_for_ibl_consumers`
- `hard_cut_toolchain_regression_green_55_of_55`
- `managed_zr_resource_build_green_job_b3a38783`
- `managed_zr_resource_lib_test_wrapper_inconclusive_same_binary_green_150_passed_3_ignored`
- `managed_current_hash_lib_test_ticket_queued_ccf45045`
- `managed_current_source_lib_green_191_passed_4_ignored_job_bf4648ee`
- `event_stream_test_owner_folder_backed_root_615_lines_test_leaf_610_lines`
- `event_stream_structure_current_hash_managed_compile_pending`
- `parent_component_path_probe_compile_and_behavior_repaired`
- `isolated_zr_resource_direct_rustc_green_default_and_feature_probe`
- `isolated_zr_resource_tests_green_113_passed_4_ignored`
- `runtime_product_managed_cargo_gate_pending`
- `consumer_manifest_tool_source_green`
- `consumer_manifest_tool_profile_gate_green`
- `assembly_consumer_closure_repaired_8_consumers_41_fixtures_green`
- `hard_cut_rust_outputs_pinned_rustfmt_idempotent_80_of_80`
- `journal_folder_facade_visibility_repaired`
- `standalone_atomic_and_transaction_tests_green_41_of_41`
- `journal_split_current_owner_unified`
- `journal_split_explicit_candidate_pending`
- `managed_core_min_compile_receipt_pending`
- `runtime25_and_runtime11_attribution_fixed_3_current_blobs`
- `historical_56_path_gate_superseded_only_root_cargo_was_in_write_set`
- `engine_runtime_performance_claims_not_admitted`
- `milestone_not_accepted`

This is the current implementation and admission record, but not yet an accepted milestone result.
The physical owner cut is present in current source; managed Cargo/product validation, integration
review, coordinator candidate acceptance, commit, and WeCom completion notification remain open.

## Current Snapshot

The review used main HEAD `f811b3bf474d70347199772a175422333dfb36f6` and coordinator baseline
epoch 420. The current `zircon_runtime/src/core/resource` tree contains:

- 57 tracked Rust files;
- 11,111 lines;
- 412,214 bytes;
- tree SHA-256 `c824c74936e8a533954de2017aed726269ac3df4edd01a18c6323cf0684b23c2`.

The fingerprint is the SHA-256 of the LF-joined, path-sorted list of
`relative-path<TAB>lowercase-file-sha256`. It does not match the parent plan's epoch-321 fingerprint
`b06500a6f558b36880d5f051d566dddb054f2bf7bc23b370abdd06f4c16b9538`; implementation must not
reuse that older manifest.

The stable ABI owner at `zircon_runtime_interface/src/resource` remains 14 Rust files, now 823 lines
and 28,441 bytes. Four Interface inputs are dirty in current source: `src/lib.rs`,
`resource/{locator.rs,mod.rs,stable_uuid.rs}`. The Runtime implementation tree has 11 Git-dirty files
with 508 insertions and 135 deletions. A tracked lexical estimate now finds resource facade paths in
599 Rust files / 798 matching lines, plus 40 tracked Markdown/TOML files. This estimate is not the
atomic migration manifest; the implementing owner must regenerate the literal-plus-structured
Rust use-tree union, including nonignored untracked consumers.

## Dependency And Ownership Review

The production Resource tree is suitable for a low-layer crate after path rewriting. Direct
dependencies are `std`, `serde`, `thiserror`, `blake3`, and the approved
`zircon_runtime_interface::resource` DTO surface. `serde_json` is test-only. Apparent references to
Asset, diagnostics, and Frameworks occur only inside hard-cut scanner test fixtures; no production
Resource file directly depends on those higher Runtime domains.

The physical cut is not currently admissible:

- Frameworks01 r9 has an immutable scope for the open failures and `zr_math`; it does not own the
  Resource tree, new `zr_resource` tree, complete manifests/lock, or the current consumer union.
- Runtime51 session `optimize-runtime51-query-merge-heap-r1-20260824` is active and owns five
  Resource blobs: `management_generation.rs`, its projection test, and
  `manager/{lazy_registration,management_projection,resource_manager}.rs`; it also owns Interface
  `resource/locator.rs`. Its validation ticket is pending, so those hashes must not be rewritten or
  transferred prematurely.
- Runtime25 session `optimize-runtime25-filesystem-capability-truth-freeze-r1-20260823` is registered
  over `core/resource/io/resource_io.rs` and related filesystem capability contracts.
- Runtime24 session `optimize-runtime24-stable-uuid-v1-hard-cut-r1-20260824` is
  `waiting_validation` over Interface `resource/{mod.rs,stable_uuid.rs}`. These stable ABI inputs do
  not migrate into `zr_resource`, but their exact current hashes must settle before the atomic
  manifest and public-surface seal are frozen.
- Most durable-transaction and ResourceManager blobs retain stale attribution to archived
  Frameworks01 r8; several other blobs are unattributed or point to stale owners. The epoch-420
  ownership matrix therefore cannot form one executable transfer set yet.
- The two previously identified mixed Editor blobs remain excluded from Frameworks01 writes. The
  stable Runtime facade means their product imports do not need rewriting for the crate split.

## Reference-Engine Findings

- Unreal keeps `FAssetRegistryState` as a dedicated registry-state owner with explicit enumeration,
  mutation, and serialization policy, while platform file primitives remain in the lower
  `IPlatformFile` boundary. The relevant rule is to keep registry/query state and durable file
  primitives separately owned even when one private crate contains both subdomains.
- Bevy keeps handles, asset IDs, asset events, `Assets<T>`, server behavior, and I/O under the
  independent `bevy_asset` crate instead of the application facade. The relevant rule is a physical
  owner with curated outward projections, not copied DTOs.
- Fyrox uses a separate `fyrox-resource` crate and separates resource I/O, registry, loaders, and
  manager state. Its registry exposes explicit loading/status and mutation behavior rather than
  allowing upper scene/editor modules to own resource storage.

These references support the parent plan's `zr_resource` boundary. They do not support moving Asset
discovery/import/project-generation algorithms into the foundation crate: those remain upper
`zr_asset` responsibilities that consume the Resource foundation.

## Locked Hard-Cut Shape

When ownership converges, the implementation batch must:

1. Create `zircon_runtime/crates/zr_resource` as the only implementation owner with dependencies on
   `serde`, `thiserror`, `blake3`, and `zircon_runtime_interface`; keep `serde_json` test-only.
2. Move production data/error/event-stream/I/O/lease/management-generation/manager/mutation/
   readiness/registry/runtime/snapshot code and internal behavior tests into that crate.
3. Move Runtime-source-reading facade/hard-cut scanner tests to the Runtime integration/absorption
   guard owner so `zr_resource` never gains a dev-dependency on `zircon_runtime`.
4. Replace `zircon_runtime::core::resource` with an explicit curated product projection. Preserve
   `core::resource::io::atomic_write` as the public IBL/upper-layer entry and keep transaction,
   recovery, staging, `PendingAtomicWrite`, readiness rows, and event-byte helpers out of the
   external product API. The approved hidden sibling-crate surface remains `assembly`; Runtime must
   not re-export it.
5. Delete the old implementation children in the same batch. The remaining Runtime projection is
   architecture, not a forwarding compatibility implementation; no duplicate owner, wildcard
   projection, old implementation module, or direct App/Editor/plugin dependency on `zr_resource`
   is allowed.
6. Atomically update root/Runtime/Interface manifests, `Cargo.lock`, Runtime roots, API guards,
   docs, examples, and the fresh literal-plus-structured consumer union. Preserve all foreign dirty
   blobs through coordinator transfer rather than regenerating them.

## Validation And Performance Boundary

No Cargo or profiling command was launched for this preflight. The previously routed
`zr_rhi_wgpu` errors still block the Runtime product gate, so repeating the same product build would
not provide new Resource evidence. After ownership convergence, the parent plan's Resource-focused
build/tests, Runtime/App/Editor product gates, public-API/rustdoc seal, hard-cut guards, and plugin
workspace gate remain required.

This record makes no compile-time, runtime latency, throughput, allocation, I/O, energy, bottleneck
removal, parity, or optimality claim. Algorithm changes remain prohibited until a coherent product
build can produce same-fingerprint WPR/ETW, I/O, memory, and available power baselines as already
defined by the durable-transaction review.

## UI12 IBL `atomic_write` facade re-audit

UI12 reported three E0432 imports against `crate::core::resource::io::atomic_write`, but the current
HEAD source does not contain that three-file shape:

- `core/resource/mod.rs` declares `pub mod io`, and `core/resource/io/mod.rs` explicitly publishes
  `pub use atomic_file::atomic_write`; the facade file SHA-256 is
  `8977312a56e9a4228c0534092c8e91882a56b449553596590a686d82b0d3bed8`.
- `ibl_bake_artifact_asset_derived.rs` and `ibl_bake_artifact_cache.rs` still consume this curated
  facade, with current hashes `a6b804ce5da2b69b4376d20c51633f5239ce859df4e5cde359db57c1300955cf`
  and `09a8bae2c523c6e5c9cce8591a49d48a97b796228078517c7ea4762f08420b18`.
- `ibl_source_cubemap_staging.rs` no longer imports `atomic_write`; current hash
  `257c0499e5a8a81e42dbb6204463271719aacc56aa0ea2851533b637b48a3a77` consumes the private durable
  transaction surface for multi-file commit/recovery instead.

The correct hard-cut decision is therefore to preserve `core::resource::io::atomic_write` as the
curated single-file durability entry. Migrating the two remaining consumers or hiding the export
would break the approved Runtime facade. Frameworks01 does not edit the three Shader/IBL files; the
reported UI12 compiler fingerprint must be discarded and rebuilt from current source. This is
source-shape evidence only because the unchanged foreign WGPU diagnostics errors still prevent a
fresh Runtime/Editor product GREEN.

## 2026-08-26 current-source rotation gate

The former epoch-420 ownership blockers are no longer executable. Runtime51, Runtime25, Runtime24,
and Frameworks01 r8 are archived, and a fresh epoch-444 ownership matrix over
`zircon_runtime/src/core/resource` reports no active foreign owner or live foreign lease. This
changes admission state only; it does not retroactively make any stale attribution current.

The implementation tree has also grown beyond the older 57-file baseline. A fresh sorted
path/bytes/lines/file-hash manifest over the current working tree records 68 Rust files, 12,666
lines, 431,914 bytes, and tree SHA-256
`6d68fe5854945b7c4a5b9233a3206208dd395ae5bdf1e65cc80077684819cae2`. The stable Interface DTO
tree remains a separate non-migrating owner with 14 Rust files, 989 lines, 28,441 bytes, and tree
SHA-256 `65c452d8742f6ba4cf4dea0a1174a8026c61cfc8d64227d9a0c50326b68f7dbf`.
The added Resource files are the folder-backed durable journal/recovery and focused optimization
test owners; the old 57-file manifest must not be used for movement or deletion.

The consumer inventory was rebuilt from 18,542 tracked plus nonignored untracked Rust candidates,
excluding `dev`, `target`, `.codex`, and the implementation tree itself. The repository Rust lexer
and use-tree parser were combined with a code-view literal matcher so grouped imports such as
`use crate::core::{resource::{...}}` cannot escape the migration manifest. The current union is 502
files: 495 literal consumers, 477 structured consumers, 7 structured-only consumers, and 25
literal-only consumers. Top-level distribution is Runtime 410, plugins 61, Editor 29, and App 2.
The sorted path/bytes/file-hash/matcher manifest was verified twice without drift; both passes have
SHA-256 `8399d652a8ba4660c8dd7096bd8ed9baa06e010fb81805ba9012fd8c756cee72`.

The seven structured-only paths cover TextureImporter, two Graphics render-framework/scene paths,
and four Scene dynamic-reload paths. They are required atomic inputs and prove that a literal-only
scope would still be incomplete. The current structured inventory contains 84 Resource leaf paths.
The dominant surface is stable identity/handle/record/marker/locator DTOs; Runtime-only assembly
remains limited to transaction/fault/staging entries, `ResourceRegistryStaging`,
`ResourceReadinessRow`, and `approximate_event_bytes`. No newly introduced upper-domain dependency
was found in the Resource implementation tree, so the locked `zr_resource` plus hidden `assembly`
shape remains valid.

Intersecting the 502-file union with current coordinator dirty entries yields 62 dirty consumer
blobs: 7 already attributed to the current Frameworks01 session, 27 attributed only to
stale/archived sessions, and 28 missing attribution. Executable foreign owners are 0. The pre/post
consumer evidence is stored outside C drive at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/consumer-union.json` (SHA-256
`25ccc526872dff777678d1d216ff820d05a377efe75a5e23acc17819057364f6`) and the ownership
intersection at the adjacent `ownership-intersection.json` (SHA-256
`be6885c0c237e43cd9e413f276198b097daccd1d172281fc10d2760bdb9c639d`).

Admission is therefore `scope_rotation_ready_current_hash`, not `source_implemented`: the next
Frameworks01 successor must atomically inherit the current r12 owned changes, register the complete
502-consumer union plus both Resource/Interface trees, manifests, roots, guards and docs, then redo
ownership preview/apply against the same hashes before editing. Any hash drift or newly executable
foreign owner returns this record to preflight. No partial crate, forwarding module, compatibility
alias, consumer subset, Cargo, performance, power, or milestone acceptance claim is made here.

## 2026-08-26 `b41b0c0b` scope-rotation refresh

The preceding 502-consumer inventory is retained as historical evidence but is no longer the
rotation input. Shared main advanced to `b41b0c0b9da31eb4d19e3f086d6027f745f11a38`, and a corrected
full rescan matched both internal `crate::core::resource` and external
`zircon_runtime::core::resource` paths through the code-view literal matcher and Rust use-tree
parser. The first corrected pass was rejected because new nonignored Rust candidates appeared
during the scan. The accepted second pass records:

- 18,554 tracked plus nonignored untracked Rust candidates;
- 503 consumers: 496 literal, 478 structured, 7 structured-only, and 25 literal-only;
- Runtime 411, plugins 61, Editor 29, and App 2 consumers;
- one added consumer, `zircon_runtime/src/scene/dynamic_scene/session/io/atomic.rs`, and no removed
  consumer relative to the 502-file inventory;
- stable HEAD, stable candidate set, zero post-scan result hash drift, and zero classification
  mismatch among unchanged files from the earlier inventory;
- canonical consumer manifest SHA-256
  `5058770014b27b59543ba50d564c1a84ef41353335776c3d7703a88f5c70454a`.

The accepted report is stored outside C drive at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/consumer-union-b41b0c0b-r2.json`
(file SHA-256 `74e61b60ba534c95af9808428979c6cd40a573ff52a13fb3b6a1cbd25ea305fd`).
The Resource and Interface aggregate shapes remain 68 files / 12,666 lines / 431,914 bytes and 14
files / 989 lines / 28,441 bytes respectively; this refresh does not substitute a differently
canonicalized tree digest for the earlier tree evidence.

The current ownership intersection is materially less converged than the earlier 62-dirty snapshot.
Of the 503 consumers, 200 are now Git-dirty: 18 are attributed to Frameworks01 r12, 104 to
non-executable sessions, 67 have no attribution, and 11 retain executable foreign attribution.
Those 11 paths are owned in coordinator state by MVP00 (eight), Runtime25 (two), and Runtime11
(one). All 11 attribution content hashes are stale relative to current bytes, but executable source
status still correctly blocks ownership transfer. There is no live lease overlapping any of the 503
consumers, so this is an attribution/Session-lifecycle blocker rather than an active file-lock
conflict. The exact intersection is stored at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/ownership-intersection-b41b0c0b.json`
(file SHA-256 `d2af0a2a89eb542a2c46311b723232785134933ef2d60cd48478961492f5de71`).

Admission is now `scope_rotation_manifest_stable_b41b0c0b` plus
`scope_rotation_blocked_by_11_executable_foreign_attributions`. Frameworks01 r12 must remain
executable until those source Sessions terminalize or explicitly return exact current hashes; only
then may a successor register the 503-consumer atomic scope and transfer the current dirty set. This
record still makes no production implementation, Cargo, performance, power, or milestone acceptance
claim, and it continues to prohibit a partial crate or compatibility projection.

## 2026-08-26 deterministic consumer-manifest tooling performance plan

Shared main advanced again to `601472078e848164d2221967c55a77fea2452928`, so the preceding
`b41b0c0b` report remains historical admission evidence rather than an implementation fingerprint.
Frameworks01 now owns a RED-to-GREEN deterministic scanner at
`tools/frameworks_01_resource_consumer_manifest.py`. The scanner enumerates Git tracked plus
nonignored untracked Rust current source, excludes deleted tracked paths and the Resource owner
tree, unions code-view literal matches with structured Rust use-tree matches, fingerprints every
candidate, and refuses output unless HEAD, candidate membership, and every candidate byte hash are
stable across the scan. Its focused fixture tests use a D/E/F temporary root on Windows and cover
tracked, untracked, ignored, deleted, owner-tree, literal-only, structured-only, and all three drift
classes.

The first full unprofiled run correctly rejected six newly added UI candidates after 97.5 seconds.
Per the optimization gate, no parser change followed that observation. A subsequent `cProfile`
run completed on the stable HEAD and established this pre-optimization baseline:

- 18,211 current-source Rust candidates and 503 consumers;
- 496 literal, 478 structured, 25 literal-only, 7 structured-only, and 471 matched by both methods;
- Runtime 411, plugins 61, Editor 29, and App 2 consumers;
- candidate manifest SHA-256
  `0fd544e34f90ca932d6b6a5d36342ccb7ed44a66ea71b3cabe2017391fb83430`;
- consumer manifest SHA-256
  `335b59fc1e19e7ae94f9426a96e6725f01103883a2d425496428208e712fccca`;
- 129.746 profiled seconds / 15,050,811 calls (134.3 seconds observed wall time).

The profile identifies a whole-module design cost rather than a use-tree hot-loop defect.
`_rust_code_view` runs for all 18,211 candidates and consumes 58.559 cumulative / 50.798 self
seconds. `_rust_use_paths` consumes 12.115 cumulative seconds. The required two-pass content seal
opens 36,422 files and consumes 23.252 cumulative seconds, while the two Git candidate inventories
consume 17.208 seconds. A same-HEAD raw-byte inventory found only 1,145 candidates (6.29%) containing
all necessary case-sensitive tokens `resource`, `core`, and either `crate` or `zircon_runtime`; the
single full read took 13.963 seconds and found no missing file.

The planned optimization is therefore a semantics-preserving two-stage scan, not a lexer rewrite:

1. Continue reading and SHA-256 fingerprinting every candidate in the first pass and re-reading
   every candidate in the post-scan content seal.
2. Before constructing a Unicode code view, require the raw bytes to contain `resource`, `core`,
   and one approved root token. A valid current matcher result cannot omit any of these tokens;
   comments and literals may produce harmless false positives but never false negatives.
3. Run the unchanged shared `_rust_code_view` and `_rust_use_paths` only for that bounded superset.
   Do not add an alias resolver, literal-only fallback, cache, timestamp, or compatibility path.
4. Preserve the exact deterministic report schema and the two manifest hashes above on the same
   HEAD. Add focused false-positive/structured-only/raw-identifier regressions before GREEN.

Post-optimization acceptance requires: exact 503-consumer classification and manifest SHA on this
HEAD; at most 1,200 code-view/use-tree calls for this source shape; profiled total time no more than
65 seconds; and a stable three-run unprofiled median no more than 45 seconds. Candidate and content
seals remain mandatory even if they dominate the remaining time. This is tooling performance
evidence only: it makes no engine runtime, allocation, I/O throughput, power, compile, Cargo, or M1
acceptance claim. Evidence is stored outside C drive at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/consumer-union-current-r3-profiled.json`
(file SHA-256 `20244204da623ecde23679fc49042e5e6a6e47b810482ba06c477082e29bc97d`)
and adjacent `resource-consumer-manifest-r3-unstable.pstats` (SHA-256
`21c6e91b94f5e84f7f7efaf0924f532458811e169f10219be22099b5f7f2faed`).

### Post-prefilter profile and bounded I/O continuation

The necessary-token stage is semantically GREEN. On the same HEAD it preserved the exact
503-consumer SHA while reducing shared lexer and use-tree calls from 18,211 to 1,145. One stable
unprofiled run covered 18,219 candidates in 54.020 seconds; the consumer SHA remained
`335b59fc1e19e7ae94f9426a96e6725f01103883a2d425496428208e712fccca` and the expanded candidate
manifest SHA was `481a54043289f990d48ba943a0322eabf2d3d26554e3c9259296f26f48e5f1af`.
The report is `consumer-union-current-r4-run1.json` (file SHA-256
`c0310fc7cff55d14077282afdd0affc04033bf22437b541f4de09c251b7b29d0`). A second run was correctly
rejected after 59.432 seconds because three foreign files changed during the content seal, so it is
not a performance sample.

The post-prefilter `cProfile` run was also rejected after one foreign Editor test changed, but its
complete call profile is admissible for bottleneck location: 60.592 seconds / 7,168,043 calls,
with 1,145 `_rust_code_view` calls consuming 9.892 cumulative seconds and 1,145 use-tree calls
consuming 1.745 seconds. The remaining dominant costs are 36,440 serial `read_bytes` calls (27.232
cumulative seconds), the post-scan content seal alone (14.877 seconds), and two Git inventories
(15.987 seconds). The profile is `resource-consumer-manifest-r4-post-prefilter.pstats` (SHA-256
`ad66cf7fd7e082a9ca8b5b0a7b0c41d89cd4e08841776a5929cc9d31c16ccdff`). This proves that further
lexer changes would optimize the wrong layer.

Before the second implementation change, the I/O design is locked as follows:

1. Add one scanner-private ordered read pipeline with exactly eight workers and at most sixteen
   in-flight files. Do not use executor defaults or submit the complete 18k set at once.
2. The first pass may overlap file reads, SHA-256, and UTF-8 decoding, but the main thread must
   consume results in canonical path order and run literal/use-tree classification in that order.
   Memory is therefore bounded by sixteen source files plus the existing fingerprints/consumer
   manifest, not the entire repository source tree.
3. The post-scan seal uses the same bounded worker count to re-read every candidate and compare
   exact byte length plus SHA-256. It may not replace hashes with size/mtime, Git dirty state, or a
   subset recheck; an unrelated file can become a Resource consumer without changing membership.
4. Missing/read/UTF-8 errors retain their current typed classification, and ordered delivery keeps
   changed-path diagnostics and deterministic output unchanged.

Focused RED must prove more than one worker participates while preserving candidate order, and the
existing deleted/missing/content/candidate/HEAD drift fixtures must remain GREEN. The original
post-optimization thresholds remain unchanged: same consumer SHA, no more than 1,200 parser calls,
profiled total no more than 65 seconds, and three stable unprofiled runs with median no more than 45
seconds. No engine runtime or power inference is allowed from this tooling-only concurrency.

### Consumer-manifest tooling acceptance result

The two-stage scanner and bounded ordered I/O pipeline are source-complete and meet their tooling
gates. The production owner is 352 lines and the focused test owner is 263 lines, both below the
repository review budget. Final non-Cargo verification is:

- manifest fixture suite `10/10` GREEN, including tracked/untracked/ignored/deleted source,
  literal-only and structured-only raw identifiers, owner exclusion, deterministic output,
  bounded parallel reads, and HEAD/candidate/content drift rejection;
- shared `runtime_domain_dependency_audit` parser regression `11/11` GREEN;
- AST parse `2/2` GREEN and scoped `git diff --check` GREEN;
- three stable unprofiled full-repository runs at 37.331, 38.308, and 38.798 seconds, median 38.308
  seconds, against the 45-second gate;
- one stable profiled run at 35.463 seconds / 9,529,810 calls, against the 65-second gate;
- exactly 1,145 code-view and use-tree calls, against the 1,200-call gate.

The stable profiled result covers 18,228 current-source Rust candidates and the unchanged 503
consumer union: 496 literal, 478 structured, 25 literal-only, 7 structured-only, and 471 matching
both methods; Runtime 411, plugins 61, Editor 29, and App 2. Its candidate manifest SHA-256 is
`9c92236283473d3dfa110a7bef9739ace35688604c3bea4a637888fa12e352bf`; the consumer manifest SHA-256
remains exactly
`335b59fc1e19e7ae94f9426a96e6725f01103883a2d425496428208e712fccca` across the pre-optimization,
three unprofiled post-optimization, and stable profiled reports. The current report is stored at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/consumer-union-current-r5-profiled-stable-r2.json`
(file SHA-256 `67ae33ed664229d4d023fe92e46fc9db3bea70a4b984b2c55ba58abb49495744`),
with adjacent `resource-consumer-manifest-r5-post-bounded-io-stable-r2.pstats` (SHA-256
`27dcdd322633d6269dbfbf725b9098b9ea835b48b3bedb34cf79e2ca4a73f9d8`).

Relative to the 129.746-second profiled baseline, the stable 35.463-second post profile is 72.7%
lower. Relative to the 97.5-second first unprofiled attempt, the 38.308-second stable median is
60.7% lower. These are current-source manifest-tool wall/profile results, not engine frame time,
compile speed, allocation, throughput, energy, or reference-engine parity results.

The physical `zr_resource` cut remains blocked. Coordinator recheck after this tooling result shows
`mvp00-current-source-convergence-r2-01a00797-20260818` still `active`,
`root-runtime25-single-pass-asset-uri-20260826` still `registered`, and
`runtime11-bounded-stream-root-20260826` still `resolving_failure`; their eight/two/one consumer
attributions remain executable. The three child-plan failure records are imported and visible as
`open`. Frameworks01 therefore does not rotate scope, edit those 11 blobs, start Cargo, claim the
Resource crate guard GREEN, request a milestone commit, or send a WeCom completion message. The
next production action remains exact-hash ownership convergence followed by one atomic 503-consumer
hard cut without compatibility paths.

### Atomic hard-cut input composer and pre-optimization plan

The next blocker-reduction slice adds a separate atomic-input composer rather than expanding the
consumer scanner into another mixed owner. Its focused fixture is `5/5` GREEN and locks the required
roles: fixed workspace/manifests/roots, Resource implementation tree, Interface Resource DTO tree,
Rust consumer union, raw docs/tools/examples references, and approved missing future crate roots.
The composer rejects ignored/deleted inputs, future-path collisions, supplemental candidate drift,
and supplemental content drift; it does not create `zr_resource`, infer source-to-destination moves,
or weaken the 503-consumer parser contract.

Three real current-source attempts were correctly rejected after 43.931, 37.451, and 60.657
seconds by Text, Graphics, and Shader06 foreign changes. Two attempted full profiles also stopped
inside the initial consumer scan due HEAD/Rust-content drift, so they are not used to justify an
optimization. A separate production-shaped supplemental profile completed the full before/after
membership and SHA path on a stable source window:

- 6,713 supplemental candidates and 112 raw textual references;
- stable candidate set and zero content drift;
- 9.423 profiled seconds / 3,572,063 calls;
- two supplemental membership inventories at 4.852 cumulative seconds;
- initial supplemental read at 2.206 seconds and exact second SHA seal at 2.365 seconds.

The profile is stored outside C drive at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/resource-hard-cut-supplemental-r1.pstats`
(SHA-256 `68fede8286d06a04bcc40d5eb6589ba66b78343cf68d9780f525f88afa4421f6`).
Together with the already accepted 35.463-second consumer profile, this identifies the structural
problem: the composer calls a consumer builder that already performs initial capture plus final
seal, scans supplemental inputs, and then performs a third full Rust candidate/hash pass through
consumer revalidation. The extra Rust pass extends the shared mutation window without adding a new
class of evidence.

Before implementation, the correction is locked as follows:

1. Split consumer snapshot construction into an internal first-pass capture and one finalize/seal.
   The existing public consumer builder remains capture plus finalize and retains its report schema,
   exact hashes, CLI, and two-pass guarantees.
2. The hard-cut composer performs consumer capture, then supplemental capture, then exactly one
   consumer finalize and one supplemental finalize. Thus every Rust and supplemental candidate has
   a first fingerprint and an exact post-capture fingerprint, while Rust is no longer read a third
   time.
3. Finalize continues to compare HEAD, filtered Git candidate membership, byte length, and SHA-256.
   No size/mtime shortcut, dirty-state proxy, watcher, cache, lock, or partial subset is admitted.
4. RED must prove the public consumer builder still calls the Rust inventory exactly twice and the
   composer also calls it exactly twice, while all existing drift fixtures remain GREEN.

Acceptance requires a stable real atomic report, unchanged 503-consumer SHA, `11/11` consumer and
expanded composer fixture GREEN, and no more than 50 seconds for a stable unprofiled real compose on
this machine. This is preflight tooling only and carries no engine runtime, power, frame-time,
compile-speed, or reference-engine parity claim.

### Atomic hard-cut input composer implementation result

The capture/finalize split is source-complete. The public consumer builder still performs exactly
two Rust inventories, and the hard-cut composer now also performs exactly two rather than the prior
three. Both paths retain exact candidate membership, byte-length, SHA-256, and HEAD validation; the
composer additionally seals supplemental membership/content and checks the three future crate paths
at both ends of the operation. It does not create the crate or mutate any consumer.

Focused verification is `13/13` consumer-manifest fixtures plus `7/7` composer fixtures GREEN. The
new RED-to-GREEN cases prove that capture/finalize preserves the public report and that a complete
compose uses exactly two Rust inventories. A second review added a final HEAD fence after each long
content seal: one fixture changes HEAD during the consumer content seal and one commits after the
composer's supplemental content seal; both now reject rather than returning a stale-HEAD report.
Existing tracked/untracked/ignored/deleted, owner-tree, determinism, collision, and
HEAD/candidate/content-drift cases remain GREEN. The production owners are 513 and 412 lines
respectively, below the repository's large-owner threshold.

The first post-change real compose correctly rejected a concurrent Editor test write. Before the
final-HEAD review correction, a stable run completed in 44.419 wall seconds, below the locked
50-second gate, on HEAD
`a71cebf35c0be232ce734e483636d6c31c664ad0`. Its atomic report contains:

- 707 unique inputs with manifest SHA-256
  `cb75da65acd2aafa0c32bd204ac4c298a75805d9af6c7f031a1b60974c2e9d2e`;
- 504 Rust consumers with manifest SHA-256
  `19020b57cfb46496d0400abe473802498dd7364d2ae8b74dc3718052d3602540`;
- 6,724 supplemental candidates with manifest SHA-256
  `c10230703897d545cb4b093917eba040669466004aa68e31edd06eb3f456009a`;
- 12 fixed workspace inputs, 68 Resource implementation files, 14 Interface Resource DTO files,
  504 Rust consumers, and 112 raw textual references; overlapping roles are intentionally merged;
- all four stability flags true and all three future `zr_resource` paths absent.

The previous 503-consumer acceptance hash was tied to the earlier HEAD, not a timeless repository
constant. The current source adds exactly one real consumer,
`zircon_runtime/src/graphics/scene/render_scene/component_projector/projection.rs`, whose
`source_level` signature now names Resource handles directly; no prior consumer was removed. Four
shared consumer files also changed content. The new 504 count is therefore a reviewed source-shape
change, while the fixture-locked parser behavior and two-pass seal remain unchanged.

That historical stable report is stored outside C drive at
`F:/zircon-profiles/frameworks01-zr-resource-preflight-20260826/resource-hard-cut-inputs-current-r2.json`
(file SHA-256 `9752c8ef78792a2c2e3b0362d9192331551e34656cb04efe36b65f4518ec3edb`).
It remains valid evidence for the two-inventory performance direction, but it predates the final
HEAD fence and HEAD has since advanced to `d4ca9a802ecd19976c653caa58614af0c2fb15f7`; it is therefore
not the terminal current-source acceptance report. Two full profile attempts were rejected by exact
source-content drift in Text and Interface UI files. Three final-HEAD-fence real runs then rejected
Tooling content drift, two newly added Rust candidates, and Runtime UI content drift after 53.765,
46.989, and 53.557 wall seconds. None is an acceptance sample and no partial output is used.

The pre-optimization supplemental profile, the accepted consumer-scanner profile, and the historical
44.419-second end-to-end sample support the structural change. A stable current-HEAD compose and a
stable full post profile remain pending and must not be represented as engine-runtime or power
evidence. The source implementation and all 20 focused fixtures are complete, but this tooling slice
is not marked accepted until that current-source artifact exists.

The physical `zr_resource` hard cut and M1 remain blocked by the same 11 executable foreign consumer
attributions. Fresh transfer preview fingerprint
`054fdfb7dda4905cf5762468cfa195b1a8d712658b8972b87d97351153f8c3db` at baseline epoch 449
reports `source_owner_executable` for all 11 paths: MVP00 remains `active`, Runtime25 remains
`registered`, and Runtime11 remains `resolving_failure`. No transfer apply, Cargo validation,
milestone commit, coordinator milestone acceptance, or WeCom completion notification is authorized.

### Resource owner move-manifest architecture review

The physical cut remains attribution-blocked, but its owner-tree transformation can be made
deterministic before that dependency clears. Review of the accepted `zr_math` cut confirms the local
target shape: one low-dependency implementation crate is canonical, while Runtime retains an
explicit curated product projection. Local Unreal `AssetRegistry` and `CoreUObject` keep public,
private, and internal module surfaces distinct and declare dependencies at the module boundary;
Bevy and Fyrox likewise place resource implementation in a dedicated crate rather than duplicating
it in product shells. Zircon must preserve that ownership model without copying their APIs or
retaining a compatibility module.

Current HEAD `d4ca9a802ecd19976c653caa58614af0c2fb15f7` contains 68 Rust files under
`zircon_runtime/src/core/resource`. The hard-cut partition is locked as follows:

1. Runtime retains exactly `mod.rs` and `io/mod.rs`, but both become generated curated projections;
   they are not byte-moved and may not re-export `zr_resource::*` or `assembly`.
2. `management_generation/tests/hard_cut.rs` and its parser-only `support.rs` move to the Runtime
   absorption `resource_foundation` guard owner. They inspect Runtime source topology and therefore
   must not make the internal crate depend on Runtime as a dev dependency.
3. The remaining 64 owner files move under `zircon_runtime/crates/zr_resource/src` with their
   relative layout preserved. Files whose Rust code view contains `crate::core::resource` require an
   explicit crate-root rewrite; raw strings/comments do not trigger that classification. The moved
   `management_generation/tests/mod.rs` also requires its module set to drop the two Runtime guards.
4. `zr_resource/Cargo.toml`, `src/lib.rs`, `src/assembly.rs`, and `src/io/mod.rs` are generated final
   surfaces, not copied facades. Workspace/Runtime manifests, `Cargo.lock`, and Runtime absorption
   module wiring are explicit patch inputs. No destination may already exist or receive two sources.
5. The hidden assembly surface carries only Runtime-internal symbols approved by the parent plan;
   product crates continue through `zircon_runtime::core::resource`, and the move manifest does not
   invent aliases, forwarding modules, glob projections, or automatic grouped-use rewriting.

Implementation will be a pure deterministic composer over one already sealed hard-cut input report.
It must verify the report schema/stability flags, exact on-disk owner hashes, complete 68-file
partition, unique destinations, approved retained/relocated paths, and absent generated destinations.
It then emits operation counts and a canonical SHA-256. This tool will not create `zr_resource`,
modify a consumer, invoke Cargo, or treat the historical `a71cebf3` report as current acceptance.
Focused RED/GREEN fixtures must cover owner hash drift, missing/extra owner input, destination
collision, raw-string false positives, crate-root rewrite classification, Runtime guard relocation,
deterministic output, and rejection of an unstable or wrong-schema source report.

### Resource owner move-manifest implementation result

The pure move-manifest composer is source-complete at 486 production lines with a 384-line focused
fixture owner, both below the repository large-owner threshold. Future ownership for the two new
tool paths was transferred to this Session with fingerprint
`6062f79794d733fbcc5bab38cd2229fc3971695adf76e0a6640a496da9568511` and apply request
`27e7656bc0a74027baaa61cf35a453ad`; no production Resource path was transferred or edited.

The fixture suite is `10/10` GREEN. It proves complete mutually exclusive owner partitioning,
code-view-only crate-root rewrite classification, Runtime guard relocation, deterministic canonical
output, unique/absent destinations, exact source report schema and boolean stability fields, atomic
input manifest integrity, required fixed-workspace roles, owner membership/content seals, HEAD
drift rejection, and rejection of higher-layer Runtime dependencies in files destined for the
internal crate. The implementation performs a second owner membership/content pass and a terminal
HEAD check; it never creates a destination or rewrites source.

A direct current-source code-view audit of the intended production partition finds exactly 64 files
destined for `zr_resource`: 36 require explicit `crate::core::resource` root rewriting and 28 are
verbatim or the one approved module-set rewrite. Actual non-Resource `crate::...` and
`zircon_runtime::...` dependencies in those 64 files are zero. The two Runtime facades and two
Runtime absorption guard files are excluded from that internal-crate dependency count by design.

No real move report is accepted yet because its source must be a stable current-HEAD atomic-input
report. Two post-implementation source-seal attempts correctly rejected newly added
`text/shaping/failure_receipt.rs` and `tools/mvp/resource-management-workload-registry.json`; neither
produced partial output. The historical `a71cebf3` report is deliberately rejected by the move tool's
HEAD fence. Therefore this slice is implementation-complete but current-source artifact pending,
and it does not authorize the physical hard cut, Cargo, milestone commit, or WeCom notification.

### Resource hard-cut patch composition architecture review

The move manifest deliberately stops before content generation. A full current-source review of the
68-file Resource owner and its Runtime consumers found that applying those move operations literally
would not compile and would weaken the approved API boundary. The important issue is not a path-copy
detail: `PreparedResourceMutation`, registry staging, readiness rows, atomic staging, and durable
transaction types are currently `pub(crate)` because Resource and their consumers live in one crate.
Blindly changing every such declaration or inherent method to `pub` would make Runtime-only commit,
fault-injection, recovery, and projection machinery visible through the product `ResourceManager` and
`ResourceReadinessGeneration` types. That would violate the parent plan's hidden-assembly contract and
the priority structure convention's curated-facade rule.

The reviewed split follows local `zr_math` and Unreal's Public/Private/Internal module separation:

1. `zr_resource::lib` owns the existing product Resource surface and explicitly exposes
   `#[doc(hidden)] pub mod assembly`; Runtime `core::resource` explicitly re-exports only the product
   set and never re-exports `assembly` or a glob.
2. Inherent product methods that are currently crate-private stay non-product. Hidden assembly free
   functions bridge `ResourceManager::prepare_commit`, registry staging construction, readiness-row
   access, and prepared commit application. The six actual Runtime callers migrate in the same patch;
   no method alias, wrapper at the old owner, or compatibility path remains.
3. Assembly-only value types become externally nameable only through the hidden module. Only the
   methods/fields required by sibling Runtime code gain Rust `pub` visibility; the product facade does
   not project them. Runtime `core::resource::io` keeps the public atomic-write pair and Resource I/O
   contract, while fault/staging/durable transaction entries remain `pub(crate)` projections.
4. `zr_resource` features preserve conditional behavior: Runtime profiling maps to
   `zr_resource/profiling`, and Runtime tests enable `zr_resource/test-support` so transaction fault
   variants do not ship in ordinary production builds. The new crate depends only on `blake3`,
   `serde`, `thiserror`, `toml`, and the frozen Interface Resource DTO crate.
5. The relocated Resource topology guard is rewritten for the new canonical owner root and include
   paths. It must reject `zircon_runtime::` and stale `crate::core::resource` backflow from the moved
   crate rather than preserving assumptions about the deleted Runtime implementation directory.

Implementation is a deterministic patch composer over a stable atomic-input report plus its exact
move manifest. It will revalidate both schemas and hashes, rebuild the move manifest from current
bytes, seal every atomic input, transform Rust code-view spans without touching comments or strings,
require exact declaration/caller replacement counts, and emit a reviewable unified patch plus a
content-hash report. It never applies the patch to the shared checkout. Fixture acceptance must prove
deterministic output, `git apply --check` and apply in an isolated E/F-drive repository, curated public
versus assembly surfaces, comment/string preservation, exact visibility promotion, feature/manifest
wiring, relocated guard paths, stale/tampered report rejection, and fail-closed behavior for any
unexpected source shape. A real patch remains unauthorized until the 11 executable foreign consumer
attributions transfer and a current stable source/move report exists.

### Resource hard-cut patch composer implementation result

The deterministic composer is source-complete, but this is not an accepted M1 result. Its fixed
contract is split between `tools/frameworks_01_resource_hard_cut_spec.py` and the composer so the
production owner remains below the large-file threshold. The focused suite is `9/9` GREEN and
proves deterministic output, isolated `git apply --check` plus apply, exact consumer-role and source
shape rejection, CRLF fail-closed behavior, code-view-only path rewriting, hidden assembly versus
product-facade separation, move-report integrity, and a terminal full-source stability fence.
The final combined manifest/move/patch regression is `39/39` GREEN in 112.892 seconds. A real
in-memory transform of 64 moved owners, two relocated guards, five generated surfaces, and six
Runtime consumers produced 77 Rust outputs with `rustfmt --check` failures `0`. The spec/composer/test
owners are 343/599/666 lines, all below the 800-line threshold.

The required second review found and corrected two compile-blocking design defects before any real
patch was emitted:

1. The first surface generator removed canonical `pub(crate) use` declarations from `zr_resource`
   `lib.rs` and `io/mod.rs`. Moved sibling modules rewrite `crate::core::resource::*` to `crate::*`
   and therefore still require those crate-internal root projections. They are now retained; only
   the Runtime product facade is curated, so no internal symbol becomes product-public.
2. The moved Resource source guard still required `ResourceRegistryStaging` to be declared
   `pub(crate)`, which is impossible across the new crate boundary. The transformed guard now
   requires the implementation type to be externally nameable, rejects its projection from product
   `lib.rs`, and requires the sole hidden `assembly.rs` projection.

The visibility closure was then re-audited against all real Runtime call sites. All 68 Resource owner
files and the six required staging/readiness/commit callers match exact transforms. The durable
transaction test-support promotion covers 47 non-module `cfg(test)` sites and leaves zero such sites
unmapped; true `mod tests` owners remain test-only. A direct two-crate `rustc` check on F drive proves
that the selected extension-trait bridge remains callable when the canonical inherent method stays
crate-private. The public `atomic_write`/`atomic_write_new` pair remains in the product I/O facade;
fault, staging, and durable transaction APIs remain Runtime `pub(crate)` projections through hidden
assembly.

The terminal stability fence now rebuilds the complete atomic source manifest after patch
composition, rather than checking only HEAD and already-known hashes. This deliberately detects new
untracked consumers or textual references created under the same HEAD. Two post-review real source
seals correctly rejected concurrent Graphics changes in
`render_scene/resource_dependencies.rs`, `scene/resources/mod.rs`, and the three
`render_asset_residency` files. Neither attempt wrote a partial report. Consequently no current move
report, sealed real patch, Cargo validation ticket, integration candidate, milestone commit, or
WeCom completion notification exists yet; the 11 foreign consumer attributions and a stable complete
source window remain the acceptance gates.

### 2026-08-27 hidden-assembly consumer closure repair

A fresh whole-module review found that the patch composer still encoded the initial six Runtime
assembly consumers rather than the complete current call surface. `asset/facade/assets.rs` and
`asset/facade/manager.rs` both call the crate-private
`ResourceReadinessGeneration::row`, but only `asset/facade/readiness.rs` received the hidden
`ResourceReadinessGenerationAssemblyExt` import. Applying the old patch would therefore leave two
deterministic E0599 errors after the physical crate cut. Making `row` product-public was rejected:
the row and its internal fields are assembly data and must remain absent from the curated Runtime
product facade.

The composer now patches eight approved consumers and fail-closes over three complete callsite
classes: readiness-row access, Resource registry staging, and prepared ResourceManager commit. Each
class scans Rust code views from the sealed atomic inputs and requires the actual path set to equal
the approved path set; method syntax, UFCS syntax, and legal Rust whitespace are all recognized. A
missing known path and an unexpected new path are both errors. The two new readiness consumers import
only the hidden extension trait. No Runtime consumer source, Resource implementation owner, facade
export, or future crate file was modified in the shared checkout.

TDD evidence is RED 8/10 with the two intended failures in 99.804 seconds, followed by focused patch
GREEN 10/10 in 113.943 seconds. A second self-review then changed the unknown-consumer fixture to
`ResourceReadinessGeneration :: row(...)`; the literal method scanner correctly reproduced RED 0/1
in 8.668 seconds. The regex closure repair is GREEN 1/1 in 8.038 seconds and the final complete patch
suite is GREEN 10/10 in 57.074 seconds. The upstream consumer/input/move chain is independently GREEN
30/30 in 179.148 seconds, for a final 40/40 focused fixture set. The final direct read-only scan of
8,264 current Runtime Rust files confirms all three method/UFCS assembly callsite sets exactly.
Untracked no-index
`git diff --check` is GREEN for the four changed code/plan files; the modified production/test owners
remain below the 800-line structure budget.

The same visibility-closure audit found a second deterministic compile blocker. The initial spec
made `ResourceReadinessRow::record` and `load_state` externally nameable through hidden assembly,
but `asset/facade/manager.rs` also reads `direct_dependency_state` and
`recursive_dependency_state`. Those two fields now receive the same assembly-only visibility
promotion. `dependency_revision`, `dependency_fingerprint`, and `payload_type_id` remain
crate-private because no external Runtime consumer uses them; the inherent generation `row` method
also remains crate-private. The focused field test is RED 0/1 in 3.924 seconds then GREEN 1/1 in
2.259 seconds, and the final complete patch suite remains GREEN 10/10 in 69.790 seconds.

The first real in-memory compose after the repair did not emit or write a patch. Its initial source
seal rejected eight concurrent content changes after 52.5 seconds:
`editor_host_event_controller.rs`, `play_hierarchy.rs`, `editor_world_sync.rs`,
`play_hierarchy_projection.rs`, `scene_inspection_publication.rs`,
`scene_hierarchy_refresh.rs`, retained-host `tick.rs`, and Runtime `linked_plugins.rs`. This is
correct fail-closed scheduling evidence, not an implementation RED. The immediate post-failure
routing check observed HEAD `b8f646ba4d369b51b163af53c5e3a3f392dc11c8`; later shared-branch
advancement does not turn that rejected compose into an acceptance artifact.

Coordinator re-audit keeps all 11 ownership blockers executable: MVP00 remains `active` for eight
paths, Runtime25 remains `registered` for two paths, and Runtime11 remains `resolving_failure` for
one path. The current state is therefore
`composer_source_repaired / 41_fixture_and_runtime_closure_green /
current_source_artifact_and_physical_hard_cut_blocked`. No Cargo, runtime profile, power claim,
milestone acceptance, commit, or WeCom completion notification is authorized by this slice.

The final-output formatting boundary was then checked separately from source formatting. All 77
current Rust inputs represented by the 68 Resource owners, eight assembly consumers, and the Runtime
absorption root were already `rustfmt +1.94.1` clean. Before correction, the hard-cut transforms
introduced formatting deltas in 19 of 80 final Rust outputs: shortened crate paths, promoted
visibility, newly injected imports, the two Runtime facades, and the relocated owner guard. The
three source-free generated crate surfaces were already clean. Leaving those deltas for a later
physical-cut owner would make the sealed patch fail its own formatting gate, so the composer now
formats all final Rust outputs in one pinned batch before computing patch bytes and hashes. Its
scratch directory is created on the invoking repository's parent drive and removed before return;
non-Rust outputs remain byte-identical.

The formatter fixture first reproduced RED 0/1 because the formatting stage did not exist, then is
GREEN 1/1 in 3.713 seconds. The complete patch-composer suite is GREEN 11/11 in 117.009 seconds,
including deterministic composition, isolated apply, assembly closure, and formatting. A direct
current-shape transform still produces 80 Rust outputs; the first pinned pass changes the expected
19 and a second pass changes zero, proving formatter idempotence over the actual source shape. With
the unchanged upstream consumer/input/move suites at 30/30, focused evidence is now 41/41. The
public `core::resource::io::{atomic_write, atomic_write_new}` projection remains intentional and the
current IBL artifact consumers are valid; their reported E0432 diagnostics came from an older
Runtime fingerprint. The facade characterization is GREEN 1/1 in 2.326 seconds and explicitly
rejects a `pub(crate)` downgrade; the final full patch suite remains GREEN 11/11 in 39.399 seconds.
No IBL source was changed.

A follow-up visibility and dependency audit scanned 8,492 non-Resource Runtime Rust files. The
external assembly surface reaches ten files naming `PreparedFileWrite`, nine naming
`AtomicWriteFault`, four naming `ResourceRegistryStaging`, four naming `DurableCommitReport`, and
the smaller durable recovery/journal type sets. Every called constructor, retire/commit operation,
report observation, journal target accessor, and staging mutation is covered by the existing exact
visibility promotions; no Runtime caller reaches the intentionally private prepared-write fields,
readiness dependency revision/fingerprint/type-id fields, or durable error constructors. The 47
transaction `cfg(test)` sites promoted to `test-support` use only the normal `blake3`, `serde`,
`thiserror`, and `toml` dependencies. `serde_json` remains confined to a true Resource test module,
so keeping it in `zr_resource` dev-dependencies does not create a feature-build dependency gap.

### 2026-08-27 content-addressed snapshot architecture and performance review

The current-source failures above exposed a scheduling defect rather than a Resource algorithm
defect: the first hard-cut manifest revision used the repository HEAD and every current Rust source
as one global stability fence. An unrelated commit or an unrelated untracked Rust file could
therefore invalidate a sealed Resource graph even when every Resource owner and consumer byte was
unchanged. This is incompatible with the coordinator's shared-current-source model and cannot be
fixed by waiting for a globally quiet checkout.

The reviewed v2 contract removes HEAD from the consumer, atomic-input, move, and patch reports. It
seals the exact semantic Resource candidate set and SHA-256 of each candidate, repeats candidate
discovery at finalize time, and performs a third terminal semantic revalidation after the
supplemental seal. Unrelated HEAD movement is accepted; a changed, removed, or newly introduced
Resource consumer is rejected by candidate-set or content drift. Move and patch composition still
rebuild their complete prerequisite reports from current bytes, so the change narrows the sealed
dependency graph without weakening it. Focused fixtures are GREEN 43/43. A real run rejected only
`graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/
candidate_publication.rs` after its bytes changed, rather than rejecting unrelated shared edits; the
next stable run completed with 538 consumers, 749 atomic inputs, consumer hash
`359bc3a61e4d53360abbc87afb3c56c1149396e0059b27fac49d2185d4b25b0b`, atomic-input hash
`d65624a2ccee9425b4ddc1eb0ff58af9e8ab45aedfa41ae1c1295a5ca8ca922b`, and 6,897
supplemental candidates in 72.796 seconds.

This dependency model follows the local UnrealBuildTool implementation rather than inventing a
repository-wide generation fence. `ActionGraph.cs` derives each action from its explicit
`PrerequisiteItems` and discovered dependency list; `ActionHistory.cs`, `ActionHistoryLayer.cs`, and
`CppDependencyCache.cs` persist producing-attribute or dependency hashes. None uses the source-tree
commit as a substitute for the action's actual inputs. The Zircon hard cut therefore treats semantic
Resource candidates and their bytes as the action prerequisites, with terminal rediscovery as the
race fence.

The required pre-optimization profile found that v2 is correct but not yet efficient enough. A
current-source in-memory `cProfile` run completed in 75.482 seconds with 17,430,296 calls, 539
consumers, 750 atomic inputs, and 6,899 supplemental candidates. The cumulative profile contains
69,198 `_bounded_ordered_map` submissions/results and 68.554 seconds of thread joins. A direct stage
sample found 18,593 current Rust files, 18,465 after owner/exclusion filtering, but only 1,189 with
the necessary raw Resource tokens; enumeration took 5.271 seconds and Python read/filter/hash took
5.373 seconds per pass. Supplemental discovery has the same structural overreach: only 120 current
tracked/untracked textual files contain a hard-cut reference token, while the current seal reads and
hashes 6,899 candidates.

The next implementation slice will preserve all three semantic inventory passes but push a safe
superset filter into Git for tracked files and scan only untracked candidate files in Python. Rust
tracked candidates require lowercase `resource`; the existing raw-token predicate and Rust
code-view parser remain authoritative. Textual tracked candidates require one of `core/resource`,
`core::resource`, `zircon_resource`, or `zr_resource`; Resource implementation owners, Interface
DTO owners, and fixed workspace inputs remain unconditional exact inputs. This reduces irrelevant
I/O and also removes unrelated documentation from the stability fence. Acceptance requires:

1. unrelated tracked/untracked Rust and textual changes remain accepted across a snapshot;
2. dirty tracked, new untracked, changed, and removed real Resource consumers/references still fail
   closed at the correct candidate/content gate;
3. all 43 existing v2 fixtures plus new prefilter fixtures pass, and stable real reports preserve the
   semantic consumer/input hashes for unchanged source;
4. three stable current-source runs have median wall time at most 50 seconds, the profiled run is at
   most 65 seconds, and the post-profile no longer attributes the dominant work to reading the full
   repository inventories.

This review authorizes only the manifest algorithm optimization. It does not authorize the physical
`zr_resource` move, Cargo execution, milestone acceptance, commit, or WeCom notification while the
11 foreign executable attributions and unmanaged target audit remain open.

The first optimized current-source series is correctness-stable but does not yet meet the wall-time
gate. All three runs at HEAD `6a6063f463714a498d2165b28fb4a06a0ac6182e` produced the same 539
consumers, 750 atomic inputs, 212 supplemental candidates, consumer hash
`958492cd44acb39609a3bb1c71f6c788fabf361d101e8f816af766510c77df64`, and atomic-input hash
`c4b94c1cf92d2ec799387e0c9e75e1841dc77016c991c6e59d1b43b2cb2b56b9`. Wall times were
49.730, 58.897, and 54.350 seconds, so the 54.350-second median correctly keeps the slice open.
The post-change `cProfile` run is below its 65-second gate at 55.819 seconds and reduces total calls
from 17,430,296 to 5,583,705, but still attributes 38.773 cumulative seconds to six semantic
candidate inventories and 31.899 cumulative seconds to 12 semantic plus six owner Git subprocesses.

The next measured reduction is structural and keeps the same report semantics. Tracked paths are
already selected by Git from current worktree bytes; reading them once again inside the prefilter and
then again in the authoritative parser is redundant. The prefilter will therefore trust only Git's
safe token superset for tracked paths, while continuing to read every untracked candidate because Git
grep does not include it. The authoritative Rust scan and supplemental hash pass still read and hash
all selected paths. Resource owner and Interface DTO roots will be enumerated directly from their
physical trees in one pass; this both removes six full-repository Git subprocesses from a build and
closes the old gap where an ignored but physically present owner file was absent from the hard-cut
input set. A direct comparison found the two separate Git inventories take 1.579 seconds per phase,
while one combined inventory takes 0.563 seconds and returns the identical current 82-file set.

That reduction is GREEN in all 49 focused fixtures and produced one stable 46.386-second ordinary
sample. A complete 58.747-second post-profile is below the 65-second gate with 5,336,493 calls, 540
consumers, 751 atomic inputs, and 212 supplemental candidates. The six owner-root Git subprocesses
are gone. However, the next same-shape ordinary series produced 68.431 and 54.408 seconds before a
third run rejected six live Graphics consumer changes; the two successful runs also had different
content hashes under the same HEAD because dirty consumer bytes changed between snapshots. There is
therefore not yet a valid three-run same-input median, and the remaining 12 Git subprocesses still
dominate the profile at 31.998 cumulative seconds.

Rust and textual discovery currently start separate tracked-grep and untracked-list pairs even
though every accepted token contains lowercase `resource`. One phase-level inventory can run one
tracked `git grep resource` and one untracked `git ls-files` concurrently, then classify Rust and
textual candidates independently. The Rust parser remains authoritative for its safe superset;
textual candidates still require the exact four-token set. Suffix-qualified pathspecs keep the
combined inventory bounded. A current-source prototype returns the same exact 120 textual paths as
the existing algorithm with zero missing/extra paths, while its two Git subprocesses complete in
8.225 seconds. The hard-cut builder will capture this combined inventory independently at initial,
finalize, and terminal phases, reducing 12 Git subprocesses to six without caching across a
stability boundary.

### Content-addressed snapshot v3 implementation result

The optimized source chain is implementation-complete. The hard-cut source report is schema 3 and
explicitly records `supplemental_terminal_snapshot`; move rejects the older stability shape. The
consumer report remains schema 2 because its semantic candidate/report contract did not change.
Each source phase now captures one `ResourceReferenceInventory`, which carries the Rust safe
superset and exact textual reference set from one concurrent tracked/untracked Git query pair.
Consumer parsing and supplemental hashing remain independent authoritative byte reads. The initial,
finalize, and terminal phases do not share a cache.

Two additional correctness repairs are part of this result. Resource implementation and Interface
DTO roots use direct physical-tree enumeration, so an ignored but present owner cannot escape the
move seal; a path resolving outside the repository is rejected. The final phase repeats future-crate
collision detection after both consumer and supplemental revalidation. Focused TDD covered the
algorithm in three RED/GREEN groups: unrelated HEAD versus real consumer drift, semantic textual
filtering plus terminal late-reference detection, and combined phase inventory plus ignored-owner
coverage. The final chain is GREEN 50/50:

- consumer manifest `16/16`;
- atomic source manifest `12/12`;
- move manifest `10/10`;
- patch composer `12/12`, including isolated apply and pinned `rustfmt +1.94.1`.

Final production/test owner sizes are 624/482 lines for consumer, 477/405 for source, 477/387 for
move, and 681/777 for patch. All remain below the 800-line structure threshold. Python compilation,
tracked/untracked whitespace checks, and stale-helper review are GREEN. There is no
`_git_semantic_candidates` or `_git_current_paths` remainder, and the combined suffix-qualified
inventory reproduces the prior 120 textual-reference paths exactly.

Three successful ordinary final-implementation samples completed in 47.362, 35.957, and 41.274
seconds; the median is 41.274 seconds, 24.1% below the preceding 54.350-second median and below the
50-second acceptance gate. Their current input counts were 542 consumers, 753 atomic inputs, and 212
supplemental candidates for the latest two-current-source shape; live shared edits changed hashes
between some individually stable snapshots, so hashes are compared only when input bytes are
unchanged. The two adjacent stable samples with unchanged input produced the same consumer and
atomic hashes. Intervening runs correctly rejected content/candidate drift in Graphics and Editor
consumers instead of emitting partial reports.

The complete final `cProfile` is GREEN in 53.361 seconds, below the 65-second gate. Relative to the
initial 75.482-second profile, elapsed time falls 29.3%. Relative to the previous 58.747-second
profile, it falls 9.2%. Git subprocess count falls from 12 to six and cumulative Git subprocess time
from 31.998 to 22.877 seconds, a 28.5% reduction. Supplemental candidates remain 212 instead of the
old 6,899 global textual files. The remaining dominant cost is the required three current-worktree
Git inventories and authoritative content reads; removing another phase would weaken drift
detection, so no further optimization is accepted in this slice.

This closes the manifest-algorithm performance defect, not Frameworks01 M1. The physical
`zr_resource` cut, managed Cargo validation, integration candidate, milestone commit, and WeCom
notification remain unauthorized while 11 executable foreign consumer attributions and the
unmanaged artifact audit are open. The public
`core::resource::io::{atomic_write, atomic_write_new}` projection remains required; no IBL or mixed
Editor owner was modified.

### 2026-08-27 durable journal folder-facade compile closure

A current-source Windows Runtime compile reached `zircon_runtime` after the lockfile dependency
graph was refreshed. Its structured fingerprint contained 587 errors, including 25 diagnostics in
`core/resource/io/transaction`: ten E0364/E0365 re-export failures at `journal/mod.rs`, their E0603
private-import cascades in transaction engine/commit/recovery, and one downstream E0282 inference
failure. This exposed a folder-split visibility defect, not a durable-I/O algorithm defect. Before
the split, the journal symbols were declared `pub(super)` directly in `transaction/journal.rs`, which
made them visible to the `transaction` owner. After the split, the same spelling in
`journal/{append,frame_codec,intent,recovery}.rs` reached only the immediate `journal` parent and
could not be re-exported one level further by the root facade.

Ownership transfer preview/apply fingerprint
`548b456b01a923bb09fdc744856f43ac5c369175edf4684746ee6986c42771c3` atomically assigned the six
previously unowned added journal files to Frameworks01 r12 without absorbing unrelated transaction
blobs. The repair keeps `journal/mod.rs` as the single internal facade and promotes only the ten
forwarded leaf declarations to `pub(in crate::core::resource::io::transaction)`. It does not use
`pub(crate)`, expose child modules, add wrappers, retain a compatibility alias, or change runtime
behavior. The resulting exact file hashes are attributed under coordinator request
`9c90a1b214e7466cab9983e68f374c82`; six-file `rustfmt --edition 2021 --check` is GREEN.

The post-fix RustRover current-source fingerprints at 07:09 contain zero Resource transaction
errors in both lib and test-lib; each is stopped only by the same two foreign Rust-2024 let-chain
diagnostics in Text/Graphics. This is useful compile diagnosis but is not managed acceptance. The
managed Windows `core-min` check job `2f7eed4398c540ea8ebb73779188205f` ran in the D-drive shared
target from 07:11:49 until its process tree exited at 07:21:06, but the coordinator recorded
`orphaned`, `exit_code=null`, and no new Runtime fingerprint. It therefore supplies no GREEN or RED
acceptance claim and must not be represented as a passing build.

The folder split is still not a committable standalone milestone. The current Session explicitly
claimed and attributed the tracked `transaction/journal.rs` and `transaction/recovery.rs` deletion
tombstones under request `80353a8b95d1446080d1665ad8166459`; the ownership matrix now reports
only its expected `deletion_requires_explicit_candidate` gate. Transfer preview fingerprint
`ff6eb8a8636613be155a807874843aa1c43042c627b2918910a4ca0f1b8bac08` and apply request
`0bb0f46fe82e42e495fb9fbbc28dfaea` moved the remaining 15 existing transaction blobs from
archived/unowned attribution into r12. The complete dirty transaction split therefore has one
executable owner, but both tombstones must still be listed in an explicit candidate; neither old
monolith will be restored. Physical `zr_resource`, managed product validation, integration
candidate, milestone commit, WeCom notification, performance, power, or optimality claims remain
pending.

A D-drive standalone Rust test assembly then isolated the complete current `atomic_file` plus
`transaction` subtree from unrelated Runtime modules. The first real execution produced
`40 passed / 1 failed`: recovery discovery returned the canonical Windows physical path
`\\?\C:\...`, while the pre-existing orphan test compared it with the caller-spelled `C:\...`
path. Source review confirmed that production must retain the canonical operation path to keep
alias/symlink admission and recovery I/O on one physical identity. Unreal's `FPaths::IsSamePath`
likewise normalizes full paths and compares case-insensitively on Microsoft platforms; it does not
justify degrading the operation path to caller spelling.

The test was hard-cut to compare against `canonicalize(root).join(file_name)` and its fixture root
now honors `ZIRCON_TEST_OUTPUT_ROOT`, then `CARGO_TARGET_DIR`, then the workspace `target` fallback.
Ownership preview fingerprint
`37a62f39e7d92b4340d8b6e6b8fd8550f7b3e2b7a2c0eb145436181f593644f1` and apply request
`8fcd39f9dd5b449697840306f4723db0` transferred the complete current recovery test blob from
archived r8 before editing. The rerun set `TEMP`, `TMP`, and `ZIRCON_TEST_OUTPUT_ROOT` to
`D:\zircon-frameworks01-r12-transaction-standalone` and passed all 41 tests in 1.92 seconds. It
covers create-only atomic publication, commit/rollback faults, owner locking, torn WAL tails,
bounded journals, recovery evidence, and Windows path identity. The executable SHA-256 is
`e9cd9e348762a42e33a8fbf047ceeb388ecb16525dc97f4c7cae7ff55f95555c`; the one C-drive directory
left by the intentional RED failure was removed exactly, and the matching Temp leftover count is
zero. This is focused behavior evidence, not a replacement for the missing managed workspace
receipt or permission to bypass the explicit deletion-candidate gate.

### 2026-08-27 current-source frozen-snapshot patch composition

A fresh schema-3 hard-cut source scan completed over the shared current tree with 760 atomic inputs,
549 Rust consumers, and 212 supplemental candidates. The canonical atomic-input SHA-256 is
`1414a150ce100377c738f9e046e44adf830e69e65bb605e8a8eeae9167cb9f24`; the consumer SHA-256 is
`bae70eb78fecddbbe3074d744ebbb592839f6b58b7da7171d1971ff803ad0ff3`. All 760 accepted input
files were then copied byte-for-byte into
`D:\zircon-frameworks01-r12-resource-snapshot-current-r3`, with each source byte buffer checked against
its manifest hash before publication. A second schema-3 scan inside that isolated Git-indexed
snapshot reproduced both hashes exactly. This materializes the already approved content-addressed
dependency model; it does not replace a changed consumer with stale bytes or relax terminal semantic
discovery.

The current 68-file Resource owner maps to 76 move operations with operation-manifest SHA-256
`057868221ab8379aceea2dacee4d0e4886cbdee81746b113a0670e01282dd840`. The real patch composer then
passed all four stability gates (`atomic_inputs`, `move_manifest`, `source_manifest`, and
`source_shape`) and emitted 150 changes: 70 additions, 66 deletions, and 14 modifications, including
the eight reviewed hidden-assembly Runtime consumer patches. The unified patch is 921,113 bytes with
SHA-256 `584f1d45d912c59a046b407fa512d7fec7f63fa1f969688824ba604979c66395`; the content report SHA-256
is `f63fb30e2bf5ab08df60bee4282d48aeafab41e334e9dc0372e2f3d77e57efed`, and the move-report file
SHA-256 is `0892e4d21e0488a4a2e2a9b36ecfd52abe7b84758ad17c4f9eddf9be65dd5b08`. A real
`git apply --check` against the frozen snapshot is GREEN. All artifacts are on D drive; no patch was
applied to the shared checkout.

The first emitted patch, SHA-256 `c8c3101941fd7c794383e694acd574b48b1d4d0ae6589d4ec61d0afd210dc2f3`,
is explicitly invalidated. Applying it in a D-drive copy exposed two `new blank line at EOF`
warnings, and byte inspection showed that CRLF owners `data.rs` and `manager/revision.rs` had every
logical newline doubled. The root cause was Windows text-mode translation in
`_format_rust_outputs`: `Path.write_text` converted already-CRLF input to `\r\r\n`, and universal
newline reads projected that as `\n\n`. A focused CRLF regression reproduced RED 0/1, then the
scratch path changed to exact UTF-8 byte writes plus one CRLF-to-LF normalization on rustfmt output.
The focused pair is GREEN 2/2 and the complete patch-composer suite is GREEN 13/13 in 46.467 seconds.
The corrected real patch is 921,113 bytes. Its 150 output/deletion hashes all match after isolated
application, `git apply --check` emits no warning, both former CRLF outputs end in exactly one LF,
and all 80 generated or modified Rust outputs pass pinned `rustfmt +1.94.1 --check`.
The final combined consumer/source/move/patch regression is GREEN 51/51 in 222.010 seconds.

The immediately preceding direct shared-tree compose correctly failed closed after three semantic
inputs changed between the source and patch seals:
`docs/plans/engine-code-review-findings-2026-06.md`,
`docs/plans/engine-code-structure-convention.md`, and
`graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/texture_slot_summary.rs`.
It wrote no patch or report. The frozen snapshot result proves that the complete structural hard-cut
is composable on one exact dependency snapshot, but it does not authorize shared-tree application.

The old fixed count of 11 executable foreign attributions is no longer current. Exact coordinator
rechecks preserve all former MVP00/Runtime25/Runtime11 blockers and find additional executable
attributions in the current 760-input union, including the Runtime64 readiness owner/record/guard,
Runtime96 probe-slot consumers, Coordinator01 failure-closeout consumers, and Hybrid GI/material
consumers. There are no live foreign leases on the specifically rechecked paths, but executable
source ownership still forbids transfer. The status therefore records an expanded foreign-owner gate
without inventing a new total from a coordinator prefix query that timed out. Physical application,
managed Cargo acceptance, integration candidate, milestone commit, and WeCom completion notification
remain pending.

### 2026-08-27 r4 frozen snapshot compile/test closure

Four focused Resource test owners changed after the r3 seal, so r3 remains historical evidence and
must not be applied as the current patch. A new byte-frozen r4 snapshot reproduces 760 atomic inputs,
549 Rust consumers, and 212 supplemental candidates. Its atomic-input SHA-256 is
`57b1a1d25dd66249081fad283f540f77f51096bc9d42dee2acde1b700dca7e47`; the consumer SHA-256
remains `bae70eb78fecddbbe3074d744ebbb592839f6b58b7da7171d1971ff803ad0ff3`. The source and
verification reports are byte-identical, 198,567 bytes, SHA-256
`4f8ebe5059ca6bbfdfe1f5d1b4859a228c5924ed6f3595833b8a08bbed793809`.

The current 68 Resource owners still form 76 move operations; the operation-manifest SHA-256 is
`5769298ac2b176eb36d01c0d9a2a5057d7649e078cba7692e7146df53d641ef1`, and the move-report file
SHA-256 is `0bd4bc63917442de7790799f29cce0b1dcc2d10e981e91741c13e66a9c742b1b`. The r4 unified patch
contains the same 150-path shape (70 additions, 66 deletions, and 14 modifications), now 923,370
bytes with SHA-256 `3603e8ba0376a459e26b8f644511070f1649d0a1e4eded7ed4c1ba1c3e52d42b`.
Its content report SHA-256 is `113a34709ad2bb22997cc9c8abdf951dc15f61d48022aad31cf6440c3be73def`.
All 150 after/deletion checks match the report, cached `git apply --check` and `git diff --check` are
GREEN, and all 80 generated or modified Rust outputs pass pinned
`rustfmt +1.94.1 --config skip_children=true --check`. Artifacts remain on D drive and the patch was
not applied to the shared checkout.

A direct Rust 1.94.1 assembly over the generated `zr_resource` first ran 117 tests with five
failures. Whole-owner review showed stale assertions rather than production regressions: a
supposedly nonprojected fixture changed `source_hash`, ready registration was compared with a
pending revision-zero fixture in two tests, a source guard compared unrelated first textual lock
and publish occurrences across functions, and an error-slot test expected `Unloaded` although the
runtime contract and transaction tests require `Error`. The four owned test files now exercise
diagnostic-only nonprojection, ready/revision-one publication, the `PreparedResourceMutation::commit`
critical section, and the runtime error state respectively. No production Resource behavior changed.

The r4 test executable SHA-256 is
`1b1c17aa93256b7a2c28788a0d0549a06d2eabef5d2ab1af6c5ea1157f9cb446`; its rerun with D-drive
`TEMP`/`TMP` passed 113 tests, failed 0, and ignored 4 in 4.80 seconds. Default metadata compilation
is GREEN at SHA-256 `c2c068c963a0a148e9633e87e2e1cc622cb80df677796528d969c07c98600a1d`, and the
`test-support` plus `profiling` feature probe is GREEN at SHA-256
`b9b29df7dacf328e3a163d16205c5533f5e410d4d82bd5c18484fac33d251747`.

These are isolated current-snapshot structure and behavior checks using already available dependency
artifacts. They are not managed Cargo receipts, do not supersede Runtime/App/Editor product gates,
and do not admit performance, energy, parity, bottleneck-removal, candidate, milestone, commit, or
WeCom claims. Shared-tree physical application remains blocked by the expanded executable foreign
attributions; the public `core::resource::io::{atomic_write, atomic_write_new}` projection remains
required, and Frameworks01 still does not edit the foreign IBL or mixed Editor owners.

### 2026-08-28 stale-owner attribution return

The Runtime25 and Runtime11 source Sessions from the earlier 11-path ownership intersection are now
`stale`. A fresh coordinator transfer preview at baseline epoch 516 preserved the exact current
bytes and returned three eligible paths with no blocking reasons: the two AssetUri consumer owners
and the dynamic-scene session-I/O atomic consumer. Fingerprint
`ecccb84f96c9b674b42377e1534f6bdb62c05cd08aa4a7e33b1e995965f8204d` was applied atomically by
request `5e179ea63d0a4c508a96f1af114693b7`; no consumer implementation was edited.

Post-apply ownership matrices report the two AssetUri hashes
`6ac2ee259b667b5a4d495748509ddbb037c029d34d1c20b34dd0df1055201e47` and
`38232ce20e8bdcfd3be90bfaf983f69f916b3e79b72eababd5636fd56dd5d9ad`, plus session-I/O hash
`04d20053813bd6f7b545d02379c2a67f26959d5d169de4b7a38212f212e7cedc`; all three have a live
Frameworks01 lease, `blockingReasons=[]`, and `state=integration_ready`. Their canonical handoffs
were returned as `fixed-2026-08-28-frameworks01-zr-resource-{asset-uri,session-io}-consumer-attribution.md`.

This removes three members of the historical 11-path blocker set, not the complete physical-cut
gate. The eight MVP00 paths still resolve to executable active Session
`mvp00-current-source-convergence-r2-01a00797-20260818`; combined transfer fingerprint
`4c8e4e5ffff913da27581654f23731c94224746d379d36195ad0a4f964fd0e19` therefore correctly keeps
them ineligible with `source_owner_executable`. The later Runtime64/Runtime96/Coordinator01/Hybrid
GI expanded owners also remain subject to a fresh full schema-3 scan. No shared-tree hard-cut patch,
candidate, Cargo acceptance, milestone commit, or WeCom notification is authorized by this partial
attribution closure.

### 2026-08-28 schema-3 current admission and ownership intersection

The next stable schema-3 source scan completed in 67.1 seconds with 778 atomic inputs, 555 Rust
consumers, and 224 supplemental candidates. Their manifests are respectively
`aae705a628e9d49943e2e829119f053fa1022bedd1701c8b1c55a6bc92cc75d9`,
`c389b604b7c5b3bbc9a4a610b438947b3c39af40c861580186b07698e32aa236`, and
`e01932748ebc8af3d7384ede62983c7d1ba0958d3397ba2b55bd769cc83db210`. The report is
`D:\zircon-frameworks01-r12-resource-current-20260828\hard-cut-source-r1.json`, 203,850 bytes,
SHA-256 `305911cfea70bfbe2ddbabfd0a77d7914fbbe91d7aa395b9816e19efce25c6a6b`.

The same source seal produced 71 Resource owner inputs and 79 hard-cut operations: 4 generated
crate-surface entries, 42 crate-root moves with rewrites, 1 module-set move with rewrite, 24
verbatim moves, 4 required consumer patches, 2 Runtime guard relocations, and 2 Runtime facade
replacements. The operation manifest is
`a58e66b3df00be36cf5c64b6322c44b4eac46a9a7882da681885b7aaa8fa7c3a`; the move report at
`D:\zircon-frameworks01-r12-resource-current-20260828\move-manifest-r1.json` has SHA-256
`0ab54c3c0f3ec180c4b9f95c14821948e81caf7dc124a9b1f4a76ae3bb55c384`. All artifacts remain on
D drive. No shared-tree patch was generated or applied.

A later byte-for-byte admission recheck found zero missing inputs but five changed hashes, all in
plan or convention records: `engine-code-review-findings-2026-06.md`,
`engine-code-structure-convention.md`, `optimize/coverage.md`, Runtime04 asset-pipeline alignment,
and Runtime15 code-structure conventions. The snapshot is therefore valid reproducible evidence,
but no longer a current application seal; a new stable scan is mandatory before any transfer or
hard cut.

At the same recheck, the repository had 10,663 dirty tracked/untracked paths and 429 intersected
the 778-input manifest. Read-only coordinator matching at baseline epoch 516 found 91 intersecting
paths attributed to Frameworks01, 137 with no attribution, 161 with terminal foreign attribution,
and 56 distinct paths with executable foreign attribution. Counts are independent overlap classes,
not a partition. The executable set is exact for that 429-path intersection:

- Shader06 `shader06-pbr-ibl-runtime-repair`: 45 paths, including root `Cargo.toml`, Hybrid GI
  material-capture consumers, Asset material loaders, ResourceStreamer owners, mesh material draw/
  pipeline consumers, shader prewarm, and variant-cache disk I/O;
- MVP00 `mvp00-current-source-convergence-r2-01a00797-20260818`: 9 paths, including the eight
  historical consumers plus `zircon_shader_prewarm/manifest/module_dependencies.rs`;
- Coordinator01 `coordinator01-delegated-failure-closeout-r3-20260823`: 1 material selection path;
- Runtime87 `root-runtime87-lazy-project-source-ambiguity-20260827`: 1 project source-path path.

Exact baseline-516 transfer previews preserve this boundary: Shader06 request
`406c72e77ece447bb29f3c67e584170e` covers 45/45 ineligible paths with fingerprint
`be8220e5f9ade8cbc9a39b7ca34878428088aeb6704daaea205ed24eef75114d`; MVP00 request
`9f0936c8240d4d09ab1949755ed97404` covers 9/9 with fingerprint
`f6e384c869183278ecdf0ed817c9601ee53addef5f952a048d1a732d060c249e`; Coordinator01 request
`9657793bef0a4b0d9ace01c337550860` and Runtime87 request
`6caa10f2fca44386a726e0e5a5f7aff3` each cover their single path. Every item reports only
`source_owner_executable`; no transfer apply was attempted.

The coordinator contained nine nonexpired leases globally, but none overlapped these 429 Resource
inputs. Absence of a live lease does not override executable attribution. The three Runtime25/
Runtime11 blobs returned above still match their recorded current hashes and Frameworks01
attribution; their short leases have expired naturally without changing ownership. Transfer of the
remaining 56 paths is forbidden until each source Session reaches a transferable terminal state or
performs a coordinator-authorized ownership handoff. The physical `zr_resource` cut, managed Cargo
acceptance, milestone commit, and WeCom completion notification remain pending.

### 2026-08-28 schema-3 read/write admission correction and physical hard cut

The previous 56-path executable-owner gate was structurally wrong. The 778/779 schema-3 atomic
inputs are a sealed **read set** used to prove source and semantic stability; they are not all
candidate writes. Treating every dirty read input as a write requirement incorrectly demanded
ownership transfer for unrelated Shader06, MVP00, Coordinator01, and Runtime87 consumers even
though the composed migration does not modify them.

The hard-cut tools now use schema version 3 without a compatibility reader. The move graph owns all
eight required hidden-assembly consumer patches and publishes a canonical write manifest containing
only move sources, move destinations, required manifests/guards/facades, and those eight consumers.
The final r5 frozen Git-indexed snapshot contains 779 atomic inputs, 555 Rust consumers, and 225
supplemental candidates. Its atomic/consumer/supplemental manifests are
`938e3cd61e99b296f7b58270a4ce285c6a4b0f319af22412adf930e4c696036f`,
`550ab51959caeb09ff7ffa47d27053b59b4bbb7cee6d78531508ac8491bef9`, and
`1051ed2c7f2efa3d72c662b6c55bce5a803d3a8cf1d5589d1d6b56e906915cd4`. The indexed verification
report is 204,101 bytes with SHA-256
`3c87313ba45afef2906924944551dff7617a61412647e313140df022b6555cb9`.

The r5 move graph contains 87 operations and 156 write paths. Its operation manifest is
`52deb1990859a63dc61115e2185bd9a84c0d31a0bd3dd150b84af50f25e68e5f`; the write-path manifest is
`12377714bc021a44fce725ef76f205fc08b8de1e8b4670837a420b380611bb88`; and the move report is
64,624 bytes with SHA-256
`1c372a919deb2e65810bd872c8e3d0e47c469617346f24181871c39aa5801009`. Coordinator preview/apply
transferred 108 eligible exact-current paths under requests
`2236edc9b713400093bdf36e61a677c9` / `b0a6aee7331742878d52e2d8bb310170`. The only foreign
executable write path was root `Cargo.toml`; coordinator delayed patch 151, request
`daa97815afc94013b1488b2229b1de6f`, inserted the exact workspace member/dependency pair and moved
current attribution without rewriting any other root-manifest content. All 156 exact leases were
then acquired with zero conflicts under request `dfee1d61f4b34fc3a2d8edb6e5746dfa`.

The deterministic patch report recognized `Cargo.toml` as the sole exact pre-applied path and
emitted 155 changes: 73 additions, 13 modifications, and 69 deletions. Together with the pre-applied
root modification, the write-set shape is 73 additions, 14 modifications, and 69 deletions. The
patch is 1,095,773 bytes with SHA-256
`4ee6898de02a4e799750fe911ae8edc7abf9dc78969d07f903ff063899900f67`; the report is 37,566 bytes
with SHA-256 `10a23223bff2e080ae549b40d9907ead9513d56ae58851aefd39a577e795915e`.
All four stability gates passed. Shared current source matched all 156 sealed before hashes before
application and all 155 emitted after/deletion hashes after application; the root pre-applied hash
remained `d1de7ecad881433a6a23762319c958316331f903e0372bab5f7976d751dfe3a9`.

The physical result has 71 files under `zircon_runtime/crates/zr_resource` as the unique Resource
implementation. The old Runtime implementation deleted 69 paths; only the curated
`core/resource/{mod.rs,io/mod.rs}` product/assembly projections remain. Tracked consumers and the
new crate contain zero references to old private `core::resource::{data,error,event_stream,lease,
management_generation,manager,mutation,readiness_generation,registry,runtime,snapshot}::*` owners,
and the Runtime facade declares none of those old behavior modules. The public
`core::resource::io::{atomic_write, atomic_write_new}` projection remains explicit. The two IBL
single-file writers still consume `atomic_write`; source-cubemap staging consumes the private
durable transaction projection. Frameworks01 did not edit any of the three IBL owners or the two
frozen Editor mixed blobs.

The final production hard-cut scripts are 395/553/732 lines and remain below the 800-line warning.
The patch test owner was reduced from 810 to 794 lines by consolidating its duplicate CRLF workspace
case into the dedicated 65-line pre-applied contract module. A pre-consolidation complete toolchain
run passed 56/56 in 122.052 seconds, the focused final patch/pre-applied owners passed 16/16 in
42.693 seconds, and the post-consolidation complete source/consumer/move/patch suite passed 55/55 in
112.380 seconds. No Cargo command or performance claim is introduced by this structural result.
Managed current-hash `zr_resource`/Runtime/App/Editor validation, integration review, milestone
commit, and WeCom notification remain pending.

### 2026-08-28 managed `zr_resource` build and test transport evidence

Windows validator job `b3a38783824d4a6798cc211af2ab6667` used the coordinator-managed retained
pool at
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
`cargo build -p zr_resource --locked` completed GREEN in about 113 seconds. The following
`cargo test -p zr_resource --locked --lib` built its test target but Cargo returned 101 while the
validator transport retained only stderr and no failed-test stdout. The job released normally with
an empty process tree.

The exact emitted executable `zr_resource-d203034e4826f4ad.exe` was then run three times without
Cargo: once with `--nocapture`, once with default capture, and once with the validator's managed
`CARGO_TARGET_DIR`/`TEMP`/`TMP`/`TMPDIR`/`CARGO_HOME`/`SCCACHE_DIR` environment. Every run completed
`150 passed / 0 failed / 3 ignored` across 153 tests in 0.86--1.29 seconds. The two printed poison
panics belong to the passing lock-recovery test and are expected. A second managed SkipBuild test
job reproduced Cargo 101 without test stdout, so this record does not promote the managed test to
GREEN and does not hide the discrepancy behind a direct-binary pass.

Acquire diagnosis also found two coordinator transport/environment events, neither of which
started Cargo: request `883fdefa25ad47dfb284f3524c71a6a8` terminally rejected a transient foreign
`D:\ZirconBuilds\tooling15-local-benchmarks` unmanaged directory, while request
`b636440db0a140878203bba5b614297f` completed after the validator's 15-second client timeout and
leased job `d2faac4c2cd0426197cc0ddf7f598f9c`. That job had no process and was explicitly released by
coordinator request `dd6eb2f935174f8abf507ca35e64e279`; later attempts used a 90-second client
timeout. The same foreign directory reappeared and again blocked a focused managed rerun, so no
unmanaged cleanup or competing Cargo lane was taken by Frameworks01.

Current-hash validation ticket `ccf45045553d4475a09e4573541371b6` is queued under submit request
`52f4fe92c25744289a4e47dbacc21803`. Its source manifest hash is
`911c09a611abaa4c0a8ca8605fceeaff2e6c075633373c35cd38d5b138a8f365` and covers all 156 sealed
write paths, including 69 deletion tombstones. It runs only
`cargo test --package zr_resource --locked --lib` in a coordinator validation copy. Per protocol it
is not polled in this slice. Runtime/App/Editor upward gates remain pending until the lowest Resource
test transport has a terminal managed result.

### 2026-08-28 current-source facade and priority-review audit

The current UI12 compiler report does not change the Resource I/O boundary. The two tracked
single-file IBL writers import `core::resource::io::atomic_write`, and current
`ibl_source_cubemap_staging.rs:16` imports the crate-private durable transaction projection rather
than `atomic_write`. Runtime `core/resource/io/mod.rs` publicly re-exports
`zr_resource::io::{atomic_write, atomic_write_new}`, while transaction and atomic-file assembly
primitives remain `pub(crate)`. The reported three-file E0432 shape therefore came from a stale or
pre-hard-cut materialization. Frameworks01 did not edit those IBL owners, and deleting the public
single-file facade would break the intended product contract.

A production-only scan excluded `tests.rs`, `tests/**`, `*_tests.rs`, and every source suffix after
the first `#[cfg(test)]`. Across the 71-file crate it found zero `CoreError`, zero
`#[allow(dead_code)]`, and zero direct `.lock().unwrap()`/`.lock().expect()` production paths. The
largest production owners are `event_stream.rs` at 778 lines, `management_projection.rs` at 680,
`management_generation.rs` at 613, and `manager/commit.rs` at 605, all below the 800-line review
warning. The same scan found 25 panic-like invariant sites: four in durable-I/O validation/state
dispatch and 21 in manager commit/receipt/projection internals. They are not external lock-poison or
F6 typed-error paths; the crate has no dependency on framework `CoreError`. No projection algorithm
or fallback was changed from this lexical result. In particular, replacing a sparse-plan invariant
with an unprofiled full rebuild would alter the already profiled management algorithm and could hide
an internal consistency defect rather than repair it.

Two priority guards still encoded pre-hard-cut paths and assumptions. The initial current-source run
made `tools.tests.test_frameworks_01_resource_conditional_write_authority` fail 2/10 because it read
the deleted Runtime atomic-file owner and expected its old local re-export. The initial
`tools.tests.test_frameworks_01_resource_crate_boundary` run failed 5 assertions because it rejected
the intentional `pub(crate)` assembly projection, expected two no-longer-exported internal symbols,
forbade extension traits from naming their receiver types, and omitted the durable journal's `toml`
dependency from its exact manifest set. The guards now read the unique `zr_resource` owner, require
the curated public Runtime facade, distinguish crate-private assembly wiring from a public product
leak, reject the retired assembly symbols, and match the generated manifest contract. Their combined
rerun is 15/15 GREEN in 21.334 seconds; the F6 Runtime error single-source guards independently pass
2/2 in 29.916 seconds. Python syntax compilation, scoped whitespace checks, and the retired atomic
owner literal scan are GREEN. These two guards are outside the r5 156-path move/patch manifest, so
the queued validation copy remains source-exact for its declared candidate paths.

The complete source/consumer/move/patch/pre-applied Python suite was then rerun with
`TEMP`/`TMP`/`TMPDIR` rooted at `D:\zircon-frameworks01-r12-python-tests-20260828`; all 55 tests
passed in 197.578 seconds with process exit 0. This replaces an earlier complete 55/55 run whose
test runner printed `OK` but whose outer shell crossed its 180-second timeout during process exit.
No Cargo command is part of either Python suite.

The final post-review integrity check compared all 155 emitted current paths against
`hard-cut-patch-r5-report.json` and found zero missing, resurrected, or hash-drifted paths. The sole
pre-applied `Cargo.toml` remains
`d1de7ecad881433a6a23762319c958316331f903e0372bab5f7976d751dfe3a9`, and the D-drive patch remains
`4ee6898de02a4e799750fe911ae8edc7abf9dc78969d07f903ff063899900f67`. A final process query found
zero Cargo, rustc, or sccache processes from this review.

`python tools/check_conventions.py --only structure --json` is not a static parser: its dry run
expands to `cargo +1.94.1 test -p zircon_runtime --lib structure_convention --locked --jobs 1`.
An exploratory invocation exceeded 125 seconds with no captured result and left its unmanaged
Python/Cargo/rustc child tree running against workspace `target`. The exact process tree was stopped
immediately after discovery and a follow-up process query found no surviving child. This invocation
is recorded as inconclusive and is not a structure GREEN/RED receipt. Frameworks01 started no
replacement Cargo job and continues to reserve managed/upward validation for the coordinator after
the pending Resource ticket becomes terminal.

### 2026-08-30 Windows parent-component probe closure

The later current-source crate contains 76 Rust files, 16,399 lines, and 566,516 bytes. The
path-sorted `relative-path<TAB>lowercase-file-sha256` manifest is
`73de0a28f59b3e4f40e82a2d233ea9a2d2f944d2f4aa18817e10f39d374e63c8`. No Resource Rust file
changed after the full managed run started at 2026-08-30 18:35:54 +08:00.

Managed validator job `17022bc8a1d84629be775fd5e56a055a` had previously passed 190 tests and
failed only `split_at_deepest_existing_ancestor_preserves_parent_components`, proving that Windows
`metadata(missing\\..)` cannot be used as the leaf-to-root existence probe. The first correction
peeled lexical parent/current components before metadata, but current Rust 1.94.1 compilation then
reported E0004 because the subsequent component match did not exhaust `CurDir` and `ParentDir`.
Validator job `6c2abb4873d5477bac94cb0ccf3df66d` preserves that RED receipt.

`split_at_deepest_existing_ancestor` now classifies the trailing component exactly once. Lexical
`.`/`..` components are retained in the unresolved tail and removed before filesystem probing;
normal components are copied into an owned `OsString` for the NotFound branch; prefix/root/empty
states cannot be popped below a physical ancestor. This removes the duplicate component scan and
does not use a wildcard or silently discard lexical traversal. Current `pathing.rs` SHA-256 is
`ee5eadbd55dafb10f098d1db102c3cf1443bc7e8c6a29dc9964e43835c107afd`.

The post-fix evidence is:

- `rustfmt --edition 2021 --check` GREEN for the exact file;
- Resource conditional-write and crate-boundary guards 20/20 GREEN;
- coordinator job `6dacbed14ba7471b855d6d58e27e1a9b`, run
  `39c2cfa014e74aeaa71ebeddb602b359`, exit 0: the exact parent-component regression passed 1/1
  with 194 filtered tests;
- coordinator job `bf4648ee2deb47efaba2dbb6ef22b000`, run
  `c93ba373c2be4b69a33c2779c78ae63c`, exit 0: `cargo test -p zr_resource --locked --lib`
  passed 191, failed 0, and ignored 4 in 12.16 seconds on the F-drive managed target.

The earlier validate-matrix wrapper returned Cargo 101 twice without libtest output while its
nested supervisor rebuilt concurrently changing Interface inputs. The coordinator-native runs above
retain stdout/stderr and terminal exit codes and therefore supersede that transport-only ambiguity.
They close the current-source `zr_resource` behavior gate only. Runtime/App/Editor upward product
validation, same-source performance/power evidence, integration review, milestone acceptance,
coordinator commit, and WeCom notification remain open.

### 2026-08-30 event-stream test-owner structure split

The priority structure convention warns at 800 lines and requires folder-backed test owners rather
than behavior plus a growing inline test suite in one production root. Current `event_stream.rs`
had reached 780 lines; its final 163 lines were five TTL, gap, cursor-contract and ignored
performance tests. The existing sibling `event_stream/publication_index_tests.rs` already owned the
same log-entry/index test domain, so the tests were moved mechanically into that leaf. No event-log
type, constant, publication path, cursor read, lock, coalescing, eviction or capacity algorithm was
changed, and no compatibility module or new path was added.

After the split, the production root is 615 lines and the folder-backed test leaf is 610 lines.
Their SHA-256 values are
`c34d625d1b7691201d61fbd60746189ad3d312c6dc9145f976f4d6739e27d313` and
`b3cd778337fa5df6216fbae0e22210963a4bafe60c85fbf9d284f3237fa63a02`.
Rust 1.94.1 `rustfmt --edition 2021 --check` passed for both current files, scoped whitespace checks
passed, and all five moved test names are present in the folder-backed owner.

The earlier 191-pass managed test result predates these two hashes and is not reused as current
compile evidence. The first current-hash validator attempt timed out outside the validator after it
acquired job `0691fe22131d4185a278bd0e03793cf9`; Frameworks01 explicitly released that unstarted job.
The second `cargo.acquire` request `716a548ccf79467ab77e39f21854d69d` completed only after the
validator's reconciliation timeout and produced unstarted job
`585bbc74bdb54cd6a9a8d901df188462`; release request
`d51d2f176bff48b5a50d8f1b8713eaa4` closed it with no Cargo process.

Coordinator-native job `4bcc417333ec45039d835225e5c41448`, run
`d6f1b5b6fdd645e0a8c76b4bbab4b424`, then compiled and tested the exact current source on
`F:/cargo-targets/zircon-engine/pool/frameworks01-resource-event-structure-20260830`. The cold
isolated run spent most of its 20 minutes 25 seconds materializing the Cargo index, downloading
dependencies and compiling the 200,026,908-byte Runtime Interface rlib; the final Resource test
binary ran 195 tests in 9.48 seconds and reported 191 passed, 0 failed and 4 ignored. Exit code was
0, terminal status was `completed`, and coordinator run-status request
`5b81beb92e98436986e4d36dfe6779e0` confirmed completion at
2026-08-31 00:01:43 +08:00. The 13 emitted warnings are pre-existing Runtime Interface unused/dead
code warnings; no `zr_resource` failure was reported. The job was already released when an explicit
finish request reconciled it, so no Cargo process or retained lease remains.

This closes current-hash compile and behavior evidence for the event-stream owner split. It remains
a structure slice rather than a milestone acceptance boundary: Runtime/App/Editor product gates,
same-source performance and power evidence, independent review, coordinator service commit and
WeCom notification are still open.

### 2026-08-31 durable-transaction test-owner structure split

The priority structure convention requires a folder-backed test owner once a test root reaches 800
lines. `io/transaction/engine/tests.rs` was approaching that limit while still containing a
cohesive pre-active abort domain. Three tests were moved mechanically into the existing
`tests/pre_active_abort.rs` owner:

- `pre_active_abort_uses_journal_first_cleanup_for_uncertain_tail`;
- `pre_active_uncertain_cleanup_preserves_original_phase_and_error_kind`;
- `pre_active_abort_keeps_journal_when_artifact_cleanup_fails`.

No production transaction state, journal ordering, cleanup behavior, public API or compatibility
path changed. The only path adjustment in the moved tests is the existing folder-owner call to
`super::test_directory(...)`. Each moved test name occurs zero times in the root and exactly once in
the leaf.

After the split, `tests.rs` is 782 lines with SHA-256
`77dba07f37512cae20f978775ea1c2343247b88c77dcdf2fc700a4293431f17d`; the pre-active abort leaf is
128 lines with SHA-256
`3f49b38df7279866e62312a251605d0731b4000521e8185207c4cc8ae29410bd`. A full `zr_resource` Rust-file
scan now reports zero files over 800 lines; the largest files are this 782-line test root, the
769-line management projection test owner, and the 754-line crate test root. Rust 1.94.1
`rustfmt --edition 2021 --check`, scoped diff checks, and the Resource conditional-write plus crate
boundary suite all passed; the final rerun completed 20/20 in 32.246 seconds.

Coordinator-native job `6a9b7c043d7f4c089a24b1562c9930a3`, run
`4b8741119d634c67aa91d59f85652196`, then tested the exact current source with
`cargo test -p zr_resource --locked --lib pre_active_abort -- --test-threads=1`. It completed with
exit 0 at 2026-08-31 00:34:28 +08:00: 4 passed, 0 failed, 0 ignored and 191 filtered in 0.20 seconds.
The stdout/stderr SHA-256 values are
`d5128230b3ce44f79cd09c5a27a66b01b33c5329ad4a4a011f771531716cbda0` and
`5ec641e48e7fd79277d904c4983e320be3d227916621946d89c0e334ee245578`.

The coordinator had deleted the shared F-drive target after the preceding full-crate job, so this
focused run performed another cold dependency/index materialization and reported a 25m41s build.
That infrastructure cost is not attributed to the code split and is not a performance result. The
job released automatically, deleted the F-drive target, and left no Cargo or rustc process. This
closes the source-structure slice only; current-hash full-crate behavior is already covered by the
191/0/4 event-stream job above, while upward product, current physical performance/power,
independent integration review, M1 commit and WeCom notification remain open.

### 2026-08-31 readiness-graph architecture and profile preflight

The Resource readiness projection was reviewed as a graph algorithm before any production
optimization. The current recursive DFS still accepts non-canonical dependency vectors, uses the
native call stack, treats a cycle back-edge as synthetic `Loaded`, and uses a 64-bit fingerprint as
semantic equality. The 64-shard copy-on-write layout can also clone approximately `N / 64` sparse
rows. The accepted investigation direction is an iterative, fail-closed SCC/incremental-work-queue
design, but no production hard cut is authorized until the current algorithm has executable RED and
profile evidence. The full review and local Unreal/Bevy references are recorded in
`2026-08-31-m1-resource-readiness-graph-architecture-audit.md`.

Test-only infrastructure now provides that boundary without changing production behavior:

- the two inline readiness tests moved to `manager/readiness_projection/tests.rs`, reducing the
  production owner from 432 to 357 lines;
- ignored behavior RED covers cycle fail-closed semantics, dependency order/duplicate
  canonicalization, and a 10,000-node chain without relying on the native stack;
- the ignored release profile orchestrator runs each workload in a separate process, so a 100,000
  node stack abort is captured as scenario evidence rather than terminating the whole matrix;
- the profile matrix covers chain, fan-out, diamond, dense, cycle and no-change workloads from
  1,000 through 100,000 nodes and records p50/p95/MAD, allocation count/requested bytes/peak live,
  output generations and graph cardinalities under an explicit non-C-drive report root;
- queue depth, touched-shard count, RSS and power remain explicitly unavailable until the
  production algorithm exposes those observations or the managed platform supplies them.

The shared counting allocator is test-only in `src/test_profile.rs`. Current SHA-256 values are
`readiness_projection.rs=9aac00f12030fff53fef7a23e388579c2b8a0099e9787cf300a4bfe4d8c525c9`,
`tests.rs=982eb7e48f32433bd0a140a48021c0696c5d4cfa2caf08a9a5708d8235783ce6`,
`behavior_red.rs=c5be80b961d6e1c1b7a14be1526dd73f18759b051eae075bb9615e079ae5f90b`,
`profile.rs=2c32a319206ccdb02dddbb304539d5b5ef4074756f61c99af6d5d17f5bca6c22`,
and `test_profile.rs=7d6f9b2740ebf8d7877fc5b5b3f6e3992a1ac253eedc91f4db13aca2145c8a8d`.
Resource crate-boundary plus conditional-write static contracts are 22/22 GREEN in 45.343 seconds;
Rustfmt and scoped whitespace checks are GREEN.

No Rust behavior/profile result is claimed yet. Managed release attempt
`2537e95c94d54f618e244ca45e3ea73a` correctly stopped because the first harness version would have
changed `Cargo.lock` under `--locked`; the new `sha2` dependency was removed and the already locked
`blake3` dependency is used instead. Current-source attempt
`64107e1527764083b78888c199cccf5f` then passed lock resolution and compiled dependencies for about
193 seconds, but failed before compiling `zr_resource`: the foreign Runtime Interface bridge still
pattern-matched `UiDispatchHostRequestKind::ActivateLink { href }` after the typed enum hard cut to
`link_target` (`E0026/E0027`). The canonical cross-plan Failure is
`failure-2026-08-31-runtime-interface-ui-activate-link-field-mismatch.md`. Production readiness
changes, performance/power claims, independent integration review, M1 commit and WeCom notification
remain blocked until the bridge is integrated and the same current source executes the RED/profile
matrix.

The next managed retry proves that the bridge source correction is effective but does not close the
profile gate. Job `28eb6b1ee6a649e79a8cac8c19dc5c21`, run
`071e0c99214e4abd965e52a0ebf9bfda`, emitted no E0026/E0027 and reached the current Runtime Interface
schema admission code. It then stopped before `zr_resource` on two E0502 borrow conflicts in
`reflect/schema_catalog/admission.rs`. The coordinator reconciled the orphaned supervisor as a
completed run (`defff507e6d54b01b839145d3c737a9d`); stderr SHA-256 is
`8ebdbf17fdc749490e2ce382d75d4d6c3dfaa5e38671a58bccf9e4fad545e0c4`, stdout is empty, and release
request `823f601d6957426e9c884c15fd27f346` removed the E-drive target with no live Cargo/rustc process.

Current `admission.rs` has already advanced after the failed compile to SHA-256
`18d866b7ecbad235a8c83d34fea59d6a28ccc10f3275e0f5e462c90e0abb2ba7` and separates immutable alias
validation from later sorting, so the exact compile snapshot appears source-repaired by its active
owner. Frameworks01 returned the evidence without editing or claiming that path. A current-source
owner validation/integration receipt and another identical Resource profile run are still required;
no latency, allocation, RSS, power or engine-parity result exists from R3.

### 2026-08-31 current-source profile attempt R4

R4 reached a different pre-crate boundary and failed in the managed validation infrastructure.
Job `84f3507f1dee480184e94f5cbaf9fdb2` ran from 03:29:51 to 03:32:24 +08:00 and released at
03:32:31 with outer exit 1. While rustc compiled `zircon_runtime_interface` through sccache, it could
not write `deps.d` below the job-scoped
`E:\cargo-targets\zircon-engine\scratch\84f3507f...\temporary\sccacheFuHIFi` directory because the
path no longer existed (OS error 3). Cargo returned 101 before compiling `zr_resource`; no report
directory or performance sample was emitted. Terminal job lookup request
`949f999a4cf64dce8fecfc368d95cf3d` confirms the job is released, and no Cargo/rustc process remains.

The unique infrastructure handoff is
`failure-2026-08-31-managed-cargo-sccache-temporary-path-lifecycle.md`, routed to the active App08
runtime-artifact-reuse/compact-validation owner without editing Tooling source. Frameworks01 will not
repeat Cargo until that owner proves the compiler/cache process tree and job scratch lifetimes are
ordered. The Resource test-only harness and 22/22 static contracts remain valid; production
readiness/management optimization, current performance and power claims, M1 acceptance, commit, and
WeCom notification remain open.

### 2026-08-31 current-source profile attempt R5

App08's first return set `SCCACHE_CLIENT_SIDE=1` and reported Pester 5/5, but the exact Frameworks01
command rejected it. Fresh job `680c28eeb45f44ada781073ea28a3e50` reused R4 job
`84f3507f1dee480184e94f5cbaf9fdb2` and failed from 03:59:12 to 03:59:30 +08:00 with outer exit 1.
The still-running sccache server PID 1660 tried to create
`scratch\84f3507f...\temporary\sccacheY8m77e`, proving that the server remained bound to R4's deleted
startup TEMP despite the new client's environment. Cargo returned 101 in `zircon_runtime_interface`
before `zr_resource`; no report or runtime sample exists. The canonical Failure remains open and now
requires a stable non-C daemon TEMP or a controlled health/rebind transition plus a realistic
`--emit=dep-info,metadata,link` regression. No further Cargo retry or production algorithm mutation
is authorized by this rejected return.

### 2026-08-31 lease-incarnation identity hard cut

While Cargo validation is blocked, the non-validation Resource lifetime slice advanced after a full
counter/consumer review. The old global wrapping residency token and manual slot ref-count are both
removed. Each payload incarnation now owns a private `Arc<ResourceLeaseIdentity>`; `Drop` transfers
its exact identity into the manager, consumes it while holding the Resource authority write lock,
and unloads only when the slot is the sole remaining owner. This prevents token reuse, manual count
overflow, stale replacement eviction, and the two-concurrent-drop false-non-final race.

The architecture and Unreal/Bevy comparison are recorded in
`2026-08-31-m1-resource-identity-rollover-architecture-audit.md`. Static identity contracts are
GREEN, all seven changed Rust owners parse under rustfmt 2021, scoped whitespace is clean, and the
771/739-line test owners remain within the structure budget. Direct replacement and concurrent final
drop tests are source-complete. Managed Rust GREEN is pending behind the open sccache Failure and the
current foreign compiler window; the slice is not accepted or committed.

### 2026-08-31 current-source profile attempt R6

App08's revised stable-daemon storage lifecycle passed its isolated realistic dep-info/metadata/link
RED/GREEN and exact origin compilation. Job `96c7732d445d4596b5e86f662d8333ed` reused R5, ran from
04:57:00 to 05:00:13 +08:00, released at 05:00:25, and returned outer exit 1. The exact command used
E-drive sccache endpoint 42261, daemon PID 31088 and stable server TEMP
`E:\cargo-targets\zircon-engine\cache\sccache-temporary`; it compiled both Runtime Interface and
`zr_resource` through real dep-info/metadata/link without the prior deleted-TEMP/OS error 3 failure.
The canonical infrastructure defect is therefore fixed on origin evidence.

The run stopped before profile execution on a Frameworks-owned E0382 in ignored readiness RED setup:
the self-cycle record read its ID after `with_dependency_ids` consumed it. The test now captures the
ID first, with SHA-256 `f7a2e749b105c7082f7e7e4078c176353653284c2c2d2a3f129ad076c2de7282`;
rustfmt/diff-check and attribution request `423fe159221c4802884798269fd8c83d` are green. A subsequent
exact acquire request `b2b1a4b0e31a4f448a3c662cf721fada` created no job because the CPU lane is
reserved by RuntimeInterface03. The origin profile, I0 managed behavior, performance/power evidence,
M1 acceptance, commit and WeCom remain pending; the Failure stays open until profile artifacts exist.

### 2026-08-31 current-source profile scheduling R7

App08 tightened the accepted R6 storage design so only sccache daemon initialization uses stable
`cache/sccache-temporary`; Cargo, rustc, and build scripts again use each managed job's isolated
`scratch/<job>/temporary`. Frameworks01 verified helper hash
`7d1eb4fe2bad2fb7bc124efcac272c187226b9a6f52dbdf9c86e4cd5342f74d9` and test hash
`4798293a9503186b1917aa5dc5074bbbc005dacd866868366f4eb529d1502cc9`.
These supersede `70858e9e...` / `fe3b98ba...`: the owner found that raw string comparison treated
Windows extended and display path spellings as different identities. The owner reports a 6/1 RED
and 7/7 GREEN realistic dep-info/metadata/link lifecycle suite, including both spelling directions.

No R7 Resource profile job exists yet. Artifact audits
`25e75f287cc147a29d7c4d7fe8acc626` and `a276efe3790a49589380e6b3d3941197`
were both clean. Two exact submissions then stopped before job creation on legitimate FIFO owners:
request `4e8f7e19df8e4006b2b70230442f0e72` on RuntimeInterface02 reservation
`fddb926811924e0d98be1f863e7aec7d`, followed by Runtime04 reservation
`b6aa834be4e8489bad9d217b1ff949a0` now consumed as leased job
`98f39ba071014dad919ce54bcb974a4e`. Frameworks01 did not cancel or bypass them. The same command
will resume only after FIFO release. Current latency, allocation, RSS and power evidence remains
absent; no production readiness/event algorithm change, M1 acceptance, commit, or WeCom notification
is authorized.

### 2026-08-31 false Resource I/O capability hard cut

The earlier instruction to retain the public `ResourceIo` contract is superseded by a whole-current-
source capability audit. The trait was sealed against every external implementation, had no
implementation or caller anywhere in the Runtime/Interface/Editor/Plugins Rust union, and its
`ResourceIoError` existed only for that dead declaration. It was a public placeholder, not part of
the functioning Resource MVP.

TDD first added a boundary rule and failed on the still-present declaration file. The hard cut then
deleted both dead source files and removed all `zr_resource` and Runtime projections. The public
`core::resource::io::{atomic_write, atomic_write_new}` pair and private durable transaction assembly
remain unchanged. Focused GREEN is 1/1, the complete Resource crate boundary suite is 8/8, and the
current product Rust scan contains zero `ResourceIo`/`ResourceIoError` matches. The architecture,
Unreal/Fyrox reference hashes, final source hashes and replacement admission contract are recorded
in `2026-08-31-m1-resource-io-false-capability-hard-cut.md`.

The later filesystem/asset-source owner must publish a new provider only after a real implementation,
mount/source lifecycle and Asset consumer exist; no compatibility restoration of the old synchronous
three-method trait is allowed. Managed Cargo, performance evidence, M1 acceptance, commit and WeCom
remain pending.

### 2026-08-31 current-source profile R8/R9

After the FIFO lane released, acquire request `721ee523ae6c45cfb1d4d90e45490496` created empty leased
job `a7829ba0ba9741b4a9f8828f5e67203a` after the wrapper timed out reconciling the accepted response.
The job had no command, start time, managed run or live process; Frameworks01 released only that
exact lease with request `7de2f5043bcc4d6bad1bfa4adfd40e22` before retrying.

R8 job `9acd911f7caf4aa583fa31000f66be0a` then compiled Runtime Interface and `zr_resource` through the
stable sccache daemon and reached the ignored test. It returned Cargo 101 because the previously
recorded command omitted the test's mandatory `ZR_RESOURCE_MANAGEMENT_PROFILE_DIR`. Direct execution
of the E-drive test binary reproduced the exact `profile.rs:188` panic. This was an origin invocation
defect, not a sccache recurrence.

R9 explicitly set the report directory to
`E:\cargo-targets\frameworks01-resource-management-current-r4\profiles\resource-management-current`
and otherwise preserved the same managed validator, package, release profile, test filter and target.
Artifact audit `b2559d6fdd2841d6894ec2f972f99189` was clean. Job
`f2f3280096d64ca699bdd9c9e4800e97` ran from 07:03:07 to 07:07:19 +08:00, released at 07:07:25,
and returned exit 0. Endpoint 42261 remained on PID 14596; no deleted TEMP, `deps.d`, OS error 3 or
unnecessary rebind occurred. The canonical sccache Failure is fixed.

The profile emitted 14 scenarios, 31 samples and 3 warmups per scenario. Artifact SHA-256 values are
metadata `f7cc3e2bc196d316790d4f590cb16394c7d2b1135dab3407ecb2b0338ba79a07`, raw samples
`8c8bb282a3d3c051f85ff1b8f5198b66e4fb4d3ed69b79adae6bbbbad2701230`, and summary
`1244bf20c9b30bf0fca4bd7f3bf850c7502a36c0f911823079718583923d05cd`.

The baseline confirms a structural current algorithm issue rather than a small allocator-only
problem. `no_projected_change_100000` is allocation-free but still costs p50 51.1567 ms / p95
76.9357 ms because it scans all 100,000 input rows. In contrast `revision_100000_1` is p50 14.3 us /
p95 17.2 us with 21 allocations and 65,124 requested bytes. `initial_build_100000` is p50
420.7386 ms with 410,112 allocations, 46,859,944 requested bytes and 27,096,820 peak live bytes;
`dense_revision_100000` is p50 284.2293 ms with 102,856 allocations and 21,106,688 requested bytes.
RSS and power remain unavailable, so this establishes a pre-change bottleneck but not an energy,
engine-parity, optimal-scale, or eliminated-bottleneck claim.
