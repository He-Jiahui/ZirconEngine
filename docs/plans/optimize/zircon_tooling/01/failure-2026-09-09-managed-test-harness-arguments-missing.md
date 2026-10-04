---
handoff_kind: failure
status: open
created_at: 2026-09-09
summary_slug: managed-test-harness-arguments-missing
origin_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
fixing_plan: docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
origin_child_dir: docs/plans/zircon_runtime/runtime/07
fixing_child_dir: docs/plans/optimize/zircon_tooling/01
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/managed-build-policy.ps1
  - .codex/skills/zircon-dev/scripts/managed-cargo-storage.ps1
  - zircon_runtime/src/core/runtime/tests/events/benchmark_evidence.rs
tests:
  - .codex/skills/zircon-dev/scripts/validate-matrix-pipeline.Tests.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - managed Windows EventBus original and Foundation regressions with explicit serial test harness arguments
  - two managed Windows EventBus Runtime07 benchmark batches with serial test harness and visible raw evidence
---

# Tooling01: managed validator cannot express required test harness arguments

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 来源执行切片：Current EventBus behavior and performance acceptance.
- 修复责任计划：`docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md`
- 交接原因：The shared Windows validator lacks explicit serial and output-capture
  controls required by existing managed performance gates.

## 失败现象与复现证据

The current `validate-matrix.ps1` exposes `TestFilter` and `IgnoredTests`, but
neither a test-thread option nor a no-capture option. `Get-CargoArgs` only
appends `-- --ignored` for an ignored test run. The managed policy deliberately
scrubs environment variables whose names start with `rust`, so setting
`RUST_TEST_THREADS=1` or `RUST_TEST_NOCAPTURE=1` in its caller does not configure
the actual Cargo test harness.

A read-only execution of the current `scrub_inherited_cargo_environment` with
`{"RUST_TEST_THREADS":"1","RUST_TEST_NOCAPTURE":"1","PATH":"sentinel"}`
returns exactly `{"PATH":"sentinel"}`. `build_policy.py` derives
`clearEnvironment` from this function; `Push-ManagedCargoEnvironment` clears
those variables before invoking `cargo_pipeline.py`, which inherits the cleaned
environment for the real Cargo child process.

Managed job `0c1cf55c6dde4b39b0dab91ba311b8b5` passed 15 lower EventBus tests
with 8 ignored and 6827 filtered out on input
`runtime07-eventbus-guards-3277-20260909`, manifest
`ef35355b6952453dac4d0f899a2d62843c17e63e394e0f6bf4748028ead278db`.
Its caller recorded `RUST_TEST_THREADS=1`, but the actual command has no
`--test-threads` argument and the policy removed that variable. This remains
valid functional evidence, but it is not serial-test acceptance. The JSON and
log remain unchanged under
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime07-eventbus-guards-3277-20260909/results/runtime07-eventbus-owner-support-3277.{json,log}`.

Inspected current hashes:

- `validate-matrix.ps1`: `f188400e7c74264c90cca2bff6d94824ad0d0dfd5471234ab81fa8e15b36da4a`.
- `managed-cargo-storage.ps1`: `3b66c808e9416335f97b54f380183b93d9b574a08e86c4029842c3cebf5bc811`.
- `cargo_command_policy.py`: `d2b2c345d34dbffea0b7da40498fe10f4f4161dc764abe9af01d94166f874bcc`.
- `cargo_pipeline.py`: `c3b8b6c9d388925d9fdbc598586185829a84c33f52a5217c2226a07054450214`.

## 最低共享层根因

Test execution configuration has no explicit validator argument path. Ambient
Rust environment cleanup is intentional; recording requested environment values
in a caller cannot substitute for binding the real harness command. The fix
belongs to validator argument construction and its managed command identity.

## 架构修复验收

- Reconcile the validator and pipeline-test ownership with the unfinished
  `build-reuse-app08-20260905` Session before editing. At diagnosis both current
  hashes match that Session's attributions and its status is stale, not terminal.
- Provide explicit positive test-thread and output-capture controls. Bind them
  to actual harness arguments after the single Cargo `--` separator, alongside
  the existing ignored-test selector. Reject invalid or inapplicable options.
- Verify the real invocation and managed snapshot command agree. Compilation
  and check command derivation must strip test-only options; compiler policy
  and owned target directories must remain consistent.
- Exercise lower argument tests, invalid combinations and an actual managed
  test run that proves the requested configuration and visible output.
- Resume the original EventBus and Foundation batches and both required
  Runtime07 benchmark batches with explicit serial execution. Preserve actual
  `EVENTBUS_BENCH_V2` output and source/command/receipt identities.
- Return evidence to the [origin failure](../../../zircon_runtime/runtime/07/failure-2026-07-17-event-bus-backpressure-and-fanout.md).

## 禁止临时方案

- Do not relax compiler environment scrubbing, inject shell text into a test
  filter, run Cargo outside the coordinator, or change benchmark assertions to
  recover captured output.
- Do not claim serial execution or raw performance metrics from the caller's
  requested environment. Do not overwrite the prior functional receipt.

## 修复结果与回传

Open state: `explicit-test-harness-options_owner-repair-pending`.
Runtime07 has not acquired or edited validator source. Its local submission
helper now rejects the ineffective thread option and no longer requests an
ineffective no-capture environment setting. Both performance batches remain
unsubmitted; no serial, performance, closeout SHA or WeCom acceptance is claimed.

## 2026-09-09 fixing-side execution

The stale `build-reuse-app08-20260905` ownership was reconciled through the
coordinator transfer fingerprint
`d4826e9900da4f8c4ed553454909cbfc6b5047c3cea25d93c4104c89db806804`.
The stable fixing Session is `failure-roll-01a07160-tooling01-r1`; its lease
covers this record, the validator/policy sources, the coordinator pipeline and
tests, and the Runtime07 benchmark evidence helper. The transfer also brought
`tools/session_coordinator/tests/test_app08_cargo_pipeline.py` into that scope
before its attribution.

The validator now exposes `-TestThreads` and `-NoCapture`. For test stages,
`Get-CargoArgs` appends one `--` separator followed by `--test-threads <n>`,
`--nocapture`, and the existing `--ignored` selector in deterministic order.
The CLI rejects a non-positive explicitly supplied thread count and rejects
test-only options when no test stage is selected. Check/build command
derivation remains free of harness-only arguments.

Focused evidence captured under the fixing Session:

- Pester harness-argument coverage: 1 passed.
- Full coordinator Cargo-pipeline unit suite: 19 passed.
- Combined command-policy and pipeline suites: 31 passed.
- Validator pipeline Pester suite: 7 passed.
- Positive dry-run emitted `cargo test ... -- --test-threads 1 --nocapture`;
  negative dry-runs rejected `-TestThreads 0` and `-NoCapture` with
  `-SkipTest`.

The first real managed Windows validation was submitted as job
`8ba0af8860b54c4ab49c5a5491847925`; its receipt is
`E:/cargo-targets/zircon-engine/pool/a7c726fcdd457d7fdb7ca72b3e112234afdbb64adbf582a15546e2d1366ad3b0/.zircon-compile/metrics/validation-8ba0af8860b54c4ab49c5a5491847925.json`.
The recorded test command contains the explicit
`-- --test-threads 1 --nocapture` arguments, proving they reached the managed
Cargo invocation. The job exited before tests at an unrelated Runtime compile
error in `zircon_runtime/src/plugin/native_plugin_loader/native_artifact_trust.rs:734`
(`E0716`, a temporary `file_name()` value was borrowed); no EventBus or
Runtime07 benchmark acceptance is inferred from this receipt. The job was
released after the failure and no cross-owner source edit was made.

Current state remains `explicit-test-harness-options_external-runtime-blocked`:
the argument-path repair is locally verified, while the original EventBus and
Foundation regressions plus both Runtime07 benchmark batches still require a
successful managed run, independent review, failure return, closeout SHA, and
WeCom result. No fixed status is claimed.

## 2026-09-11 rolling repair verification update

The stable fixing Session `failure-roll-01a07160-tooling01-r1` renewed its
leases and re-attributed the complete owned source and test set before this
record update. No validator, coordinator, or Runtime07 source was changed in
this pass; the existing lowest-layer repair remains the subject of the evidence
below.

Fresh local dynamic evidence against the currently attributed hashes is:

- `python -B -m unittest tools.session_coordinator.tests.test_cargo_command_policy tools.session_coordinator.tests.test_app08_cargo_pipeline -v` completed 33/33.
- `Invoke-Pester -Script .codex/skills/zircon-dev/scripts/validate-matrix-pipeline.Tests.ps1 -PassThru` completed 7/7.
- `Invoke-Pester -Script .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1 -TestName 'Explicit test harness Cargo arguments' -PassThru` completed 1/1, including the single Cargo `--` boundary and `--test-threads 1 --nocapture` ordering.
- A child `validate-matrix.ps1 -DryRun -SkipBuild -SkipTest -RunConventionClippy -ManifestPath zircon_plugins/Cargo.toml` returned exit code 1 and `Convention Cargo gates require the repository root Cargo.toml.`, matching the direct convention-gate assertion.

The broader `Validate matrix CLI dry-run parsing` Pester describe displayed
individual successes through the serial/no-capture and invalid-harness cases,
but the Pester 3.4.0 invocation stopped before a final suite summary while
entering later unrelated convention-gate coverage. That incomplete aggregate
run is explicitly excluded from pass/fail evidence; it is neither reported as
a full green suite nor treated as a validator regression.

Managed Cargo has not been resubmitted. `E:\Git\zr_vm` remains materially dirty
at this check, so its existing coordinator admission blocker remains unchanged.
No Cargo process, EventBus/Foundation regression, Runtime07 benchmark result,
or closeout is inferred from the local evidence. The next action is a sealed
non-Cargo coordinator ticket for the Python policy/pipeline suite if its
dependency gate admits it; the original Windows Cargo and benchmark acceptance
must wait for the external worktree condition to change.

The dependency gate admitted that narrow ticket on 2026-09-11. Ticket
`9b2eefa6701242418892218cdbab6109` is queued for the exact four-file manifest
`6700d13d434409be3700bdaa4e2e69b7bcae8f43a6a38dd0b1f77d6fc8023eec` and the
33-test Python command recorded above; its admission blockers are empty. This
is a pending managed validation result, not acceptance evidence. The fixing
Session is therefore waiting for that result and for the separate external
Cargo precondition.

The ticket reached coordinator materialization on 2026-09-11 and failed before
the Python command started. Its copy job `0adbd316f0fe48a789fb6c45c1423d33`
reported `validation_copy_dependency_archive_failed` during
`template_dependencies`: the request had incorrectly declared the two
untracked, Session-owned pipeline overlay files as baseline dependency roots,
so the pinned `git archive` could not contain them. This is a validation-input
construction error, not a test failure or source regression. The replacement
request must use the tracked `tools/session_coordinator` tree as its readonly
baseline dependency root and preserve the exact four-file source manifest as
sealed overlays. No acceptance, return, or closeout is inferred from the
failed materialization.

After correcting the dependency-root declaration, coordinator accepted
replacement ticket `3997a6d435c544a887a8e15ab827e8bc`
(`tooling01-harness-policy-pipeline-20260911-r2`) with no admission blockers.
It seals the same four-file manifest
`6700d13d434409be3700bdaa4e2e69b7bcae8f43a6a38dd0b1f77d6fc8023eec`
and uses tracked `tools/session_coordinator` only as the readonly template
root. The ticket is queued; no worker execution or pass is claimed yet.

The replacement worker did execute the sealed copy and Python command. Ticket
`3997a6d435c544a887a8e15ab827e8bc` / job `1e5d3f5da6274e79a21a2806425dd601`
ended with exit 1 after 33 tests were collected: 30 passed and three errors.
All three are dependency-closure errors, not harness-argument assertions:
the sealed copy lacks the owner-held `build_policy.py` module needed by two
policy tests, and the copied `cargo_runner.py`/pipeline implementation lacks
the `_start_registered` seam required by the duplicate-start regression. The
managed receipt records `ModuleNotFoundError` and `AttributeError` and no
Cargo process was launched. This result is retained as a real failed dynamic
run; the ticket is not reusable as green evidence. The lower-layer owner
dependencies must be sealed at matching current hashes before another exact
validation attempt.

To separate the source repair from those foreign dependency seams, two
dependency-independent managed subsets were admitted against the same sealed
manifest and tracked readonly root:

- `64a87885d86c470da1f9451961928810` / request
  `tooling01-harness-policy-subset-20260911-r3` runs the ten policy tests that
  do not import the owner-held `build_policy.py` product-DLL branch.
- `8a75c18977754cf9b91a94bd81971df9` / request
  `tooling01-harness-pipeline-subset-20260911-r4` runs the twenty pipeline
  tests that do not require the foreign `_start_registered` duplicate-start
  seam.

Both tickets are queued with empty admission blockers. They are scoped
evidence slices, not a full-suite pass; the three excluded tests and the
original EventBus/Foundation/Runtime07 Cargo acceptance remain open.

The coordinator completed both dependency-independent slices on 2026-09-11.
Ticket `64a87885d86c470da1f9451961928810` / request
`tooling01-harness-policy-subset-20260911-r3` ran the sealed ten-test policy
command and returned exit code 0 (`Ran 10 tests ... OK`). Ticket
`8a75c18977754cf9b91a94bd81971df9` / request
`tooling01-harness-pipeline-subset-20260911-r4` ran the sealed twenty-test
pipeline command and returned exit code 0 (`Ran 20 tests ... OK`). These are
fresh managed dynamic receipts against the existing source manifest and are
reusable only for the covered assertions. The two product-DLL policy tests
still require the owner-held `build_policy.py` closure, and the duplicate-start
pipeline test still requires the owner-held `_start_registered` seam. The
full 33-test ticket therefore remains failed/blocked, and no Cargo, EventBus,
Foundation, Runtime07 benchmark, failure return, or closeout is inferred.

## 2026-09-21 managed full-suite receipt reconciliation

The preceding statement about the full 33-test ticket is superseded by the
later terminal coordinator receipt. Ticket
`1a1514d0be8142aeb8eb17e94a923536` sealed the complete twelve-path manifest
`362f3537da625640149758356f9fbdcd2ac81f0c3604b1ebd32812d80acf6f04`
and ran the exact command
`python -B -m unittest tools.session_coordinator.tests.test_cargo_command_policy tools.session_coordinator.tests.test_app08_cargo_pipeline -v`.
Job `86a573d8c00b4863a1301591ce3cb0f5` / run
`1a1514d0be8142aeb8eb17e94a923536` exited `0` with `Ran 33 tests in
0.865s` and `OK`; coordinator events `10144` through `10152` retain source
sealing, run linkage, terminal evidence, and completed cleanup. This is valid
managed evidence for that historical sealed source only.

It is not current-source acceptance for the whole twelve-path closure. Three
manifest paths have since changed under their existing owners:

- `tools/session_coordinator/build_policy.py`: sealed
  `72012d7262f70fb4e150bbf6b57e0a46784c0cf4574597166adbaf8ea0ff47a3`,
  current `bbe17eb2505020e0c05a1743fd9583ebad9eefab4dc8719d1a93b6d9d36ef498`.
- `tools/session_coordinator/cargo_pipeline.py`: sealed
  `7db5f3d7a4155fe61898ecb2bde58604227ad5a5ab6bbc83a0faeeba470424ee`,
  current `bea6f67c24220ec2ff24d448c64f4c4ab397452d816867f40ce3b95ca43c357f`.
- `tools/session_coordinator/tests/test_app08_cargo_pipeline.py`: sealed
  `8870ff216c41b1fc3c60b9e818c21fa3929f5e65c0389e7e5f436391e5b91fe6`,
  current `13076f9343b9452a7c7b27ccc9c2a2bafb5cc1805ca79d32d590ca2f81775be8`.

Successor Session `failure-roll-01a084c8-tooling01-harness-r2` owns only this
failure record and deliberately does not transfer or reattribute those mixed
owner sources. Against the current working bytes, the same local Python suite
completed `34/34`, and the targeted Pester `Explicit test harness Cargo
arguments` case completed `1/1`; both are current-source diagnostic evidence,
not coordinator-managed acceptance. A fresh immutable full-suite ticket must
be submitted by, or through an audited transfer from, the current source
owners rather than absorbing their unfinished work into this lifecycle.

The original EventBus/Foundation regressions and both Runtime07 performance
batches remain separately unexecuted on a matching current source/configuration
snapshot. Their managed Cargo admission is still blocked by the unchanged
foreign `E:\Git\zr_vm` worktree. No source edit, benchmark acceptance,
canonical return, closeout commit, or WeCom result is claimed by this
reconciliation; the failure remains `open` / `waiting_validation`.

## 2026-09-30 current managed harness-argument lower evidence

Stable fixing Session `failure-roll-01a0df1a-tooling01-harness-r3` acquired the
eight exact current script/document paths through audited ownership transfer
`4980c095f8ff467dbb749d888dd3d2fb`, preserving the existing source repair.
The seven-script manifest is
`b5ed846f03825752da2de903d98ef2ec502585dd413302dcb986ecf69bbd6023`,
pinned to baseline `628` / `bc02eefafead65dbf5050482110e8175250a5e77`.

Ticket `6d1c786b00bb4a4a9c3765b5f8194805` failed before Pester assertions:
the bundled PowerShell 7.6.5 installation lacks the built-in Utility module,
so Pester could not resolve `Add-Member`. Its failed receipt is retained.
Replacement ticket `a505c94520554138aa2c02e74cea90eb` uses the same source
manifest with Windows PowerShell Desktop `5.1.26100.9444` and Pester `3.4.0`,
both checked by the sealed command. On 2026-09-30 its managed run exited `0`
and actually executed `Explicit test harness Cargo arguments`: `Passed: 1
Failed: 0`. This covers the single Cargo `--` boundary and the ordering of
`--test-threads 1 --nocapture --ignored` in the real `Get-CargoArgs` output.

This is one current lower gate. The pipeline Pester and full Python
policy/pipeline gates, original serial EventBus/Foundation regressions, and
both ignored Runtime07 benchmark batches remain pending on matching complete
source/configuration snapshots. No Cargo ran for this result; the lifecycle
remains `open`, with no failure return or closeout acceptance.

## 2026-09-30 managed CLI refusal and compile-stage argument evidence

Ticket `f97857717f6c4fcc9174ff3249de0f6b` exited `1` before target cases:
its clean resolver assertion used the LF Git blob hash, while pinned
`git archive` under `core.autocrlf=true` materialized CRLF. The blob is
`27d315830c00ef91e8c5241a454f7036a96a2c85fa5a04077b0815f8a64e5a08`
(40,127 bytes); the exact archive member is
`32494eea05fd496503a4e160dcc5cb19e505bb2a441e98ce90ff04b3ba31635f`
(41,107 bytes). The reproduced nine-entry input manifest matches the failed
copy's durable hash
`6726aea5c76369da9e93696ad66ccb85448109becc777e11545547bf1993559d`.
The failed ticket and successful managed cleanup remain preserved.

Replacement ticket `b5da1b112b124038a7ff848c7580746e`, under the same stable
Session and baseline, preserves the six-script manifest
`e581f926e71fe2fe797bb69007505871eed60514652c7a1987b24d6dede792c1`.
Its command separately pins the materialized resolver hash; no source or Git
setting changed. Independent archive/spec reviews passed before submission.
Managed job `5f60e27962814b7688be0834c801b76b` exited `0` on 2026-09-30:
`HARNESS_CLI_GATES Passed: 4 Failed: 0`. The real Windows PowerShell 5.1 CLI
rejects non-positive `TestThreads` and `NoCapture` without a test stage;
real `Get-CargoArgs` check/build commands keep `--locked` and exclude harness
switches. The run also verifies all seven executed source/dependency hashes.

These four cases extend the previous one-case argument-ordering lower result.
The complete pipeline Pester/Python gates, original serial EventBus/Foundation
regressions and both Runtime07 benchmark batches still require matching managed
evidence. The lifecycle remains `open`; no Cargo, return or closeout is claimed.

## 2026-10-01 exact pipeline test postimage reconciliation

The stable fixing Session is unchanged. The full pipeline Pester test file is
ignored and absent from its pinned Git baseline; an empty tracked status did
not prove that it was a clean baseline dependency. Its preserved archived-owner
object is `2a0aa2fe0f032bd24f6b35263f2aac0f3a43edebb36ebb3c55f307ca4e87756b`
(8,323 bytes), while the reviewed current whole-file candidate is
`3e12c63a1641c76dcb6aa6fa42fbeabd0f298988300bbd9e9e4c7e9f3b590bed`
(8,420 bytes). Both raw objects and their exact diff remain preserved.

The current delta adds Base64 transport for JSON command arrays at the Windows
PowerShell 5.1 native Python boundary and corrects Cargo profile wildcard-key
quoting in the two check-identity cases. Existing assertions and comments are
retained. Independent whole-file static review found Critical/Important/Moderate
`0/0/0`; it did not execute any of the seven pipeline cases.

Root explicitly adopted that existing reviewed candidate, without claiming
authorship of its earlier delta. Supported transfer preview
`87a3d0683bc347dab4edce38ed09b334`, fingerprint
`a8bfed5363dab9f48efe6edd904ba8971beae99f1d8b566330f52ffa56eddfe4`,
and apply `0ce2a8fc11db431d8b8c88643ab7f5c5` completed for only this exact
file. The atomic apply extended the same fixing Session's scope from eight to
nine paths and attributed its current hash at epoch `628`. No script bytes or
foreign Python sources changed. Exact lease release
`e2c913cf66f6441ebaf8ee157cd43409` completed with `released=1` and no patches.
The overlapping audit group received one completed-transfer notice, preserving
historical comment provenance and the product exclusion.

This resolves the test-file ownership and sealing prerequisite. It does not
execute the full pipeline gate: its build-policy/check-command Python closure
still requires exact owner-safe current inputs and managed acceptance. The
original serial EventBus/Foundation regressions, both Runtime07 performance
batches, return, review and closeout remain open. Existing tickets are unchanged.

## 2026-10-02 exact Python closure and full pipeline ticket

The full seven-case Pester path requires four current Python inputs in addition
to the owned validator scripts: `build_policy.py`, `build_configuration.py`,
`cargo_command_policy.py` and `cargo_pipeline.py`. The normal source basis is
the trusted Git archive of `bc02eefafead65dbf5050482110e8175250a5e77`;
missing objects in an old epoch's physical store are not evidence that a path
is absent from that Git basis. The required current inputs and the six reusable
baseline dependencies were independently reviewed; static C/I/M was `0/0/0`.
This review ran no Pester or Cargo and supplied no dynamic acceptance.

Root adopted these four exact current postimages through normal preview
`73c0bbdfee704e01856e091d96baa4fa`, fingerprint
`aed8c6c90e0d205f7e02a5224053f24652499809e44feee2f9feea7dc4d7fe1f`,
and apply `1c96541662504266a61e6bc398146681`. The source Session
`astra-tooling-dirty-external-snapshot-20260926-01a0df17` was stale at the
preview and retained its original status, baseline and historical scope.
The supported apply extended the same Tooling01 fixing Session from nine to
thirteen paths and attributed only the four current inputs at epoch `628`.
All existing bytes were retained; root does not claim earlier delta authorship.

Normal request `04e8be5a1c444e38989fb4d8673e7de2` admitted ticket
`379eb84b85984d43b5d61584d5cd9582` with eleven current source overlays and
source manifest `855a7b0a868e3d40a02f69203c97e1abe2a201369407f0c9092a5390bcecf1d7`.
Its Windows PowerShell Desktop `5.1.26100.9444` / Pester `3.4.0` command verifies
all seventeen executed input hashes and requires seven passed, zero failed and
zero skipped cases. Python use is the actual policy dry-run and check-command
producer; no Cargo command is executed by these seven case bodies.

The ticket became terminal `failed` at `materialization_submit` with
`TransactionAdmissionBusy`: SQLite could not acquire the initial transaction
lock. No Pester case or test-result event was produced. This is a coordinator
materialization failure, not a failing assertion or a passing test; its sealed
manifest, original command and terminal receipt remain preserved. Exact release
`25f833572d644cdfa2bfcae475aa5816` released all eleven source leases with
`processed_patches=[]`, without changing any source byte.

Independent source/spec review is
`.codex/tmp/failure-roll-20261002-tooling01-full-seven-admitted-spec-independent-review-sol.json`
(SHA-256 `56237bf1c2e721f573172136dea1bc28ac2806990dbc8800baf532dec9dd9dc2`).
The lifecycle remains open. A fresh request needs current normal admission and
exact source/lease checks after the contention condition changes; the failed
request is not replayed. The complete Python suite, original serial
EventBus/Foundation regressions and both Runtime07 benchmark batches remain
separate unaccepted gates. No return, closeout SHA, WeCom or push is claimed.
