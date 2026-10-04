---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-09-08
summary_slug: project-root-junction-verbatim-fixture
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
related_code:
  - zircon_runtime/src/asset/tests/project/package_assets.rs
tests:
  - managed Windows static/no-default/locked zircon_runtime --lib asset::tests::project::package_assets
  - managed Windows project-path normalization and registration consumer gates
---

# Runtime04: junction fixture uses incompatible paths and cmd quoting

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：Frameworks02 registration acceptance continued through Runtime04's symlink fixture.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：The Runtime04 package-asset test helper owns the shared junction fixture used by project roots, package roots and root-alias deduplication.

## 失败现象与复现证据

The original `registration` batch reported two alias-root failures before the
independent Runtime11 terminal-observer hang. A narrow managed Windows
static/no-default/locked rerun, job `5269d9421f8846dcb05efaf7388b0edb`, completed
all nine `asset::tests::project::package_assets` tests: 6 passed, 3 failed,
0 ignored, 6840 filtered out. All three failures occur in
`create_directory_alias`, before production identity or registration assertions,
with `Local volumes are required to complete the operation.` They affect
project-root publication, package-root publication and Runtime85 root deduplication.
The original symlink escape test passed in this complete narrow batch.

Input `frameworks01-readiness-payload-3237-20260908` has manifest
`5dc7c3d6891874a69436ba9e7641c9e0863eb7ed548c05db7306233c5da9543d`.
Its `results/runtime04-package-assets-diagnostic-3237.{json,log}` retain the
source-bound receipt and complete failure output. The test file matched HEAD
and the immutable input, SHA-256
`044094915ddca0d298d99ac1e71af054c5a39a28468d7dc4684b0204a810b3c5`.

## 最低共享层根因

The support chain is managed output-root selection, fixture path construction,
the mklink process boundary, then production canonical identity and registration.
Managed `CARGO_TARGET_DIR` is an operational Windows verbatim path. The fixture
formats that path directly into `cmd /D /S /C mklink /J`. The existing
`ProjectPaths::display_path` contract supplies the external-tool path view while
preserving the original operational path for filesystem I/O. The focused RED
regression below confirms the actual verbatim arguments at this boundary.
The shell program was also passed through `Command::arg`, whose C-runtime
escaping is incompatible with `cmd /C`. The path-view-only candidate isolates
this second defect. The complete repair applies display paths at this private
tool boundary and uses `CommandExt::raw_arg` with the shell's outer quote pair.

## 架构修复验收

- Construct an actual junction from canonical verbatim local paths containing spaces;
  assert physical identity and payload I/O through the alias.
- Use the existing project-path conversion at the shared fixture's external-tool
  boundary, retaining operational paths for Rust filesystem operations.
- Run project-path normalization, all package-asset tests and the original
  registration consumers. Resume the complete registration batch only after
  [Runtime11's independent hang](../11/failure-2026-08-23-task-terminal-delivery-bounded-dispatch.md) is resolved.
- Bind exact source, obtain C0/I0/M0 review, return and close out before acceptance.

## 禁止临时方案

- Do not skip failed junction creation or replace physical identity with lexical equality.
- Do not modify the production registry, globally strip operational prefixes, add a new
  path-normalization implementation, or weaken the separate symlink privilege fixture.
- Do not change system privileges, enable zr_vm or absorb another active owner's changes.

## 修复结果与回传

Open state: `focused_regressions_passed_review_and_formal_closeout_pending`.
Fixing Session `failure-roll-01a07160-runtime04` retained its existing identity.
Exact-path transfer `858fdc604f85a2c270d156a19ec6b463258e4f30e434a6a2722e1ae3e5b3bbef`
preserves preimage snapshot 3244. Only the shared fixture and this canonical
record were acquired. Related upstream record:
[Runtime04 symlink fixture](failure-2026-07-17-project-root-symlink-privilege-fixture.md).
The similar App loader fixture remains with its own active owner in
[App08](../../../optimize/zircon_app/08/failure-2026-09-08-runtime-library-junction-fixture-creation.md).
The focused repaired tests pass as recorded below. Formal return, commit and
notification remain pending.

The existing lower conversion regression
`asset::project::paths::tests::normalize_windows_final_path_strips_supported_verbatim_prefixes`
executed in managed job `1bffddc052c9449481e295092a46fab9`: 1 passed, 0 failed,
0 ignored. It covers DOS, UNC, case-insensitive UNC and preservation of an
unsupported volume prefix through the actual `ProjectPaths::display_path` API.
The current path owner, Windows implementation and test source match that
3237 input; no foundation behavior change is needed.

Snapshot 3245 adds actual-argument diagnostics and the verbatim/space-path
regression without changing mklink behavior. Its input
`runtime04-junction-red-3245-20260908`, manifest
`aa02262d17065ed46dc6683213994e0396813f75f975c98f9877d51284870ecc`,
completed managed job `e7d621bb7fb0442391366f569e807b64`: 6 passed, 4 failed,
0 ignored, 6840 filtered out. All four failures print `\\?\E:\cargo-targets\...`
for both link and target. The new case reached mklink with `project alias` and
`physical root` under `zircon_asset_junction verbatim paths_1788880890377761400`;
it failed with the same local-volume error. Compilation and the remaining six
tests passed. The receipt and complete diagnostics are in that input's
`results/runtime04-junction-red-3245.{json,log}`.

Path-view-only candidate snapshot 3247, SHA-256
`d048168e217511f5e240fb634698939c9c55a97ce3314c5ed84b69386f10eae3`,
converts both arguments using `ProjectPaths::display_path` only when constructing
the mklink command. The new test and original production identity assertions are
unchanged from RED. Rust formatting and scoped diff checks pass. Derived input
`runtime04-junction-3247-20260908`, manifest
`3479d55f772dcf1ccba728db54aaa4450c2d470588ac37b6b8d887c11f3f2497`,
differs from RED only by that fixture conversion. Its full package-asset rerun
completed as job `4d1291efa7394af3ab2cf1773686eb20`: 6 passed, 4 failed,
0 ignored. All four errors changed from the local-volume message to
`The filename, directory name, or volume label syntax is incorrect.`
The `runtime04-junction-green-3247` artifact name is only an attempt label;
its receipt is `not_accepted`, not GREEN evidence. Path conversion alone did
not fix the shell argument boundary.

The helper also passed its complete, already quoted shell program to
`Command::arg`, applying C-runtime argument escaping that `cmd /C` does not
consume. The correction uses `CommandExt::raw_arg` with an outer shell quote
pair and the existing quoted display-path operands. This follows Rust's
[Windows CommandExt contract](https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html#tymethod.raw_arg)
for programs with non-C-runtime escaping. The helper accepts only these private,
generated fixture paths. No production API or path identity rule is changed.
The same four physical-identity and alias-I/O regressions must pass unchanged
before the repair can be accepted.

The final source snapshot 3251, SHA-256
`e5479d32794b9f30f33477214f0840997216dfbe5bfb7dc46114078924d31dbb`,
adds this shell-argument correction. Derived input
`runtime04-junction-3251-20260908`, manifest
`55328b53070a78a8374cf55f629fc8501d01dee2f2517a2ddc2db94fbf1bfca9`,
differs from the 3247 input only in this source file. Managed Windows
static/no-default/locked job `836e5a3c4dd347929364305b085984a6` ran the
complete package-assets group: 10 passed, 0 failed, 0 ignored, 6840 filtered
out. All four formerly failing cases reached their original assertions; the
new regression also verified alias payload I/O. The separate symlink escape
test passed without changing its privilege handling. Terminal receipt and
test names are retained in
`results/runtime04-junction-command-line-3251.{json,log}` under that input.
Current source still matches snapshot 3251; scoped rustfmt and diff checks pass.

This is complete focused support and consumer evidence. The broader
`registration` gate remains dependent on Runtime11's separate hang and other
reported owners. Operational matrix receipts still require formal fixing-Session
binding. This lifecycle remains open.

The existing independent task `优化协调器验证效率` reviewed source snapshot
3251 and record snapshot 3254 with Critical 0, Important 0, Moderate 0.
Report: `.codex/tmp/runtime04-junction-3251-review-20260908-result.txt`.
Current, ObjectStore and attributed source hashes matched before and after
review, and no reviewed-path ownership conflict was found. This completes
focused source/evidence review. The archived-reviewer Session lifecycle issue
prevented formal review binding in that attempt. Tooling06 has since returned
the lifecycle fix in [Tooling06](../../../zircon_tooling/session_coordinator/01/fixed-2026-09-10-resumed-reviewer-session-remains-archived.md).
Runtime04 has not yet obtained a fresh formal review binding.
No formal return, closeout SHA or WeCom notification has been generated for this
repair; the accepted test/review evidence is retained without resubmission.

### 2026-09-19 rolling successor source-contract admission

- Successor Session `failure-roll-01a084c8-runtime04-junction-r1` reclaimed the
  shared junction fixture and this canonical record at baseline epoch `611`.
  Ownership transfer fingerprint:
  `fd431a351b0424f2271eb01981984cfc64c45904c503d8bce6616a4e965b36ad`.
  Current fixture hash is
  `e5479d32794b9f30f33477214f0840997216dfbe5bfb7dc46114078924d31dbb`.
- Static request `failure-roll-01a084c8-runtime04-junction-20260919-r1`
  admitted ticket `e45596c5174845fc9c58cd67328ff5bf` with sealed manifest
  `5b1d4e419a873b59fd5e12058f61cc790df2383356ecce9fcb48731077e24133`.
  Its checker verifies the Windows `display_path` + `raw_arg` command boundary,
  the exact `/D /S /C` invocation, and the verbatim-space physical identity
  regression. Status is `queued` pending terminal evidence.
- This ticket is static-only. Fresh managed Cargo, the Frameworks02 registration
  consumer gate, independent review binding, fixed return, closeout, and the
  external `E:\\Git\\zr_vm` clean-worktree prerequisite remain pending.

### 2026-09-19 static terminal result

- Ticket `e45596c5174845fc9c58cd67328ff5bf` passed at
  `2026-09-19T07:17:16.019438Z` in job
  `19f53cbf5bc14315a43544c6ef0ec351`, exit code `0`, with marker
  `RUNTIME04_JUNCTION_FIXTURE_COMMAND_BOUNDARY_CURRENT_SOURCE_CONTRACT_PASS`.
  Cleanup event `10914` completed successfully.
- The result confirms the current source still uses the existing
  `ProjectPaths::display_path` view and Windows `CommandExt::raw_arg` boundary,
  plus the verbatim-space physical identity/payload assertions. It is static
  evidence only; fresh managed Cargo, Frameworks02 registration consumers,
  review binding, return, closeout, and clean `E:\\Git\\zr_vm` remain pending.

### 2026-09-21 independent current-source review receipt

Reviewer Session `review-runtime04-junction-r1` independently rechecked the
single sealed production path without editing it or changing the Windows
privilege/symlink fixtures. Current SHA-256 remains
`zircon_runtime/src/asset/tests/project/package_assets.rs`=`e5479d32794b9f30f33477214f0840997216dfbe5bfb7dc46114078924d31dbb`;
the fixing plan is `846617fcd6662e660d8653599a549e6467213a5f861f8f60e12bd08272a77169`.
The failure record was `d4fec2325915f7b76e73b7dd950779bd886fc8905d0bb063a60d75fe014b23e2`
before this receipt append.

Static evidence:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check`
  and scoped `git diff --check` passed (only the repository's existing LF/CRLF
  warning was emitted).
- Source probe passed as `RUNTIME04_JUNCTION_SOURCE_REVIEW_PASS`. The Windows
  helper is limited to this fixture, imports `CommandExt`, preserves operational
  paths for Rust filesystem I/O, converts only the `cmd.exe` operands through
  `ProjectPaths::display_path`, passes the exact `/D /S /C` argument vector, and
  uses `raw_arg(command)` rather than C-runtime `.arg(command)`. The regression
  creates a canonical verbatim-disk parent containing spaces, asserts physical
  junction identity, writes through the alias and reads through the physical
  root, then removes both the alias and temporary tree. Unix symlink behavior
  and the separate privilege fixture remain untouched.
- The corrected static ticket `e45596c5174845fc9c58cd67328ff5bf` and its
  `RUNTIME04_JUNCTION_FIXTURE_COMMAND_BOUNDARY_CURRENT_SOURCE_CONTRACT_PASS`
  marker are consistent with the current source; the earlier 10/10 package
  asset run is retained as historical managed evidence only, not reused as a
  new Cargo acceptance result.

Independent review result: Critical=`0`, Important=`0`, Moderate=`0`. The
managed Windows `zircon_runtime --lib asset::tests::project::package_assets`
run and original Frameworks02 registration consumer gate still require fresh
coordinator admission; the external `E:\\Git\\zr_vm` dirty-worktree blocker is
not a test result. Canonical `fixed-*`/return, closeout and WeCom notification
remain pending, so this failure stays `open`.
