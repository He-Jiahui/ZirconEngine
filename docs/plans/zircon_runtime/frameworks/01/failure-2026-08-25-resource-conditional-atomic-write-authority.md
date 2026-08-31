---
handoff_kind: failure
status: open
created_at: 2026-08-25
summary_slug: resource-conditional-atomic-write-authority
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/mod.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/asset/project/paths/identity.rs
  - zircon_runtime/src/asset/project/paths/tests.rs
  - zircon_runtime/src/asset/project/paths/windows.rs
  - zircon_runtime/src/asset/project/meta_preview_state.rs
  - zircon_runtime/src/asset/project/meta_write_authority.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/project/manager/relocation.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs
  - zircon_runtime/crates/zr_resource/src/io/atomic_file/mod.rs
  - zircon_runtime/crates/zr_resource/src/io/atomic_file/tests/mod.rs
  - zircon_runtime/src/core/resource/io/mod.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/engine.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/engine/tests.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/schema.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/stage.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/journal/append.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/journal/intent.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/journal/tests.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/recovery/discovery.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/recovery/evidence.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/recovery/replay.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/recovery/tests.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/recovery/validation.rs
  - zircon_editor/src/core/extension/mod.rs
  - zircon_editor/src/core/extension/toolkit/mod.rs
  - zircon_editor/src/core/extension/toolkit/registry.rs
  - zircon_editor/src/core/extension/toolkit/save/context.rs
  - zircon_editor/src/core/extension/toolkit/save/mod.rs
  - zircon_editor/src/core/extension/toolkit/save/report.rs
  - zircon_editor/src/core/extension/toolkit/save/source_write_authority.rs
  - zircon_editor/src/core/extension/toolkit/save/source_write_authority/tests.rs
  - zircon_editor/src/core/extension/toolkit/tests/saving.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/editor_error.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - tools/tests/test_frameworks_01_resource_conditional_write_authority.py
tests:
  - Resource atomic-new concurrent publication race
  - Editor06 same-path concurrent document save authority matrix
  - Editor06 same-path save versus external-effect deterministic wait
  - Editor06 animation source baseline serialization
  - Editor06 external-source conflict injection between preflight and publication
  - Editor06 published-but-not-durable baseline and save-report guarantee
---

# Frameworks01: conditional atomic write authority is not a compare-and-swap

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：DocumentToolkit UI-asset save and external-conflict handling
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：the current Editor save path consumes a new public Resource I/O primitive, while
  both the primitive and its consumers have drifted beyond archived owners and have no executable
  current-source attribution.

## 失败现象与复现证据

At HEAD `0fd7df4ecdd157f9505cd51013780e3225cfb83c`, coordinator baseline epoch 435:

- `core/resource/io/atomic_file/mod.rs` SHA-256 is
  `0b192fe6bc73802bc7f83f3fb968d3d11313c0f25aa5b15c545811041bdc1746`;
- `core/resource/io/atomic_file/transaction.rs` SHA-256 is
  `3737f42b650706ba6c230b1c1df6e46e0413c935dc6298cb09fb5350a2ead18e`;
- `core/resource/io/mod.rs` SHA-256 is
  `8977312a56e9a4228c0534092c8e91882a56b449553596590a686d82b0d3bed8`;
- `asset_editor_sessions/save.rs` SHA-256 is
  `ed1a180db42669f78e1b550447e6ee4fb31826c65331a2c51622d37e7f9c286f`.

The coordinator reports the Resource files against archived Frameworks01 r8 attribution with stale
hash/baseline and no lease. The Editor save file points to archived Editor06 attribution with the
same stale/no-lease state. Frameworks01 therefore leaves all production files unchanged.

`atomic_write_new` has a sound no-replace primitive shape: Unix publishes with a create-only hard
link, and Windows calls `MoveFileExW` without `MOVEFILE_REPLACE_EXISTING`. A target created after the
pre-check is still rejected by the OS. The existing test only covers a pre-existing target, so an
injected concurrent publication test remains required, but this API does not need to be conflated
with the conditional-write defect.

`atomic_write_if_unchanged` is not an atomic compare-and-swap. It stages bytes, reads and compares
the current target, drops that observation, then calls the ordinary replacement path. Another
Editor session, watcher callback, source-control tool, or external process can write after the read
and before `ReplaceFileW`/rename. The final replacement then overwrites that generation and returns
`AtomicWriteCompare::Replaced`. `asset_editor_sessions/save.rs` treats this result as proof that no
external conflict occurred and advances its disk baseline, so the race becomes silent last-writer
wins rather than a typed conflict.

## 最低共享层根因

The Resource primitive is being asked to own two different contracts:

1. durable publication of bytes at a path; and
2. product-level document write authority and external-conflict arbitration.

A portable path rename can provide the first contract but cannot compare arbitrary file contents
and replace the same directory entry as one filesystem transaction. The current function name and
result type overstate the guarantee. The comment mentions a higher-level source identity protocol,
but the implementation and Editor caller do not hold such a token across compare and commit.

Unreal provides the relevant boundary rather than a drop-in algorithm: Core config persistence may
write a temporary file then move it, while Editor asset operations separately gate package writes
through checkout/source-control policy. This supports keeping durable file mechanics below the
document/source authority; it does not validate a bytes-based conditional replacement facade.

## 架构修复验收

- Keep `atomic_write` as the curated single-file durability entry and keep `atomic_write_new` only
  with create-race tests proving no replacement on Windows and Unix.
- Do not accept `atomic_write_if_unchanged` under its current CAS-like name or result contract.
  Either remove it in the hard cut or expose only an explicitly best-effort preflight that cannot be
  mistaken for write authority.
- Editor06/DocumentToolkit must own one normalized-path save authority. Two engine sessions saving
  the same source path must serialize through one lease/token owner; document dirty/save tokens and
  the disk baseline must be committed by that same transaction owner.
- External source-control and non-cooperating process changes require an explicit product policy.
  For MVP, source-controlled assets must pass checkout/writability admission and the UI must report
  that external conflict detection is best-effort unless the source participates in the same lease.
  If lossless cross-process conflict preservation is required, use versioned generations plus an
  atomic owner pointer/journal; do not claim that a path rename is a content CAS.
- Add deterministic race hooks/tests at post-compare/pre-publication, two same-path Editor sessions,
  create-new contention, and source-control rejection. The tests must prove no silent last-writer
  success for cooperating engine writers and must preserve the exact residual guarantee for
  non-cooperating writers.
- Regain current-source coordinator ownership for the complete Resource API, tests, Editor caller,
  related recovery/local-copy consumers, guards, and docs before implementation. Do not patch one
  side or leave a compatibility alias.

## 禁止临时方案

- Do not hold only the UI session map lock and call the current helper after releasing it; that does
  not serialize different sessions for the same source path.
- Do not add another read immediately before rename and call the remaining race window atomic.
- Do not make transaction internals public, duplicate durable-write code in Editor, or weaken the
  existing external-conflict outcome to unconditional overwrite.
- Do not classify current unit tests or the stale UI12 compiler snapshot as acceptance evidence.

## 2026-08-26 implementation preflight

The current call graph confirms that this is a save-authority defect, not a hashing or rename
micro-optimization:

- `DocumentToolkitRegistry::begin_save` excludes re-entry by `DocumentId` only. Two UI asset
  `ViewInstanceId` values that resolve to the same physical source path receive independent
  document IDs and independent save leases.
- `open_ui_asset_editor_by_id` inserts sessions by view instance and does not deduplicate the
  physical source path. The current Resource helper is therefore the only shared guard between
  those saves, but it releases its observation before publication.
- `save_ui_asset_editor_canonical` advances `disk_source`, `disk_source_digest`, and the persisted
  source revision after `AtomicWriteCompare::Replaced`. Because that result is not a CAS receipt,
  current memory state can certify a baseline whose immediately preceding external generation was
  silently overwritten.
- The existing `core/resource/io/transaction::TransactionOwnerLock` serializes one durable journal
  directory. It does not key by document source identity and must not be reused as a hidden global
  Editor save registry.

The Unreal reference supports the same owner boundary. `FAssetEditorToolkit::SaveAsset_Execute`
routes packages to `FEditorFileUtils::PromptForCheckoutAndSave` rather than publishing files from
the toolkit. `FileHelpers.cpp::InternalCheckoutAndSavePackages` performs source-control checkout
admission before the package save and marks new packages for add only after successful save. Zircon
should copy that responsibility split, not Unreal's C++ API: Resource owns durable bytes; the
DocumentToolkit/Editor asset layer owns source identity, writability admission, conflict policy,
and the in-memory saved baseline.

The approved MVP hard cut is:

1. Add one Editor-owned `DocumentSourceWriteAuthority` shared by all UI document sessions in an
   `EditorUiHost`. It resolves the existing source and active project root through canonical
   `ProjectPaths` identity, rejects a source outside the active project, and keys a lease by the
   normalized physical source path.
2. Hold that lease across the final baseline read, Resource `atomic_write`, `disk_source` update,
   conflict clear, and persisted source-revision commit. A second cooperating session for the same
   path observes the first committed generation before it can publish and returns the typed
   external-conflict path instead of silently succeeding.
3. Perform typed filesystem writability admission before publication. The current product has no
   source-control provider contract; therefore MVP may report only filesystem admission plus
   best-effort detection for non-cooperating external writers. It must not claim checkout or
   lossless cross-process CAS semantics.
4. Delete `atomic_write_if_unchanged`, `AtomicWriteCompare`, their public facade exports, and their
   old unit test in the same candidate. Keep `atomic_write` and `atomic_write_new`; add a
   deterministic create-new contention test to prove the no-replace primitive under a real race.
5. Add deterministic authority tests for two same-path sessions, distinct-path concurrency,
   normalized aliases, read-only admission, and an injected non-cooperating write between compare
   and publication. The last test documents the residual best-effort guarantee; it must not be
   mislabeled as conflict preservation.

The initial source wave covered `core/resource/io/atomic_file/{mod.rs,tests/mod.rs}`,
`core/resource/io/mod.rs`, the folder-backed Editor source-write authority, its tests/module wiring,
`EditorUiHost` construction, and `asset_editor_sessions/save.rs`. Independent review then proved
that this opt-in surface was incomplete: animation saves and UI asset undo/redo external effects
could still write or remove the same canonical source without the authority, and save callers could
not distinguish a pre-publication failure from a post-publication durability-barrier failure. R12
therefore expanded only to the cooperating writer and save-report paths listed in `related_code`.
The unrelated in-flight split of durable `transaction/{journal,recovery}/**` remains untouched.
Coordinator baseline epoch 436 originally reported the Resource and Editor inputs against
archived/stale attribution with no live leases. R12 transferred and attributed every expanded blob
before editing; the unrelated transaction journal/recovery split stayed outside its leases and
source changes.

For `P` simultaneously active source paths and `B` bytes in the current/new documents, the planned
authority costs `O(log P)` path admission with `O(P)` retained path identity and `O(B)` compare plus
durable write I/O. The authority adds no per-frame work and no global scan. This slice is a
correctness hard cut; no latency, throughput, allocation, power, or Unreal-parity claim is approved
before managed tests and a dedicated profile.

## 2026-08-26 implementation state

The Resource/Editor hard cut is now implemented in current source but is not yet accepted:

- coordinator transfer `3098bc07218c4d4cafd4495fac60ce75` moved the original eight exact paths
  into r12; transfer `b82da4b538994cea99e200c8670f8390` added the existing
  `DocumentToolkitRegistry` owner after the design review placed source-write authority beside its
  document save/close leases;
- Resource exports only `atomic_write` and `atomic_write_new`. The CAS-like
  `atomic_write_if_unchanged` function, `AtomicWriteCompare` result, old unit test, and every
  Runtime/Editor Rust reference were deleted together; no compatibility alias remains;
- the UI12 editor-build fingerprint that reported unresolved `atomic_write` imports in three IBL
  files does not change that facade decision. Current `ibl_bake_artifact_asset_derived.rs` and
  `ibl_bake_artifact_cache.rs` remain valid single-file durability consumers of the public
  `core::resource::io::atomic_write` export. Current `ibl_source_cubemap_staging.rs` no longer uses
  that API because its source-cubemap plus derived-artifact bundle is a multi-file durable
  transaction and now consumes crate-private `io::transaction` primitives. The reported three-file
  shape is therefore a stale compiler snapshot/facade integration mismatch, not permission to
  delete the public export or patch Shader06-owned IBL behavior;
- `DocumentToolkitRegistry` now retains one `DocumentSourceWriteAuthority` per Editor host. It
  resolves aliases through `ProjectPaths`, rejects sources outside the active project, serializes
  by normalized physical path, and admits only filesystem-writable sources. The lease is exposed
  only through a scoped closure, so a caller cannot return it past registry ownership;
- the UI asset save keeps its source lease through durable publication, `disk_source`/digest
  replacement, conflict clear, and persisted-revision acknowledgement. Imports and workspace
  refresh occur only after the lease is explicitly released;
- the first independent review returned `C1/I2/M1`: animation save and asset undo/redo bypassed the
  authority; an `atomic_write` error after visible replacement could leave the in-memory baseline
  stale; the best-effort external-writer guarantee was absent from `DocumentSaveReport`; and the
  timeout-based wait test did not prove Condvar admission. Transfer
  `7edcc10fa443472696e446c4b0ff620d` expanded the immutable owner scope to the exact cooperating
  writers/report paths before those findings were corrected;
- all current production document writers now enter the same normalized-path authority. UI asset
  canonical saves, animation saves, and UI asset undo/redo replace/remove effects hold the lease
  through the filesystem effect and in-memory baseline update. The typed outcome distinguishes
  `NotPublished`, `PublishedNotDurable`, `SourceChanged`, and durable best-effort publication. A
  post-publication durability failure advances the observed disk baseline but does not acknowledge
  the document save token as persisted;
- the publisher classifies the pre-publication observation as `Missing`, `MatchesReplacement`,
  `DiffersFromReplacement`, or `Unknown`. A publisher error is `PublishedNotDurable` only when the
  post-image matches and the pre-image proves this call changed the path; same-content and unknown
  observations conservatively remain `NotPublished` instead of certifying another writer's bytes;
- `SaveCtx` can record the guarantee only by consuming a crate-private
  `DocumentSourceWriteReceipt` minted by a durable authority outcome. `DocumentSaveReport` exposes
  both cooperating-writer serialization and best-effort external-conflict detection. Callers can
  no longer self-certify the guarantee; this is still not a cross-process CAS claim;
- sixteen Editor authority tests cover alias identity, deterministic same-source Condvar admission,
  save-versus-cooperating-external-effect ordering, distinct-path concurrency, stale and missing
  conditional baselines, read-only and out-of-project admission, remove/create behavior, the named
  nonparticipating-writer best-effort outcome, all pre-publication observation classes, and
  published-but-not-durable attribution. A toolkit test covers the save-report guarantee. The
  Resource test stages two publications behind a barrier and requires exactly one create-only
  winner plus one `AlreadyExists` loser;
- the current candidate contains 22 attributed Rust blobs. Exact `rustfmt --check`, scoped
  `git diff --check`, recursive production-writer scans, and the retired-API scan are GREEN; the
  Runtime+Editor Rust tree has zero `atomic_write_if_unchanged`/`AtomicWriteCompare` references.
- `python -m unittest tools.tests.test_frameworks_01_resource_conditional_write_authority -v`
  first produced the intended TDD RED with exactly 2 of 7 tests failing: complete cooperating-writer
  coverage and the report guarantee were absent. The final primary run is `7/7` GREEN in 6.747
  seconds; the independent reviewer reproduced `7/7` GREEN in 6.263 seconds; the final post-record
  recheck is `7/7` GREEN in 8.826 seconds. The guard locks the public hard cut,
  registry/source-authority ownership, normalized-path/writability contract, bounded deterministic
  wait/release instrumentation, complete production-writer admission, typed publication
  attribution, unforgeable report evidence, UI baseline commit ordering, and the two-staging
  create-only race fixture.

The second independent review by session
`frameworks01-interface08-lifetime-review-r1-20260825` is final at `C0/I0/M0`. It verified the
authority blob SHA-256
`78227abf4b98cc36cf419096d7729efee49bbf6448210b6f3054186a011ebc85` and guard SHA-256
`2d1055f530b11074280991fb37e16cef618a84288b0f15a52b800c99c4f197d8`, including the nested
`Result` error mappings in UI asset external effects, the non-escaping scoped lease, conservative
same-content/unknown publication attribution, the receipt visibility boundary, and bounded
release completion. No review finding remains assigned to this candidate.

Windows managed test job `dc439b2ec8a14db6a0a1b4d2ea34fbfe` used only
`D:\cargo-targets\zircon-engine\frameworks01-r12-resource-runtime` and ran from
16:45:57--17:11:00 UTC before exiting `101`. The shared Runtime lib-test graph failed before
executing any test with 416 errors and 1,517 warnings across 139 error-bearing Rust files. Exact
log correlation reports zero direct diagnostics in the eight owned implementation files. Foreign
clusters include 40 Platform-test errors, 40 native-plugin fixture errors, 14 UI-text errors, and
the concurrently introduced Resource transaction split with 10 journal-root, 7 engine, and 2
recovery-discovery errors. The job is released with an empty process tree; this is a current-source
validation blocker, not a GREEN or a failure assigned to the conditional-write implementation.

Windows managed Editor test job `2d839ac9d78b4b56a829bb015784a36f` used only
`D:\cargo-targets\zircon-engine\frameworks01-r12-resource-editor` and ran from
21:24:24--21:32:51 UTC before exiting `101`. It stopped while compiling the shared
`zircon_runtime` dependency, so the Editor authority test binary was not generated and zero tests
executed. Rustc reported 80 previous errors and 118 warnings; an exact correlation of the complete
stderr log found 81 error headings and zero mentions of the nine owned Rust implementation paths.
The failures are shared current-source errors, including the foreign Resource transaction
journal/recovery visibility split and Platform host/window-registry errors. Release request
`3269486b865e4c51ba8e4aa27c244c04` confirms an empty process tree, `released` status, and queued
D-drive target cleanup.

Windows managed production build job `af4dbe2fcadf47a0a1e9c660c1966c33` reused only the
coordinator compatibility pool at
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`.
It ran from 06:36:02--06:40:42 Asia/Shanghai and exited `101` while compiling the shared
`zircon_runtime` dependency, before `zircon_editor` or any Editor authority test binary was built.
Rustc reported 83 previous errors and 119 warnings. Observed foreign clusters include animation
compiler export drift, render/frame-extract visibility, the separately owned Resource transaction
journal/engine split, ECS schedule/tick-policy drift, Platform host/window-registry visibility and
const construction, and material partial moves. No diagnostic was observed against the three
owned Resource atomic-file facade files, and the 15 Editor authority tests executed zero times.
The coordinator finished and released the job at 06:40:44 with an empty process tree; the retained
D-drive pool does not represent product GREEN.

Three scoped isolated-copy attempts then separated repository compile drift from coordinator copy
admission, but none produced Rust evidence. Cargo-copy `b5a8fc35080148928c87fc59aaecf992`
failed in `closure_planning` before Cargo because the sibling `zr_vm` descriptor was absent. The
retry pinned immutable `zr_vm` commit `61b79becf64efdae8406385ba2c880620831b4b3`, mount `zr_vm`,
and only its binding/sys crate roots; copy `79657cb067264d0dad6db28ab28dd9d6` then failed in the
same pre-Cargo stage because the loaded closure planner scanned the unrelated dirty Runtime test
`runtime_environment_wgpu_cubemap_sampling_contract.rs`, whose worktree source references deleted
worktree path `core/framework/render/environment/skybox.rs` even though both exist consistently at
HEAD. This is the already registered Coordinator01
[`wrapped-cargo-package-closure-scope`](../../../zircon_tooling/session_coordinator/01/failure-2026-08-25-wrapped-cargo-package-closure-scope.md)
failure, not a Resource/Editor compile diagnostic. Full explicit copy
`405fc4d9c26347b4bd5c936cc01b5650` successfully materialized all 21 overlays with input manifest
SHA-256 `350790704da044d45e689dc9d10331740d61ceb5a2c4dcc655fd84da9b303876`
and external-source SHA-256
`984a7062d9607791aa97e338032e547e00d658a81795a56973d7109a32c2c404`, but its intentionally
explicit source root has no workspace `Cargo.toml`; run `c40f6af8ce574115b3cdad3500d22c5c`
therefore exited `101` at Cargo root discovery with zero compilation and zero tests. Expanding that
copy with foreign dirty workspace paths is forbidden; a passed compile ticket waits for the
Coordinator01 package-closure forward fix and daemon load.

## 2026-08-26 normalized identity follow-up

A second source review found that the authority's normalized-path claim was still false for an
uncreated Windows target. `ProjectPaths::resolve_path` correctly canonicalizes the deepest existing
ancestor and then preserves the uncreated tail. The authority then discarded the resolved type and
inserted the resulting `PathBuf` into a `BTreeSet`. Rust path ordering is case-sensitive, so
`assets/Panel.zui` and `ASSETS/panel.zui` could occupy two independent lease entries even though the
default Windows filesystem path comparison addresses one target. This was a structural admission
defect, not an `atomic_write` or rename optimization.

The hard-cut repair adds `ResolvedProjectPathIdentity` to the resolver-owned project path surface.
It retains the operational `ResolvedProjectPath`, implements one total ordering, calls
`CompareStringOrdinal(..., ignore_case = 1)` on Windows, and uses native path ordering elsewhere.
It deliberately has no `Hash` implementation and does not create a lowercase/string-lossy
compatibility key. `DocumentSourceWriteAuthority` now stores
`BTreeSet<ResolvedProjectPathIdentity>` and derives every filesystem operation from the retained
operational path, so lease identity and I/O identity cannot drift. The public project facade exports
the type because Editor consumes the contract; the Resource facade remains unchanged.

The local Unreal reference confirms this platform boundary. In
`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/Paths.cpp`,
`FPaths::IsSamePath` converts both paths to absolute form, removes duplicate separators, uses
`Stricmp` only under `PLATFORM_MICROSOFT`, and uses case-sensitive `Strcmp` elsewhere. Zircon keeps
that filesystem-facing split while resolving the deepest existing ancestor before comparing the
uncreated tail. Unreal's separately generic `FPathViews::Equals/Less` string-view ordering is not
used as the document write-authority model.

The initial line-budget review used PowerShell `Measure-Object -Line` and reported 996 lines after
extracting Windows-only operations and identity ordering. That number ignored blank lines and was
not the Runtime15 physical-line metric. A later independent review measured 1,119 physical lines,
found the inline test owner still above the 800-line production budget, and also found the older
lossy identity helper still active in four production consumer groups. The correction is recorded
below; the 996-line statement is not acceptance evidence.
For `P` active source leases and `L` UTF-16 path units, admission remains sorted-set scale
`O(L log P)` with `O(P * L)` retained identity; document comparison/publication remains `O(B)` for
`B` bytes plus filesystem durability cost. This correction is off the frame loop and makes no
latency, allocation, power, or engine-parity claim without a dedicated profile.

TDD first ran only
`test_uncreated_windows_case_aliases_share_one_physical_identity` and produced the intended RED:
the static contract found `BTreeSet<PathBuf>` instead of the resolver identity. After the repair,
the complete Python architecture guard is `8/8` GREEN in 14.444 seconds. It now also rejects a
lowercase-derived identity key. Exact `rustfmt --check`, scoped `git diff --check`, and the retired
Resource CAS API scan are GREEN; the Runtime and Editor Rust trees contain zero
`atomic_write_if_unchanged` or `AtomicWriteCompare` references. Current SHA-256 values are:

- project facade `0eb41a57f7900a30dc6874492a4090f686f72b3e49afe5e5f7c585cd13d988cd`;
- project paths `8f47af576936fe2053ab4b0a80604c8bc685ca1f0938a73d53d400ed24a73364`;
- resolved identity `e4dfe1f8373e8b7032a38809919c7d52f3088611506547b9f072c467a07e795a`;
- Windows path operations `394da4ff6d801eee7fdbccc9eeb6d642f0dfb018f62e44b46e0ec09a10cfdae6`;
- Editor source authority `f1c8f05cac02c35b0a40b766e74a7c893d8314aa8de32db5e377dbcdc0b1fc51`;
- Python guard `6bbcacce9a793cd9ea6f713d0bbc04418fc22c8e0d9190429f8060e0315573ae`.

Coordinator transfer-apply request `af422a87732744caa35c0b5e15a463f2` moved the archived
current `project/mod.rs` blob into r12 after preview fingerprint
`176db018609ceade586460482c97e9731645d60162489f5bef45871112882da5`; the three path implementation
blobs were already transferred by request `0eb600aaf6ce49e698dc454c8511c3f2`. Exact Runtime source-copy
job `6b2532659b3a414ead4ce5290a69f9f2` was removed before closure planning because coordinator
artifact governance found foreign `D:\ZirconBuilds\tooling15-wave103-runtime-20260826-214953` and
`D:\ZirconBuilds\tooling15-wave104-runtime-20260826-220711` outputs. A managed current-source
Runtime test request was rejected by the same gate before Cargo admission. No Rust compiler or
behavior test ran, no target output was created by this follow-up, and the foreign artifacts were
not deleted or adopted.

## 2026-08-26 typed identity consumer hard cut

The fresh independent review of the normalized identity follow-up returned `C0/I2/M2`, not ready.
Its two Important findings were structural rather than cosmetic:

- `ProjectPaths::filesystem_identity_key` still converted resolved paths through
  `to_string_lossy().replace(...).to_lowercase()` and silently fell back to lexical paths on
  resolution failure. Prepared IBL write deduplication, project transaction recovery, targeted
  meta-path deduplication, and meta write-lock striping therefore used a second identity contract
  that contradicted Runtime25 `FILESYSTEM-P1-026`;
- the true physical size of `paths.rs` was 1,119 lines, while
  `source_write_authority.rs` was 714 lines and still carried 396 lines of inline tests. The
  repository budget counts `source.lines().count()`, not nonblank PowerShell lines.

The hard cut removes `filesystem_identity_key` completely. `ProjectPaths::resolve_identity` is the
only coordination entry and every resolution error now propagates. Prepared-write maps use
`BTreeMap<ResolvedProjectPathIdentity, _>`; targeted meta paths are resolved once and deduplicated
by the same ordering; `ProjectRecoveryPolicy` retains typed artifact, registry, cache, and
asset-root identities and performs Windows-aware component containment without a string key. IBL
bundle layout remains owned by `IblSourceCubemapStagingStore`: recovery passes its original journal
target to that semantic validator and does not alter the three IBL consumers or the public
`core::resource::io::atomic_write` facade.

The fixed 64-way hash-striped meta lock was replaced by one active
`BTreeSet<ResolvedProjectPathIdentity>` plus a `Condvar`. A multi-path writer resolves, sorts, and
deduplicates its identities, waits until the complete set is free, and admits the set atomically;
drop releases the exact set. For `P` active identities, `K` requested identities, and path length
`L`, admission is `O(K * L log K + K * L log P)`, retained memory is `O(P * L)`, and unrelated
paths no longer serialize through hash collisions. Two Rust behavior tests encode same-identity
blocking and distinct-identity concurrency, but they have not executed while Cargo admission is
blocked.

Editor source admission now resolves the project root through `resolve_existing`, requires a real
directory, and checks typed containment before returning a source identity. The Windows FFI call
has an explicit safety contract. Tests moved without semantic changes to folder-backed owners:
`paths.rs`/`paths/tests.rs` are 433/674 physical lines and
`source_write_authority.rs`/`source_write_authority/tests.rs` are 317/393. The new meta authority is
185 lines.

TDD extended the architecture guard to 10 tests. The first run produced the intended RED with four
assertion/subtest failures from the two new contracts: legacy identity consumers remained, the
meta authority file was absent, both folder-backed test owners were absent, and `paths.rs` exceeded
the physical budget. The first complete GREEN was 14.235 seconds; after strengthening the guard to
require the new algorithm, the current exact run is `10/10` GREEN in 19.061 seconds. Exact
`rustfmt --edition 2021 --config skip_children=true --check`, scoped `git diff --check`, and the
recursive `filesystem_identity_key` scan are GREEN with zero legacy references.

Coordinator ownership-transfer apply request `910ae0278b6847e894d663d866cedb38` moved the initial
consumer/test-owner set into r12; request `6dd2b8fa96d044cda1d57408d2d73798` added the previously
unowned relocation caller. Latest artifact audit request `777dd00da49744a396419b81dbef3d16`
reports six foreign unmanaged outputs:
`D:\ZirconBuilds\tooling15-wave105-runtime-20260826-231249`,
`D:\ZirconBuilds\tooling15-wave106-runtime-20260826-233142`,
`D:\ZirconBuilds\tooling15-wave107-runtime-20260826-234524`,
`E:\ZirconBuilds\mvp-perf-projects`,
`E:\ZirconBuilds\mvp-resource-management-comparisons`, and
`E:\ZirconBuilds\mvp-resource-management-projects`. They were not deleted or adopted. An earlier
managed lease attempt while the coordinator runtime descriptor was absent created neither request
nor D-drive target; no Cargo command, Rust compiler, behavior test, profile, or power measurement
ran for this snapshot.
Runtime25 `FILESYSTEM-P1-027`/`P1-029` still own the handle-relative open/create TOCTOU closure;
this in-process typed identity change does not claim to solve it.

## 2026-08-26 post-review structural closure

The first review of the typed-identity consumer cut returned `C0/I3/M1`, not ready. It found three
remaining module-level defects: `Path::exists` collapsed inaccessible/broken paths into uncreated
tails; project recovery validated only a canonical target while later operating the journal's raw
directory entry; and an active-set-only meta authority allowed a sustained single-path write stream
to barge ahead of an older multi-path waiter.

The repair is fail-closed at each boundary:

- ancestor probing now uses `fs::metadata` plus `fs::symlink_metadata`. Only a true `NotFound` with
  no extant link/reparse entry is treated as an uncreated component; every other I/O error
  propagates, and a path with no accessible physical ancestor is rejected instead of becoming a
  lexical identity. Component containment and broken directory-link behavior have direct tests;
- recovery retains typed artifact/registry/asset-root identities but also resolves the raw parent
  directory. Registry recovery requires the exact raw registry leaf. Artifact and import-source
  recovery reconstruct a relative entry from the resolved raw parent plus the original filename
  and apply the same namespace/extension/ResourceId layout predicate used for the canonical target.
  Relocation and meta entries require both canonical and raw-parent containment. IBL remains a
  separate semantic owner and still validates the original `document.target()`; the three IBL
  consumers and public `core::resource::io::{atomic_write, atomic_write_new}` facade are unchanged;
- meta admission now stores active identities and a ticketed `VecDeque` of waiters under one
  Mutex/Condvar state. A request may proceed only when it has no active conflict and no earlier
  conflicting waiter. The ordered-vector merge check avoids a hash/string identity and still lets
  a later disjoint request pass an earlier blocked waiter. Checked ticket exhaustion fails before
  enqueue; poison recovery and exact Drop release remain explicit.

The next focused review returned `C0/I1/M2`: raw-parent containment alone still allowed a valid
canonical artifact/import target to authorize an arbitrary raw leaf inside an allowed root. The
shared relative-layout predicates and raw-leaf reconstruction above close that bypass. The final
independent review by `resource_path_identity_review` returned `C0/I0/M0`, `Ready`; it found no
remaining type, visibility, lifetime, borrow, deadlock, starvation, poison, Drop, empty-set, or
ticket-overflow issue in the reviewed source.

TDD for this review wave first made the architecture guard RED on the missing fair queue and then
RED again on the missing raw-entry branch contract. The final exact command
`python -B -m unittest tools.tests.test_frameworks_01_resource_conditional_write_authority -v` is
`10/10` GREEN in 13.866 seconds. Exact Rust 2021 `rustfmt --check` over the 14 owned source/test
files, scoped `git diff --check`, the recursive legacy-helper scan, and the Resource atomic facade
check are GREEN. Current physical lines and SHA-256 values are:

- `paths.rs`: 449,
  `3a0c26d46bf7cc327bac945de838b30e3c00ca3ba54972da40a4eab9f1f260cc`;
- `paths/tests.rs`: 725,
  `7014cd38c25e3e18c4ccdd5a2b4e52f13aa21ae5cb1608037dea2702e5ea0677`;
- `meta_write_authority.rs`: 371,
  `e5f90a488ce03f0402593fa3f4ea2110f3c814d15bdc159aa8fb6523932a7f12`;
- `manager/durable_transaction.rs`: 630,
  `7d3730dd3df2cf4456ca725449be978751e0f16de4c097b3b015adf9424c2bb8`;
- the Python guard: 311,
  `a9e290ed1256dd1d38ebaa7d9d5040b36242a1dbcaef46cd40e74bdb8ed74f8c`.

Test fixtures use `ZIRCON_TEST_OUTPUT_ROOT`, then `CARGO_TARGET_DIR`, then the E-drive workspace
`target`; this slice creates no C-drive artifact. For `P` active identities, `W` queued requests,
`K` identities in a new request, and path-unit length `L`, sorting/deduplication is
`O(K * L log K)`, active conflict admission is `O(K * L log P)`, and the ordered earlier-waiter
scan is linear in the conflicting queued identity vectors. Retained identity memory is bounded by
active plus queued requests. This is a correctness/progress design bound, not measured latency,
power, or engine-parity evidence.

Latest coordinator artifact audit request `05a2ae944da84de8a8e3ab31f22b49b1` reports nine foreign
unmanaged outputs: Tooling15 waves 105 through 110 under `D:\ZirconBuilds` plus
`E:\ZirconBuilds\mvp-perf-projects`, `E:\ZirconBuilds\mvp-resource-management-comparisons`, and
`E:\ZirconBuilds\mvp-resource-management-projects`. They were not deleted or adopted. The audit
stops the managed request before Cargo, so no Rust compiler, behavior-test, profile, power, or
product-frame evidence exists for this exact snapshot.

## 2026-08-27 facade and journal compiler-blocker reconciliation

The latest UI12 report does not change the Resource public boundary. Stable HEAD already exports
`atomic_write` from `core::resource::io`; current source adds `atomic_write_new` beside it. The two
current single-file IBL consumers therefore keep importing the public `atomic_write` facade, while
`ibl_source_cubemap_staging.rs` keeps using crate-private multi-file transaction primitives. Its
current line 16 is a transaction import rather than `atomic_write`, which confirms that the reported
three-file/line-number shape came from an older materialization. Frameworks01 did not edit any IBL
consumer.

The `atomic_write_new` implementation is now one current-session ownership unit: archived r8
`atomic_file/transaction.rs` was transferred by preview fingerprint
`38bf1dc9866992f72464b81bea2270dcebcc8fca6a5307c6d8de30f958627a35` and apply request
`7bb1d53f49644dec83c954f3c1aad19a`; the implementation, facade, module root, and create-race test
are all attributed and leased by r12. Source review preserves the existing create-only algorithm:
Unix publishes through `hard_link`, and Windows uses the no-replace rename path. The pre-check is only
an early error; the OS create-only operation remains the race authority.

A Windows standalone `rustc --test` build compiled the current folder-backed `atomic_file/mod.rs`
directly to `D:\zircon-frameworks01-r12-atomic-standalone\atomic_file_tests.exe` and executed all
11 module tests with `11 passed / 0 failed` in 0.69 seconds. This includes pre-existing target
preservation, two-thread create-only contention with exactly one winner, parent-directory durability
failures, and Windows replacement recovery. The executable SHA-256 is
`3d91d4b12a3bded9fcfa77a8e4d89741c2ad17ce8d07fa5b0ee2990f4dc93a74`. `TEMP`, `TMP`, and
`ZIRCON_TEST_OUTPUT_ROOT` all pointed to that D-drive directory; no test artifact was placed on C.
This is focused behavior evidence, not the missing workspace managed-Cargo receipt.

The separately split durable journal had 25 compiler diagnostics because leaf items retained the
old monolithic file's `pub(super)` visibility after gaining an extra module level. The folder root
remains the internal facade; required leaf symbols now use exact
`pub(in crate::core::resource::io::transaction)` visibility. Current-source Rust fingerprints show
those Resource transaction diagnostics reduced from 25 to 0, and exact Rust 2021 rustfmt is GREEN.
This is diagnostic evidence only: managed `core-min` job
`2f7eed4398c540ea8ebb73779188205f` ended `orphaned` with a null exit code and no Runtime
fingerprint, so it is not accepted validation. The archived-r8 deletions of the old
`transaction/journal.rs` and `transaction/recovery.rs` were subsequently claimed and attributed by
r12 under request `80353a8b95d1446080d1665ad8166459`; the remaining transaction blobs were unified
under transfer fingerprint `ff6eb8a8636613be155a807874843aa1c43042c627b2918910a4ca0f1b8bac08`
and apply request `0bb0f46fe82e42e495fb9fbbc28dfaea`. The ownership matrix now retains only
the expected `deletion_requires_explicit_candidate` gate for those tombstones, so the full split
must be submitted as one explicit candidate and cannot be committed as journal-only files.

## 2026-08-28 durable transaction presence fail-closed review

The canonical `zr_resource` transaction owner and its real Asset/IBL/project consumers were
reviewed before changing the low-level recovery algorithm. Unreal's asset save path keeps checkout
and save admission above file publication, while Bevy's filesystem store and Godot's `FileAccess`
confirm that temporary-file mechanics belong below that product authority. The existing Zircon
boundary remains correct: `core::resource::io::{atomic_write, atomic_write_new}` is the curated
single-file facade, and multi-file journal/recovery stays private. UI12's three unresolved
`atomic_write` diagnostics therefore remain stale-build evidence rather than a reason to edit the
foreign IBL consumers or remove the public facade.

The structural review found nine production `Path::exists()` decisions across `engine.rs`,
`stage.rs`, `recovery/discovery.rs`, and `recovery/evidence.rs`. `Path::exists()` collapses every
metadata error to `false`; an inaccessible target, damaged link/reparse entry, or other metadata
failure could consequently enter an allow-missing branch and continue cleanup or rollback as if
the evidence were absent. This contradicted the fail-closed physical identity contract already
adopted by `ProjectPaths`.

The repair introduces one private `FilePresence::{Missing, Present}` classification. It uses
`symlink_metadata`, treats only `ErrorKind::NotFound` as missing, rejects symlinks and non-files,
and propagates every other I/O error. Journal-directory discovery and creation now use the same
explicit rule; recovery maps presence-query failures to typed invalid-journal evidence instead of
continuing with partial state. Production transaction sources contain zero `.exists()` calls; the
remaining 16 calls are assertions in the folder-backed Rust test owners.

The same state-machine review found a second correctness split. Live rollback already attempts
every published document and retains the first restore error, but crash recovery returned on its
first failed `restore_document`. A persistent failure in the first reverse-order document could
therefore prevent every otherwise recoverable document from receiving a restore attempt on every
startup. Recovery now uses the same best-effort-all invariant: it records/attempts every eligible
document in reverse publication order, retains the first typed error, and returns it before any
`RollbackCompleted`/cleanup transition. When state appends remain safe, attempted documents remain
marked `RollingBack` in the active journal and can be retried idempotently; the existing torn-tail
fallback remains unchanged when an append fails. No terminal phase is written while one restore
failed.

TDD first ran the presence static case RED and reported the four production files. The recovery
case then ran RED on the missing first-error accumulator. The current full architecture guard is
`12/12` GREEN in 17.298 seconds. Exact Rust 2021 `rustfmt --check` over the six production/test
files and scoped `git diff --check` are GREEN. Rust behavior sources encode both the `NotFound`
classification and a three-document recovery where document 2 fails but restore attempts remain
`[2, 1, 0]` and terminal phase attempts remain zero. Those Rust tests have not executed because the
existing managed Cargo receipt remains outstanding. These are correctness repairs, not performance
optimizations; no latency, allocation, power, or cross-engine parity claim is made, and no
additional profile is justified before managed behavior execution.

## 2026-08-28 immutable intent 与 pre-active cleanup 恢复闭环

继续复核完整 WAL 后确认了两个更低层的崩溃一致性缺陷。第一，immutable intent 原来通过可替换式
`atomic_write` 发布，而 transaction id 只有 `PID-counter`；PID 重用、遗留证据或错误配置的多个 journal
owner 可能令同名新事务覆盖尚待恢复的旧 journal。第二，pre-active abort 原来先删除 journal，再删除
staging/backup；后续 artifact 清理失败会留下没有 recovery owner 的孤儿证据。若 prepared/Active append
本身失败，尾部还可能不完整，此时继续追加 cleanup transition 同样不安全。

实现现已硬切为三条不变量：immutable intent 使用 OS create-only 的 `atomic_write_new`，同名 journal
返回 `AlreadyExists` 且原字节保持不变；私有 WAL 新增只能从 `Intent` 进入的 `CleanupIntent`，live
namespace 从未发布时按“持久化 cleanup phase -> 清理 artifacts -> 删除 journal”执行；调用方显式传递
`journal_append_safe`，prepared/Active append 失败只保留完整 intent/artifacts 交给 restart 的 torn-tail
truncation，绝不在不确定尾部追加或开始清理。restart 也必须先成功记录 `CleanupIntent`，append 失败时
保持 journal 与 staging 不变；已进入 cleanup phase 的部分清理允许缺失私有 artifacts，但仍验证已 prepared
文档的 live target/retirement 保持原代。

三个聚焦 TDD 门依次在旧 `atomic_write`、缺失 `CleanupIntent` 和缺失 `journal_append_safe` 处 RED，随后
转为 GREEN。当前完整 Python 架构 guard 为 `14/14` GREEN（10.080 秒），精确 Rust 2021 rustfmt 与 scoped
diff-check 为 GREEN。新增 Rust 行为源码覆盖“同名 immutable intent 不覆盖旧证据”、“artifact 清理失败时
journal 保留”和“restart cleanup transition 失败时完全不清理”；这些 Rust tests 尚未执行，且本轮没有
managed Cargo、性能、功耗或产品帧证据。public `core::resource::io::{atomic_write,
atomic_write_new}` facade 与 IBL consumers 均未改变。

## 2026-08-28 atomic publication 与 WAL 最终结构复核

继续下钻 `zr_resource::io::atomic_file` 后发现，事务层的 fail-closed 规则还没有覆盖底层 publication
primitive。原实现仍在若干位置使用 `Path::exists`/`Path::is_dir`，Unix backup fallback 的 `fs::copy`
能够覆盖既有 evidence，Windows `ReplaceFileW` 传入了官方明确不支持的
`REPLACEFILE_WRITE_THROUGH`，missing-target staged replacement 还会在竞态中退化成可替换发布。修复统一
引入 `PathEntry::{Missing, File, Directory, Other}`，只把 `NotFound` 视为缺失；backup 通过 hard-link 或
`OpenOptions::create_new` copy 创建并先完成 file/parent durability barrier；Windows 在独立持久化 backup
后调用 `ReplaceFileW(backup=NULL, flags=0)`；缺失目标始终使用 OS no-replace publication。Windows restart
recovery 只接纳完整保留命名规则且为 regular file 的 backup candidate。

独立 review 随后发现 C1：durable `commit_file` 在 publication 前设置 `staged.committed=true`。准备时缺失
的目标若由外部进程在 publish 前创建，底层虽然正确返回 `AlreadyExists`，rollback 却可能删除该外部
文件；若目标更早出现，动态 `PathEntry` 分支甚至会直接 replace。修复不再从 commit-time path presence
推导模式，而是使用准备阶段的 `target_existed`：expected-missing 固定走 create-only；内部 typed error
明确区分 `NotPublished` 与 `MayHavePublished`，只有成功或后者才设置 rollback eligibility。新增回归测试
证明 pre-publication conflict 返回冲突、`committed` 保持 false，且外部字节不被覆盖或删除。测试输出根
也统一为 `ZIRCON_TEST_OUTPUT_ROOT`、`CARGO_TARGET_DIR`、workspace `target` 的既有顺序，不再硬编码仓库
输出目录。

TDD 静态门先在缺失 publication state/transaction-aware create-only contract 处 RED，最终完整 guard 为
`15/15` GREEN（13.891 秒）；独立 reviewer 在最终四个关键哈希上复跑同一 guard 为 `15/15` GREEN
（17.990 秒）。完整 atomic/transaction Rust 源码的 Rust 2021 `rustfmt --check` 为 GREEN。最终独立复核
为 `C0/I0/M0`, `Ready`：atomic transaction
`dd8237bea05ca8aa05f99f5dab54d97eda52cc8898042ab02fa238405c4c3368`，transaction commit
`9d33dec63db6b4157d2053c955c71c21ed6669de92ffc81e215dcd41dda368cd`，engine tests
`53407d24f721c3177864f437166ad76cc406a9be8878d77b46e6ace6fb2e8cea`，Python guard
`1c631ea90cb4d827e8c18b02b41c9178e400114129b94dcf78ee4034bb9b0abc`。

受管 Windows Cargo 已提供以下聚焦行为证据，全部产物位于 coordinator-managed D 盘 target：immutable
intent create-only 通过；两个 pre-active abort/uncertain-tail 用例通过；真实 multi-file
`one_generation_can_durably_retire_a_source_and_its_sidecar` 通过；C1 回归
`prepublication_conflict_preserves_external_target_and_skips_rollback` 通过，最后一项含编译耗时 189.3 秒。
一次完整 `zr_resource --lib` 受管运行曾返回 wrapper exit 101，而同一 D 盘 test binary 的只读诊断执行为
`159 passed / 0 failed / 3 ignored`；直接 binary 不是 acceptance receipt。最终 atomic-module 请求
`0fe992d83b234d96857d704c542e05fc` 的协调器终态为 `failed/cargo_reuse_pool_busy`，兼容池由作业
`93aad82c12fa4f6a9d74e9c8c8d03c22` 占用，因此该请求没有启动 Cargo，不能登记为失败测试或通过证据。

Runtime25 `FILESYSTEM-P1-027`/`P1-029` 仍负责 handle-relative open/create 与非协作进程的内容级 TOCTOU。
本切片只保证 cooperating engine transaction 与明确观测到的 pre-publication conflict，不宣称
`PathEntry` 是 filesystem CAS，也不声称消除了 external existing-target/retirement mutation。此次是
正确性和恢复状态机修复；没有 latency、allocation、功耗或跨引擎性能对标数据。

## 2026-08-29 expected-missing concurrency oracle correction

The latest coordinator-built full `zr_resource` diagnostic failed
`concurrent_missing_staged_replacements_have_exactly_one_winner`. Reusing that exact D-drive executable for 50
isolated repetitions produced 9 failures (`18%`), each with two successful outcomes. Independent review established
that this did not test the durable transaction contract: both workers called the public `replace_staged_file` helper,
whose `Barrier` is reached before its own `path_entry` observation. After the first worker publishes, the second can
legally observe `PathEntry::File` and enter the existing-target replacement branch. The barrier therefore synchronized
worker entry but did not freeze the helper's branch authority. The result is valid replace-helper behavior and is not
evidence that Windows `MoveFileExW` violated no-replace publication.

A temporary Windows hard-link production change based on that invalid conclusion was withdrawn before managed
validation, candidate creation, or commit. The final production source retains the reviewed platform split and returns
to atomic transaction hash `dd8237bea05ca8aa05f99f5dab54d97eda52cc8898042ab02fa238405c4c3368`:
Unix uses `hard_link`, while Windows uses `MoveFileExW(MOVEFILE_WRITE_THROUGH)` without
`MOVEFILE_REPLACE_EXISTING`. No production atomic change survives this follow-up.

The corrected regression is named
`concurrent_expected_missing_transaction_publication_has_exactly_one_winner`. It runs 32 independent two-thread
rounds and calls `publish_staged_file_for_transaction(..., false)` directly, fixing the transaction's prepared
`target_existed=false` authority before the barrier. Every round requires exactly one success and one `AlreadyExists`.
The current atomic-test hash is
`6d72f1255be5e73216b4a2fd037911472a6f0093bc58fa7f16950d846c0b3640`; the corrected Python guard hash is
`42158146ca2e558ab0fef7f251d1bd639c5ed113b61d7ca7fde3beaeb94ec5fc`. The prior static result targeted the
withdrawn oracle and is superseded. The corrected architecture guard is `15/15` GREEN in 14.401 seconds; exact Rust
2021 format and trailing-whitespace checks are GREEN. Independent re-review matched all seven declared atomic/manager
hashes, independently passed the same 15-case guard and exact format check, and returned `C0/I0/M0`, Ready for managed
Cargo validation. No Cargo or file write occurred in review. Latest managed request
`ae6177392b9b41d6a666b9518bbef145` ended `cargo_reuse_pool_busy` before Cargo because job
`a75927364a084b159716eb83ffbcfa88` owned the compatible target. A later no-PID coordinator runner completed job
`7f6e2375e1064fc78cae255540ac4e2e` / run `cd33a5f667874bdca61bbeae9d50ec97` with exit 0: current-source compile
took 3m15s and the corrected managed filter passed `1/0/169` in 0.22s. Its D-drive executable SHA-256 is
`8800bbbc17595aa13c4948d9dfa4bb400a13dfa3cb2f943d199ecec6ec7e2ea0`; coordinator stdout/stderr hashes are
`3abb43d380415c55b353f415275b066b57fd07aa19758505a4d56f045c8a936a` and
`d3ff3e089ceed2aa4a91002afe446298c167207f5b746e0c2672b9b5369ef53e`. The exact binary then passed the full
library directly with `167 passed / 0 failed / 3 ignored` in 2.26 seconds. That direct full run was diagnostic evidence
and is superseded by the final managed full-library receipt below. This correction makes no latency, power,
product-frame, or cross-engine parity claim.

## 2026-08-29 current-source managed Resource receipt

The first warning-hard-cut Resource snapshot received a complete managed Windows test-target receipt. Coordinator job
`8b31357dce8a42f89e0c51212d3774fc` / run `1cde4a6add6542b7916f6dc2bb16a5af` executed
`cargo test -p zr_resource --locked --lib` against the D-drive compatible pool and returned
`167 passed / 0 failed / 3 ignored` in 3.27 seconds after a 53.73-second build. The job exited 0, released at
03:41:39 Asia/Shanghai, and has no live process. Its 5,412,352-byte executable SHA-256 is
`d901c0e372ecfa7b1c1714b402e3e224b335176a5121db0cbfea60969242ea14`; immutable stdout/stderr hashes are
`f41648a67514424b8086a441d6349ec83e0b2305c6cf4fa101a4d2fec64df90c` /
`43ea5f6b4a03a346e668d0d9ff18c136bdecc231940ccd39fa79147830075a78`.

The initial physical warning-boundary hard cut removed only compiler-confirmed unused crate-private/root aliases and
one zero-consumer projection method. Independent read-only review matched all four changed hashes, passed the
crate-boundary guard `5/5` plus exact Rust 2021 format, found zero product consumers of the retired paths, and returned
`C0 / I0 / M0`. That test-target compiler emitted zero `zr_resource` warning summaries. The later Editor upward
compile correctly built the normal Resource library and exposed three production-only warning groups, so the earlier
warning-free statement was not treated as final production evidence.

Exact whole-crate consumer review showed five helpers are test-only:
`ResourceEventLogEntries::{is_empty, values}`, `ResourceManagementRow::from_record`, and
`ResourceManagementGeneration::{from_rows, from_sorted_rows}`. Production uses indexed event operations,
`from_record_reusing_identity`, `from_parts`, and `from_sorted_rows_with_hash_authority`; no cross-crate or
test-support consumer exists. The final hard cut marks only those five helpers `#[cfg(test)]`. Exact hashes are
`5ab275d50d501e48b82b6133ac5c8d95ba6e53440da1637e1acbd4b17e212280` for `event_stream.rs` and
`333e8d82759576fd8a10dfa236fe184b8b4b9caf08c50e4d917eb2c7aa62bf79` for
`management_generation.rs`. Independent follow-up review rechecked all consumers and replacements, passed exact
format plus the crate-boundary guard `5/5`, and returned `C0 / I0 / M0` without writes or Cargo.

Final normal-library check job `a86b9a1cc126443db28241b511870863` / run
`791f01d779254d56b153377a87b6d79a` executed `cargo check -p zr_resource --locked --lib`, exited 0 after 2m31s,
released with no live process, and emitted zero `zr_resource` warnings. Immutable stdout/stderr hashes are
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` /
`282b989fc798470bc169f042b8146438353f42c1cb6e3715e45c5bb09be771cb`.

Final full-library job `1dcb8301b7b047d58f246c823fe6b42d` / run
`0b8fc25d8a52467d9c7013e24ace3972` then returned `167 passed / 0 failed / 3 ignored` in 2.69 seconds after a
2m10s test-profile build. It exited 0, released with no live process, and emitted zero `zr_resource` warnings. The
5,412,352-byte executable SHA-256 is
`86e85e0fe4f32107dac75cb119fb0547620a23ad9058cce45a08062576a91f8d`; immutable stdout/stderr hashes are
`b5e8bba492d79814147c1555394f9d1ca96ed16fb78094d79d806442ab2f1714` /
`161a40f0ae9e19fb3e66d05e36b4025514770853200b353808ee330d2e968f37`. The remaining eleven detailed warnings and
one summary belong to the foreign `zircon_runtime_interface` shared source. The complete 15-case conditional-write
architecture guard passed again in 14.540 seconds, and the final Resource crate-boundary guard passed `5/5` in 9.128
seconds.

This receipt closes the Resource library execution gap and supersedes the earlier direct-only full-library evidence.
Editor upward job `41919258d25041c39d9c74d57d4c989f` / run `b6657885fc6944009fcf4ad2d43dd75a` did not reach
the Editor06 authority tests: it stopped on the single foreign E0004 at
`zircon_runtime_host/src/foreign_output/item_count.rs:80`, whose match does not cover
`WorldQueryResult::TransformSnapshot`. Diagnostic owner hashes are
`2782a5a5a3533762b0a1d435b2b6faf1cd6b6dfce4b353901f0eb39aa1c4bf27` for `item_count.rs` and
`56f357dddd79e119ca894b195966658025731b67bc102377a386fda62a7aaa47` for
`zircon_runtime_interface/src/world_sync/query.rs`. This slice does not edit those foreign owners. The canonical
handoff therefore remains open until upward Editor authority validation and coordinator Failure closeout/return are
accepted; no fixed artifact, milestone commit, WeCom notification, performance, power, or product-frame claim is made
here.

## 2026-08-29 latest lower-layer repair validation

本轮继续保持该跨计划 Failure 为 `open`，但已完成其最低共享层的两个防御性收口。恢复校验在访问
`documents[0]` 前拒绝空 folded journal；pre-active CleanupIntent transition 失败时保留原始
`DurableTransactionError::Operation` 的 phase、exact path、`io::ErrorKind` 与 source chain，并把 transition
故障作为私有 context 附加。intent 入口拆为纯 `plan_intent` 与锁内 `persist_intent`：全部 normalized
live/artifact/owner-lock/journal namespace antichain 校验发生在 journal directory 创建、owner lock、pending
scan 和 WAL 写入之前；`create_intent` 仅保留 `#[cfg(test)]` wrapper，不构成旧架构兼容入口。

证据：独立 reviewer fresh result 为 `C0/I0/M0, Ready`（engine `26c0cb4...`，intent `7781ec21...`，journal
mod `4238c926...`，后续 error-chain engine `fd09e104...`）；architecture guard
`python tools/tests/test_frameworks_01_resource_conditional_write_authority.py` 为 `15/15 OK`；exact Rust
2021 rustfmt 与 `git diff --check` 为 GREEN。受管 Windows focused test `pre_active_abort_preserves_original_operation_when_cleanup_transition_fails`
在 job `0b6ac465c557498f98c6f3075329668e` 的 D 盘 lane 通过；修正 Windows 普通 Win32 lexical-alias fixture
后，`lexical_target_alias_in_journal_is_rejected_before_recovery_io` 在 job `1e9425f870c047c88428f4f8dcd636d4`
通过；完整 `zr_resource --lib` job `dc58a408ca7143dba2d5fb606163e47e` 为 `191 listed / 0 failed`，production
`cargo build -p zr_resource --locked` job `79f813a207844ec2bdb5ba2a41b070b2` 为 exit 0。所有编译和测试
target 均在 D 盘 managed pool，未向 C 盘写入产物；foreign `zircon_runtime_interface` 的 11 条 warning
未由本 Session 吸收。

该 lower-layer gate 不等于整个 Failure 已 return：Editor06 upward authority test 仍被 foreign
`zircon_runtime_host/src/foreign_output/item_count.rs:80` 的 `TransformSnapshot` exhaustiveness error 阻断，
且尚未完成完整 process-kill/every-transition crash matrix。因此当前仍不声称 fixed artifact、里程碑提交、
企微同步、性能/功耗或跨引擎 parity；待 foreign owner 收敛后由 origin gate 重新验证，再走 coordinator
`failure return` 原子移动。

## 修复结果与回传

Open. Architecture/root-cause review, production implementation, typed identity hard cut, raw
recovery-entry validation, fair meta admission, the `10/10` static guard, exact format/diff checks,
legacy-key removal, file-budget split, atomic/WAL fail-closed publication, immutable intent, and
pre-active cleanup ordering are complete. The earlier atomic snapshot received `C0/I0/M0`, and
multiple focused managed Rust behavior tests pass including the C1 conflict-preservation case. The
apparent Windows double-winner was a test-oracle defect: the oracle now targets the transaction's
expected-missing publication branch directly, and the production atomic snapshot remains
`dd8237bea05ca8aa05f99f5dab54d97eda52cc8898042ab02fa238405c4c3368`. Because the test owner changed,
the corrected source was independently re-reviewed at `C0/I0/M0`, and the corrected static guard is `15/15` GREEN.
The final production check is warning-free and the final managed Resource library passes `167/0/3`, superseding the
earlier direct-only and pre-follow-up full-library evidence. Runtime25 owns the explicitly documented handle-relative
TOCTOU residual. Current Editor06 upward authority execution is blocked by the foreign runtime-host exhaustiveness
error; fixed return, milestone commit, and coordinator-owned WeCom synchronization remain pending. Current state is
`source_repaired / corrected_oracle_static_green / independent_review_green / production_check_green /
current_source_managed_resource_library_green / editor_upward_blocked_by_foreign_current_source /
fixed_return_pending`;
this record makes no performance, power, milestone, or product-acceptance claim.
