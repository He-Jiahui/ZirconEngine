# Claude engineering review repairs

Scope: the 2026-10-17 review, on `fix/claude-review-20261017` only.
No push, PR, sibling ZrVM changes, or Jenkins installation changes are authorized.
The preexisting Linux `.codex/config.toml` overlay is preserved.

## Validation limits

This Linux checkout has no verified Jenkins activation, Windows drive-root build
storage, the pinned Rust toolchain, or PowerShell. Python and source checks run locally;
compiled Rust, TLS integration and Windows acceptance remain pending unless
explicit evidence below says otherwise. No build outputs are created here.

## Items

- C1: Separate source contracts from Rust compilation. Use one per-member fmt
  contract, refresh stale source assertions, add path-budget tests, verify
  referenced unittest modules are tracked, and grandfather only review-baseline
  documentation/test-layout debt. The baseline reads Git objects at the reviewed
  revision; CI fetches full history. Existing debt stays visible in reports.
  Validation: 61 focused Python tests passed; docs and test-layout gates report
  zero new violations (36,858 and 63 existing respectively). Nine existing path
  budget violations and 30 existing layering edges are also explicitly baselined.
- H1: Remove managed-storage enforcement from the reusable interface build script;
  retain the generated availability fallback. Move the profile contract workflow
  onto the guarded Windows output lane with Rust 1.94.1. Build-script and workflow
  source checks passed; cross-platform Cargo validation requires approved storage.
- H2: HTTP reuses the rustls handshake verifier (chain, hostname, pin), forbids
  pinned redirects and plaintext TLS material. Replace the old untrusted/mismatched
  hostname fixture with a trusted loopback CA/leaf. Add wrong-pin/no-request
  regression and retain a successful pinned request. Fixture and source checks
  passed; compiled TLS tests are pending the Windows validation lane.
- H3 (in-repo parts): Disable fsmonitor/hooks on Jenkins Git entry points, pin
  the delivery import and execute the verified bytes without rereading. Narrow
  the canonical Windows profile: Git data, Jenkins runtime/state, workflows and
  executable helper directories are read-only. Commit the profile through the
  index while preserving the preexisting Linux overlay. 29 focused Python tests
  passed (four Windows-only cases skipped); 205 production Python files parsed.
  Skipped: relocating the live Jenkins install/authorization DB, immutable
  external bootstrap and interpreter pinning; those need operator installation
  changes. No service configuration or runtime was modified or activated.
- H4: Self-hosted jobs run only on main/master pushes; PR source contracts use
  GitHub-hosted runners. Limit tokens to contents:read and disable persisted
  checkout credentials. Add workflow concurrency and fresh run/attempt/job/matrix
  roots for Cargo, rustup, temp, targets, Miri and Zig. Physical/reparse guards
  reject reused output roots. 48 focused Python contracts passed. YAML parser
  unavailable here; actual Windows runner execution remains pending.
- M1: Parse WebSocket URIs with the connection library's HTTP URI parser; policy,
  pin host and connection share the parsed URI. Normalize schemes and hosts.
  Add query/fragment loopback-spoof and uppercase-scheme regressions. Source
  wiring checks passed; Rust regressions await the guarded build lane.
- M2: Replace blanket source-tree/text ignores with specific state/cache/log
  exclusions. New tests, skills, examples, documentation layouts and CMake/license
  text are visible to ordinary Git add. CI checks tracked-but-ignored paths and
  representative new files; only the two exact L4 legacy cache outputs remain
  temporarily allowlisted. Three focused ignore-contract tests passed.
- M4: Cache candidate validation by tree, runtime, recipes and pinned inputs.
  A fetched, fast-forward origin/main receives a separate full root/plugin
  workspace check/test/clippy batch before local synchronization; divergence,
  dirty state, unknown native outcomes and failed validation retain local main.
  Already-synchronized merged PRs can finish queue reconciliation. Waiting for an
  open PR is a successful scheduled observation. Four new real-Git queue tests
  and seven existing PR-gate scenarios passed with Windows read pins mocked for
  portable logic coverage. Native validation/service acceptance stays pending.
- M5: Remove the deleted Pester suite and exact aggregate count, check every
  suite exists, and require at least one test per listed suite in aggregate.
  Supply Rust 1.94.1 explicitly. Check the physical D: drive before toolchain/
  cache setup and give Cargo/toolchain/temp paths a unique run root. A hosted
  image without approved physical storage now fails with an explicit requirement.
  Workflow/suite/toolchain source checks passed; PowerShell execution is pending.
- M3: Pin both bindings in plugin/WoC manifests and all three lockfiles to
  public upstream revision `3c1352ecb6ddfab02752d46682cc376558d5f481`. Restrict cargo-deny
  to that git URL and revision-qualified sources; explicitly fetch locked plugin
  dependencies in CI. Update the owning audit and version-pairing docs. Upstream
  manifests were inspected through read-only HTTP; no sibling checkout/session
  was accessed. TOML/source contracts pass; locked Cargo/native validation is pending.
- M6: Preserve 321 generated audit/receipt files (60,187,713 logical bytes) in
  the ignored task artifact archive, with SHA-256/length manifest; Git also retains
  their originals. Artifact-store import by Jenkins remains pending. Reject new
  source commits of these generated reports. A reproducible metadata-only sweep
  repaired 16,128 unambiguous moved references and retired 20,730 declarations
  with no current owner. Historical prose is untouched. The raw audit now checks
  6,360 documents with zero invalid references, so docs no longer need a debt
  exemption. 45 focused Python checks passed. Acceptance results are unchanged.
- M7: Make `SystemParam` unsafe with explicit access, aliasing, state ownership
  and item-lifetime obligations. Mark all 18 implementations (including tuple
  generation and probes) unsafe with local justifications. Removed-component
  readers now conservatively declare their retained World access. Add a complete
  compile-fail example for safe implementations. Source inventory and diff checks
  pass; compiled ECS/doctest acceptance is pending the guarded Rust lane.
- L1: Correct the Cargo hook denial to `tools/dev/local-cargo.ps1`; document
  its Bash-only, fail-open text matching and known bypasses in the hook and
  current operator guide. It is explicitly advisory. A focused import/decision
  check passes without creating state; wrapper existence and diff checks pass.
- L2: Remove the commandlet environment RAII mutation. Pass the resolved
  project root through App/editor snapshot, capture, materialization and empty-
  scene paths; keep project/fingerprint validation. Existing normal-host evidence
  retains its read-only startup environment opt-in. Explicit forwarding and
  no-mutation source checks pass; editor compilation/capture is pending.
- L3: Label literal allocation counts as `_expected` across 73 affected
  fixtures, and tab-layout formula bytes as `_estimated_bytes`. Timing and actual
  allocator counters remain unchanged. Update the owning Python contracts and
  add a repository guard against unlabeled constant allocation evidence. Repair
  stale moved-fixture reads in those contracts. 14 focused tests pass; static
  comparison confirms timing fields are unchanged. No performance run is claimed.
- L4: Remove the staged registry and two contact-shadow shader-cache outputs.
  The prewarm success test reproduced the exact custom-ID registry through its
  shared relative `stage/` fake; it now owns a temporary output. The writer rejects
  relative destinations before any mkdir. Ignore generated stage output and guard
  against tracking stage/shader caches, removing the M2 allowance. Contact-shadow
  tests already use the physical-output-checked project helper. Seven focused
  Python tests pass with subprocesses mocked; no compiler was invoked.
- L5: Stage under a versioned parent with a per-generation OS file-lock lease
  and UUID-bound marker. A parent lifecycle lock closes the create/sweep race;
  startup sweeps only unlocked, marked, plain-file generations. Live, malformed,
  unmarked and reparse paths are retained. Cleanup follows DLL/directory guard
  release. Add live/stale/foreign and symlink regression tests. Source/lifecycle
  checks pass; Rust/Windows tests remain pending. Legacy unmarked roots cannot
  establish ownership and are intentionally retained for operator review.
- L6: Pin every third-party action in all four workflows to a full commit.
  Resolve checkout/setup-python/Pages tags through read-only upstream GitHub refs;
  reuse the existing reviewed Rust/cache/upload pins. Every Rust installation now
  supplies 1.94.1 or the existing Miri nightly explicitly. Extend CI contracts to
  all workflows and update tag-dependent assertions. 21 focused Python tests
  pass; workflow execution remains pending. Resolved pins: setup-python v5
  `a26af69be951a213d495a4c3e4e4022e16d87065`, configure-pages v5
  `983d7736d9b0ae728b81ab479565c72886d7745b`, upload-pages-artifact v4
  `7b1f4a764d45c48632c6b24a0339c27f5614fb0b`, deploy-pages v4
  `d6db90164ac5ed86f2b6aed7e0febac5b3c0c03e`.
- L7: Replace generic mutex proofs with `ZrVmLockGuard`, constructed only
  inside the process-lock module. Its private mutex guard retains ownership and
  a Cell marker prevents sharing one token between threads. Update owner and
  instance consumers; retain the existing poison/concurrency regression. Token
  privacy and consumer inventory checks pass; compiled VM validation is pending.
- L8: Propagate an absolute admission deadline through scoped dispatch and
  use `send_timeout`, sharing the existing five-second host admission budget.
  Timeout reports AdmissionIncomplete (ABI Error), disconnect reports Terminated;
  admitted commands still wait without an execution timeout for the Drop signal.
  Add full-queue/shutdown-lock and admitted-borrow completion regressions. Source
  checks pass; compiled concurrency tests remain pending the guarded Rust lane.
  Follow-up review fixes the completion regression's Send capture by moving its
  receiver and borrowed output into the callback.
- L9 (tooling): Parse actual include_str! literal arguments through the linked
  catalog module tree, resolve exact manifest paths, and ignore comments, fake
  code strings, backup filenames and orphan modules. Add Rust regressions for
  these cases and the current split manifest table. Derive the Jenkins default
  root from its driver path; its focused Python test passes. Remove 17 nonexistent
  saved documentation grants and add the six current owners. Canonical TOML and
  source checks pass; preserve Linux overlay bytes through the index. Compiled
  catalog tests remain pending.
- L9 (WebSocket): Replace all three production queue polling loops with
  registered capacity notifications. Frame drains and manager event drains wake
  readers; registration covers an early-drain race, and writer failure cancels
  readers. Capacity and cancellation use distinct Notify objects. Clone callbacks
  under the registry lock, then invoke after releasing locks. Recover only the
  primitive enum/frame/event locks without asserting that partial events are valid.
  Move existing admission tests out of production source; add notification-race
  and poisoned-buffer regressions and update explicit test drains. Source checks
  and the test-layout gate pass (zero new violations); compiled socket/concurrency
  tests remain pending. RPC protocol-state poison handling follows.
