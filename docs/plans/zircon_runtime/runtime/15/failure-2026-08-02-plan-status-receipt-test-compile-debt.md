---
handoff_kind: failure
status: open
created_at: 2026-08-02
summary_slug: plan-status-receipt-test-compile-debt
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
related_code:
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
  - tools/tests/test_runtime_receipt_hard_cut.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - docs/engine-architecture/runtime-architecture-review-m0.md
tests:
  - python -m unittest tools.tests.test_runtime_receipt_hard_cut tools.tests.test_runtime_schedule_frame_loop_audit
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter runtime_absorption
---

# Runtime15：计划状态 receipt tests 污染 Runtime lib-test 编译面

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-08-02 全仓 plans/source/test 独立审阅
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：该子树由 Runtime15 的结构与测试 owner 挂载，并被其 file-budget/architecture review guard 反向约束；Performance01 不应在并发 Runtime owner 之外拆除整棵测试树。

## 失败现象与复现证据

`zircon_runtime/src/tests/runtime_absorption/mod.rs` 通过 `mod plan_status;` 把计划生命周期检查编入 `zircon_runtime` lib-test。2026-08-02 current worktree 静态盘点为：

- `plan_status.rs` 加子树共 711 个 Rust 文件、38,656 行；
- 仅 48 个 `#[test]`，主体读取 `docs/plans`、archive 和状态表措辞，而非运行时产品行为；
- `status_output_tables/` 单独包含 654 个 Rust 数据文件，`#[test]` 数为 0，只为上层 receipt tests 提供计划状态常量；
- `docs/engine-architecture/runtime-architecture-review-m0.md` 与 Runtime15 的 structure-convention guards 又枚举这些测试文件，形成“测试树约束文档、文档再约束测试树”的自引用。

精确盘点使用 `Path.rglob("*.rs")` 统计文件/行，并逐文件计数 `#[test]`。当前子树还有 4 个并发修改文件（Runtime02/03/08/13 cargo-gate anchors）；直接整树删除会覆盖其他 Session 的未提交工作，因此本轮只保留 failure，不抢写这些文件。

## 最低共享层根因

计划状态、历史证据和 Cargo gate 可见性被实现成 Runtime Rust lib-test，再为满足文件预算持续拆成常量/路由/镜像子模块。结果是计划叙述成为编译契约、计划改字触发运行时测试失败，且没有行为测试的表数据也参与编译。通用计划 schema、handoff 与 output-record 完整性应由仓库 tooling validator 持有；Runtime lib-test 只应验证源码边界与运行时行为。

## 架构修复验收

- 在保留所有并发修改语义后删除 `mod plan_status;`、`plan_status.rs` 与完整 `plan_status/` 子树，不保留 facade、archive fallback 或兼容模块。
- 删除 structure-convention/file-budget 中仅用于维护 plan-status 文件形状、路径和状态镜像的 guards；保留真正检查生产 owner、模块挂载、文件预算和禁止 API 的源码 guards。
- 从 `runtime-architecture-review-m0.md` 和 Runtime15 structured metadata 删除已退役测试路径；计划 YAML、handoff、output-record、链接与 lifecycle 校验统一走现有 Python/Coordinator tooling。
- 运行 managed `zircon_runtime` lib-test gate，确认删除只减少 receipt tests，不丢失 runtime behavior/structure coverage；随后运行 handoff validator 与 plan-output audit。

## 禁止临时方案

- 不把 654 个零测试表文件搬到另一个 Rust 模块，也不新增 alias、re-export、`cfg`、ignore 或空壳来保留旧路径。
- 不修改计划状态/历史措辞来满足 receipt tests，不把 archive fallback 当作 current truth。
- 不删除真正读取生产源码并验证模块边界、预算或禁止依赖的 structure guards。

## 修复结果与回传

Open state: `receipt_tree_and_status_mirror_hard_delete_implemented_review_green_managed_validation_pending`。

- `mod plan_status;`、receipt root、row-data/status-slice 镜像与仅维护这些形状的结构守卫已直接删除；未新增 facade、archive fallback、alias、re-export、`cfg` 或空壳。
- `assert_contains_all` 只检查显式 source；专用 plan-status Python 审计模块及总审计器入口已删除，计划 lifecycle 由编号文档与 Coordinator/Python 工具持有。
- M0、模块规范与 Runtime15 计划的退役路径元数据和 receipt-only 历史章节已清理；生产 owner、模块预算、禁止 API、dead-code 与 lock-poison 守卫继续保留。
- 本地精确 Python 回归通过 5/5；Python 模块语法检查通过。全量 `audit_runtime_structure.py --json` 的本地 125 秒运行超时，未作为 GREEN/RED 证据。
- 独立二次审查已完成，结果为 Critical/Important/Minor = `0/0/0`；仍待 managed Runtime lib-test、handoff validator、plan-output audit 与原子提交回执，在这些证据完成前不宣称 fixed/accepted。
- 静态 M0.3 审计发现并删除 `naming_boundary/split_layout.rs` 对退役 `support/status_evidence.rs` 的最后一个 `include_str!`；receipt guard 现在扫描所有存活 Rust 的六类退役 route fragment，红测精确命中后回归恢复 5/5，未恢复旧文件。
- current-source validation manifest 已冻结为 1,865 路径，其中 1,858 个 JSON `null` 删除墓碑、7 个存活验证输入，canonical JSON 为 305,657 bytes，SHA-256 为 `856cec8429e8fb16d3391fd33be7cc60c568271c299b57121ceb81cfc1fb4237`。该 manifest 不纳入记录自身或其他 failure 记录，避免状态回写造成自引用哈希漂移；所有实际编译/静态验证输入仍完整封存。
- 该 manifest 超过 Windows process command-line 上限，不能再使用 inline JSON 参数；Coordinator01 已以 [fixed return](../../../mvp/00/fixed-2026-08-04-validation-ticket-large-manifest-cli-transport.md) 提供同一严格校验路径上的 UTF-8 stdin transport，并有 managed `30/30` 证据。此前没有 Runtime lib-test ticket 的记录早于该 return；下一次 current-source snapshot 必须通过 stdin 重新提交完整 manifest，不省略任何删除路径。failure 继续保持 `open`，因为 Runtime lib-test、handoff validator 与 plan-output audit 尚无该刷新 snapshot 的 terminal evidence。
