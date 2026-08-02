---
handoff_kind: fixed
status: fixed
created_at: 2026-07-31
resolved_at: 2026-07-31
summary_slug: asset-pipeline-audit-behavior-test-discovery
origin_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
failure_scope: local
plan_link_mode: required
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/diagnostics.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/single_flight.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool/task_pool.rs
tests:
  - python tools/tests/test_runtime_asset_pipeline_audit.py
---

# Runtime04: asset pipeline audit child behavior-test discovery

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 来源执行切片：Runtime04 current child-guard audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：该 local failure 的审计 inventory、behavior source set 与 mirror counts 均由 Runtime04 自身所有。

## 失败现象与复现证据

`python tools/tests/test_runtime_asset_pipeline_audit.py` 的 current child-guard audit
遗漏六个已经存在于 Rust child modules 的 behavior anchors。直接复跑执行 2 tests，唯一失败
是同一六项 `missing_behavior_test_anchors`；另一项通过。这是静态审计 failure，不是 Cargo 证据。

## 最低共享层根因

`asset_pipeline_boundary.py` 只读取声明 child modules 的
`zircon_runtime/src/asset/tests/pipeline/worker_pool.rs`，没有读取
`diagnostics.rs`、`single_flight.rs` 与 `task_pool.rs` 中的真实测试体。
同时 production worker policy 已拆入 `worker_pool/options.rs` 与
`worker_pool/completion.rs`，旧 source inventory 也未包含两个真实 owner。

## 架构修复验收

- 三个 worker-pool child test modules 同时进入 guard inventory 与 behavior source set。
- 两个 production worker policy owners 进入 source inventory，不删除或弱化任何 anchor。
- Runtime04 source/guard mirror counts 与真实 owner 集合一致。
- `python tools/tests/test_runtime_asset_pipeline_audit.py` 全绿且 `risks = []`。

## 禁止临时方案

- 不得复制六个 behavior tests 到 parent module、改名绕过 inventory 或放宽 missing-anchor 判定。
- 不得删除真实 production/test owner 来恢复旧计数。
- 不得把 Python static gate 解释成 Runtime04 broader Cargo acceptance。

## 修复结果与回传

- 根因：审计只扫描 parent module 和过时 source owner 集合，无法发现 child test bodies 与拆分后的 worker policy owners。
- 架构修复：source inventory 加入两个 production owners，guard/behavior inventory 加入三个 child test modules；现有 anchors 保持唯一。
- 验证：coordinator 应用受保护脚本 patch 70/72；最终 `python tools/tests/test_runtime_asset_pipeline_audit.py` 为 2 passed / 0 failed，detail 为 `source=24`、`guard=20`、`worker_missing=0`、`behavior_missing=0`、`risks=0`，scoped diff-check 与二审 C0/I0/M0 通过。
- 回传：Runtime04 local audit gate 可恢复；broader asset/worker managed acceptance 仍独立保持 `in_progress`。
