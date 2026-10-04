---
handoff_kind: failure
status: open
created_at: 2026-08-25
summary_slug: engine-compatibility-caret-range-exhaustiveness
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_editor/51
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/project/engine_compatibility/directional_range.rs
tests:
  - .\tools\build\build-editor.ps1 -Ephemeral
---

# Editor51: engine compatibility caret range match is not exhaustive

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Windows Editor product build and visual-acceptance gate
- 修复责任计划：`docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md`
- 交接原因：Editor51 owns project engine/BuildSet compatibility preflight. The failing file is a new untracked current-worktree implementation under that contract; UI12 does not own semantic-version range parsing.

## 失败现象与复现证据

On 2026-08-25, `\.\tools\build\build-editor.ps1 -Ephemeral` entered the managed Windows Cargo lane at target `F:\cargo-targets\zircon-engine\ephemeral\check\ba774352932c4e7daea94f508ab8064d` and failed with one compiler error:

```text
error[E0004]: non-exhaustive patterns: `(1_u64..=u64::MAX, Some(_), _)` not covered
  --> zircon_runtime_interface\src\project\engine_compatibility\directional_range.rs:178:23
```

The upper-layer expected path was the normal Editor product build through `zircon_app`; no UI-specific fallback is acceptable. Plausible lower support layers were comparator parsing, comparator-to-range projection, caret upper-bound selection, and Editor51 project preflight integration. The compiler proves the lowest failing layer is `caret_range` upper-bound selection before any Editor/UI code can be validated.

## 最低共享层根因

`caret_range` matches `(base.major, minor, patch)` and uses a guard arm `(major, Some(_), _) if major > 0`. Rust exhaustiveness checking does not treat that guard as covering every `major >= 1` value, so the match remains non-exhaustive for `(1..=u64::MAX, Some(_), _)`. The repair must express the nonzero-major domain as an exhaustive pattern while preserving SemVer caret behavior for `^1`, `^1.2`, `^0.2`, `^0.0.3`, and overflow boundaries.

## 架构修复验收

- Add focused lower-layer range tests for nonzero-major caret comparators, zero-major/minor/patch cases, and checked overflow behavior.
- Compile and run the focused `zircon_runtime_interface` engine-compatibility tests through the managed Windows validator.
- Re-run `\.\tools\build\build-editor.ps1 -Ephemeral`; UI12 can resume the current-source product screenshot gate only after the normal `zircon_app` path passes this lower layer.

## 禁止临时方案

- Do not add a wildcard that silently returns `None`, weaken compatibility decisions, or bypass preflight in Editor/App.
- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or UI12 product acceptance criteria to hide the failure.

## 修复结果与回传

### 2026-08-27 current-source continuation

- The untracked current-source implementation now expresses the nonzero-major domain as the exhaustive pattern `(1.., Some(_), _)`, with no `if major > 0` guard. The zero-major nonzero-minor, zero-minor patch, omitted patch, and checked `u64::MAX` overflow branches remain explicit.
- Focused source inspection confirms four regression tests cover `^1.2`, `^0.2`, `^0.0.3`, and maximum-major overflow. `rustfmt +1.94.1 --edition 2021 --check zircon_runtime_interface/src/project/engine_compatibility/directional_range.rs` passes, and a structured static contract verifies every required branch/test marker.
- Managed compilation cannot yet be claimed: the shared validator currently fails closed before Cargo acquire with `unmanaged_artifacts_detected` for the foreign path `D:\\ZirconBuilds\\tooling15-local-benchmarks` and an empty `cleanupReservations` list. This continuation does not delete or adopt that Tooling15 artifact and does not retry under unchanged governance state.

Open state: `部分推进，待受管编译与 Editor 产品回归`; no fixed/return claim is made.

### 2026-08-27 managed focused validation

- Artifact governance is no longer the admission blocker: the fresh audit returned
  `unmanaged: []`, and the managed validator started Cargo job
  `afdeb83a911b4aff83f04a719aad658b`.
- The job ran `cargo test -p zircon_runtime_interface --locked --lib caret_`, reached
  real `rustc`, then released normally with wrapper exit `1` / Cargo exit `101`.
- Compilation stopped before the caret tests on nine foreign current-source lib-test
  errors: missing public projections for `UiHitRouteNode` and
  `UiTextShapeArtifact`, crate-name imports inside `binding_value_contracts.rs`, two
  `ProjectManifestSummary` initializers missing `project_guid`, and one stale
  `UiTextShapeArtifact::as_ref` call. No diagnostic names `directional_range.rs`.
- The focused caret test count is therefore zero. This is forward compile evidence,
  not a GREEN result; the failure remains open until those lower-layer owners restore
  the package test boundary and the same managed filter executes successfully.

### 2026-08-31 exact-owner continuation

- Editor51 已接管当前精确源码 blob
  `zircon_runtime_interface/src/project/engine_compatibility/directional_range.rs`，
  SHA-256 为
  `B4BCE07D63A2604548CC9B3CD53C7AFE4CB6EBC512CD7A93EAD5C6017D5E6FC7`。
- 生产分支保持五类显式穷举且无 `_` fallback：`^1`、`^1.2` 取 next-major，
  `^0.2` 取 next-minor，`^0.0.3` 取 next-patch，`^0.0` 取
  next-minor；major、zero-major minor、zero-major/minor patch 三类
  `checked_add` 溢出均直接断言 `caret_range(...).is_none()`。
- 独立只读终态 review 为 `Critical / Important / Minor = 0 / 0 / 0`；
  `rustfmt --edition 2021 --check` 与 scoped `git diff --check` 均通过。
- 已提交受管 Windows 库级编译 ticket
  `d63bd88448ae40a0ad9f444d5b227990`（request
  `caret_compile_check_20260831_0135`）和聚焦测试 ticket
  `580ff315af974093972fb9e06e895486`（request
  `caret_focused_tests_20260831_0135`）。两者 source manifest hash 均为
  `21deb6eaa2a3cdcb595190e8cca8e9a997b6724bc4b7d05c50834052f310ba8a`，
  当前状态均为 `queued`。

Open state: `源码与静态复核完成，待受管编译、聚焦测试及 Editor 产品回归`；
在 GREEN 结果形成前不创建 fixed/return 记录。

### 2026-09-08 ticket reconciliation and current regression

The two original tickets have terminal GREEN results with the original owner
`root-editor51-autosave-policy-materialization-20260831` unchanged:

- Compile ticket `d63bd88448ae40a0ad9f444d5b227990`, job
  `991f178139bd4971854399802d44928a`, exit 0 at
  `2026-08-31T01:51:31.990883+00:00`.
- Focused ticket `580ff315af974093972fb9e06e895486`, job
  `516e156258104140be81c3c07978c453`, exit 0 at
  `2026-08-31T01:52:15.471452+00:00`.

Current `directional_range.rs` remains SHA-256
`b4bce07d63a2604548cc9b3cd53c7afe4cb6ebc512cd7a93ead5c6017d5e6fc7`.
Managed Windows static/no-default/locked job
`69c68a09fcc74e55a5f065c9a7ae1d0d` actually executed all eight
`project::engine_compatibility` tests: 8 passed, 0 failed, 0 ignored. This covers
all five caret shapes and three checked-overflow cases on that exact source.
Immutable input `interface02-reflection-catalog-3213-20260908`, manifest
`140cc434c0b430b58e937b125e5c7994a6fc0896c580046a5361ff876485f24e`;
result files `editor51-caret-compatibility-3213.log` and `.json` are under its
`results` directory.

The canonical record alone was transferred to stable continuation Session
`failure-roll-01a07160-editor51` with audited fingerprint
`fdcb692a2444c44cbbcb196f2efb8f4ee02ba31a83dada79b186aa89ce9319f1`.
Existing source and ticket ownership are preserved. The Editor product build,
formal return and closeout still need acceptance; neither earlier static
checks nor the eight lower-layer tests replace the required product build.

## 产出记录与时间

| 时间（Asia/Shanghai） | 状态 | 完成项目 | 量化证据 | 后续门禁 |
|---|---|---|---|---|
| 2026-08-31 09:35 | 部分推进 | 接管精确源码；补齐 5 类 caret 形态与 3 类 overflow 回归；完成独立终态 review；提交 2 个托管 Windows 验证 | SHA-256 `B4BCE07D...E6FC7`；C/I/M=`0/0/0`；rustfmt=`0`；diff-check=`0`；tickets=`d63bd884...`、`580ff315...` | 两个 ticket GREEN 后重跑 `build-editor.ps1 -Ephemeral`；未通过前保持 `open` |

## 2026-09-19 rolling successor formal source binding

- Successor Session `failure-roll-01a084c8-editor51-caret-r2` reclaimed the exact failure
  artifact and implementation path through coordinator transfer fingerprint
  `370fd85e4fb6abcfe1aa5fdb05784c063510d42c12bcf42ba607b9cb62c3243e` at baseline epoch
  `611`. The transfer changed no source bytes; current implementation hash remains
  `b4bce07d63a2604548cc9b3cd53c7afe4cb6ebc512cd7a93ead5c6017d5e6fc7`.
- A successor formal Windows source-contract ticket will assert the exhaustive caret match,
  all five caret-shape branches, the three checked-overflow tests, and rustfmt. It defers the
  required managed Editor product build, independent C/I/M review, fixed return and closeout;
  the prior lower-layer GREEN evidence remains supporting evidence only.
- Request `failure-roll-01a084c8-editor51-caret-20260919-r1` admitted ticket
  `87fd130f951048d9907895d35866c5e5` with sealed manifest hash
  `bcf0eb41c1630f3d0b53c0e27a96b467ff3279ef2e99c203a29b52d833d74272`; status is `queued`.
  The immutable command is source-contract-only and does not claim the deferred product build
  or any closeout state.

### Ticket correction after validator-scope assertion failure

- Ticket `87fd130f951048d9907895d35866c5e5` executed and failed at
  `2026-09-19T05:25:59.109872Z` because its broad source check treated the unrelated
  `comparator_range` wildcard arm `_ => None` as a caret implementation fallback. The
  exhaustive `caret_range` match itself was not disproven; this is a validator-scope error,
  not a source or test failure. Its sealed manifest hash was
  `bcf0eb41c1630f3d0b53c0e27a96b467ff3279ef2e99c203a29b52d833d74272`.
- The failed ticket remains retained as non-reusable evidence. A successor command will scope
  the wildcard assertion to the extracted `caret_range` function and leave unrelated
  comparator operations out of that check, without changing source bytes.
- Corrected request `failure-roll-01a084c8-editor51-caret-20260919-r2` admitted ticket
  `32f11f7c60724776b50a34ab166f7df0` with sealed manifest hash
  `a12eb445b54d9f113ca57cda8c98cb567f9c0f2f70a237515843b7fccf87b1e8`; status is `queued`
  pending the coordinator terminal result.

### Corrected source-contract ticket terminal result

- Corrected ticket `32f11f7c60724776b50a34ab166f7df0` completed `passed` at
  `2026-09-19T05:28:53.661379Z` (exit code 0), emitting
  `EDITOR51_ENGINE_COMPATIBILITY_CARET_SOURCE_CONTRACT_PARSE_PASS`. Its sealed manifest hash
  is `a12eb445b54d9f113ca57cda8c98cb567f9c0f2f70a237515843b7fccf87b1e8`.
- This is a current-source static contract result only. The managed Editor product build,
  upward acceptance, independent C/I/M review, canonical fixed return and closeout remain
  pending; the external `E:\Git\zr_vm` dirty-worktree blocker is retained.
