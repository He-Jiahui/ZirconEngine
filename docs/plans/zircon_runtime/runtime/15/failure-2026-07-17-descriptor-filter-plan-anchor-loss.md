---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: descriptor-filter-plan-anchor-loss
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/script_host.rs
tests:
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::production_file_budget::texture_descriptor_settings::runtime_15_texture_descriptor_settings_parser_is_child_owner -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::runtime_dead_code::script_host::runtime_15_script_host_value_descriptors_do_not_suppress_dead_code -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1
---

# Runtime15：descriptor filter plan/status anchors were lost

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M3 RuntimePlugin lifecycle `descriptor` focused gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：两项失败都验证 Runtime15 structure/status evidence routing；Frameworks02 不应删减结构守卫或复制 Runtime15 历史正文来通过自己的上行门。

## 失败现象与复现证据

Windows managed job `386e3d872b224189b488a0d37e564f34` / run `53f2e2c699974d9cb51bcbb9b3b040e5` 执行 `cargo test -p zircon_runtime --lib --locked --jobs 1 --color never descriptor -- --test-threads=1`，结果为 242 passed / 3 failed / 7996 filtered out，exit 101。本交接覆盖其中两项：

1. `runtime_15_texture_descriptor_settings_parser_is_child_owner`
   - 在第一个 `Runtime 15 plan` source 上短路，缺少完整 5-anchor 集合：切片标题、状态 slug、parent path、child settings path 与测试名。
   - 同一测试随后还会检查 runtime index、engine review/structure priority plans、module/importer/render-assets docs、status row、status map 与 date map；不能只补首个 panic 后就宣称完成。
2. `runtime_15_script_host_value_descriptors_do_not_suppress_dead_code`
   - 在第一个 `Runtime 15 plan` source 上短路，现有标题与测试名存在，但缺 `runtime_15_script_host_value_descriptors_coremin_check_passed`。
   - 同一测试还要求 runtime index、两个 priority plans、module convention 与 script host ledger 的三锚集合一致。

独立 current-source 静态矩阵确认：Runtime15 parent 对 texture 为 0/5、对 script 为 2/3；runtime index 对 texture 为 1/5、对 script 为 0/3；两个 priority plan 对 texture 均只有 parent path 1/5、对 script 均为 0/3。`docs/zircon_runtime/structure/module-convention.md` 保留两组完整锚，script function ledger 也保留 script 3/3，说明生产/模块 owner 证据仍存在，断裂的是 current plan/status 路由。

## 最低共享层根因

Runtime15 parent/index 与 priority plan 的 current evidence aggregation 在压缩或 child-owner 拆分后没有同步迁移这两组可执行锚。测试仍以多个 aggregate documents 作为并列消费者，导致已有 child/module truth 无法通过 current parent/status route 被发现。

## 架构修复验收

- 先确定每组锚的 canonical current child record；parent/index/priority plans 应链接或由守卫读取该 child owner，不得重新复制大段历史成为第二事实源。
- 两个 exact tests 分别通过，且同一测试内的所有后续 sources、status row/status/date map 断言都实际执行，不能只消除首个短路。
- 重跑 Frameworks02 `descriptor` filter 时两项消失；Render07 的 SSR history 失败独立处理。
- 更新状态时保持 `engine-code-structure-convention.md` 与 `engine-code-review-findings-2026-06.md` 的优先级和 hard-cut 规则，不降低 required-anchor 集合。

## 禁止临时方案

- 不得删除测试、缩短 required anchors、从 `descriptor` filter 排除结构守卫，或用 ignored/allowlist 绕过。
- 不得把 archive aggregate 恢复为 current truth，不得在多个父计划复制同一完整状态正文，不得添加兼容路径。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
