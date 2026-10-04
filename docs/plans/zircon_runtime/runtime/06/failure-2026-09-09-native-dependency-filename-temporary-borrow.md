---
handoff_kind: failure
status: open
created_at: 2026-09-09
summary_slug: native-dependency-filename-temporary-borrow
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_artifact_trust.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --locked --lib --tests
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --locked --test native_plugin_loader_contract native_runtime_hot_update_loads_real_fixture_from_export_manifest -- --exact --nocapture
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --locked --lib asset::tests::project::manager::targeted_import
---

# Runtime06: native dependency filename borrow blocks test compilation

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Targeted metadata precondition current-source validation, snapshot 3331.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md`
- 交接原因：The lowest failing production owner is native DLL artifact admission.

Runtime04 was validating the targeted metadata preconditions from snapshot
`3331` against the complete current compiler dependency closure. Native DLL
artifact admission belongs to Runtime06. The existing file is attributed to
Session `astra-full-domain-20260905`; the reporting Session does not adopt its
uncommitted implementation.

## 失败现象与复现证据

Managed Windows job `353114f74d5a417b802942d4587a86f5` failed Cargo check with
exit 101 before any test executed. The command was the repository managed
`validate-matrix.ps1` with package `zircon_runtime`, `-NoDefaultFeatures`,
`-SkipBuild`, `-LibTests`, static linking, automatic linker and reuse storage,
filter `asset::tests::project::manager::targeted_import`; Cargo remained locked.
The complete input manifest is
`a51d5734995d5d019fd18d3e3bc97adc2d6cd62ecefc04abd64075a0c283eb65`.
The original log is `.codex/tmp/runtime04-current-targeted-meta-3331-r1.log`.
The input was captured and manifest-verified as
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-current-meta-3331-20260909`;
its results directory retains the managed receipt JSON and original log.

The compiler reports E0716 at `native_artifact_trust.rs:734`:
`entry.file_name().to_str()` borrows a temporary `OsString`, which is dropped
before `normalized_native_dll_name(name)` consumes the borrowed string.
The observed working-file SHA-256 is
`eee8077309d46be7931ae4a63d1d4d07013161585a4b8db3c50dc892ebabdd22`.

## 最低共享层根因

`capture_native_dependency_closure` must retain the owned filename for the
duration of normalization, or borrow the filename from an already retained
path. This is a native-loader Rust lifetime error independent of the targeted
asset metadata change and of the separately skipped `zr_vm` worktree.

## 架构修复验收

- Preserve the DLL regular-file, UTF-8, normalized-name and ambiguity checks.
- Compile the actual current dependency closure on managed Windows with
  `--locked`, and run the existing dependency-admission regression cases.
- Rerun the originating targeted-import filter with nonzero executed tests,
  including its five metadata-precondition regressions.
- Keep the original failure receipt and bind any source fix to its owner.

## 禁止临时方案

Do not disable native admission, exclude the failing module, weaken the
precondition tests, revert another Session's implementation or substitute an
older unrelated compile pass for the originating gate.

## 修复结果与回传

Open. No Runtime04 targeted metadata acceptance or Runtime06 fix is claimed.
Runtime04 continues dependency-independent migration and management work.

## 2026-09-09 Runtime06 owner repair

The exact source path was transferred from `astra-full-domain-20260905` after
that owner explicitly authorized the handoff; the remaining Astra write scope
was preserved. `capture_native_dependency_closure` now stores
`entry.file_name()` in a local `OsString` before borrowing its UTF-8 view, so
the filename remains alive through `normalized_native_dll_name`. DLL extension,
regular-file, UTF-8, ambiguity, import-graph and digest checks are unchanged.
Source SHA-256 at the 2026-09-09 repair boundary:
`8988336c6aa6e520cc2b3c0d22bc09760f421b80ca8022f75c52e81f107a50d7`.

Scoped `git diff --check` passed. The first exact managed Cargo submission
(`runtime06-native-dependency-borrow-validation-20260910-r1`) was rejected at
admission with `validation_ticket_external_worktree_dirty` for
`E:\\Git\\zr_vm`; no Cargo process or test executed and no pass is claimed.
The source repair remains pending the same clean external dependency and the
original targeted-import regression.

## 2026-09-11 failure rolling repair

- Re-verified the owner source and regression scope under the stable session `failure-roll-01a084c8-runtime06-native-borrow`; `native_artifact_trust.rs` retains the owned `OsString` through UTF-8 normalization, and the native loader contract fixture remains attributed. Rustfmt (edition 2021) and scoped `git diff --check` pass.
- Corrected snapshot 3419 freezes the exact failure record, `native_artifact_trust.rs` (`8988336c6aa6e520cc2b3c0d22bc09760f421b80ca8022f75c52e81f107a50d7`), and `native_plugin_loader_contract.rs` (`c80df608db9e66b33761ec03c71ce9550c0b0cf3d3d8b744efc118159d5b1d0d`). Request `runtime06-native-dependency-borrow-20260911-r1` submitted `cargo +1.94.1 check -p zircon_runtime --no-default-features --locked --lib --tests` with the originating targeted-import test retained as the required follow-up.
- Coordinator admission returned `validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`; no ticket, Cargo check, or dynamic test evidence exists. This lifecycle remains open and the stable session is `waiting_validation`; after the external owner supplies a clean revision, rerun the owner check and the exact targeted-import regression before any fixed return or closeout.

## 2026-09-19 failure rolling successor r2

- Session `failure-roll-01a084c8-runtime06-native-borrow-r2` received the three-path transfer from archived owner `failure-roll-01a084c8-runtime06-native-borrow` (fingerprint `89b6904533d9b870429e3b30c49e317d61b85d72f0781b35b63d66ed8c948463`), then claimed leases and current baseline attribution without changing source bytes. The stable lifecycle key is unchanged; the fixed/return artifacts remain absent.
- Current source still retains `entry.file_name()` in `file_name: OsString` through the UTF-8 borrow before `normalized_native_dll_name`, and the native loader contract fixture remains in scope. The current source hashes are `native_artifact_trust.rs=8988336c6aa6e520cc2b3c0d22bc09760f421b80ca8022f75c52e81f107a50d7` and `native_plugin_loader_contract.rs=c80df608db9e66b33761ec03c71ce9550c0b0cf3d3d8b744efc118159d5b1d0d`.
- Static source-contract validation is being resealed under the successor; managed Windows `--locked` check, exact targeted-import execution, upstream Runtime04 acceptance, independent review, failure return, and closeout remain pending the external `E:\\Git\\zr_vm` clean revision.

### 2026-09-19 static ticket materialization failure (preserved)

- Ticket `27e85742c25a45179395457bbedf3dfe` was rejected during coordinator materialization before command execution (`validation_copy_dependency_archive_failed`, job `70ad6cf25b5d4031bcb64bdc7967bc1f`). No source assertion or test result is inferred.
- The failure was caused by declaring the current untracked successor-owned `native_artifact_trust.rs` as a baseline template dependency; it is an overlay input, not a path available in the pinned Git archive. The next ticket keeps the source in the sealed overlay but declares only the tracked contract test as the template dependency root.

### 2026-09-19 corrected static validation

- Corrected ticket `32a426d849f34724b458a92c01498716` (request `failure-roll-01a084c8-runtime06-native-borrow-20260919-r3`) retained the untracked production source as an attributed overlay and used only the tracked contract test as the pinned template dependency.
- Coordinator job `030e29862aa4460a82251c96260b2570` / run `32a426d849f34724b458a92c01498716` finished exit 0 with stdout `RUNTIME06_NATIVE_DEPENDENCY_BORROW_SOURCE_CONTRACT_PARSE_PASS`; cleanup completed. This proves the OsString lifetime/source contract only. Managed Cargo, exact targeted-import execution, Runtime04 upstream acceptance, independent review, fixed return, and closeout remain pending.

### 2026-09-20 independent review

- Reviewer Session `review-runtime06-native-borrow-r2` inspected the source-sealed failure record,
  `native_artifact_trust.rs`, and the attributed native-loader contract fixture. The production
  source hash is `8988336c6aa6e520cc2b3c0d22bc09760f421b80ca8022f75c52e81f107a50d7`; the contract
  fixture hash is `c80df608db9e66b33761ec03c71ce9550c0b0cf3d3d8b744efc118159d5b1d0d`. The reviewer
  held the failure-document lease during the audit.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. `capture_native_dependency_closure`
  now retains `entry.file_name()` in an owned `OsString`, borrows its UTF-8 view only while calling
  `normalized_native_dll_name`, and leaves regular-file, UTF-8, ASCII/extension, ambiguity,
  transitive-import, and digest checks intact. The contract fixture's authority-bound Windows
  paths preserve tamper rejection and real-fixture ABI checks; unrelated callback/global-hook
  ownership is outside this lifecycle.
- Scoped Rustfmt (edition 2021, root-only) and `git diff --check` passed, and the reviewer’s
  source probe emitted `RUNTIME06_NATIVE_BORROW_REVIEW_CONTRACT_PASS`. No source was edited by the
  review. Managed `--locked` Cargo, exact targeted-import execution, Runtime04 upward acceptance,
  external `E:/Git/zr_vm` clean revision, fixed return, and closeout remain pending.

## 2026-09-28 current-source reconciliation under successor r3

- Both earlier fixing Sessions were archived. Registration request
  `3798ba211c564a90bf05f6d217658aeb` created the executable primary Session
  `failure-roll-01a0df1a-runtime06-native-borrow-r3` without changing the lifecycle key or
  the ownership of any historical ticket. Exact three-path transfer preview
  `1c63ec3199424aa5be73993e83839439` and apply
  `7828afedd8774d50aa9f0d27d6654f77` used fingerprint
  `446ac65042d6c5597c73930f4a2affc224fbcc2bc0e384186e979b312f6e72dc`.
  Coordinator attribution at epoch 628 covers the current source and fixture bytes.
- Before this record update, the failure record was SHA-256
  `09890cdec5c86e98e45b739c78365a79044734694c2d3791a703b86c9483cda0`.
  The current production source is
  `native_artifact_trust.rs=185c00a963b4aad91d75ade95e2590345683bc9a3f618bcda1519448577d129b`;
  the current contract fixture is
  `native_plugin_loader_contract.rs=f4cb644952ce15934e92cdab4188bc90e9aec0066d92925db2f6c942baafdb16`.
  These differ from the September 20 reviewed source and fixture. That review and passed static
  ticket `32a426d849f34724b458a92c01498716` remain historical evidence for their exact old
  inputs, not current-source dynamic acceptance.
- The current `capture_native_dependency_closure` still retains the owned `OsString` through the
  UTF-8 view and normalized DLL-name check. The Windows real-fixture test writes to its export
  source after loading; the loader stages and loads a verified copy in a distinct temporary
  directory, so this source write does not target the mapped DLL. This is a source-level finding;
  the test has not been executed for these current bytes.
- The pending managed Windows batch is the locked `zircon_runtime --no-default-features` library
  and test compilation, the actual Windows native-loader contract test
  `native_runtime_hot_update_loads_real_fixture_from_export_manifest`, the original
  `asset::tests::project::manager::targeted_import` filter including its five metadata
  precondition regressions, and the Runtime04 upward asset/worker-pool gates. Each exact filter
  must execute a nonzero number of tests. External `E:/Git/zr_vm` capture, direct Cargo route
  proof, current-source independent review, failure return, and closeout remain open.
