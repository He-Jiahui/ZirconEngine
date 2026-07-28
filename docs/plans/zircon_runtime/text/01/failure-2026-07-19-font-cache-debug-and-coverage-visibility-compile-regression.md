---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: font-cache-debug-and-coverage-visibility-compile-regression
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/fallback.rs
  - zircon_runtime/src/text/font/vertical_metrics.rs
tests:
  - cargo test -p zircon_editor --lib --no-default-features --locked --jobs 1 --no-run --message-format short --color never
  - cargo test -p zircon_runtime --lib text::font --locked --jobs 1 --color never -- --test-threads=1
---

# Text01：font cache Debug 与 coverage helper 可见性编译回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行者：`editor-layout15-material-state-priority-business-20260717`
- 来源执行切片：whole-editor current-source no-run 诊断 job `7a118e773c5d4e1ca351d06999c1a7c1` / run `16027c4887524c58abd79b268ff7de54`
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：全部错误位于 Text01 正在收敛的 generation-owned face metadata、fallback cache 与 coverage helper 边界，Layout15 不应修改 font database 内部合同。

## 失败现象与复现证据

Layout15 的受管 Windows job 执行 `cargo test -p zircon_editor --lib --no-default-features --locked --jobs 1 --no-run --message-format short --color never`，于 2026-07-19 02:48:24 +08:00 自然结束并释放，exit 101、tests 0、live PIDs 为空。raw stderr 位于 `.codex/state/session-coordinator/cargo-runs/7a118e773c5d4e1ca351d06999c1a7c1/16027c4887524c58abd79b268ff7de54/stderr.log`，共 8 条编译错误：

- `database.rs:169/170`：`EffectiveInstanceCache` 与 `FallbackCaches` 未实现 `Debug`，使包含它们的 database 状态派生失败（2 条 E0277）。
- `fallback.rs:267/284/314/337/339`：fallback owner 无法调用私有的 `face_covers_all`、`face_covers_codepoint` 与 `face_coverage_count`（5 条 E0624）。
- `vertical_metrics.rs:16`：vertical metrics owner 无法调用私有的 `face_vertical_advance_units`（1 条 E0624）。

该 job 启动后，Frameworks root 文件与 Text01 三个相关源文件均发生过 owner 修改，因此它只能作为 current captured-source 诊断，不能作为 Layout15、Text01 或 Editor15 的 immutable acceptance。错误出现前 rustc 已越过此前的 Frameworks、Editor05 与 Layout15 编译根因。

## 最低共享层根因

Text01 将 face metadata/cache 收敛到新的内部 owner 时没有原子迁移派生与 sibling consumer 可见性：聚合状态仍要求全字段 `Debug`，而 fallback/vertical consumer 已迁到 sibling 模块，所需只读 helper 仍停留在原模块私有级别。

## 架构修复验收

- 由 Text01 决定 cache 是否应提供语义安全的 `Debug`（可省略内部大表）或让聚合 owner 使用手写 `Debug`；不得删除有用的 database 诊断能力来规避 E0277。
- coverage 与 vertical helper 只放宽到 `text::font` 所需的最窄 sibling 可见性，保持 face metadata/cache 的单一 owner；不得扩成 crate-wide/public API。
- 先运行 Text01 font focused gate，再运行新鲜 source-bound `zircon_runtime --lib` 与 `zircon_editor --lib --no-run` 上行门禁；记录 job/run/raw log、测试计数和无 source race 证据。
- 既有 `font-face-metadata-reparse` 与 `font-fallback-candidate-rebuild` 性能验收继续有效；本 compile fix 不得回退 generation-owned cache 或恢复重复解析/候选重建。

## 禁止临时方案

- 不得移除 cache 字段、关闭整个 database `Debug` 合同或使用 test-only 派生绕过。
- 不得把 coverage/vertical helper 设为 `pub`/`pub(crate)` 以扩大架构面；只允许 sibling owner 所需的最窄边界。
- 不得把 source-raced job `7a118e...` 记录为任何计划的 GREEN。

## 修复结果与回传

Open state: `implementation_complete / managed_validation_pending`。`EffectiveInstanceCache` 与 `FallbackCaches` 均提供省略内部大表、只展示 bounded report 的语义安全 `Debug`；face metadata/coverage helper 使用 `pub(in crate::text::font)`，严格开放给 font sibling 而不扩成 crate public。vertical production consumer 进一步改为按 face 借用并复用 `FontVerticalMetrics`。待 fresh focused/broad source-bound 验证、独立复审、受管提交与 fixed return。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
| --- | --- | --- | --- |
| 2026-07-19 02:48 +08:00 | `open / Text01 待修复` | 已记录 2 条 cache `Debug` E0277、5 条 fallback coverage E0624 与 1 条 vertical metrics E0624；未修改 Text01 业务源码。 | job 自然 exit 101、tests 0、PIDs 清空；该运行存在 source race，仅作诊断，待 Text01 fresh source-bound 验证与 fixed return。 |
| 2026-07-19 03:50 +08:00 | `implementation_complete / managed_validation_pending` | cache `Debug`、font-sibling visibility 与 vertical metadata view 已完成；结构守卫锁定不得扩成 `pub(crate)`；rustfmt/scoped diff 通过。 | Plugins01 job `c1fe7621...` 在最后一次 Text Rust 写入后启动 rustc，诊断出 7 条 Text + 1 条 foreign Scene；7 条 Text 已修复。该 job 自身为 orphaned/source-raced，不计 GREEN，待 fresh managed job。 |
| 2026-07-19 08:20 +08:00 | `implementation_complete / review_green / managed_validation_pending` | `face_glyph_id` 仅开放到 `crate::text` 的 SDF consumer；metadata/coverage helper 继续保持更窄的 `crate::text::font` 可见性；结构 guard 同步锁定。 | 独立终审 0/0/0 Ready。Editor03 source-raced job `0873f135...` 编译到当前 Text 后仅有 5 条 foreign plugin-bridge 错误，不能替代 fresh Text gate。 |
| 2026-07-28 01:45 +08:00 | `implementation_complete / managed_broad_runtime_passed / editor_upward_running` | Managed Runtime job `8f1c073d40ce4bee8483c046e6ee6b9b` / run `48f0711c4ca1468d90b7545df7c6e047` completed the declared `text::font` broad compile-and-test return. | Exit 0: `79 passed / 0 failed / 2 ignored / 8922 filtered`; current `zircon_editor --lib --no-run` job `4eefa547982a4bd896813d9fad698f21` remains the required upward compile contract. |
| 2026-07-28 02:42 +08:00 | `Text01_runtime_return_passed / external_editor_return_failed` | The same fresh editor upward job `4eefa547982a4bd896813d9fad698f21` / run `ceff37fc13224768af1c365287f242e5` compiled current Runtime/Text and then exited 101. | The 56 errors are all in `zircon_editor` API/DTO/test owners (private fields, pane projection, event initializer, plugin lifetime, and test type drift); no Text01 source diagnostic occurred. This failure is not closed until its external editor return is repaired and rerun. |
