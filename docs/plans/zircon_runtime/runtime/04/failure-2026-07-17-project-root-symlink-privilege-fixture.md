---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: project-root-symlink-privilege-fixture
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/tests/project/package_assets.rs
tests:
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never asset::tests::project::package_assets::project_root_registration_rejects_a_canonical_symlink_escape -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never registration -- --test-threads=1
---

# Runtime04：Windows symlink privilege fixture classification

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M3 RuntimePlugin lifecycle `registration` focused gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：失败发生在 Runtime04 拥有的 project-root 资产测试夹具；Frameworks02 不应修改资产生产校验、申请系统权限或以插件层 fallback 掩盖该环境能力分类。

## 失败现象与复现证据

Frameworks02 Windows managed job `cbbe13aff0db495181c4ec16e984c51f` / run `a4e893e4be0c44859e38fc19b0697986` 执行 `registration` filter，结果 211 passed / 5 failed / 8025 filtered，exit 101。`project_root_registration_rejects_a_canonical_symlink_escape` 在进入 `PackageAssetRegistry::register_project_roots` 前创建目录符号链接失败，Windows 返回 OS error 1314（客户端没有所需的特权），fixture 随即 panic。

## 最低共享层根因与硬切

生产 project-root canonical escape 校验尚未执行；故障仅属于 Windows 测试夹具的能力探测。当前 Rust/Windows 组合没有把 1314 稳定映射为 `ErrorKind::PermissionDenied`，而 helper 只识别后者。最低修复是在 Windows helper 中把 raw OS 1314 与 `PermissionDenied` 一并视为“本机无法构造该 fixture”，沿用已有清理后提前返回路径；生产注册器、canonicalization 和 escape error 均不得改变。

## 架构修复验收

- exact test 在具备 symlink 权限时继续验证 `CanonicalProjectAssetRootEscape`；不具备权限时仅跳过无法构造的 fixture，测试进程不得 panic。
- Frameworks02 `registration` 重跑时该 panic 消失。

## 禁止临时方案

- 不得申请或修改系统权限，不得把 symlink 替换成 junction，不得弱化生产 root containment 校验，也不得增加兼容 API、alias 或 shim。

## 修复结果与回传

Open state: `Windows 1314 fixture classification is present / current-source Cargo blocked before the exact test`; no Cargo pass is claimed.

## 产出记录与时间

| 时间 | 状态 | 证据与后续 |
|---|---|---|
| 2026-08-27 | `open / managed compile reached / exact test not executed` | Windows helper 仍同时识别 `PermissionDenied` 与 raw OS error `1314`，且 production project-root containment 源码未改动。Fresh managed `validate-matrix.ps1` job `2b227f610a374f2f878af4c69b9b4121` 进入 `cargo test -p zircon_runtime --locked --lib project_root_registration_rejects_a_canonical_symlink_escape`，随后以 wrapper exit `1` / Cargo exit `101` 终止于 587 条外部 current-source 编译错误；首条为 `text/glyph_artifact/geometry.rs:320` 的 Rust 2024 let-chain，随后还有 `scene_renderer/ui/render/rich_text.rs:95` let-chain、`runtime_interface/src/runtime_api/host_requests.rs` compile-time resource 缺失，以及 Runtime11/Render owner 的可见性与导入错误。`package_assets.rs` 没有 rustc 诊断，目标测试计数为 0；这些证据不能替代 exact/registration GREEN，故本 failure 保持 open，等待外部编译债收敛后由 Runtime04 重跑同一受管门。 |

### 2026-09-08 current fixture executed; registration batch did not complete

The existing fixture source still classifies raw OS 1314 and PermissionDenied.
Its current SHA-256 is
`044094915ddca0d298d99ac1e71af054c5a39a28468d7dc4684b0204a810b3c5`;
production `project/package_asset_registry.rs` is
`cb66c869d35d9e1b1eb360b0abb3e67fa5afa04da0e94daecca97516cf6fd09b`.
Both match immutable input `frameworks01-readiness-payload-3237-20260908`,
manifest `5dc7c3d6891874a69436ba9e7641c9e0863eb7ed548c05db7306233c5da9543d`.
No fixture or production source was changed by this continuation.

Managed Windows static/no-default/locked job
`82a958b201a94a8eb40887701173698f` compiled and started all 345 tests selected
by `registration`. The named original test
`asset::tests::project::package_assets::project_root_registration_rejects_a_canonical_symlink_escape`
executed and reported `ok`. This does not establish which fixture-capability
branch ran, or claim a separate `--exact` execution. Before a Runtime11 hang,
the log observed 290 `ok` and 27 `FAILED` results; these are partial counts,
not a completed harness summary or an accepted registration gate.

The hung terminal-observer ordering test is linked to the existing
[Runtime11 bounded terminal delivery failure](../11/failure-2026-08-23-task-terminal-delivery-bounded-dispatch.md).
After confirming its inline callback/barrier self-deadlock and the managed
process ancestry, only test leaf PID 9132 was terminated. The Cargo/validator
parents completed naturally and recorded wrapper exit 1. The source-bound
receipt and log are under the input's `results/` directory as
`runtime04-symlink-registration-3237-r2.{json,log}`. The first attempt had no
accepted job because the reuse pool was busy; this retry did not duplicate one.

The two physical-path alias-root tests also reported failure, separately from
the original symlink test. Their fixture uses a junction; this observation does
not prove the same root cause as the existing
[App08 junction fixture failure](../../../optimize/zircon_app/08/failure-2026-09-08-runtime-library-junction-fixture-creation.md).
The remaining registration failures span other owners and require their own
diagnosis. The full upward gate and formal acceptance remain open; the broad
batch must not be repeated unchanged while its confirmed hang is unresolved.

### 2026-09-08 completed package-assets support batch

The separate [junction-fixture repair](failure-2026-09-08-project-root-junction-verbatim-fixture.md)
now has source snapshot 3251, hash
`e5479d32794b9f30f33477214f0840997216dfbe5bfb7dc46114078924d31dbb`.
It changes only the junction helper, its diagnostics and a new junction test;
the symlink helper and this lifecycle's production containment behavior remain
unchanged. The earlier whole-file hash is historical preimage evidence.

Managed Windows static/no-default/locked job
`836e5a3c4dd347929364305b085984a6` ran all 10 package-assets tests with
10 passed, 0 failed, 0 ignored. The original symlink escape test reported
`ok`, as did both previously failing alias-root registration tests. Input
`runtime04-junction-3251-20260908`, manifest
`55328b53070a78a8374cf55f629fc8501d01dee2f2517a2ddc2db94fbf1bfca9`,
retains `results/runtime04-junction-command-line-3251.{json,log}`.
This completed focused batch supersedes the earlier before-test block for
the symlink fixture. The capability branch remains unobserved, and the full
registration gate, formal binding and closeout remain pending.
