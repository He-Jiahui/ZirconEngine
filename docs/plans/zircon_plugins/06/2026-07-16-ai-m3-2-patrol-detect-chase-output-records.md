---
related_code:
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/selector.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase/fixtures.rs
implementation_files:
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/selector.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/mod.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase/fixtures.rs
tests:
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase.rs
  - zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase/fixtures.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime tests::scenarios::patrol_detect_chase --locked --jobs 1 -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --jobs 1
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-16 严格按 zircon_plugins 架构计划持续完成插件功能
doc_type: milestone-detail
---

# 2026-07-16 AI M3.2 巡逻-发现-追逐产出记录

Plan: docs/plans/zircon_plugins/06-ai.md
Milestone: M3.2
Slice: M3.2 / M3-T4
Status: completed
Files: ["docs/plans/zircon_plugins/06/2026-07-16-ai-m3-2-patrol-detect-chase-output-records.md", "docs/zircon_plugins/ai/runtime.md"]

## Scope delivered

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | T4 真实场景 | `完成` | 2026-07-16 | 真实 Level、插件扩展、Perception、Behavior 和中立 Navigation 动态组件共同驱动巡逻、发现、追逐与到达成功。 |
| M3 | Reactive selector | `完成` | 2026-07-16 | higher-priority perception decorator 变为 eligible 后，先 abort lower branch，再执行 chase host mutation。 |
| M3 | 结构收束 | `完成` | 2026-07-16 | selector 执行职责从约 900 行 executor owner 抽到独立 folder-backed owner；统一复用 subtree abort owner。 |
| M3 | Testing | `完成` | 2026-07-16 | direct/nested/policy scenario 10 项与 AI full package 96/96 均通过，doctest 受管 exit 0。 |

## 架构边界

- 场景只依赖共享 `navigation.Component.NavMeshAgent` 合同，并在测试 World 注册最小中立 descriptor；AI runtime 没有新增对具体 Navigation 插件 crate 的依赖。
- Selector 对显式可判定的 blackboard/perception decorator 在 host mutation 前抢占 lower-priority branch；tri-state side-effect-free probe 覆盖 nested Sequence/Selector/RandomSelector/Parallel，known-false guard 复用 cached failure，uninspectable external guard 才延后到自身 tick。
- 两条抢占路径共同调用 `abort_subtree`，因此 MoveTo destination 清理、外部 runtime `on_abort` 与活动节点状态移除只有一个 owner，不建立上层测试专用成功旁路。
- 端到端场景通过真实 scene systems 保证 `ai.perception_tick -> ai.behavior_tick`，并用目标限定的 `NavAgentTickReport` 完成 chase；测试不启用 Navigation debug capture。

## Fresh testing evidence

- 源码与场景测试由协调器集成提交 `ad2c6f98` 承载；本切片清单仅绑定该提交之后仍有差异的最终验收记录与模块文档，不重复声明已经进入 HEAD 的源码。
- Focused scenario：Windows managed job `ceae105af06b4885b8852dc51a6ff695`，2 passed / 0 failed / 86 filtered out，exit 0。
- Observer policy regression：Windows managed job `c0619de2d83a46c48a39569f70b593a3`，1 passed / 0 failed，证明 `None`/`Self` 不会抢占 lower-priority running branch，exit 0。
- Full AI package：首轮 refreshed Windows managed job `bdcd816e50d7425ba6fdd655ff9c9181` 编译通过并完成 96 项执行，95 passed / 1 failed；唯一失败为测试 fixture 的 Decorator child-count 合同，已补齐 succeeded child。最终 post-split Windows managed job `8e1df08d245749a79e0cd1c63e1b5b28` 为 96 passed / 0 failed，doctest 通过，exit 0。
- 静态：触及 Rust 文件 `rustfmt` 通过，精确 `git diff --check` 通过；plugin structure audit 的 manifest、free-function registration、compatibility shim、SDK mirror 与 distribution boundary 均为 0 违规。生产 owner 最大 710 行；场景 behavior owner 405 行、folder-backed fixture owner 506 行。

## Review

- 最终独立只读复审已完成，Critical 0 / Important 0；post-split managed gate 已通过。

## 未完成边界

- M5 行为树图编辑器、运行时节点高亮、Blackboard 面板与 Perception overlay 仍未完成。
- EQS-style 环境查询未由 M3-T4 引入；整体 AI package 继续保持 `Experimental / Partial`。
