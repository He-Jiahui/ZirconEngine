---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: canonical-descriptor-dot-segments
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/project/canonical_descriptor_identity.rs
  - zircon_runtime_interface/src/project/tests/project_identity.rs
  - zircon_editor/src/core/project/project_preflight/preflight_receipt.rs
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib project::tests::project_identity::
  - cargo test -p zircon_editor --locked --lib core::project::project_preflight::
---

# Interface02: literal dot segments in canonical descriptor identity

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：完整接口库动态验收的 project identity 回归。
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 交接原因：canonical physical descriptor 身份的词法准入属于 Interface02，Editor preflight 直接消费。

## 失败现象与复现证据

Windows locked/static 作业 `893e5568e6364fd9944fd431c04c6ffa` 执行原有
`canonical_descriptor_identity_rejects_nonphysical_path_shapes` 失败；完整结果
747 passed、16 failed、101 ignored。输入 `interface-cache-generation-3159-20260908`，
manifest `e275ded38d2cd913d9bac17ee5ec8fdcff1f9e17e75306de307a82d6b5c38038`。
原夹具在受管 Windows verbatim TEMP 路径上调用 `PathBuf::join("..")`，把需检测的
ParentDir 在进入 constructor 前规范化掉，构造器实际收到的是合法绝对路径。

## 最低共享层根因

夹具必须用 OsString 保留字面点段；此外 constructor 依赖 `Path::components()`，
该 API 会规范化普通绝对路径中的 CurDir，因此不能完整证明“不含 . 或 ..”的
既有 DTO 契约。Editor preflight 从 filesystem authority 取得已经解析的操作路径，
此修复只收紧公开构造与反序列化的词法验证，不替代文件系统真实身份解析。

## 架构修复验收

- 原测试在 Windows 受管 TEMP 和普通 C 盘绝对路径上实际检查原始 `.`、`..`。
- 构造器与 serde 拒绝字面点段，继续接受合法绝对路径并保留非 Unicode OS bytes。
- 原 ProjectIdentity round-trip 与 Editor preflight 直接消费 gate 通过。
- 独立审查 C0/I0/M0、正式受管绑定、failure return 和 closeout 完成。

## 禁止临时方案

- 不删除 ParentDir 断言，不把输入先 canonicalize 后作为拒绝夹具，不调用真实 I/O
  或用 lossy Unicode 转换验证这个 data-only DTO。
- 不降低 lexical contract，不增加第二套项目 filesystem authority。

## 修复结果与回传

Open state: `precise-fixture-red-validation-pending`。
Session `failure-roll-01a07160-interface02`，前置快照 3161；测试快照 3166 的 hash 为
`1f3f0fc9288148929c10c6d1baab31a45d0a87b75bb684efbef5c859daa247eb`。
它保留原 relative/serde/round-trip 测试，改用 OsString 构造原始点段，并加入不依赖
实际文件存在的普通 Windows 绝对路径。生产 constructor 尚未修改，先取得动态红态。
未 failure return、未 commit、未发送企微。

保留原始点段的受管红态作业 `971de0c483af45aaabfb499a48e2807e` 在普通 Windows
绝对路径的 `.` 断言失败；此时 codec 与 manifest fixtures 已转绿，完整接口库
751 passed、12 failed、101 ignored，输入 manifest
`4a1f09d59338ae525a09dba4becf2c02fa7e4ac64135a0cce1ad7bc7c040f3e3`。

最终源码快照 3170 在既有绝对路径 admission 后用 `OsStr::as_encoded_bytes()` 和
平台 `is_separator` 判断精确 `.`/`..` 段，不分配、不执行 I/O、不损失 OS bytes。
constructor hash `8ddbdccc30b3cca19d044097c271ce210b88359e7bb177b6b3697506c238b7ee`；
test hash `90d9948d933b1201cd850cb832958fa7bdc583d245067133f6a3be1384d3dca3`。
测试同时覆盖 serde 拒绝、`.zircon/project..toml` 合法文件名，以及含未配对 UTF-16
surrogate 的 Windows OS 路径不被 lossy 转换。

受管绿态作业 `dd18d653fe3c494c91fbe57f5959580a` 实际通过四个 project_identity
测试，输入 `interface-project-identity-3170-20260908`，manifest
`1f5a61e72075db714e26e348064c43abf268af11edf80465b15536ef40f639c9`；
全库 754 passed、11 failed、101 ignored、0 filtered。日志为
`results/interface-library-3170.{json,log}`。当前状态为
`source-repaired_lower-regression-green_upward-review-closeout-pending`；
Editor preflight、独立审查与正式 failure 验收仍待完成。

Independent review in the existing task "优化协调器验证效率" completed on
2026-09-08 with C0/I0/M0. Report:
`.codex/tmp/interface02-runtime09-3173-review-20260908-result.txt`.
The reviewer checked raw OS-path admission, Windows/Unix separators, the direct
Editor preflight caller, and exact current/attributed/ObjectStore/input hashes.
Upward Editor verification and formal failure closeout remain pending.

The original fixing Session was reactivated and the two changed interface paths
were transferred from the stale Astra attribution after their hashes were
reconciled. The exact upward command
`cargo +1.94.1 test -p zircon_editor --no-default-features --locked --lib core::project::project_preflight:: -- --test-threads=1 --nocapture`
was submitted once on 2026-09-09, but coordinator admission rejected it with
`validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`. No Cargo
process ran and no new acceptance or closeout is claimed; retry after that
external repository is clean.

## 2026-09-19 rolling successor source reconciliation

- Successor Session `failure-roll-01a084c8-interface02-canonical-r1` reclaimed
  the canonical failure record and the two already-repaired Interface02 source
  paths at baseline epoch `611`. Ownership transfer fingerprint:
  `e9ddc3359d6516cf29629be09073a37da15ea11aac6bbb6bd6e5e482a4c7d4e1`.
  Current hashes remain constructor
  `8ddbdccc30b3cca19d044097c271ce210b88359e7bb177b6b3697506c238b7ee` and
  test `90d9948d933b1201cd850cb832958fa7bdc583d245067133f6a3be1384d3dca3`;
  this slice did not alter source bytes.
- The prior managed Interface02 project-identity job
  `dd18d653fe3c494c91fbe57f5959580a` remains reusable only for the lower
  `zircon_runtime_interface` regression: four focused tests passed, with the
  recorded full-library result `754 passed, 11 failed, 101 ignored`. The
  existing independent review remains C0/I0/M0. These receipts do not satisfy
  the upward Editor preflight gate.
- The exact upward Editor command was previously rejected before Cargo by
  `validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`; no retry
  is claimed in this successor slice. A fresh managed Editor gate is still
required after that external worktree is clean, followed by canonical
failure return and closeout.

### 2026-09-19 current-source static validation

- Static request `failure-roll-01a084c8-interface02-canonical-20260919-r1`
  admitted ticket `0b3ae21e53144a1da54c3d6ffcf5364f` with sealed manifest
  `ca19d6c3cc67797bae13f5f95bc8ae87d5ca8ff788f2cafbdc633499716c052d`.
  The checker verifies byte-preserving lexical `.`/`..` rejection, serde and
  literal-filename coverage, and the Windows non-Unicode path regression.
- This is a static-only ticket; its fresh upward Editor Cargo gate, review
  refresh, canonical fixed return, closeout, and the external `E:\\Git\\zr_vm`
clean-worktree prerequisite remain pending. No dynamic acceptance is claimed.

### 2026-09-19 static checker script correction

- Ticket `0b3ae21e53144a1da54c3d6ffcf5364f` ran job
  `26e3beb82e81415ca8f536aceefeb8d3` and terminated `failed` at
  `2026-09-19T07:08:32.319906Z` before any source or Cargo work. Its Python
  checker embedded the Rust byte-literal assertion with unescaped nested
  double quotes, producing a `SyntaxError` at line 7. The coordinator marked
  the result `failureCacheExcluded=true`; this is validator-script evidence,
  not a product failure or acceptance result.
- No source bytes changed. A corrected checker will use a single-quoted Python
  literal for the Rust fragment and rerun against a fresh manifest.

### 2026-09-19 corrected static retry

- Corrected request `failure-roll-01a084c8-interface02-canonical-20260919-r2`
  admitted ticket `d1bdb9c84bbf4bb282eb6e01c2f68b66` with sealed manifest
  `9f0c615ee848f33cb2d382e7fe6125e646975785181282b871d82ccd62239700`.
  The checker uses single-quoted Python literals for the Rust byte-pattern and
  keeps the same source-contract assertions; status is `queued` pending the
  coordinator terminal result.
- This remains static-only. The fresh managed Editor upward gate, review
  refresh, canonical return/closeout, and clean `E:\\Git\\zr_vm` prerequisite
  remain pending.

### 2026-09-19 corrected static terminal result

- Corrected ticket `d1bdb9c84bbf4bb282eb6e01c2f68b66` passed at
  `2026-09-19T07:13:21.235229Z` in job
  `23a0574f53e54e9f8a97d0d11e997b91`, exit code `0`, with marker
  `INTERFACE02_CANONICAL_DESCRIPTOR_DOT_SEGMENTS_CURRENT_SOURCE_CONTRACT_PASS`.
  Cleanup event `10905` completed. The earlier ticket `0b3ae21e...` remains
  retained as a checker-SyntaxError receipt and is not reused.
- This terminal result is static source-contract evidence only; fresh managed
  Editor preflight Cargo, review refresh, canonical fixed return, closeout, and
  clean `E:\\Git\\zr_vm` remain pending.

### 2026-09-20 independent source review refresh r1

The reviewer inspected the current Interface02 constructor and regression
fixtures together with the direct Editor preflight consumer.  The lexical
admission remains byte-preserving (`OsStr::as_encoded_bytes()`), uses the
platform separator predicate, rejects only literal `.`/`..` path segments after
the absolute-path check, and continues to accept literal filename dots.  Serde
deserialization delegates to the same constructor, while Editor preflight
constructs the identity from its resolved operation path without introducing a
second filesystem authority.  The non-Unicode Windows fixture remains
platform-gated and does not perform lossy conversion or I/O.

Read-only checks:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` over
  `canonical_descriptor_identity.rs`, `project_identity.rs`, and
  `preflight_receipt.rs`: `INTERFACE02_RUSTFMT_PASS paths=3`.
- `git diff --check` over those three paths: pass (only Git line-ending
  notices).

Current source hashes inspected:

```text
zircon_runtime_interface/src/project/canonical_descriptor_identity.rs 8ddbdccc30b3cca19d044097c271ce210b88359e7bb177b6b3697506c238b7ee
zircon_runtime_interface/src/project/tests/project_identity.rs 90d9948d933b1201cd850cb832958fa7bdc583d245067133f6a3be1384d3dca3
zircon_editor/src/core/project/project_preflight/preflight_receipt.rs 18a8cca3e66b527f4bef110b76c115adbd15b9205b15ead7e65a008323c5ab14
```

Independent review result: `Critical=0 Important=0 Moderate=0`.  The prior
lower-interface four-test receipt is retained only for that lower scope; no
fresh upward Editor Cargo result, product acceptance, failure return, or
closeout is inferred here.
