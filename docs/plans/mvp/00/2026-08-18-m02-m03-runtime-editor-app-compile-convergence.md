---
record_kind: milestone
status: validation_pending
created_at: 2026-08-18
plan: docs/plans/mvp/00-current-source-baseline-recovery.md
milestone: M0.2-M0.3
---

Plan: `docs/plans/mvp/00-current-source-baseline-recovery.md`

Milestone: `M0.2-M0.3`

Status: validation_pending

# Runtime04 resolver and Runtime-to-Editor-to-App convergence

## Scope Delivered

- Converge 54 Runtime lib-test compile errors caused by current public API hard cuts across
  service admission, weak core handles, ECS storage/query/schedule, detached entity batches and
  dynamic scene components.
- Keep the Runtime04 migration resolver on its generation-owned lookup boundary; the compile
  convergence does not add a filesystem fallback, compatibility alias or upper-layer bypass.
- Publish `EditorHostRunConfig::with_hub_handshake` as the single documented cross-crate builder
  and add an App-side compile contract for the real editor-host composition boundary. Editor-owned
  behavior tests verify both the no-handshake default and the mailbox path derived from the exact
  project root plus session token without exposing the retained-host handshake DTO.
- Repair the current-source Runtime04 acceptance regressions exposed only after the compile
  blockers were removed: report display paths no longer leak Windows verbatim prefixes,
  authoring documents precede newly minted sidecars in the durable transaction, and scale/fault
  fixtures count both transaction artifacts without weakening resolver or rollback contracts.
- Parse flattened material texture references independently from sibling slot metadata such as
  `fallback`, `uv_channel`, and `transform`, so strict persisted-reference decoding retains both
  the reference and the non-reference fields instead of silently treating the full slot as extras.
- Remove the remaining allocation from the common unlabeled resolver lookup: normalized
  `res://` keys are borrowed directly, while only labeled subasset locators allocate a stripped
  base key. Generation construction also reserves its physical-path, forward and reverse maps
  from the known projection/binding cardinality instead of repeatedly growing at 100K entries.

## Performance Evidence

- `MigrationResolverIndex` owns forward and reverse `HashMap` indexes; each post-generation
  resolver lookup is average `O(1)` and does not walk roots or access the filesystem.
- The existing 100,000-entry resolver matrix now emits separate `RESOLVER_INDEX_BUILD` and
  `RESOLVER_INDEX_LOOKUP` samples for one and four roots, including 1/1K/100K references and
  2/2K/200K forward-plus-reverse lookups. The samples retain the zero-filesystem-probe contract
  and separate one-time generation construction from steady lookup cost.
- Unit contracts pin the key ownership rule: an unlabeled lookup returns the exact borrowed
  locator, while a labeled lookup owns the correctly stripped base locator. Existing labeled
  migration/reload coverage continues to protect subasset semantics.
- At 100,000 references and four roots, the retired path could visit up to 400,000 root candidates
  and enter filesystem-backed source resolution. The replacement acceptance case produces
  200,000 forward-plus-reverse results through generation-owned map lookups with zero resolver
  filesystem probes.
- The current acceptance matrix covers `1 / 1,000 / 100,000` references with `1 / 4` roots and
  verifies exact forward plus reverse result counts.
- Production migration counters require `resolver_filesystem_probes = 0` and
  `full_value_clones = 0`, with exactly one read and one parse per authoring document, for dry-run,
  apply, unchanged and one-percent-change phases.
- The managed sweep emits one `MIGRATION_SCALE` sample for every 1/1K/100K file, reference and
  directory case plus the four-root case. Each sample reports migration-only elapsed milliseconds
  together with the production visit/read/parse/probe/clone/output-byte counters, so terminal
  validation and the WeCom commit notice can carry measured scale data without treating fixture
  construction time as resolver latency.
- Directory-scale cases make the zero resolver-filesystem-probe and zero full-value-clone values
  hard assertions at 1/1K/100K, rather than relying on emitted diagnostics alone.
- The first current-source managed run compiled the complete Runtime lib-test harness and executed
  all 67 Runtime04 tests in 12.75 seconds: 52 passed, 14 failed, and the one 1/1K/100K managed
  sweep remained intentionally ignored. All 9 resolver-index tests passed, including the
  filesystem-free generation and 1/1K/100K reference plus 1/4-root matrix contracts.
- That first run was a cold isolated build, not a resolver benchmark: build plus test wall time was
  55 minutes 16.91 seconds and the observed peak `rustc` working set was approximately 9.68 GiB.
  These figures are retained as validation infrastructure cost and are not presented as hot-path
  migration latency.

## Static Testing Evidence

- `rustfmt --edition 2021 --config skip_children=true --check` passed for all 106 lease-bound Rust
  paths after the Runtime, App, Editor harness and resolver-performance repair batches. Restricting
  child traversal keeps the formatter check on the exact Session manifest instead of walking into
  unrelated dirty module trees.
- Scoped `git diff --check` passed; reported messages are only the repository's LF-to-CRLF
  checkout warnings.
- The Hub builder source guard confirms the public method exists and the retired `pub(crate)`
  visibility does not remain.
- Resolver source guards confirm one generation build call and no filesystem API or retired
  persisted-source fallback in the resolver/index lookup boundary. A repository-wide static trait
  census confirms all 14 `EditorRuntimeGateway` trait implementations expose submit, poll and harvest;
  Editor integration tests no longer import through private retained-host root modules.

## Managed Testing Evidence

- Runtime04 first source-copy job `7bd9cae7d5c646329df44099b331e586`, run
  `e1aee22e308e443da0b476484b21f7fc`, completed naturally with exit 101 and no remaining
  Cargo/Rust process. Compilation succeeded; the 14 test failures identified Windows report-path
  presentation, document/sidecar transaction ordering, stale sidecar cardinalities, two malformed
  current-reference fixtures, and one flattened texture-slot reload regression for follow-up.
- Replacement batch source-copy job `0a2547c8b7e84fd4b3f08d19c5b9de6f` failed during external
  source closure planning before Cargo started because its sibling descriptor was encoded as an
  array instead of the required single object. No compile or test result is attributed to it; a
  corrected 22-file batch replaces it.
- Corrected batch source-copy job `c2385ffc1b4246068428747133ccdefa` was accepted with the
  complete 22-file overlay, but stopped before Cargo at the owned-overlay gate because one
  post-attribution edit made the overlay hash stale. All 22 source paths were then renewed and
  attributed together before scheduling its successor.
- Successor batch source-copy job `7f61b837316e4ea7a652511a290fbfd4`, run
  `dc336f73ca8949008507732a35b01ffa`, compiled `zircon_runtime`, `zircon_editor`, `zircon_app`,
  the editor-host plugin closure and the Runtime04 production changes before stopping naturally
  with exit 101 in the App integration-test target. Its only remaining diagnostics were eight
  current-source errors in `zircon_app/tests/editor_mvp_authoring.rs`: that test still accessed the
  retired `ProjectManager.project_info/world` fields and called the removed
  `EditorProjectDocument::load_from_project` helper.
- The App integration test now reads the same activated project generation through the public
  `ProjectInfo::from_project` and `Scene::load_scene_from_uri` contracts. It does not restore the
  removed aggregate fields, expose Editor's test-only loader or add an upper-layer compatibility
  path. The resulting 23-file Rust scope passes rustfmt and scoped diff checking.
- Every source-copy job pins sibling `zr_vm` commit
  `503fb72163cd20ddf32a38f8a330083712f5d648` and includes only its binding and sys crate roots.
- Prepared scale copy `254ce9aab178476386c1a37b728b9bc2` and Editor product copy
  `fba07069b88145a89cc897c186447084` never started Cargo and were superseded when the App test fix
  changed the source fingerprint.
- Unified successor copy `7c2f7822b5cb4e37b69fa69b2be30eb8`, run
  `8725d2e09a6d4b8db074bf83688eec5b`, used manifest
  `7c48db06867cd0e07e23d22a9c4bfb96b52a4f4eeff895842caee538c709587f`. It compiled the repaired
  `editor_mvp_authoring` target and reached the App lib-test harness before the first stage stopped
  naturally with exit 101 after 2,291.352 seconds. The only remaining error was a test-only
  `EditorStartupPreparation` destructure that did not mention the newly added `startup_metrics`
  field; the later product/focused/default/scale stages correctly did not start.
- The App entry owner now conditionally ignores exactly `startup_metrics` in that destructure when
  `cfg(test)` is active. It does not add a wildcard pattern, so future production fields continue
  to make the startup composition fail compilation until they are handled deliberately.
- Unified 24-file source-copy job `4bc5c40df9224a3aa69f641e4cffbd7f`, run
  `a7056f5be1544e5688e258915987d6f8`, used manifest
  `b1fbdca1401d6a31c8782a2a036126ee4ee02405dc5abe9fd35cd682b6f8db38`. Runtime, App,
  `editor_mvp_authoring`, and the repaired App lib-test target compiled before the first stage
  stopped naturally in the Editor lib-test harness with exit 101 after 2,617.538 seconds. The
  remaining four stages did not start.
- The retained 64 KiB diagnostic tail contained 15 actionable Editor errors from the larger
  current-source test-harness backlog: five sibling-helper scope failures, six private-path
  imports, and four `EditorRuntimeGateway` test doubles missing the required operation methods.
  The helpers now have explicit test-scope imports or re-exports, tests use the retained-host
  public crate surface, and the four gateway doubles return the same capability-missing contract
  as the existing operation-aware doubles. No production compatibility path was restored.
- Diagnostic source-copy job `501f244a7bfe4f19a5d838a3964d6095`, run
  `aca88e21dccc42d2b38d6c51c48cbf99`, completed naturally with exit 101 after 2,918.765 seconds.
  Consolidated request `c995fb520d704277b9d859f55687c276` stopped in the first stage with 217
  diagnostic lines: 216 short-format compiler entries across 57 Editor files plus the terminal
  crate error. No product-build, focused App, default-check or scale stage started, so this run
  contributes no performance result.
- The complete short-format output corrected the earlier retained-tail interpretation. Its 89
  distinct messages converged into test-child helper visibility, explicit hard-cut imports,
  moved test-only types/functions, two lifetime annotations, one missing command accumulator and
  seven identical descriptor/property closure-capture errors. The repair uses test-local imports,
  `pub(super)` helper visibility and the current owner paths; it does not add an upper-layer alias.
- The two `E0106` helpers now bind returned text to the node or presentation input explicitly.
  Neither repair introduces an owned clone, leaked value or static workaround.
- Snapshot `1819` binds all 107 milestone paths to baseline epoch 333 after the batch repair.
  Exact-path rustfmt and scoped diff checking are green; a successor managed source copy is still
  required before any compiler, test or performance acceptance claim.
- Successor source-copy job `1b4923e9067f4621891ff210aeb7e244`, accepted by request
  `47eb976ea53b447e8177ac082c0afe9f`, failed during `overlay_ownership` because its asynchronous
  worker found 62 of the 106 Rust paths without current Session content-hash attribution. Cargo
  never started, so the job contributes no compiler, test or performance evidence.
- All 107 milestone paths were reclaimed without conflicts. First replacement source-copy job
  `0539e4b255054f18b17d39116a9bf231` was accepted asynchronously by request
  `603bacd7137c4308bd4513093113e363`, but failed at the same gate. This proves lease refresh alone
  was insufficient; it also contributes no Cargo evidence.
- The Session then refreshed content-hash attribution for all 107 milestone paths and bound them in
  snapshot `1823`. Second replacement job `626268c05ed642e19ba36cf0b36d3b00`, accepted by request
  `25f95ce7de434c689007c49e247f0808`, overlays the 106 Rust paths and pins sibling `zr_vm` commit
  `503fb72163cd20ddf32a38f8a330083712f5d648`. It materialized 17,852 closure files with input
  manifest hash `82f20740ad3cd1801bd7589752b966ba618323a51facc5f82c8149f0edd5d9f5`.
- The unified five-stage run was submitted against that copy. Its client command timed out before
  returning an immediate request receipt, but the coordinator reports the copy as `running`; no
  duplicate run was submitted. The terminal run ID, compiler/test results and performance samples
  remain pending evidence.
- While the managed run remained active, a fresh SHA-256 audit compared the worktree with snapshot
  `1823`: all 106 Rust paths still match the materialized source exactly. Only this delivery record
  differs because it continues to receive asynchronous validation notes.
- The exact 107-path snapshot set has 97 changed code paths plus this record and nine clean closure
  inputs. `git diff --check -- <snapshot paths>` exits 0; no broad directory status is used as a
  submission manifest because unrelated Sessions currently modify hundreds of Editor paths.
- Unified source-copy job `626268c05ed642e19ba36cf0b36d3b00`, run
  `865f36c2eef64270bccb7c9f06cc66c4`, completed naturally with exit 101 after 3,127.249 seconds
  in the first Runtime04-plus-editor-host stage. The full short-format stream contained 212
  compiler entries across 62 Editor paths; the remaining four stages correctly did not start.
  This terminal run contributes no test or performance acceptance result.
- The 212 entries were converged as one support-first batch rather than validated one-by-one. The
  repairs update sibling jobs tests to test-only lifecycle probes, consume non-`Copy` tickets and
  retained frame data explicitly, bind `Arc` snapshots before borrowing, use current
  authoring-world/resource-generation/plugin-catalog contracts, and remove retired startup-session,
  runtime-level and retained-pane test assumptions. The workbench property mutator remains private
  in production; only a `cfg(test)` crate-visible forwarding method was added for the external test
  module.
- The expanded validation manifest now contains 161 lease-bound paths: the previous 107 milestone
  paths, the newly exposed Editor compiler owners, lifecycle records, and one test-only bridge
  owner. All 155 Rust paths pass exact-path `rustfmt --check`; scoped `git diff --check` exits 0.
  Unrelated dirty paths owned by other Sessions remain outside the source manifest and submission
  scope.
- Snapshot `1827` binds the complete 161-path batch to baseline epoch 333. Successor source-copy
  job `3df1e76f63a74736847b2da11400078f` was accepted asynchronously with all 155 Rust overlays and
  sibling `zr_vm` pinned at `503fb72163cd20ddf32a38f8a330083712f5d648`; Cargo closure
  materialized 17,852 closure files with input manifest hash
  `415e317277caa8305ebf93a57d17d60bf26e7c7fbf96e7534adfac565548130d`.
- The unified five-stage validation request `263d9c31f9c14e7bbb261921526449cf` was accepted against
  that durable copy. Run `e3e10d14a4e742dc881a8fc1d2ba04e9` completed naturally with exit
  101 after 2,303.405 seconds in the first stage. The exact terminal stream contained 11 Editor
  test-harness errors across six files; the product build, focused App, default three-package check
  and exact scale stages correctly did not start, so this run contributes no performance sample.
- The 11 errors were repaired as one support-first batch: pending-decision tests now distinguish
  public option IDs from core ticket selections, dynamic inspector fixture values satisfy the
  owned-string contract, document round-trip assertions read generation-resolved locators,
  capability toggles borrow their identifier, and event assertions keep journal snapshots alive
  while borrowing records. No production compatibility path or hot-path clone was introduced.
- Snapshot `1828` binds the full 168-path Session scope after the batch repair. Successor source-copy
  job `1dbbbfd1345b4d69b4d0ec5ac963d81c`, accepted by request
  `1e7fb8e12677428bb896444539a886c7`, overlays the same 155 Rust paths and pins sibling `zr_vm` at
  `503fb72163cd20ddf32a38f8a330083712f5d648`. It materialized the current-source closure with input
  manifest hash `103e094b1764f4bab2b885ea799333db8f0cfcca7b2d364e3375f8e775ad1f6a`.
- Unified five-stage request `ae562853c44244089c2cefac334d14b1` was accepted against that durable
  copy. The client reached its 15-second `command_post_timeout` reconciliation boundary while the
  coordinator reported the copy as `running`; no duplicate run was submitted. Run
  `674ffcfa89c9488f9a85afe81000f42e` later completed naturally with exit 101 after 4,056.921
  seconds in the first stage. Its only compiler error was an Editor cross-crate unit test directly
  constructing the now-owner-private fields of `NativePluginLoadReport`; the remaining four
  stages correctly did not start, so the run contributes no product or performance acceptance.
- The Editor test now creates a temporary native editor package manifest and obtains its missing
  DLL failure through `load_discovered_native_editor_plugins`. The registration assertion consumes
  the real package id, `library-open` stage and `artifact missing` diagnostic. It adds no public
  report constructor, compatibility API or test bypass; exact rustfmt, diff checking and a source
  guard confirm that the private report literal is gone.
- Snapshot `1830` binds the complete 173-path M0.2-M0.3 batch after that repair, including 156
  exact Rust paths, the nine F0 acceptance overlays and the three open failure records. The repaired
  native-registration test owner has SHA-256
  `3cf5b6baca7c2b97568ac2edd67ab9689d7d37fd69770937ed223e77a4743d28`.
- Source-copy job `85c16f1adb1349ebb4c372958ec1cdb6` failed during pre-Cargo closure planning
  because the validation target root was mistakenly supplied as an additional source `--path`.
  Cargo never started, so this is coordinator invocation evidence rather than a compiler, test or
  performance failure.
- Replacement source-copy job `77799ccc2c524af2a211aac189a03f2b`, accepted by request
  `00e9220946c4464b9d24a4c53c7f8993`, pins sibling `zr_vm` commit
  `503fb72163cd20ddf32a38f8a330083712f5d648` and its two binding crate roots. The coordinator
  materialized a 17,852-file closure with input-manifest hash
  `103e094b1764f4bab2b885ea799333db8f0cfcca7b2d364e3375f8e775ad1f6a`. A mandatory
  156-path post-materialization hash audit rejected the copy before Cargo: 155 Rust paths matched,
  while `native_registration/manager.rs` still contained its pre-repair bytes because that path's
  Session content-hash attribution was stale. No run was submitted against the stale copy.
- The native-registration path was re-attributed under its live lease. Successor copy
  `fcd64a75e19b41569d8fec9f9cb49cde`, accepted by request
  `8ad019eaf6204e209f3b27b37b4a54f8`, began materializing the same pinned closure asynchronously.
  It was superseded before Cargo when performance review found that the scale gate only placed weak
  lower bounds on inventory visits; no run was submitted against that copy.
- The scale acceptance owner now requires exact entry visits, directory reads and directory sorts
  for all nine executed 1/1K/100K file/reference/directory cases, the four lifecycle phases and the
  four-root case. A duplicated inventory walk or directory read can no longer pass by merely
  emitting a larger counter. Weak visit assertions are absent and the nine scenario call sites all
  use the same exact-generation helper.
- Snapshot `1831` binds the resulting 173-path source at baseline epoch 333; the strengthened scale
  owner has SHA-256 `3f44f5d179942e3e859c7a80d1718e8b000950b8b5080e11a1ad22767d249993`.
  Source-copy job `bceb399278af466098a32cead7d3a268`, accepted by request
  `d1b362e37ddf43908c796dd70228bab1`, materialized that exact source with input-manifest hash
  `bd4fbc4660303cbee99d85ab32da575cefed26987aa8e7c350b6c669dc92c0fc`. The mandatory
  post-materialization audit compared every one of the 156 Rust paths against the live worktree;
  all 156 hashes matched and no stale overlay was submitted.
- Unified ten-stage request `e4a4e259749d401b84fef80ff28aadec` was accepted against that durable
  copy. Its script SHA-256 is
  `E6D718BB9F7BA5F18741E4CB42B22EFF77E107919A2FF6AD799ADF8F59A9C9AE`; the batch combines the
  Runtime04 parent/Editor host compile gate, Editor and Runtime product builds, focused App host
  tests, the three-package default all-target check, and five sequential performance repetitions.
  Each repetition runs the exact resolver-index matrix and the exact managed 1/1K/100K
  file/reference/directory sweep with one Cargo worker and one test thread. The client reached its
  15-second `command_post_timeout` reconciliation boundary after coordinator acceptance; no
  duplicate run was submitted, and the terminal run id and receipts remain pending evidence.
- While that copy materialized, the complete current-source scope passed exact non-Cargo gates:
  `rustfmt --edition 2021 --config skip_children=true --check` was green for all 156 Rust paths and
  `git diff --check` was green for all 173 snapshot paths. The resolver hot-path guard found zero
  filesystem/fallback APIs in `resolver_index.rs`, and `run.rs` retains exactly one
  `MigrationResolverIndex::build` generation call.
- After the ten-stage batch was accepted, F0/App01 review found that both product binaries discarded
  the boolean result from `shutdown_process_log`, allowing a flush timeout or output failure to
  retain a successful process exit. Editor and Runtime now preserve the shutdown receipt and
  return failure with a stable `component=diagnostic_log` recovery diagnostic when it is false.
  Process-level shutdown treats an absent sink as already complete, so `zircon_runtime --help` and
  pre-initialization argument failures keep their intended result; dynamic-session lease release
  behavior is unchanged. Six follow-up owner paths pass exact rustfmt, source-contract checks and
  `git diff --check`; they were created after snapshot `1831` and therefore remain explicitly
  pending a successor source-bound Cargo batch. Snapshot `1832` binds the resulting 179-path
  source, including 162 Rust owners, at baseline epoch 333. Successor source-copy job
  `982ca39ec87a4252b090d75dd6119a97`, accepted by request
  `bf19c07dbb7f43ccad31fc1fbac3a9b7`, materialized asynchronously without starting Cargo. Its
  mandatory 162-path Rust audit found 156 matches and rejected all six newly claimed follow-up
  paths as stale baseline bytes; its unchanged input-manifest hash
  `bd4fbc4660303cbee99d85ab32da575cefed26987aa8e7c350b6c669dc92c0fc` corroborates that the new
  overlays were not included. Releasing and reclaiming the six leases did not refresh their
  separate content-hash attribution. Snapshot `1833` recorded the correct worktree hashes, but
  replacement copy `e96087d2024f4fc9976d0a3d19532c97`, accepted by request
  `87851e5ea7454f96b6157194411d8e8f`, still materialized the old six overlays. Its mandatory Rust
  audit again matched 156/162 paths and rejected the same six files without starting Cargo. The
  correct `baseline attribute` operation then refreshed all 179 owned paths; snapshot `1834` binds
  those hashes at baseline epoch 333. Successor copy `4f187d8b5647483ea1555d79411c4f15`, accepted by
  request `fd6a96286d3a49f483a27b7aaf33d13b`, materialized with the distinct input-manifest hash
  `c8c751fc482ee14d64a42d3253e48c4b71311d92f223d0ad139c15cfd0464cae`, confirming the six F0
  overlays entered the copy. Independent transaction review then found that the Runtime04 capacity
  change had appended sidecar writes after authoring documents even though the durable transaction
  engine preserves caller order and the existing sidecar crash-window test requires index zero to
  publish the sidecar. The worktree restored sidecar-first publication while retaining exact
  preallocation. The mandatory audit consequently matched 161/162 Rust paths and rejected only
  `run.rs`: copy SHA-256 `f9580bdfbb0d09a1d8efe8302c4c874848962f9052d60627a5df326195a4c7f9`
  versus repaired worktree SHA-256
  `f7b8ab06ec67520e21c7d80e18d315dff22b3c288a1745b97a25367649a95a1e`. No second Cargo run was
  started while the ten-stage batch was active.
- Current-source successor copy `3a8faca39487478dbdcc48b3ea511972`, accepted by request
  `3d09adf41a6641e4b61650824c60e519`, materialized with input-manifest hash
  `332a6853d35b6952506c215a91a03eb63d866ec12df3a5805e917b9d7ee58e15`. Its mandatory audit
  matched all 162 leased Rust paths against the live worktree, including the repaired sidecar-first
  `run.rs`; it remained idle while the first Cargo batch owned the validation lane.
- The first ten-stage copy then completed naturally as run
  `695ba161c7a24965bc0d4178110a3253` with exit 101 after 4,438.343 seconds. The first stage stopped
  after 4,438.171 seconds on two pre-existing IBL integration-test API drift errors: the render root
  omitted `IBL_BAKE_ARTIFACT_PAYLOAD_CHECKSUM_SIZE`, and the test still called the removed arbitrary
  `IblBakeArtifactDescriptor::with_producer` mutator. No product, shutdown or performance stage
  started, so this run contributes no performance sample.
- The checksum constant is now re-exported from the render owner. The integration test constructs
  its GPU runtime descriptor through `current_for_runtime_cache_request`, preserving the constrained
  producer invariant instead of restoring the retired mutator. Both repaired paths pass exact
  rustfmt and `git diff --check` and remain pending the successor source-bound batch.
- Independent F0 staging run R7 passed from detached worktree
  `E:\zircon-profiles\mvp00-f0-source-bound-01a00797-20260818` with eight exact overlays, source
  fingerprint `036C280826D4DA235CCA115BFA1105DBC78BD66FD4C70A9A88FC8BE38F7474D5`, and wall time
  156.055 seconds. The release/device-path support batch also passed in 8.228 seconds, including
  both device-path Pester cases.
- The first full acceptance pass against that batch reached the F5 retained-metadata negative case
  after 406.863 seconds. It correctly failed closed when the reopened Editor PNG had been removed,
  but the no-follow manifest-entry prelock surfaced a localized Win32 file-not-found exception
  instead of the stable staging-evidence diagnostic expected by the test.
- The manifest-entry lease boundary now translates only Win32 error 2/3 into a relative-path
  diagnostic stating that the manifest entry does not exist in the staging root. A focused probe
  deleted `captures/editor-after-reopen.png` after publishing the tree manifest and observed that
  exact fail-close diagnostic.
- Diagnostic staging run R9 passed in 124.417 seconds with nine exact overlays and source
  fingerprint `94D420493492AFBD9479A238C77453A2C5C7779C51C31A975A19196BBEDD0D69`.
  Acceptance run R5 then confirmed after 372.727 seconds that production rejected the missing
  reopened PNG with the intended `does not exist in the staging root` error. PowerShell's error
  formatter wrapped `the` and `staging` onto separate lines, so the old literal-space test regex
  produced a false negative. R6 confirmed after 434.946 seconds that ANSI/source-excerpt markers
  also occur inside the wrapped diagnostic, so whitespace-only normalization was still too weak.
  The assertion now passes the captured failure through the existing ANSI normalizer and anchors
  all three semantics under DOTALL: the exact reopened-capture relative path, `does not exist`, and
  `staging root`. It preserves the original failure detail and cannot pass on an unrelated error.
- Intermediate source fingerprint
  `5944BDEE1F0A173876C249C8DB3CC53B808DF2D73F5F6550B70080C450501936` passed isolated staging R10
  in 173.922 seconds. The final nine-overlay source fingerprint is
  `7AEDD19E79E1814060E7DBA4B12429C15C7A055301469F482240D0320924BA66`; isolated staging R11
  passed its full contract in 133.194 seconds. Full acceptance R7 then passed against the same
  source-bound overlay set with exit code 0 in 496.943 seconds. This closes the source-bound F0
  staging and acceptance harness batch without claiming the still-pending Cargo product gates.
- Current-source job `d5004232e7594bec846f4f3c7fdfa972` was created with ordinary path
  materialization instead of Cargo closure planning. Run `5ac09f35080a474aad69815f14bfff63`
  therefore exited in 0.339 seconds because the source root had no `Cargo.toml`; no crate was
  compiled and this is a validation-copy invocation failure, not a source failure. Replacement
  Cargo-closure job `61892f3ca1494d788a7f727b9cad5cab`, accepted by request
  `28bb10b2fe5449d9b6aba1a8244fb182`, overlays all 181 Session paths, pins the same two `zr_vm`
  binding roots, but was rejected before Cargo at `owned_overlay` after this record changed while
  the asynchronous worker was materializing it.
- Baseline attribution request `75488ba8c1294179a81b99eae07fffde` refreshed all 181 Session
  paths together. Snapshot `1837` binds the resulting source at baseline epoch 333. Successor
  Cargo-closure job `b04d27fb572742db86a3a3c3d6c77770`, accepted by request
  `2d16ad91e00646fd9e3b8589306e2ee1`, materialized 17,869 files with input-manifest hash
  `160e40f694f28ef5e7c00f138efd98d0e7078070c82ab0725fa8bd051a8f57fd`.
  The post-materialization audit matched all 181 owned paths, including all 164 Rust paths; the one
  absent path is the optimization record intentionally created only after terminal acceptance. The
  two external `zr_vm` binding roots contain exactly the nine files from pinned commit
  `503fb72163cd20ddf32a38f8a330083712f5d648`; all nine Windows-checkout files match their pinned
  Git blobs after CRLF normalization.
- The successor ten-stage script has SHA-256
  `1fe427322a92e73e66df78ebd7028132052c1dd92ea2fd44c8540dbbd22e97ba`.
  Coordinator request `4f5ca557dcf84a3a86fa8ce020547d9b` reports submission `accepted`; the
  client reached its 15-second post-response reconciliation limit, so no duplicate run was
  submitted. Compiler, product, test and five-repeat performance receipts remain pending.
- That run completed naturally as `078dd974f0a74ae09c028e172a30d5b9` with exit 101. Its first
  stage stopped after 2,888.985 seconds (`2,889.157` seconds total) because
  `runtime_text_rich_blocks.rs:31` still compared the shared `Arc<str>` parse result directly with
  `&str`. No later product, profile or performance stage started, so it contributes no performance
  sample. The test now borrows `parsed.text.as_ref()` and preserves the shared-text hard cut.
- Independent Runtime04 review also removed the missing-subasset parent fallback. Stale GUIDs may
  repair only to an exact surviving label; a missing label now returns a typed dangling-subasset
  error with stable same-source candidates in both GUID and path-hint resolution, and migration
  classifies it as a dangling reference. The historical 2026-07-14 fallback acceptance is marked
  as superseded by the new open handoff.
- The first R3 materialization job `e09df45512d6405dbdfecac7e80adc8d` failed before Cargo at
  `overlay_ownership` for three newly edited paths. After refreshing their content-hash attribution,
  snapshot `1838` froze 186 paths with zero drift. Successor copy
  `874e7b02341544e6b95814570179f10b` materialized 17,869 files with input-manifest hash
  `38ef29bee248e601cc3e7b9d57dbaf86ebd33e14d10761b9de08666de5651a2d`; all 186 overlay hashes
  match and sibling `zr_vm` remains pinned at
  `503fb72163cd20ddf32a38f8a330083712f5d648`.
- R3 batch script SHA-256 is
  `ae6c62c56f10723fdbc7cbeb2f05e0a65fb85a184da6b6c4fb0b69e28fd63fcd` with zero parser errors.
  It adds three focused resolver/importer/rich-text tests and all seven F0 profile checks to the
  existing product, shutdown, default-all-target and five-repeat performance stages. Coordinator
  request `571b83a8b5984bce82356f71555a5607` accepted the single batch; its client reached the
  15-second reconciliation limit and no duplicate run was submitted.
- That R3 batch completed naturally as run `f730ec81064b4bae9453593cc88cf92e` with exit 101. Its
  first stage stopped after 2,874.132 seconds (`2,874.324` seconds total) because
  `runtime_environment_wgpu_cubemap_sampling_contract.rs` passed owned `String` rotations into two
  `&str` source-contract helpers. No product, profile or performance stage started, so this run
  contributes no performance sample. Both call sites now borrow `&rotation` and preserve the
  helper contract.
- The next two source-copy jobs, `c53c19f5b82f415f82710db31757f9d1` and
  `162e333d62624846a63b3cafc4d2f3e0`, failed before Cargo at owned-overlay validation for eight
  independently completed App01, Runtime03 and Identity24 paths. Baseline attribution request
  `0e1c368ee95d4b5298ed3d8ffa1a4c81` refreshed the complete Session source. Copy
  `27d906dfa4074fcaaccdc882e6c9021e` then materialized snapshot `1841`, but became obsolete after
  the WGSL contract repair and was intentionally never run.
- Snapshot `1842` freezes the unified 201-path source, including the WGSL repair, with zero drift.
  Cargo-closure job `630d6c46609347feb7933fe5b612f7eb`, accepted by materialization request
  `dfcac0402cad43ca9d9f1a209b4daf1e`, pins sibling `zr_vm` commit
  `503fb72163cd20ddf32a38f8a330083712f5d648` and materialized with input-manifest hash
  `35a993d7e0f3fad5b3e83f17e2730a40a8d9485737a0ee72dbca1475a565342c`. The mandatory
  post-materialization audit matched all 201 snapshot overlays against the source copy.
- The unified ten-stage script has SHA-256
  `a1edef55290b5e45496ef36f14640bdfeb3002f46138abf752358f6c5beb8030`, is 5,780 bytes and has
  zero PowerShell parser errors. It batches focused App cadence/plugin, Runtime diagnostic/profile,
  Scene allocator, full WGSL contract, source-guard, three all-target feature configurations and
  performance metrics. Coordinator request `744c4c3cb4d24adcaf5330f633b14e2e` accepted the run; the
  client reached its 15-second post-response reconciliation limit, and coordinator status is
  `running`. No duplicate run was submitted. The terminal run id, stage receipts and measured
  performance output remain pending evidence.
- That batch completed naturally as run `b8f53978e6d9408abcade9f38fb0ecdf` with exit 101. Its
  first stage stopped after 801.353 seconds (`801.377` seconds total) because the UI text profile
  child declared `record_text_prepare_profile` as visible only to `prepare_report`, while that
  parent re-exported it to the next-level `text` owner. No later product, profile or performance
  stage started, so this run contributes no performance sample. The child function now uses the
  narrow `pub(in super::super)` visibility required by `text`; a source-contract test locks the
  child visibility, parent re-export and consumer chain.
- While that batch compiled, Runtime03 periodic logging gained a current-only store projection that
  preserves path/unit/current/EMA/min/max but omits retained history and subsystem tags. At the
  review scale of 541 series and the default 64-measurement history window, periodic history clones
  fall from at most 34,624 to zero. Identity24 project-load exhaustion now preserves both the
  document path and typed `SceneError::EntityIdExhausted` source. Both changes pass exact rustfmt,
  source-contract and diff gates; Cargo evidence remains pending.
- Snapshot `1843` froze the 210-path successor source with zero drift before the UI text visibility
  failure became terminal. It was intentionally not materialized or run and is superseded by the
  visibility repair. The next unified script has SHA-256
  `aaab84f6a56103cee44419b261a864828cbcabf224f7de29b92e654760b7ccc7`, is 8,241 bytes and has
  zero PowerShell parser errors. Its 11 stages retain the compile/product/config gates and add five
  sequential Runtime04 repetitions: 17 raw sample groups per repeat, 85 total. This script was used
  only for closure planning and was superseded before a Cargo run when scope review restored the
  original missing-subasset, rich-text and UI-visibility regression reproductions.
- Snapshot `1844` freezes the resulting 212-path source with zero drift, including the narrow UI
  text visibility repair and its source-contract test. Initial Cargo-closure job
  `fa7860ad176943cb9005a94e1c57335d` materialized 17,852 files with input-manifest hash
  `63a498ab59271adce53fa30daae44bdcb4e31c6295ae492b9c588329a3f75093`, but its closure omitted
  19 leased plan, contract-document and PowerShell paths that were not Cargo inputs. The mandatory
  212-path audit therefore rejected that copy before Cargo despite all 193 present overlays
  matching; no run was submitted against it.
- Successor copy `35ebe74c10674df1a6912f716978a31b`, accepted by materialization request
  `8f0f945a6a5e4934ba6e7c27435fb899`, explicitly adds every snapshot path to the same Cargo closure
  and pinned `zr_vm` roots. It materialized 17,872 files with input-manifest hash
  `70c542cb02a69ae5377dc488e417b71a34bbda538beac6aca99a9ab327766949`; the post-materialization
  audit matched all 212 snapshot expectations, including the one intentionally absent optimization
  record, and confirmed all four restored regression-test sources were present.
- The submitted 12-stage script has SHA-256
  `4654767ed7bc4bb2c56d24822a232b775bf1f2c075e20ae594921b5d06d0cb2d`, is 9,476 bytes and has
  zero PowerShell parser errors. In addition to the compile/product/config and five-repeat
  performance gates, one batched stage executes the original missing-subasset resolver/importer,
  UI text visibility and rich-text integration reproductions. Coordinator request
  `13b22c9dffef4fc79fcd8ceaf0a6fa87` accepted the single run; the client reached its 15-second
  post-response reconciliation boundary, so no duplicate was submitted. Run
  `a1d800fd78dd4c43a2a8896c0dccad0a` then completed naturally with exit 101 after 1,462.213
  seconds. The first stage stopped on `E0282` at
  `zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/runtime_lines.rs:122`;
  no Runtime04 performance sample ran, so this receipt is compiler evidence rather than a failed
  performance gate.
- The compiler failure was caused by reading `.len()` from an unannotated `collect()` result in a
  profiling counter before the function-tail return could establish its collection type. The
  fallback collection is now explicitly `Vec<RuntimeTextLine>`. This is a type-only repair with no
  runtime branch or allocation change, and the touched file passes exact rustfmt and diff checks.
- Independent allocator review then found that project normalization admitted the initial restored
  entity cursor before accounting for the camera and directional-light nodes it may create. A
  persisted maximum of `u64::MAX - 2` could still overflow on the second default spawn, while
  `u64::MAX - 3` could return a world whose next cursor was the reserved maximum. Admission now
  precomputes both missing-default decisions, validates the post-default cursor before any
  normalization mutation, and reuses those decisions for the actual spawns. The regression covers
  both near-exhaustion cases and preserves the exact project path plus typed exhaustion source.
- Baseline attribution request `148020ee2f884f5bb5bfe9e0b43bcbf7` and snapshot `1845` froze the
  preceding 216-path source, including the rich-text rustfmt repair and four pending optimization
  records. Copy `64258635779b4c01b85eb244d2711cd0`, accepted by request
  `88b32f76143a43c2998e7e6e918b9082`, materialized 17,876 files with input-manifest hash
  `a61b65fdb6dee929a759fe0dbe73d468e0eb6260f4f1daeb7795460891eade39`. Its copy audit matched
  `216/216`; the worktree audit then rejected it as obsolete on exactly the two allocator owner/test
  paths above. No Cargo run was submitted against snapshot `1845`.
- Baseline attribution request `3effd3de585246a9aace87b060fef26d` and snapshot `1846` freeze the
  resulting 217-path source, adding the Editor fallback-line type repair to the allocator repair and
  four pending optimization records. Cargo-closure copy `4872e9b11aa74bcc80da787a06404441`,
  accepted by materialization request `65877399690b480fb523feaaab419d1e`, materialized 17,876 files
  under the F-drive validation root with input-manifest hash
  `1c6e6c40d7d576d5b575a3778eba32d4301ee9008dc23a6aa77dc67e9ac2371d`. The exact post-copy audit
  matched all `217/217` snapshot expectations: 216 present files plus the one intentionally absent
  fixed record.
- The submitted successor script has SHA-256
  `154e085363c2a947dd28d676fc644f303a9732dfdf28b1ddf0970296d23a0f33`, is 10,075 bytes and has
  zero PowerShell parser errors. It retains the 12 batched stages and 19 Cargo commands, adds the
  Editor16 cross-crate handshake behavior test, and requires default-node allocator reservation in
  the source guard. Coordinator request `f71486fef10b4dbf899843f0b885bcac` accepted the only run;
  the client reached its 15-second post-response reconciliation boundary, so no duplicate was
  submitted. Run `5a1ee52cdb27449b86878a76ee712792` completed naturally with exit 101. Its
  first stage completed the cold test build in 30m45s and then failed the focused 10 Hz cadence
  regression because a Continuous pump had left the constructor's initial `frame_requested` bit
  set before transition to LowPower. No later product, configuration or Runtime04 performance
  stage started, so this run contributes zero performance samples.
- Continuous cadence now clears an already-satisfied frame request on every unconditional pump.
  The production focus/occlusion handlers still create a new explicit request after a real window
  transition, so early wake semantics remain while stale cross-mode state no longer bypasses the
  100 ms timer. The file passes exact rustfmt and diff checks and remains pending the successor
  source-bound batch.
- The successor run against copy `a2aedf3b72e4455b81a5fe12b6f9a636`, run
  `dd2c8ec6f998499a9aa84d7d0428fe8d`, completed the App cadence stage and then stopped in the
  profiling stage with two compile-only test harness errors: the layout profiling test imported
  `layout_profile_metrics_enabled` from the wrong module level, and the inactive dynamic profiling
  macro test left its generic name argument inferred as `()`. The test-only repairs use the parent
  module import and a typed lazy `String` helper; they do not change production behavior or capture
  work. No product, default-configuration or Runtime04 performance stage started in that receipt.

## Review

- Reviewed all 41 changed production-path candidates from snapshot `1823`. Runtime04 resolver
  borrowing/preallocation, sidecar-before-authoring crash-window ordering, flattened material
  reference round-tripping, Windows display paths, the App/Editor handshake surface, the RHI
  re-export and the schedule borrow repair preserve their intended owner contracts.
- The remaining Editor changes are test-owner imports, test-only `pub(super)` visibility, explicit
  lifetimes or equivalent closure rewrites. No compatibility alias, extra runtime branch or hot-path
  clone was introduced. Final source-bound diagnostics remain pending the managed run receipt.

## Residual Scope

- This record remains `validation_pending` until Runtime04 tests, the three-package compile batch
  the managed scale sweep, and the editor-host product/focused gate have terminal coordinator
  receipts.

## 2026-08-19 Async Stage-4 Repair

- The primary 12-stage batch request `8e473e810b4f48458c6f2be89cacf6a8` completed as run
  `e0cf610bc058413babd6a43f33d7f740` with exit code 101. Stages 1-3 passed in
  `2069.382s`, `1583.967s`, and `2.216s`; stage 4 stopped in `2.313s` at
  `scene::tests::world_basics::world_state::project_load_rejects_invalid_orphan_local_transform`.
  The failure was a panic in `typed_api/projection_rebuild.rs` because deserialization projected an
  orphan `local_transforms` row before the typed transform validator could report its zero scale.
  No product/configuration/performance stage ran and this receipt contributes no performance sample.
- The repair validates the raw persisted transform map before projection, checks every persisted
  component map against the registered entity set, and makes direct `World` deserialization reject
  orphan rows with a serde error instead of an `expect` panic. The original invalid-orphan test
  still requires `SceneError::ZeroScaleTransform`; a new valid-orphan regression requires typed
  `SceneError::MissingEntity` from project loading.
- The successor batch starts at the repaired scene stage and continues through WGSL, repaired
  regressions, source guards, all-target checks and five Runtime04 performance repeats. Its script
  is `zircon-validation-1847-scene-preflight.ps1` with SHA-256
  `4bc6b2cf87f7e0c6c7f6689414857d6573b153a906df45cd621342e890a16c73` and zero PowerShell parser
  errors. Snapshot `1856` freezes 229 current-source paths with input-manifest hash
  `6df6c9dcf7e7a0438a1fc1b25b90c9b28fcebc885afbb5d28e8346cda5cbd588`.
  Cargo-closure materialization job `fd641468df2e4c4689ce28fe509a111e` completed before the
  validation command was accepted as coordinator request `7324bb0d5b094bbaa4f3fff0e035918a` at
  2026-08-19T04:18:01Z. The client timed out only while reconciling that accepted request; the
  copy status is `running`, so this is an asynchronous submission log, not a test or performance
  result. Terminal coordinator evidence remains required before this milestone can pass.

## 2026-08-19 Async Stage-4 Visibility Repair

- The asynchronous successor run `5b69486f879443dcacca8377d82e514f` for job
  `fd641468df2e4c4689ce28fe509a111e` reached Cargo and exited `101` after
  `1196.677s`. It stopped in the first stage while compiling `zircon_runtime`; no scene test,
  product gate, all-target check, or performance sample ran.
- The only diagnostic was `zircon_runtime/src/scene/world/mod.rs:52:16: error[E0365]` because a
  `pub(super)` `WorldPersistentState` was re-exported from its parent module. The repair removes
  that re-export and imports the owned type directly from `world` in `project_io::document`,
  preserving the internal visibility boundary without changing the preflight behavior.
- A new snapshot and materialization are required because both the source fix and this receipt
  update change the frozen input. The next batch must retain the same scene-first ordering and
  continue through the product, all-target, and five-repeat performance stages before any result
  is accepted.
- Replacement snapshot `1857` was materialized as job `cee4201b4beb4009ba274f4d91e57439` with
  input-manifest hash `1a2c4f1892300f8b03251e5cb6894752d70faa30392eb1bd09fb4f04579b2f71`.
  Its validation submission was accepted as request `43ada476483d4be28d9af7fbea4ce269`; the
  client again timed out only during post-response reconciliation. The job is `running` and has
  no terminal evidence yet.

## 2026-08-19 Async Stage-4 WGSL Contract Repair

- Replacement run `57d669e1c1cf4c6292c3927d9ce4f431` completed for job
  `cee4201b4beb4009ba274f4d91e57439` with exit code `101` after `2499.555s`. The repaired
  persisted-component admission stage passed all 5 focused tests in `1803.952s`; this confirms
  invalid orphan transforms now return their typed error and valid orphan component rows now
  return `MissingEntity` without projection panics.
- The next WGSL contract stage ran 23 tests and failed 6 source-shape assertions in `695.575s`.
  The shader already retained the intended runtime behavior: normalized sky wrappers, final skybox
  intensity, metallic diffuse suppression, zero-weight sky suppression, planar early return, and
  normalized-reflection reuse. The assertions had become brittle after harmless delegation to
  `*_with_prepared_inputs` and `*_after_planar`, compound guards, and CRLF-preserving source
  extraction. No product/configuration/performance stage ran, so this failed receipt contributes
  no performance sample.
- The contract test now asserts semantic function calls and ordering across the delegated helpers
  instead of exact multiline source formatting. It still requires defensive normalization at the
  public boundary, planar return before continuation, zero-weight guards before their samples, and
  normalized PBR reflection. A new frozen batch is required before this record can report any
  performance result.

## 2026-08-19 Default All-Targets Framework Cutover Repair

- Successor job `791d58c5a9fe4418be91b0c5fdff146f` retained terminal evidence after the
  coordinator removed its copy. The persisted-scene stage passed `5/5` in `2905.131s`, the WGSL
  contract stage passed `23/23` in `753.573s`, the repaired regression batch passed in
  `2448.776s`, and the source guards passed in `0.166s`. The default all-targets stage then exited
  `101` after `596.389s`; total elapsed time was `6704.072s`. No configuration or Runtime04
  performance stage ran, so this receipt contributes no new performance sample.
- All six diagnostics came from `zircon_shader_pbr_viewer`: the viewer still imported the removed
  `SceneViewportSurface`, called private `SceneRenderer::render`, requested the removed direct
  surface creation/render methods, and retained two mutable `self` borrows while writing screenshot
  and GPU-timing evidence. Commit `7a20f921b` intentionally moved viewport and native-surface
  ownership into `RenderFramework`, so restoring those old APIs would violate the accepted hard
  cutover.
- The viewer now owns a `WgpuRenderFramework` plus a viewport handle. Offscreen frames use
  `submit_frame_extract` followed by framework capture; native-window frames use
  `present_frame_extract`; bind, unbind, debugger capture, GPU timing drain, and specialized Base
  pipeline admission all stay inside the framework lock boundary. A startup constructor returns
  the existing renderer startup report without exposing renderer ownership. The screenshot and GPU
  evidence values are harvested before mutating the outer app, eliminating both overlapping borrows.
- `scene.rs` and `app.rs` remain single-purpose viewer orchestration files at 1062 and 1086 lines.
  This blocker repair does not add a second responsibility, so a broad split is deferred; the
  smallest follow-up boundary is the GPU-evidence and screenshot state machine in `app.rs`.

## 2026-08-19 Viewer Framework Contract Batch Repair

- Frozen job `2ffd48830e8149ad8c5f794317ceb2f9` completed as run
  `70b8bc3ef5ab48bab715e0681c18a79a` with exit code `101`. Its first stage built the cold viewer
  test target in 53m38s, then finished 116/126 tests in 1.42s; stage elapsed time was `3222.335s`
  and total batch time was `3222.365s`. The default/configuration checks and Runtime04 repeats did
  not start, so this receipt contributes no performance sample.
- Seven failures were source-contract assertions that still required the pre-cutover direct
  renderer spelling or single-line formatting. They now assert the framework capture, debugger,
  Base prewarm and timing boundaries without depending on line wrapping. The GPU distribution
  contract separately preserves report/status collection before resolution and requires the
  pending timing state to arm every sampled frame.
- Two project-asset assertions were test defects: one found its own forbidden deletion string,
  while the other rejected a serialized empty `textures` table even though it contained no texture
  references. The guards now avoid self-matching and accept only an absent or empty texture table.
- The remaining project-asset failure was Windows path exhaustion in the deep validation copy.
  Private staging and displaced-tree names are shortened from the long descriptive prefix to
  `.zpv4-s-<pid>-<sequence>` and `.zpv4-i-<pid>-<sequence>`. The representative deepest source
  path falls from 232 to 198 characters, preserving 34 characters of headroom without changing
  the published immutable `viewer-assets-v4` path or its atomic rename protocol.
- `app.rs` now snapshots `gpu_timing_evidence_pending()` before borrowing the scene and requests a
  frame timing report while that state is true. This repairs the real App02 scheduling hole without
  affecting normal viewer redraws when `--gpu-timing-report` is absent. Exact rustfmt, diff checks,
  and all repaired source anchors pass; a new current-source Cargo batch remains required.

## 2026-08-20 Current-Source Overlay Dependency Repair

- Validation-copy job `5e543b01614444ccb6e26c1babee5f44` completed as run
  `aa314b671a3c43e3855763279a8904ce` with exit code `101` after `2910.903s`.
  The viewer framework target compiled cold and passed all `128/128` tests in `1.42s`.
  The next `zircon_runtime` test target stopped at compile time because
  `scene/tests/world_basics/world_state.rs` used `RenderOverlayExtract.highlights` while the
  immutable copy still contained the old baseline `RenderOverlayExtract.selection` definition.
  No configuration, product, or performance stage ran, so this receipt contributes no latency
  sample.
- The failure was not a stale test. The shared worktree contains the complete Editor05 hard cut
  from `SelectionHighlightExtract` to the runtime-neutral `HighlightSet`, but its 23 supporting
  Rust paths and four failure records were still attributed to inactive or archived Sessions and
  therefore could not enter the 261-path copy. Reverting the assertion would restore the deleted
  compatibility contract and leave the remaining consumers uncompilable.
- With no foreign live lease, the coordinator transferred those 27 exact current-source paths to
  the MVP00 convergence Session. Snapshot `1889` freezes the definition, renderer consumers,
  viewport controller modules, boundary tests, and their handoff records together. All 23 Rust
  files pass Rust 1.94.1 `rustfmt --check` and the scoped diff check. The replacement manifest is
  284 paths; it must rerun the same combined batch before the baseline can be integrated.

## 2026-08-20 Highlight Capacity Dependency Repair

- The 284-path replacement job `855522afe57b43de8d563872a72dea25` completed as run
  `2b2620240be34875b81e4ea3bc530e54` with exit code `101` after `1857.005s` in the first
  runtime compile stage. The only compiler diagnostic was `E0599` at
  `dynamic_api/session/extract_stats.rs`: the immutable copy called
  `HighlightSet::entity_capacity()` while its `highlight_set.rs` still came from HEAD and did not
  expose that method. No test binary, configuration check, or performance stage ran, so this
  receipt contributes no correctness or latency sample.
- The current worktree already contains the matching crate-private capacity accessor and its
  canonicalization regression assertion. That one direct dependency had never appeared in any
  coordinator snapshot or attribution, so reverting the stats call would undercount retained
  overlay capacity and would hide the incomplete current-source closure.
- The coordinator now leases and attributes
  `zircon_runtime/src/core/framework/render/highlight_set.rs` to the MVP00 convergence Session.
  The replacement manifest is 285 paths and must rerun the same runtime-first combined batch.
  The previously recorded 284-path scoped diff check passed; the added Rust file must also pass
  pinned Rust 1.94.1 formatting and the replacement scoped diff check before resubmission.

## 2026-08-20 Current-Source Combined Batch Submission

- The exact 285-path successor was materialized as job
  `2bd89ab3a1e84a1abf31a2f105b1a0fc` with input-manifest hash
  `3944eb72839d8ad9c19f3eeecb262f163ea48f9c9df53739efb8a90e0ba0f2b5`.
  Coordinator request `f899cabf4f9946fca16c45d5654330cf` accepted its only run. The copy remains
  `running` under root PID `53340`; this is asynchronous submission evidence and not a compiler,
  test, or performance acceptance claim.
- The combined, primary, and follow-up scripts have SHA-256 values
  `4736aea0f3b7355c041be3624e2da00b16ff3b79d1a8e4c44ac4f6d52df5ae12`,
  `ea367a941c246e2c1e44b44b745557be27e596020892181dbccf6b7efb3972b3`, and
  `9a48e883e6f21d8d294aefd2b0459e3bc2ffcffdb198b1b5c341920392e65a89`.
  While the first Cargo stage compiled, validation preflight proved that libtest can prefix its
  first benchmark row with `test ...`. The not-yet-executed Runtime02 child parser was corrected
  to extract all three `EVENTBUS_BENCH_V2` markers from anywhere in a line; its SHA-256 is
  `8c6b4025ead4d9a87e185ab6a8a119f8245d73a224c0f3e1bb3ff78e104c2f2a` and its PowerShell AST
  has zero parser errors. No Cargo or rustc process was interrupted.

## 2026-08-20 Default All-Targets Private Frame Repair

- Current-source job `2bd89ab3a1e84a1abf31a2f105b1a0fc` completed as run
  `c1ec9d708d684b939fb2ad15f379342f` with exit code `101` after `5434.974s`.
  The focused Runtime framework contract passed, and the Viewer target passed all `128/128`
  tests; their combined stage elapsed `4856.254s`. The default all-targets stage then failed
  after `578.228s` on one `E0603` diagnostic in
  `m1_runtime_editor_boundary_contract.rs`: the external test still imported the now
  crate-private `ViewportRenderFrame`.
- `ViewportRenderFrame` is deliberately internal after the framework ownership cutover. The
  public boundary is `RenderFramework::submit_frame_extract`, so re-exporting the old frame type
  would restore an unusable compatibility surface. The same stale call remained in three Shader06
  integration-test paths that the compiler had not reached yet.
- All four integration-test owners now build a public `RenderFrameExtract` from their synthetic
  snapshot and submit it through `submit_frame_extract`. The Shader06 source contract explicitly
  rejects the old runtime-frame entry point, and the internal helper is now declared
  `pub(crate)`. Exact Rust 1.94.1 formatting and scoped diff checks pass, while a source scan finds
  zero `ViewportRenderFrame` or `.submit_runtime_frame(` matches under `zircon_runtime/tests`.
- The three Shader06 test paths and internal helper had no live foreign lease. They are now part
  of the convergence Session together with the already-owned M1 boundary test, increasing the
  next exact current-source overlay from 285 to 289 paths. A fresh combined batch remains required;
  no configuration or performance stage ran in the failed receipt.
- Repair snapshot `1904` was submitted for Cargo-closure materialization as job
  `b949b5ec0fa3478ea1619446e5b3022d` under coordinator request
  `dc76b3247d4c4f89a8a27388abb6f439`. The copy is still `materializing`; this is an
  asynchronous submission receipt and does not claim the 289-path batch has passed.
- Static preflight then removed two stale `output_size` parameters made unused by the public
  frame-extract migration. Exact formatting and diff checks still pass, but that edit correctly
  caused b949 to stop before copying or Cargo with `validation_copy_attribution_stale`. Snapshot
  `1906` freezes the three updated Shader06 paths; the next materialization must use this current
  attribution instead of reusing the rejected job.
- Fresh 289-path job `df54dcddb4d84de3882933ba3a29051d` materialized with input-manifest
  hash `e78ed8ea4833d2e735ad7e40094e8d3d623525c0c1c1e79d9d0866fd35e86379`.
  Its six directly repaired/recorded paths match the current worktree byte-for-byte. Successor
  script `zircon-validation-1906-default-all-targets-successor.ps1` has SHA-256
  `2102a426882dd87de8cc33473ff659338bef5073ad925979ae79d2755d3f5fe0` and zero
  PowerShell parser errors. It reuses only the 1 Runtime and 128 Viewer tests already passed by
  run `c1ec9d708d684b939fb2ad15f379342f`, starts at the failed default all-targets stage, and then
  retains every configuration, five-repeat performance, and follow-up stage. The batch is now
  `running` under coordinator root PID `37200`; this is asynchronous submission evidence only.

## 2026-08-20 Realtime IBL Generation-Ticket Closure

- Successor run `5ab3e7108a7a494db9b8acc03e4b0bc5` completed job
  `df54dcddb4d84de3882933ba3a29051d` with exit code `101` after the default
  all-targets stage ran for `562.934s`. The compiler reported 12 `E0609`
  diagnostics in `runtime_shader_pbr_realtime_ibl_export.rs`: its current-source
  assertions referenced generation, recipe fingerprint, work slot, scheduled and completed
  workgroups, and terminal reason fields that were absent from the immutable copy's
  `RealtimeIblGpuTimingReport`. No later configuration, performance, or follow-up stage ran.
- Those report fields already exist in the current worktree and are produced by the matching
  generation-ticket state machine. The failure is another incomplete current-source closure,
  not a reason to delete the integration assertions or restore the obsolete full-update path.
- The coherent closure consists of 11 modified files: GPU timing metadata, graph planning,
  compiled-graph cache, runtime submission, time-slice scheduling, WGPU recording, and their
  direct tests. No modified core environment DTO is required by their imports. The largest
  file is 465 lines, all 11 files pass pinned Rust 1.94.1 formatting and scoped diff checks,
  and the convergence Session now owns the complete closure. A fresh immutable snapshot and
  default-all-targets successor remain required before acceptance.
- Full 300-path snapshot `1911` was submitted for Cargo-closure materialization as job
  `6b98a7da50a54cee8358728a34b6d327` under coordinator request
  `a314ee9f43af45febcdb9bb530f4ab11`. The copy is `materializing`; no main Cargo run was
  submitted while the Plugins11 r4 batch owns the validation slot. This is an asynchronous
  materialization receipt only.
- The asynchronous worker then rejected job `6b98a7da50a54cee8358728a34b6d327`
  before copying or Cargo with `validation_copy_overlay_not_owned` at the first newly claimed
  IBL path. The 11 files were formatted after their leases were acquired, so their lease
  baselines did not yet attribute the final bytes. The next replacement refreshes content-hash
  attribution for all 300 Session paths together before taking a new immutable snapshot.
- Attribution request `a8ccb21e993042fcab7cc3ef5484f9e8` refreshed all 300 paths and snapshot
  `1914` froze them. Replacement copy `8d86fa48fe0e47f5b3a6f14cb0cd0be8`, accepted by request
  `e49f6ef6dc5b45dd90a6ceb4a2b78260`, began materializing but was superseded before any Cargo
  submission when static closure audit found two unowned direct consumers.
- `render_scene.rs` enables per-ticket GPU timing when the direct renderer has a valid readback
  frame, while `write_scene_uniform.rs` keeps procedural environment bindings until the first
  realtime ticket reaches publication. Leaving either file at HEAD would respectively suppress
  the required timing reports or sample unpublished realtime resources. Both files are now
  owned, exactly formatted, and diff-clean, increasing the complete closure to 302 paths. The
  obsolete 300-path copy will not be run.
- Snapshot `1915` materialized the exact 302-path replacement as job
  `2dd228647e2142fbb7b1401509783f91` with input-manifest hash
  `3e4b2629e8e1a6109cc605e00537979bea6058f2a6cc36d3c2ce4c1a4c04c06e` under
  `F:\cargo-targets\verify`. All 13 IBL closure paths match the current worktree byte-for-byte.
  Cargo remains intentionally unsubmitted while the Plugins11 batch owns the real validation
  slot; this is materialization evidence only, not correctness or performance acceptance.

## 2026-08-20 Shader Prewarm Current-Source Test Closure

- Successor run `d866aa83c77941ceaa72f3634ed97975` stopped in the default
  all-targets stage after `515.284s` with 16 `zircon_shader_prewarm` test-target
  compile errors. No configuration, Runtime04 performance, or follow-up stage ran.
- Eight diagnostics came from asset-inventory tests that did not import their
  parent module's snapshot path helpers; the old immutable baseline also omitted
  the current parent implementation. Two include-DAG assertions compared
  `&Vec<T>` directly to arrays. Six manifest assertions still read the removed
  `ShaderVariantPrewarmRequest.wgsl_source` field instead of resolving its
  generation-owned `ShaderVariantPrewarmSource` through `manifest.source_for`.
- The tests now import the three private snapshot helpers, compare indexed
  dependency slices, and resolve WGSL through the public manifest/source
  relationship. The current `asset_inventory.rs` is included unchanged as the
  direct helper owner. All four closure files pass pinned Rust 1.94.1 formatting
  and scoped diff checks; the largest is 834 lines.
- The convergence Session now leases the complete four-file support closure,
  increasing the next current-source overlay from 302 to 306 paths. Its
  successor must restart at default all-targets and retain every later stage.
- Snapshot `1927` materialized the exact 306-path successor as job
  `5fd737cdad9e4bc498d5a82c83d4f0da` under coordinator request
  `1d1490b4f46a4f0f87b392ebd858a594`, with input-manifest hash
  `316ee671f3993117402e7f095702ce9f03c6e68d65ed4a4e3592b504d17e9dad`.
  Snapshot preview reports `0/306` current-source changes, and all four
  shader-prewarm closure files match the isolated source byte-for-byte.
  The successor and follow-up scripts retain SHA-256 values
  `2102a426882dd87de8cc33473ff659338bef5073ad925979ae79d2755d3f5fe0`
  and `9a48e883e6f21d8d294aefd2b0459e3bc2ffcffdb198b1b5c341920392e65a89`;
  both have zero PowerShell parser errors. Cargo remains unsubmitted in this
  receipt, so it is materialization evidence only.

## 2026-08-20 Cube-Mip Test Import Repair

- Coordinator run `6a402b701f4c49b9b7f43d0fb0eebc8c` completed job
  `5fd737cdad9e4bc498d5a82c83d4f0da` with exit code `101`. The default
  all-targets stage ran for `765.230s` and stopped on one `E0422` diagnostic;
  no configuration, performance, or follow-up stage ran.
- `realtime_ibl_graph_plan/tests.rs` matched the new ticketed prefilter
  operation with `CubeMipRange`, but its explicit sibling-module import listed
  only `CubeFaceRange` through the parent module. The production graph planner
  and time-slice operation remained consistent; this was a test-target support
  import omission rather than a runtime behavior regression.
- The test now imports the existing crate-private `CubeMipRange` directly from
  `realtime_ibl_time_slice`. No production behavior changed. Pinned Rust 1.94.1
  formatting, scoped diff checks, and the exact source anchor must pass before
  taking a replacement snapshot.

## 2026-08-20 Exact-Format Snapshot Supersession

- Snapshot `1928` froze all 306 Session paths after the cube-mip import repair,
  and source-copy job `b66b36a3094345f0a26c25cdfd8aad37` materialized that
  snapshot under `F:\cargo-targets\verify` with input-manifest hash
  `4f72f98c226ab884a41b092d1dba3904c34d433190c8c8357d29932546eaa25e`.
- The mandatory exact-path Rust 1.94.1 formatter sweep then found one owned
  source, `zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs`,
  whose current bytes needed formatting. The full 247-Rust-path check passes
  after that correction, and scoped diff checking remains clean apart from
  checkout line-ending notices.
- Because the format-only edit occurred after materialization, job
  `b66b36a3094345f0a26c25cdfd8aad37` is superseded and will not run Cargo.
  A fresh attribution, immutable snapshot, and source copy are required so the
  next compiler/test receipt binds the exact formatted worktree bytes.
