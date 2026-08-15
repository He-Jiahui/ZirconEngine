---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: native-discovery-metered-collector-contract
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_runtime/frameworks/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/contract.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/service.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
tests:
  - collector rejects candidate, diagnostic, read-byte, and scratch-byte limits before unbounded public payload allocation
  - cargo +1.94.1 test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1 -- --nocapture --test-threads=1
---

# Frameworks04: Native discovery metered collector contract

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片：Runtime11 native-plugin discovery bounded refresh publication contract
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md`
- 交接原因：Frameworks04 owns the native discovery authority, filesystem/manifest collection projection, and the collector boundary consumed by Runtime11. Runtime11 owns task admission, cancellation, terminal tickets, and immutable publication, but cannot enforce resource limits after a foreign collector has already allocated its public payload.
- 生命周期键: `native-discovery-metered-collector-contract`

## 失败现象与复现证据

Runtime11's `NativePluginDiscoveryRefreshPayload` exposes already-built `Vec<NativePluginCandidate>` and `Vec<String>` plus collector-reported `read_bytes` and `peak_scratch_bytes` (`discovery_refresh/contract.rs:134-142`). `NativePluginDiscoveryRefreshService` checks those counts only after `NativePluginDiscoveryCollector::collect` returns (`discovery_refresh/service.rs:310-370`). A collector can therefore allocate or retain arbitrary candidate/diagnostic data before Runtime11 rejects it, and can report scalar byte values smaller than its real allocation.

The existing Runtime11 budget tests exercise scalar accounting only. They do not prove that candidate/diagnostic/read/scratch ceilings are enforced before public collection allocation, so the current contract cannot support its stated bounded-RSS acceptance claim.

## 最低共享层根因

The collector contract transfers ownership only after an unmetered aggregate has been materialized. Limits are represented as trusted post-hoc metadata rather than authority-owned reservation or sink admission. The correct lower repair belongs at the Frameworks04 discovery collector/authority boundary, where directory entries, manifest bytes, parse diagnostics, and candidate materialization are produced.

## 架构修复验收

- Replace aggregate-return-only collection with one canonical metered collector contract: Frameworks04 receives immutable limits and a Runtime-owned bounded sink/reservation interface, checks cancellation/deadline between units, and must acquire capacity before retaining candidate data, diagnostics, read buffers, or scratch space.
- The sink records actual admitted units and byte usage. Collector self-reported counters are telemetry only and cannot authorize publication or satisfy Runtime11 budget checks.
- Focused Frameworks04 tests make an oversized candidate, diagnostic, manifest-read, and scratch request fail at admission before an unbounded public `Vec`/`String` payload is formed; cancellation and last-good publication behavior remain intact.
- Runtime11 reruns `cargo +1.94.1 test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1 -- --nocapture --test-threads=1` through a fresh managed reservation and then reruns its burst/cancellation/shutdown upward fixtures.

## 禁止临时方案

- Do not retain the aggregate payload API behind a second post-hoc validation pass, trust collector-provided byte counters, or merely clamp a report after allocation.
- Do not add a parallel scanner, editor-local cache, compatibility collector, unbounded channel, or test-only allocation bypass.
- Do not move Runtime11 scheduler/ticket/publication ownership into Frameworks04 or weaken the native discovery resource-bound acceptance criteria.

## 修复结果与回传

Open state: `待修复`; Runtime11 keeps its bounded refresh service and terminal observer contract open, but does not claim a real RSS bound, Native Refresh GREEN result, fixed artifact, or commit until Frameworks04 returns a verified metered collector contract and the managed upward gate runs.

## 当前实施状态（2026-07-29）

- Frameworks04 source session `frameworks04-native-discovery-metered-collector-r1-20260729` now owns the metered collector contract. The former public aggregate payload is `pub(super)` implementation detail only; `NativePluginDiscoveryCollector::collect` receives a Runtime-owned `NativePluginDiscoveryRefreshSink` and returns only its input identity.
- Candidate and diagnostic values now use an admitted `materialize` closure; read and scratch capacity use an admitted `execute` closure. The sink records the admitted counters, and the old collector-provided `Vec` and byte counters are no longer a publication input. Read-byte addition is checked and rejects arithmetic overflow rather than saturating.
- Focused tests contain a source-contract guard plus candidate, diagnostic, read-byte, and scratch-byte probes. The original guard was RED before the public payload hard cut and is statically GREEN after it; it additionally rejects public `commit(prebuilt)`.
- Static evidence: `rustup run 1.94.1 rustfmt --check --config skip_children=true` passed for the six changed Rust files; `git diff --check` reported no diff error (only the pre-existing Windows line-ending notice for `native_plugin_loader/mod.rs`).
- Independent review corrected the test import to the parent loader export and rejected the first `commit(prebuilt)` shape. The correction removed that method, but the follow-up review found the remaining closure-capture loophole: a collector can still allocate/read before passing an already-built value into a closure. This is the same open lower authority/materialization root cause, not an acceptance result.
- CPU warm reservations `c491a7163f6543a2bf82ee8ab8bcd45c` and `72ea9d5e87314f25965bde8a827f6d43` were released before any job bound because source corrections invalidated their snapshots. The current pending compile/regression row is `78fb773ee1fa41c2ac10e146912241b9`, command fingerprint `02a909fa654503cb338ca75149585aedad1537f016e86a3a795787e41fe05854`, source snapshot `1278` / manifest fingerprint `04c331759c954e6a13279459d5d6f3111fd85a63a0ddb1a67599845f4a2a4102`. Its consume has correctly returned `cargo_cpu_reservation_not_fifo_head`; no job has run.

Remaining acceptance includes a collector/authority design that prevents the closure-capture loophole, the raw terminal result of the exact managed lib-only gate, and the Runtime11 upward cancellation/burst/shutdown rerun. This record remains open: no fixed artifact, milestone commit, or RSS-bound claim is made yet.

## 恢复设计记录（2026-07-29）

- Read-only authority audit established that the refresh collector trait and
  `NativePluginDiscoveryRefreshService::new` have no production construction site: their current
  consumers are the refresh fixtures. The real native-discovery path is
  `NativePluginDiscoveryAuthority -> collect_plugin_manifests -> candidate_from_manifest_path`.
- The authority collector currently accumulates directory entries and manifest paths, while
  `candidate_from_manifest_path` reads an entire manifest before producing a candidate. Moving
  only the final candidate commit behind a sink therefore cannot establish the required
  before-allocation or before-read bound.
- The recovery design is to make the existing authority's cold/refresh walk the canonical metered
  path: stream directory traversal without a second scanner, reserve bounded scratch before
  retaining traversal state, use file metadata to admit a manifest read before materializing its
  buffer, reserve parse scratch before parsing, and admit each candidate or diagnostic before it
  is retained. The generic test collector must not remain a production extension point capable of
  bypassing those operations.
- `collect_manifests.rs` and `candidate_from_manifest.rs` are outside this session's immutable
  source scope. Before changing either file, Frameworks04 must release the unbound reservation
  `78fb773ee1fa41c2ac10e146912241b9`, retire this narrow source lease without a fixed claim, and
  register a successor Frameworks04 recovery session containing the canonical authority and both
  lower materialization files. That preserves source-bound Cargo evidence and avoids a parallel
  scanner or scope mutation.

## 后继实施状态（2026-07-29）

- The narrow session released `78fb773ee1fa41c2ac10e146912241b9` before it bound a job and
  released only its own eight leases. Successor session
  `frameworks04-native-discovery-authority-materialization-r2-20260729` now owns the exact
  authority, traversal, manifest-materialization, refresh-contract, and budget-test scope.
- The public `NativePluginDiscoveryCollector`, sink, request, and permit exports were hard-cut.
  `NativePluginLoader::discovery_refresh_service` is now the only public construction path and
  creates an authority-owned collector. Crate-local fixtures retain injection solely to exercise
  Runtime11 scheduler/ticket behavior; no foreign production collector can bypass admission.
- `collect_manifests.rs` now supplies one streaming traversal primitive to both the existing
  synchronous collector and the metered authority visitor. The metered visitor reserves scratch
  before retaining traversal paths, reserves diagnostics before formatting them, and delegates
  each manifest to a path that reserves candidate capacity, metadata-sized read capacity, and
  parse scratch before materializing the candidate. This reuses the canonical scanner rather than
  adding a parallel scanner.
- Static evidence after the hard cut: Rustfmt check passed; the public-surface source guard passed
  (no loader export or public collector/sink constructor); and `git diff --check` had no diff
  errors (only existing CRLF notices). A real temporary-manifest regression now requires an
  authority-backed refresh to fail with `ReadBytes` before snapshot publication when metadata
  exceeds the admitted read budget.
- Fresh source snapshot `1280` has manifest fingerprint
  `e9d5de061604e488bd16aaebb1a5d657446250c8eee33f9f0ced7c63e7aae6f6`.
  Its exact managed gate is reservation `92ba6ba7a565474a96397483b6ee1b4b` for
  `cargo +1.94.1 test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1 -- --nocapture --test-threads=1`.
  Consumption correctly returned `cargo_cpu_reservation_not_fifo_head`; no job has run and no
  test result is claimed. The record remains open pending the true terminal gate, independent
  review, and Runtime11 upward cancellation/burst/shutdown evidence.

## 审阅否决与继续恢复（2026-07-29）

Independent review rejected the authority-materialization r2 shape; its static checks are not
acceptance evidence.

- The candidate and diagnostic reservation result signatures still carry obsolete `<'_>`
  lifetimes although their permit structs no longer do, producing a deterministic Rust E0107
  compile blocker.
- More importantly, the new async service did not replace the public synchronous
  `discover`/`load_discovered_*` path. That path still initializes an unmetered authority cache,
  reads and parses manifests directly, and can run concurrently with a separately created refresh
  service whose publication does not update that cache. This violates the one-canonical-scanner,
  one-generation, and no-bypass requirements.
- Restricting the trait, sink, and constructor to `pub(crate)` is insufficient: any runtime module
  can still inject an I/O-producing collector. The production collector contract must be private
  to the authority, with test injection isolated behind `cfg(test)` rather than crate-wide
  visibility.
- The streaming walker needs a cancellation/admission checkpoint before each directory read and
  entry inspection. The metadata-then-`read_to_string` path also has a growth TOCTOU and cannot
  prove that actual bytes read stay below admission; bounded streaming read or an equivalent
  authoritative size-checked reader is required.
- Existing synchronous discovery source guards also need repair and new parity coverage must prove
  that discovery/load consumers share the same authority publication rather than maintaining two
  caches.

The pending r2 reservation `92ba6ba7a565474a96397483b6ee1b4b` is compile/regression evidence
only and must be released before the required source redesign. Frameworks04 will register a new
immutable scope including the synchronous load owner, preserve this failure as open, and make no
fixed, RSS-bound, Cargo-green, or milestone-commit claim until the canonical path is verified.

## r3 设计约束（2026-07-29）

The r2 reservation has been released and r3 owns the expanded authority, synchronous-load, and
refresh-test scope. A second independent review confirms that the remaining repair cannot be a
local API redirect. The following r3 constraints are now part of this open failure's acceptance:

- One authority-owned service uses a fixed authority budget. The former configurable public
  service factory is removed rather than returning a global service while silently ignoring a
  caller budget; no external caller receives a handle that can shut down canonical discovery.
- Synchronous discovery obtains a shared in-flight ticket from the authority under a short mutex,
  waits only after releasing that mutex, and observes the current winner after a supersede. It
  must never schedule independent root work or wait from the collector's I/O task. Initial failure
  becomes a report diagnostic; later failure keeps the last-good report and appends the terminal
  diagnostic for the attempted generation.
- The old explicit manifest/remove calls retain no second incremental cache. They are explicitly
  bounded full-root refresh notifications: callers must apply filesystem removal before notifying;
  a still-existing manifest is intentionally rediscovered. This is a hard semantic cut, covered
  by the loader's only in-tree callers and tests.
- Read admission is a two-phase protocol. A fixed-size stack buffer is filled only after an upper
  bound is admitted; EOF, partial reads, errors, cancellation, and overflow return unused bytes
  before the next unit. Source-buffer growth obtains scratch admission before `try_reserve_exact`,
  and parsing obtains a separate conservative scratch admission. Metadata is telemetry only and
  cannot authorize a read.
- Production service construction has no generic collector trait or constructor. The fixed
  authority callback is private to `native_plugin_loader`; fixture collectors exist only under
  `cfg(test)`. Root identities are likewise internal authority values, so callers cannot invent
  aliases to defeat same-root coalescing.

This is still a design/recovery record, not a fixed return: r3 has no completed Cargo job, review
acceptance, Runtime11 upward rerun, managed commit, or RSS-bound claim.

## r3 source-bound validation status（2026-07-29）

- r3 re-claimed its exact 16-path scope without conflict after the default lease elapsed, then
  created source snapshot `1282` for the 15 Rust paths. The manifest fingerprint is
  `962d9bf153db5a70136f9836b93b196793efa86ca52026d61d1fd01043fd6460`.
- `rustfmt --check` passed for the changed Rust paths. The static hard-cut scan found no public
  `discovery_refresh_service`, production `NativePluginDiscoveryCollector`, obsolete permit
  lifetimes, or closure-style reservation execution.
- Exact source-bound lib-only reservation `4a970d9c94164d07b71c01bd21f6f87b` was created for
  `cargo +1.94.1 test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1 --
  --nocapture --test-threads=1`. Its consume correctly returned
  `cargo_cpu_reservation_not_fifo_head`: pending Plugins09 reservation
  `6aa1533ac5df4cca8361e244b6a1bf0a` is the predecessor. A subsequent independent static review
  found that a superseded forced refresh would re-submit in its wait loop, so this unbound
  reservation was released before correcting that livelock; no job or Cargo result exists.

The released reservation is non-acceptance evidence only. This failure remains open and has no
fixed return, review acceptance, Runtime11 upward rerun, managed commit, or RSS-bound claim.

## r3 复审否决与 r4 恢复（2026-07-29）

The replacement r3 reservation `49111db0cd9d4332befc19e99f0c9d52` is pending behind the same
Plugins09 FIFO predecessor and has no job. Independent review rejects the current r3 source shape
before that reservation can be meaningful:

- Exact-limit manifests currently attempt an extra admitted byte solely to prove EOF, rejecting a
  valid file at `max_read_bytes`. The current vector may also grow to `max_read_bytes` after only
  a `max_scratch_bytes` ledger entry, which is unsound when those limits differ.
- A completed first-refresh failure remains in the authority's in-flight map forever, so a normal
  later `discover()` cannot retry. The map also needs bounded terminal cleanup.
- Spin/yield waiting is not an acceptable terminal notification and can starve the same I/O pool
  that must execute collection. Ticket completion needs a blocking notification plus an explicit
  no-wait-on-collector-lane rule.
- The fixed authority factory remains visible to the parent loader module and takes arbitrary
  budgets; `shutdown()` is still callable on the service type. Root identities are lexical rather
  than canonical, so aliases can consume multiple root slots.

The reviewer read an earlier r3 snapshot before the single-forced-submit loop correction; current
source resets `force_refresh` after its first submission and has a regression guard for that
specific latest-wins livelock. It does not resolve the four findings above.

The pending `49111db0cd9d4332befc19e99f0c9d52` reservation must be released before r4 source
changes. r4 must include `discovery_refresh/ticket.rs` in its immutable scope, preserve this
failure as open, and make no Cargo-green, fixed, commit, or upward-runtime claim until a fresh
source-bound gate and review accept the repair.

## r4 validation status（2026-07-29）

- r4 owns the same recovery scope plus `discovery_refresh/ticket.rs`. It replaces spin waiting
  with ticket terminal notification, makes forced full-root notifications merge into one queued
  successor, clears completed in-flight tickets for retry, rejects synchronous waits from the
  collector I/O lane, and gates production service construction with an authority-private
  capability.
- Manifest collection now uses metadata only for early rejection and fixed logical source-buffer
  capacity. Actual reads remain chunk-admitted, exact read-limit files do not require a probe byte,
  and source plus parse scratch are separately admitted. New focused coverage addresses terminal
  notification, exact read boundary, and read/scratch separation.
- Snapshot `1286` covers the 16 Rust paths with manifest fingerprint
  `80b67fc80b4f7a54bad85b8eac96b16d263c11572a3d297f63a945f8d2e4d845`.
  Exact reservation `0d18d7a7b2f34baf970141cb7b9b0b49` is pending for the canonical lib-only
  command. Its first consume correctly returned `cargo_cpu_reservation_not_fifo_head` behind
  Plugins09 reservation `6aa1533ac5df4cca8361e244b6a1bf0a`; no job or Cargo result exists.

The failure remains open with no fixed return, review acceptance, Cargo-green result, commit, or
Runtime11 upward rerun claim.

## r4 审阅续证据（2026-07-29）

Independent r4 review accepts that capability-gated construction, terminal ticket cleanup, and
Condvar waiting remove their earlier bypasses, but rejects the current source before the pending
`0d18d7a7b2f34baf970141cb7b9b0b49` reservation can be used:

- The collector I/O-lane flag currently clears before `complete_generation` delivers terminal
  observers. An observer that calls synchronous discovery can therefore block the same worker
  pool. The guard must cover collection, completion, and observer delivery.
- A task queued behind saturated I/O workers reaches no collector checkpoint, while an unbounded
  ticket wait has no pre-start deadline terminal. Ticket waiting must time out at its immutable
  deadline and terminalize the ticket safely.
- The fixed source buffer does not verify post-read handle length, so a manifest that grows after
  metadata may publish a truncated prefix. The opened handle must be checked before and after the
  bounded read and changes must produce a typed failure.
- The fixed allocation needs a fallible `try_reserve_exact` path, and root canonicalization must
  be bounded-cached by the authority so warm discovery/generation projection does not stat on each
  call.

New r4 tests must cover queued-before-start deadline, terminal-observer re-entry, handle growth
rejection, and warm root identity reuse. The pending reservation is to be released before these
source changes; this failure remains open with no test, review, fixed, commit, or upward-runtime
claim.

## r5 前向修复与二次审查（2026-08-01）

- Session `frameworks04-native-discovery-authority-materialization-r5-20260801` continued from the
  integrated r4 source. Its first independent review was C0/I3/M0: it found an unmetered
  `CollectingManifestVisitor` aggregate/test bypass, a test-only unbounded manifest read, two stale
  source-string assertions, and missing behavioral coverage for queued-before-start deadline,
  production read stability, and warm root reuse.
- The non-conflicting owner scope now hard-cuts the aggregate collector and its collection result;
  all retained discovery paths use the canonical visitor traversal. The test-only
  `fs::read_to_string` path and disconnected stability helper are deleted, and dead read/change
  candidate-error variants are removed rather than retained as compatibility contracts.
- The queued-deadline regression now saturates a real one-worker `TaskPool`, verifies the ticket
  terminalizes before collector start, releases the worker, and proves late collector completion
  cannot replace `DeadlineExceeded`, publish a snapshot, or erase the typed last failure. The
  manifest-length regression calls the production `ensure_bounded_read_is_stable` helper, and the
  source guards match the current multiline completion and authority failure-report calls.
- The first r5 re-review was C0/I1/M0. Every metered-collector finding above passed; the remaining
  Important item was the lack of a behavioral warm-root no-restat regression. The successor test
  now discovers through a lexical `..` alias, removes the canonical root, and requires warm
  generation and report projection to preserve the published identity and generation. A repeated
  canonicalization/stat path would lose that canonical identity after removal instead of serving
  the last-good snapshot.
- Static evidence for the five changed Rust files is GREEN under Rust 1.94.1 `rustfmt --check` and
  scoped `git diff --check`; the latter reports only existing CRLF conversion notices. The final
  independent re-review is C0/I0/M0, Source Ready: it confirms the alias-removal regression fails
  if warm lookup re-canonicalizes to the now-missing lexical path and therefore behaviorally guards
  the cached identity. No Cargo command has been run directly or claimed GREEN.

This record remains `open` / `resolving_failure` until a fresh coordinator-managed lib-only gate
returns. Pending or running coordinator work delays only accepted closeout; it does not convert
Source Ready evidence into a fixed artifact or stop subsequent Goal execution.

Coordinator receipt note: exact-scope lease refresh succeeded as request
`f9dd87d5a97d442ea2579c56544d1921`. The following snapshot request
`ae1c9d9529a6432db2934f4e75e338b6` timed out during coordinator health preflight with explicit
`submission: not_submitted`; therefore there is no snapshot, reservation, job, or test result to
reuse or claim. Per receipt-driven execution policy, no recovery polling or resubmission loop was
started. The locally recomputed ordinal, LF/no-final-LF nine-Rust-path source fingerprint is
`d3f1dc5efb84b2533cfc3171ab98c0ae6f6b2d50aed2ce0643f9632987773408`; it records Source Ready
identity only and is not a substitute for a coordinator snapshot.

## r6 集成源码验证恢复（2026-08-08）

- r5 已复审通过的 authority、ticket、bounded-read 与 focused test 源码已进入共享 main；当前
  `discover`/`discovery_refresh` canonical owner 范围相对 HEAD 无额外实现 diff，未回滚或重建旧
  aggregate collector、公开 factory、spin wait、unbounded read 或第二缓存。
- successor `frameworks04-native-discovery-authority-validation-r6-20260808` 已无冲突领取 failure
  与 canonical authority/test exact14，准备基于当前不可变源码提交 Rust 1.94.1 lib-only focused
  gate。r5 的旧 timeout 仍是 `submission: not_submitted`，不可复用为 receipt 或验证结果。
- 状态保持 `open` / `resolving_failure`：实现与独立二次审查 C0/I0/M0 已完成；新的受管 receipt、
  terminal GREEN、Runtime11 upward gate、fixed return 与 milestone commit 仍 pending。
